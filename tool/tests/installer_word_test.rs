//! Scenario test of wave 0055: the installer takes what it promises.
//!
//! The hand is a shell script, so the probe RUNS it against a real
//! git repository; only `cargo` is a stub (see `common::versions`).
//!
//! Measured on f744252 (final review 2026-09-06, bugs R-5, R-10,
//! R-11; tests R-4): `KEEL_REF=<branch>` -- the very form the launcher
//! and `keel version` advise -- was refused as "no such version",
//! because a fresh clone knows a remote branch only as
//! `origin/<name>`; `KEEL_REPO` was ignored in silence once a source
//! clone stood; the launcher set `KEEL_HOME` without exporting it,
//! so the binary read `~/.keel` and `keel version` found no version
//! from inside its own home; and two promises of the launcher's
//! contract -- a missing `.keel-current` and a stranger's `keel` on
//! PATH -- were held by hand alone.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;
use common::versions::{World, git, install, no_releases, run_in, world};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn installer() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("install.sh")
}

fn stub_path(w: &World) -> String {
    format!(
        "{}:{}",
        w.stub.display(),
        std::env::var("PATH").unwrap_or_default()
    )
}

/// The real install.sh, with a repository of the probe's choosing.
fn install_from(w: &World, repo: &Path, git_ref: &str) -> (String, i32) {
    let out = Command::new("sh")
        .arg(installer())
        .current_dir(&w.dir)
        .env("PATH", stub_path(w))
        .env("KEEL_REPO", repo)
        .env("KEEL_HOME", &w.home)
        .env("KEEL_BIN", &w.bin)
        .env("KEEL_REF", git_ref)
        .env("KEEL_RELEASES", no_releases(w))
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

fn head_of(w: &World) -> String {
    git(&w.home.join("source"), &["rev-parse", "HEAD"])
}

/// proves: the-installer-takes-what-it-promises@73b251 -- the advice
/// `KEEL_REF="<ref>" sh install.sh` named a hand that refused a
/// branch; `KEEL_REPO` was a word without a hand once `source`
/// stood; and the launcher's own home was a word its binary never
/// heard.
#[test]
fn the_installer_takes_what_it_promises() {
    let dir = sandbox("installerword");
    let w = world(&dir);

    // -- a branch of the remote, by its name ---------------------------
    // Not checked out in the remote, so a clone knows it only as
    // `origin/feature`.
    git(&w.repo, &["checkout", "-q", "-b", "feature"]);
    fs::write(w.repo.join("tool/feature.txt"), "on the branch\n").unwrap();
    git(&w.repo, &["add", "-A"]);
    git(&w.repo, &["commit", "-q", "-m", "the branch"]);
    let feature_tip = git(&w.repo, &["rev-parse", "HEAD"]);
    git(&w.repo, &["checkout", "-q", "main"]);
    let (said, code) = install(&w, Some("feature"));
    assert_eq!(
        code, 0,
        "a branch of the remote installs by its name -- the very form \
         the launcher and `keel version` advise:\n{said}"
    );
    assert_eq!(
        head_of(&w),
        feature_tip,
        "and the tree built is the branch's tip, not main's:\n{said}"
    );
    assert_eq!(
        fs::read_to_string(w.home.join("versions/feature/.keel-ref"))
            .unwrap_or_default()
            .trim(),
        "feature",
        "under a home named by the ref that was asked for:\n{said}"
    );

    // -- KEEL_REPO names the source, standing clone or not ------------
    let other = dir.join("other");
    git(&dir, &["clone", "-q", w.repo.to_str().unwrap(), "other"]);
    fs::write(other.join("tool/other.txt"), "from the other repository\n").unwrap();
    git(&other, &["add", "-A"]);
    git(&other, &["commit", "-q", "-m", "the other tip"]);
    let other_tip = git(&other, &["rev-parse", "HEAD"]);
    let (said, code) = install_from(&w, &other, "main");
    assert_eq!(code, 0, "a second repository, named, installs:\n{said}");
    assert_eq!(
        head_of(&w),
        other_tip,
        "and the tree built comes from the repository that was named, \
         though a clone of the first one stood in `source` -- a name \
         ignored in silence under an \"installed\" is the bug this \
         scenario exists for:\n{said}"
    );
    assert!(
        said.contains(other.to_str().unwrap()),
        "and the change of source is said aloud, by the address:\n{said}"
    );

    // -- the launcher exports its home --------------------------------
    // No KEEL_HOME in the environment: the launcher bakes in the home
    // it was installed with, and the binary it hands over to must see
    // the same one -- `keel version` reads the versions of its home.
    let project = dir.join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let out = Command::new(w.bin.join("keel"))
        .arg("--version")
        .current_dir(&project)
        .env_remove("KEEL_HOME")
        .env("PATH", stub_path(&w))
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "the launcher runs from its baked-in home:\n{said}"
    );
    assert!(
        said.contains(&format!("home: {}", w.home.display())),
        "and the binary it runs sees that home -- exported, not merely \
         set in the launcher's own shell:\n{said}"
    );

    // -- the contract's promises, held by the probe -------------------
    // No `.keel-current` while versions stand: the refusal says that,
    // not "nothing is installed" (review 0041 R-10).
    fs::remove_file(w.home.join(".keel-current")).unwrap();
    let (said, code) = run_in(&w, &project, &["--version"]);
    assert_eq!(
        code, 2,
        "without a current version the launcher refuses:\n{said}"
    );
    assert!(
        said.contains("no version is marked current"),
        "and says which of the two reasons it is:\n{said}"
    );
    assert!(said.contains("2.0.0"), "naming what stands here:\n{said}");

    // A stranger's `keel` on PATH is replaced, said aloud, and a copy
    // kept (review 0041 R-7).
    let dir = sandbox("foreignkeel");
    let w = world(&dir);
    fs::create_dir_all(&w.bin).unwrap();
    fs::write(w.bin.join("keel"), "#!/bin/sh\necho stranger\n").unwrap();
    let (said, code) = install(&w, None);
    assert_eq!(
        code, 0,
        "the install stands over a stranger's keel:\n{said}"
    );
    assert!(
        said.contains("is not this launcher; replacing it"),
        "and says the stranger was replaced:\n{said}"
    );
    assert_eq!(
        fs::read_to_string(w.bin.join("keel.before-launcher")).unwrap_or_default(),
        "#!/bin/sh\necho stranger\n",
        "with a copy of it kept beside the launcher:\n{said}"
    );
    let launcher = fs::read_to_string(w.bin.join("keel")).unwrap();
    assert!(
        launcher
            .lines()
            .take(2)
            .any(|line| line.contains("keel launcher")),
        "and the launcher stands where the stranger stood:\n{launcher}"
    );
}
