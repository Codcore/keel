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
        flow.contains("sh release.sh"),
        "it builds with the one script:\n{flow}"
    );
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
