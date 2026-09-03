//! Scenario tests of wave 0021-git-speaks-to-the-project, transform
//! one-git-hand: every court judges the project it was pointed at,
//! never the repository that spawned it.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// What a git hook hands its children -- the probe's own sandboxes
/// must be as deaf to it as the tool is (school 0020).
const GIT_ENV: [&str; 10] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
    "GIT_CEILING_DIRECTORIES",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
];

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0021g-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    fs::create_dir_all(dir.join("keel/reviews")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    for name in GIT_ENV {
        command.env_remove(name);
    }
    let out = command
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// keel run exactly as a git hook would leave the environment: the
/// foreign repository named in every addressing variable.
fn keel_from_hook(args: &[&str], foreign: &Path) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .env("GIT_DIR", foreign.join(".git"))
        .env("GIT_WORK_TREE", foreign)
        .env("GIT_PREFIX", "")
        .env("GIT_INDEX_FILE", foreign.join(".git/index"))
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

/// The judged project: a crate, a git repository on the wave's own
/// branch, and one wave whose scenario carries no tag at all -- so
/// the closure court must stay red.
fn project(name: &str) -> PathBuf {
    let dir = sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    write(&dir, "tests/steady_test.rs", "#[test]\nfn steady() {}\n");
    write(
        &dir,
        "keel/waves/0030-unproven.md",
        &format!(
            "---\nscenarios:\n  never-proven: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [never-proven]\n    files: [src/lib.rs]\n{}---\n\n## Why\n\nwhy words\n\n## scenario: never-proven\n\nbody of never-proven\n\n## transform: t\n\nthe work of t\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    git(&dir, &["init", "-q", "-b", "0030-unproven"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "state"]);
    dir
}

/// A stranger's repository, whole and quiet: whatever the courts do,
/// not one byte of it may change.
fn foreign(name: &str) -> PathBuf {
    let dir = sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(&dir, "README.md", "the stranger\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the stranger's own state"]);
    dir
}

/// Every file of a tree with its bytes -- the honest way to say
/// "not one byte changed".
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = fs::read(&path) {
                out.push((
                    path.strip_prefix(root).unwrap().display().to_string(),
                    bytes,
                ));
            }
        }
    }
    out.sort();
    out
}

/// proves: courts-deaf-to-the-environment@f930de -- holds what the
/// 0020 review measured and the wave repairs: a git hook hands its
/// children GIT_DIR and its kin, those outrank -C, and a court that
/// inherits them judges the repository that spawned it. Under that
/// environment every court must still judge the project it was
/// given -- above all the closure court, which greened on an
/// unproven wave -- and the stranger's repository must not change
/// by a byte.
#[test]
fn courts_deaf_to_the_environment() {
    let dir = project("judged");
    let stranger = foreign("stranger");
    let before = snapshot(&stranger);

    // The closure court: red on this project's unproven wave, and
    // never green because a stranger's repository looked tidy.
    let (out, code) = keel_from_hook(&["close", dir.to_str().unwrap()], &stranger);
    assert_eq!(
        code, 1,
        "close stays red on the project's own unproven wave:\n{out}"
    );
    assert!(
        out.contains("0030-unproven") && out.contains("in progress"),
        "close names this project's wave and its lack:\n{out}"
    );

    // The form court: this project's own findings, none of the
    // stranger's documents.
    let (out, _) = keel_from_hook(&["check", dir.to_str().unwrap()], &stranger);
    assert!(
        out.contains("0030-unproven"),
        "check reads this project's documents:\n{out}"
    );
    assert!(
        !out.contains("the stranger"),
        "nothing of the stranger's tree reaches the verdict:\n{out}"
    );

    // The commit judgement: the branch of this project, by name.
    write(&dir, ".git/COMMIT_MSG_PROBE", "red: never-proven\n");
    let msg = dir.join(".git/COMMIT_MSG_PROBE");
    let (out, _) = keel_from_hook(
        &["gate", msg.to_str().unwrap(), dir.to_str().unwrap()],
        &stranger,
    );
    assert!(
        out.contains("never-proven") && !out.contains("is named as no wave"),
        "the gate keeps its subject: this project's branch and scenario:\n{out}"
    );

    // The reviewer's package: built for this project's wave.
    let (out, code) = keel_from_hook(&["review", dir.to_str().unwrap()], &stranger);
    assert_eq!(code, 0, "review builds the package here:\n{out}");
    assert!(
        out.contains("0030-unproven"),
        "the package is this project's wave:\n{out}"
    );

    // The step hand and the stage eye speak of this project too.
    let (out, _) = keel_from_hook(&["next", dir.to_str().unwrap()], &stranger);
    assert!(
        out.contains("0030-unproven"),
        "next speaks of this project's wave:\n{out}"
    );

    // The planning hand counts this project's numbers: 0030 is
    // taken here, whatever the stranger holds.
    let (out, code) = keel_from_hook(&["plan", "0030-twin", dir.to_str().unwrap()], &stranger);
    assert_eq!(
        code, 2,
        "plan refuses a number this project already holds:\n{out}"
    );

    // And the stranger is exactly as it was -- not one byte.
    assert_eq!(
        snapshot(&stranger),
        before,
        "no court writes into the repository of the environment"
    );
}
