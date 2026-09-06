//! Scenario test of wave 0048: the launcher fetches a missing version
//! aloud.
//!
//! The concept's line, the operator's decision: a project pins a
//! version that is not installed, and `keel` fetches its release,
//! verifies the checksum, says what it took and from where, and runs
//! it -- or refuses honestly with the ready command. Judged on the
//! world of wave 0041 with a release server that is a directory
//! served over `file://`, holding what `release.sh` writes.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;
use common::versions::{World, host, install, world};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// A release server: `<dir>/<tag>/keel-<version>-<host>.tar.gz` and its
/// `.sha256`, built the way release.sh builds them, holding a `keel`
/// that answers `keel <answers>` -- the tag's own version unless a
/// probe wants a binary that lies. `tamper` writes a checksum that is
/// not the archive's.
fn serve(dir: &Path, tag: &str, version: &str, tamper: bool) -> String {
    serve_answering(dir, tag, version, version, tamper)
}

fn serve_answering(dir: &Path, tag: &str, version: &str, answers: &str, tamper: bool) -> String {
    let home = dir.join("releases").join(tag);
    fs::create_dir_all(&home).unwrap();
    let build = dir.join(format!("build-{tag}"));
    fs::create_dir_all(&build).unwrap();
    fs::write(
        build.join("keel"),
        format!("#!/bin/sh\necho \"keel {answers}\"\necho \"args: $*\"\n"),
    )
    .unwrap();
    let name = format!("keel-{version}-{}.tar.gz", host());
    let (said, code) = sh(
        &build,
        &format!(
            "chmod +x keel && tar -czf '{}' keel && cd '{}' && sha256sum '{name}' > '{name}.sha256'",
            home.join(&name).display(),
            home.display()
        ),
    );
    assert_eq!(code, 0, "the release server is built:\n{said}");
    if tamper {
        fs::write(
            home.join(format!("{name}.sha256")),
            format!("{}  {name}\n", "0".repeat(64)),
        )
        .unwrap();
    }
    format!("file://{}", dir.join("releases").display())
}

/// A release whose archive is random bytes and whose `.sha256` is of
/// something else: a launcher that checks BEFORE unpacking speaks of
/// the checksum; one that unpacks first speaks of the unpacking
/// (review 0048 R-5).
fn serve_garbage(dir: &Path, tag: &str, version: &str) {
    let home = dir.join("releases").join(tag);
    fs::create_dir_all(&home).unwrap();
    let name = format!("keel-{version}-{}.tar.gz", host());
    fs::write(home.join(&name), b"this is not a gzip archive at all\n").unwrap();
    fs::write(
        home.join(format!("{name}.sha256")),
        format!("{}  {name}\n", "0".repeat(64)),
    )
    .unwrap();
}

