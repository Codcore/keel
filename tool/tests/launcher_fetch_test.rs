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
use common::versions::{World, install, world};
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

fn host() -> String {
    let out = Command::new("rustc").arg("-vV").output().unwrap();
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
        .expect("rustc names a host")
}

/// A release server: `<dir>/<tag>/keel-<version>-<host>.tar.gz` and its
/// `.sha256`, built the way release.sh builds them, holding a `keel`
/// that answers `keel <version>`. `tamper` writes a checksum that is
/// not the archive's.
fn serve(dir: &Path, tag: &str, version: &str, tamper: bool) -> String {
    let home = dir.join("releases").join(tag);
    fs::create_dir_all(&home).unwrap();
    let build = dir.join(format!("build-{tag}"));
    fs::create_dir_all(&build).unwrap();
    fs::write(
        build.join("keel"),
        format!("#!/bin/sh\necho \"keel {version}\"\necho \"args: $*\"\n"),
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

    // No release for the pin: a refusal with the ready command, as
    // before -- never another version run in silence.
    let missing = project(&dir, "missing", "4.0.0");
    let (said, code) = launch(&w, &missing, &releases, &tmp, &["--version"]);
    assert_eq!(code, 2, "a pin with no release refuses:\n{said}");
    assert!(
        said.contains("no release") && said.contains("v4.0.0") && said.contains("KEEL_REF="),
        "naming what was not there and the command that installs by hand:\n{said}"
    );
    assert!(!said.contains("keel 1.0.0"), "{said}");

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
