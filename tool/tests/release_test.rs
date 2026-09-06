//! Scenario test of wave 0048: a release is built by one script.
//!
//! `release.sh` is the one build of a release -- the workflow runs it
//! on a tag, a person runs it by hand -- and what it writes is what
//! the launcher fetches: `keel-<version>-<target>.tar.gz` with one
//! file `keel` inside, and `<archive>.sha256` beside it. Judged on the
//! world of wave 0041: a repository shaped like keel's and a stub
//! `cargo`, so the probe pays milliseconds instead of a release build
//! and still runs the real script, the real tar and the real sha256.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;
use common::versions::{host, world};
use std::path::Path;
use std::process::Command;

/// The real release.sh -- the copy standing in the tree it releases,
/// as the workflow and a person have it -- with the stub cargo first
/// on PATH.
fn release(tree: &Path, stub: &Path, args: &[&str]) -> (String, i32) {
    let script = tree.join("release.sh");
    let path = format!(
        "{}:{}",
        stub.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("sh")
        .arg(&script)
        .args(args)
        .current_dir(tree)
        .env("PATH", path)
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

fn sh(dir: &Path, line: &str) -> (String, i32) {
    let out = Command::new("sh")
        .args(["-c", line])
        .current_dir(dir)
        .output()
        .unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

/// proves: a-release-is-built-by-one-script@27bfe9
#[test]
fn a_release_is_built_by_one_script() {
    let dir = sandbox("release");
    let w = world(&dir);
    // The world's repository is the tree the script runs in: its
    // tool/Cargo.toml says 2.0.0, and the stub cargo builds a binary
    // that answers so.
    let tree = w.repo.clone();
    // The script lives beside tool/ in the tree it releases: copy it
    // in, as the workflow and a person have it.
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("release.sh");
    std::fs::copy(&script, tree.join("release.sh")).expect("release.sh exists at the root");
    let (said, code) = release(&tree, &w.stub, &["--out", "dist"]);
    assert_eq!(code, 0, "one script builds the release:\n{said}");

    // The archive and its checksum, named by the version the built
    // binary ANSWERS and the target rustc names -- not a number
    // copied from a manifest.
    let target = host();
    let name = format!("keel-{}-{target}.tar.gz", w.new_version);
    let archive = tree.join("dist").join(&name);
    assert!(
        archive.is_file(),
        "the archive is written as {name}:\n{said}"
    );
    let sum_file = tree.join("dist").join(format!("{name}.sha256"));
    assert!(sum_file.is_file(), "with its checksum beside it:\n{said}");
    assert!(
        said.contains(&name) && said.contains(".sha256"),
        "and the script says what it wrote:\n{said}"
    );

    // The checksum file is one sha256sum would write, and it checks.
    let sum = std::fs::read_to_string(&sum_file).unwrap();
    assert!(
        sum.trim().ends_with(&format!("  {name}")),
        "the checksum names the archive the way sha256sum -c reads it: {sum}"
    );
    let (checked, code) = sh(&tree.join("dist"), &format!("sha256sum -c '{name}.sha256'"));
    assert_eq!(code, 0, "and sha256sum -c accepts it:\n{checked}");

    // Inside: exactly one file, `keel`, and it answers the version.
    let (listed, _) = sh(&tree.join("dist"), &format!("tar -tzf '{name}'"));
    assert_eq!(listed.trim(), "keel", "one file inside, `keel`:\n{listed}");
    let unpack = dir.join("unpack");
    std::fs::create_dir_all(&unpack).unwrap();
    let (_, code) = sh(&unpack, &format!("tar -xzf '{}'", archive.display()));
    assert_eq!(code, 0);
    let (version, _) = sh(&unpack, "./keel --version");
    assert!(
        version.contains(&format!("keel {}", w.new_version)),
        "the binary inside answers the version in the archive's name:\n{version}"
    );

    // The version is what the BUILT binary answers, never the
    // manifest's number (review 0048 R-8): a tree whose manifest says
    // 9.9.9 and whose binary answers 2.0.0 releases 2.0.0.
    std::fs::write(tree.join("tool/answers"), "2.0.0\n").unwrap();
    std::fs::write(
        tree.join("tool/Cargo.toml"),
        "[package]\nname = \"keel\"\nversion = \"9.9.9\"\n",
    )
    .unwrap();
    let (said, code) = release(&tree, &w.stub, &["--out", "dist2"]);
    assert_eq!(code, 0, "{said}");
    assert!(
        tree.join("dist2").join(&name).is_file() && !said.contains("9.9.9"),
        "the archive carries the binary's own answer, not the manifest's:\n{said}"
    );

    // The tag being released must be the binary's version with a `v`
    // in front: a release under `v3.0.0` whose binary answers 2.0.0
    // is one no pin ever finds (review 0048 R-1) -- refused, and
    // nothing written.
    let (said, code) = release(&tree, &w.stub, &["--out", "dist3", "--tag", "v3.0.0"]);
    assert_ne!(
        code, 0,
        "a tag the tree does not answer to is refused:\n{said}"
    );
    assert!(
        said.contains("v3.0.0") && said.contains("2.0.0") && said.contains("tag the commit"),
        "and the refusal names both numbers and what to do:\n{said}"
    );
    assert!(
        !tree.join("dist3").exists()
            || std::fs::read_dir(tree.join("dist3"))
                .unwrap()
                .next()
                .is_none(),
        "nothing is written under a wrong tag"
    );
    let (said, code) = release(&tree, &w.stub, &["--out", "dist4", "--tag", "v2.0.0"]);
    assert_eq!(code, 0, "the matching tag passes:\n{said}");
    assert!(tree.join("dist4").join(&name).is_file());

    // `--out` without a value is the script's word, not the shell's
    // (review 0048 R-12); and no `--out` at all lands inside the
    // ignored build tree, never in `git status` (R-11).
    let (said, code) = release(&tree, &w.stub, &["--out"]);
    assert_eq!(code, 2, "{said}");
    assert!(said.contains("--out needs a directory"), "{said}");
    let (said, code) = release(&tree, &w.stub, &[]);
    assert_eq!(code, 0, "{said}");
    assert!(
        tree.join("tool/target/dist").join(&name).is_file(),
        "the default out is tool/target/dist, inside the build tree:\n{said}"
    );

    // The workflow runs THIS script on a tag, attests what it built,
    // and publishes the archives with their checksums -- held by its
    // text, since neither a tag push nor `gh` runs here.
    let flow = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(".github/workflows/release.yml"),
    )
    .expect("the release workflow exists");
    assert!(
        flow.contains("tags:") && flow.contains("\"v*\""),
        "it runs on a version tag:\n{flow}"
    );
    assert!(
        flow.contains("sh release.sh") && flow.contains("--tag \"$GITHUB_REF_NAME\""),
        "it builds with the one script, and hands it the tag being released:\n{flow}"
    );
    for os in [
        "ubuntu-latest",
        "ubuntu-24.04-arm",
        "macos-latest",
        "macos-15-intel",
    ] {
        assert!(
            flow.contains(os),
            "it builds every target the launcher can name -- {os}:\n{flow}"
        );
    }
    assert!(
        flow.contains("attest-build-provenance"),
        "it attests the provenance of what it built:\n{flow}"
    );
    assert!(
        flow.contains("gh release") && flow.contains("dist/"),
        "and publishes what the script wrote:\n{flow}"
    );
    assert!(
        flow.contains("rust-toolchain") || flow.contains("rustup"),
        "on the toolchain the project pins, not the runner's own:\n{flow}"
    );
}