/// The installed launcher, run in a project, with the release server
/// and a temp dir of the probe's own -- so what a fetch leaves behind
/// is seen.
fn launch(w: &World, project: &Path, releases: &str, tmp: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(w.bin.join("keel"))
        .args(args)
        .current_dir(project)
        .env("KEEL_HOME", &w.home)
        .env("KEEL_REPO", &w.repo)
        .env("KEEL_RELEASES", releases)
        .env("TMPDIR", tmp)
        .env(
            "PATH",
            format!(
                "{}:{}",
                w.stub.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
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

fn project(dir: &Path, name: &str, pin: &str) -> PathBuf {
    let project = dir.join(name);
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("keel.toml"),
        format!("lang = \"uk\"\nversion = \"{pin}\"\n"),
    )
    .unwrap();
    project
}

/// proves: the-launcher-fetches-a-missing-version-aloud@420e44
#[test]
fn the_launcher_fetches_a_missing_version_aloud() {
    let dir = sandbox("fetch");
    let w = world(&dir);
    let (said, code) = install(&w, Some(&w.old_ref));
    assert_eq!(
        code, 0,
        "the launcher is installed with one version:\n{said}"
    );
    let tmp = dir.join("tmp");
    fs::create_dir_all(&tmp).unwrap();
    let releases = serve(&dir, "v2.0.0", "2.0.0", false);
    serve(&dir, "v3.0.0", "3.0.0", true);

    // A pin nobody installed, with a release to take: fetched,
    // verified, installed into a home of its own with every record,
    // said aloud -- and run.
    let pinned = project(&dir, "pinned", "2.0.0");
    let (said, code) = launch(&w, &pinned, &releases, &tmp, &["--version"]);
    assert_eq!(code, 0, "the missing version is fetched and run:\n{said}");
    assert!(
        said.contains("keel 2.0.0"),
        "and it is the version the project pinned:\n{said}"
    );
    let name = format!("keel-2.0.0-{}.tar.gz", host());
    assert!(
        said.contains("fetched")
            && said.contains(&name)
            && said.contains("v2.0.0")
            && said.contains("file://"),
        "the launcher says WHAT it took and FROM WHERE:\n{said}"
    );
    assert!(
        said.contains("sha256") && said.contains("verified"),
        "and that the checksum was checked:\n{said}"
    );
    let home = w.home.join("versions").join("v2.0.0");
    assert!(home.join("keel").is_file(), "the binary stands in its home");
    assert_eq!(
        fs::read_to_string(home.join(".keel-version"))
            .unwrap()
            .trim(),
        "2.0.0"
    );
    assert_eq!(
        fs::read_to_string(home.join(".keel-ref")).unwrap().trim(),
        "v2.0.0"
    );
    let sum = fs::read_to_string(home.join(".keel-sum")).unwrap();
    assert!(
        sum.trim().len() == 64 && sum.trim().chars().all(|c| c.is_ascii_hexdigit()),
        "the binary's own sha256 is recorded, as for a built version: {sum}"
    );
    assert!(
        home.join(".keel-sha").is_file(),
        "and where it came from is recorded"
    );

    // The second run does not fetch again: the version stands.
    let (said, code) = launch(&w, &pinned, &releases, &tmp, &["--version"]);
    assert_eq!(code, 0, "{said}");
    assert!(
        !said.contains("fetched"),
        "a version that stands is not fetched twice:\n{said}"
    );

    // A checksum that does not match: a refusal, and NOTHING installed
    // -- no home, no half-unpacked archive in the temp dir.
    let tampered = project(&dir, "tampered", "3.0.0");
    let (said, code) = launch(&w, &tampered, &releases, &tmp, &["--version"]);
    assert_eq!(
        code, 2,
        "a release whose checksum does not match refuses:\n{said}"
    );
    assert!(
        said.contains("checksum") && said.contains("nothing was installed"),
        "and says why, and that nothing landed:\n{said}"
    );
    assert!(
        !said.contains("keel 3.0.0"),
        "no other binary ran in its place:\n{said}"
    );
    assert!(
        !w.home.join("versions").join("v3.0.0").exists(),
        "no home was left behind"
    );
    let leftovers: Vec<_> = fs::read_dir(&tmp)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        leftovers.is_empty(),
        "and the temp dir is clean after a refusal: {leftovers:?}"
    );

    // The order is checksum FIRST, unpacking after (review 0048 R-5):
    // an archive that is garbage under a wrong checksum is refused
    // for the checksum, and never unpacked.
    serve_garbage(&dir, "v5.0.0", "5.0.0");
    let garbage = project(&dir, "garbage", "5.0.0");
    let (said, code) = launch(&w, &garbage, &releases, &tmp, &["--version"]);
    assert_eq!(code, 2, "{said}");
    assert!(
        said.contains("does not match its published checksum") && !said.contains("did not unpack"),
        "the checksum is judged before anything is unpacked:\n{said}"
    );

    // A release whose binary answers another number than its tag
    // (review 0048 R-2): refused, nothing installed -- and so never
    // fetched again and again on every run.
    serve_answering(&dir, "v6.0.0", "6.0.0", "6.0.1", false);
    let lying = project(&dir, "lying", "6.0.0");
    let (said, code) = launch(&w, &lying, &releases, &tmp, &["--version"]);
    assert_eq!(
        code, 2,
        "a release that answers another version refuses:\n{said}"
    );
    assert!(
        said.contains("answers 6.0.1") && said.contains("nothing was installed"),
        "and says which number it answered:\n{said}"
    );
    assert!(!w.home.join("versions").join("v6.0.0").exists());

    // A temp dir that cannot be made (review 0048 R-3): a refusal that
    // names it, nothing fetched into `/`, nothing installed -- not a
    // success with litter, which is what a function run as an `if`
    // condition does when its `mktemp` fails and nobody looks. A pin
    // not yet installed, so the road is walked at all.
    serve(&dir, "v8.0.0", "8.0.0", false);
    let nowhere = dir.join("nowhere").join("deeper");
    let (said, code) = launch(
        &w,
        &project(&dir, "notmp", "8.0.0"),
        &releases,
        &nowhere,
        &["--version"],
    );
    assert_eq!(
        code, 2,
        "a temp dir that cannot be made is a refusal:\n{said}"
    );
    assert!(
        said.contains("temp dir")
            && said.contains("nothing was installed")
            && !said.contains("verified"),
        "and it names the temp dir, and claims no fetch:\n{said}"
    );
    let litter = Path::new("/").join(format!("keel-8.0.0-{}.tar.gz", host()));
    let littered = litter.exists();
    let _ = fs::remove_file(&litter);
    let _ = fs::remove_file(Path::new("/").join(format!("keel-8.0.0-{}.tar.gz.sha256", host())));
    assert!(
        !littered,
        "no archive was written into / in place of the temp dir"
    );
    assert!(
        !w.home.join("versions").join("v8.0.0").exists(),
        "and no home was made"
    );

    // No release for the pin: a refusal that says what would work --
    // a ref, or waiting for the release -- not the install command
    // that walks to the same 404 (review 0048 R-13); never another
    // version run in silence.
    let missing = project(&dir, "missing", "4.0.0");
    let (said, code) = launch(&w, &missing, &releases, &tmp, &["--version"]);
    assert_eq!(code, 2, "a pin with no release refuses:\n{said}");
    assert!(
        said.contains("no release") && said.contains("v4.0.0") && said.contains("pin a ref"),
        "naming what was not there and the road that works:\n{said}"
    );
    assert!(
        !said.contains("KEEL_REF=\"4.0.0\""),
        "and not the command that would fail the same way:\n{said}"
    );
    assert!(!said.contains("keel 1.0.0"), "{said}");

    // And nothing in this world ever asked github.com for anything:
    // the world's own release server, an empty directory, stands in
    // KEEL_RELEASES for every install and launch (review 0048 R-4).
    assert!(
        !dir.join("curl-http.log").exists(),
        "no http(s) request left this machine:\n{}",
        fs::read_to_string(dir.join("curl-http.log")).unwrap_or_default()
    );

    // A pin that is a git ref, not a version: not turned into a
    // release address at all -- the old road, with its own refusal.
    let branch = project(&dir, "branch", "plan/0048-something");
    let (said, code) = launch(&w, &branch, &releases, &tmp, &["--version"]);
    assert_eq!(code, 2, "{said}");
    assert!(
        !said.contains("no release") && said.contains("KEEL_REF="),
        "a ref takes the git road, and no release URL is built from it:\n{said}"
    );
}

/// The real install.sh against the world, with the release server set
/// and the stub `cargo` on PATH or not -- without it, the only road
/// to a binary is the release.
fn install_with(w: &World, git_ref: &str, releases: &str, cargo_on_path: bool) -> (String, i32) {
    let installer = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("install.sh");
    let inherited = std::env::var("PATH").unwrap_or_default();
    let path = if cargo_on_path {
        format!("{}:{inherited}", w.stub.display())
    } else {
        // Every real cargo off the PATH too: a bare PATH of the
        // system's own directories, which carry git, tar and sha256sum
        // but no cargo (asserted below).
        inherited
            .split(':')
            .filter(|dir| !dir.contains("cargo") && !dir.contains("rustup"))
            .collect::<Vec<_>>()
            .join(":")
    };
    let out = Command::new("sh")
        .arg(installer)
        .current_dir(&w.dir)
        .env("PATH", path)
        .env("KEEL_REPO", &w.repo)
        .env("KEEL_HOME", &w.home)
        .env("KEEL_BIN", &w.bin)
        .env("KEEL_REF", git_ref)
        .env("KEEL_RELEASES", releases)
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

/// proves: the-installer-takes-the-release-before-the-source@7dd774
#[test]
fn the_installer_takes_the_release_before_the_source() {
    let dir = sandbox("installrelease");
    let w = world(&dir);
    let releases = serve(&dir, "v2.0.0", "2.0.0", false);

    // The PATH without cargo really has none -- or the case below
    // proves nothing.
    let bare: String = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|dir| !dir.contains("cargo") && !dir.contains("rustup"))
        .collect::<Vec<_>>()
        .join(":");
    let out = Command::new("sh")
        .args(["-c", "command -v cargo"])
        .env("PATH", &bare)
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "the bare PATH must not carry cargo, or this probe measures nothing"
    );

    // A version tag with a published release: installed from the
    // archive -- no cargo, no clone, no build -- with its records, and
    // the launcher runs it.
    let (said, code) = install_with(&w, "v2.0.0", &releases, false);
    assert_eq!(code, 0, "the release is taken without cargo:\n{said}");
    assert!(
        said.contains("fetched") && said.contains("v2.0.0") && said.contains("verified"),
        "and the installer says it took the release and checked it:\n{said}"
    );
    let home = w.home.join("versions").join("v2.0.0");
    assert!(
        home.join("keel").is_file(),
        "the binary stands in its home:\n{said}"
    );
    assert_eq!(
        fs::read_to_string(home.join(".keel-version"))
            .unwrap()
            .trim(),
        "2.0.0"
    );
    assert_eq!(
        fs::read_to_string(home.join(".keel-ref")).unwrap().trim(),
        "v2.0.0"
    );
    let sum = fs::read_to_string(home.join(".keel-sum")).unwrap();
    assert_eq!(
        sum.trim().len(),
        64,
        "the binary's sha256 is recorded: {sum}"
    );
    assert!(
        !w.home.join("source").exists(),
        "no clone was made for a release that was there:\n{said}"
    );
    let plain = project(&dir, "plain", "2.0.0");
    let tmp = dir.join("tmp");
    fs::create_dir_all(&tmp).unwrap();
    let (said, code) = launch(&w, &plain, &releases, &tmp, &["--version"]);
    assert_eq!(code, 0, "{said}");
    assert!(said.contains("keel 2.0.0"), "the launcher runs it:\n{said}");

    // A ref with no release -- the older tag has no archive on the
    // server -- is built from source, as before the wave, and the
    // installer says why.
    let (said, code) = install_with(&w, &w.old_ref, &releases, true);
    assert_eq!(
        code, 0,
        "a ref without a release still installs from git:\n{said}"
    );
    assert!(
        said.contains("no release") && said.contains("building"),
        "and the road taken is said aloud:\n{said}"
    );
    let old_home = w.home.join("versions").join(&w.old_ref);
    assert_eq!(
        fs::read_to_string(old_home.join(".keel-version"))
            .unwrap()
            .trim(),
        w.old_version,
        "built and recorded as before"
    );
    assert!(
        w.home.join("source").join(".git").is_dir(),
        "through the one clone the git road keeps"
    );

    // A version with no release and no tag of that name -- keel's own
    // pin `0.1.0` is one (review 0048 R-14) -- is built from the
    // branch the remote leads with, and counts only if that branch
    // answers the version: the world's head is moved to answer 3.0.0
    // with no tag, so a pin of 3.0.0 takes that road, and 7.7.7 is
    // refused with the road that works.
    let empty = format!("file://{}", dir.join("empty-releases").display());
    fs::create_dir_all(dir.join("empty-releases")).unwrap();
    fs::write(
        w.repo.join("tool/Cargo.toml"),
        "[package]\nname = \"keel\"\nversion = \"3.0.0\"\n",
    )
    .unwrap();
    common::versions::git(&w.repo, &["add", "-A"]);
    common::versions::git(&w.repo, &["commit", "-q", "-m", "three, untagged"]);
    let (said, code) = install_with(&w, "3.0.0", &empty, true);
    assert_eq!(
        code, 0,
        "the lead branch answers 3.0.0, so 3.0.0 installs from it:\n{said}"
    );
    assert!(
        said.contains("branch the remote leads with") && said.contains("3.0.0"),
        "and the road is said aloud:\n{said}"
    );
    let home = w.home.join("versions").join("v3.0.0");
    assert_eq!(
        fs::read_to_string(home.join(".keel-version"))
            .unwrap()
            .trim(),
        "3.0.0"
    );
    assert_eq!(
        fs::read_to_string(home.join(".keel-ref")).unwrap().trim(),
        "v3.0.0"
    );
    let (said, code) = install_with(&w, "7.7.7", &empty, true);
    assert_ne!(code, 0, "a version nothing answers to is refused:\n{said}");
    assert!(
        said.contains("answers keel 3.0.0, not 7.7.7") && said.contains("pin a ref"),
        "with what the branch answers and what would work:\n{said}"
    );
    assert!(
        !w.home.join("versions").join("v7.7.7").exists(),
        "and no home is left"
    );
}
