//! Scenario test of wave 0068: the closing court is not narrower than
//! the check.
//!
//! Measured before the work, on one tree and one commit: a branch
//! touching a file no transform of its wave names gives
//! `keel check` exit 1 and `keel close` exit 0. The court that lets a
//! branch into a merge was blind to what the cheaper court beside it
//! had already found -- and it spent thirteen minutes of battery
//! before saying so.

mod common;

use common::{Sandbox, keel_sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
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

fn keel(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
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

fn all_decided() -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        block.push_str(&format!("  {cut}: \"n/a, бо ця пісочниця грає інше\"\n"));
    }
    block
}

/// A crate with git, the cargo adapter and one chore wave, standing on
/// the wave's own branch with a trunk behind it.
fn project(name: &str) -> Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "pub fn a() {}\n");
    write(
        &dir,
        "keel/waves/0001-a-chore.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
            all_decided()
        ),
    );
    write(&dir, "keel/reviews/0001-a-chore.md", "# Рецензія\n\nok\n");
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-chore"]);
    dir
}

/// proves: the-cheap-court-runs-first-and-locally@242d9a
#[test]
fn the_cheap_court_runs_first_and_locally() {
    // --- a finding of `keel check` is a blocker of `keel close` ---
    let dir = project("closesees");
    write(&dir, "src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    // A file no transform of the wave names: §4.6 drift.
    write(&dir, "build.rs", "fn main() {}\n");
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "tidy: the work and a stranger"],
    );

    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 1,
        "the fixture must really be red for check, or this probe \
         proves nothing:\n{checked}"
    );
    assert!(
        checked.contains("build.rs"),
        "and red for the drifted file by name:\n{checked}"
    );

    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_ne!(
        close_code, 0,
        "the court that lets a branch into a merge is not narrower \
         than the one beside it: what `keel check` calls a finding, \
         `keel close` calls a blocker:\n{closed}"
    );
    assert!(
        closed.contains("build.rs"),
        "and it names the same file, not a number:\n{closed}"
    );

    // --- and it stops BEFORE the battery ---
    //
    // A probe that only reads the exit code would pass over a court
    // that ran thirteen minutes of tests and only then looked. The
    // whole point of the wave is the six seconds.
    assert!(
        !closed.contains("battery:"),
        "the documents are judged first: the battery does not run at \
         all when the cheap court has already found something:\n{closed}"
    );

    // --- where check is silent, close reddens nothing extra ---
    let dir = project("closequiet");
    write(&dir, "src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "tidy: only what was named"]);
    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 0,
        "the quiet side is quiet for check:\n{checked}"
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(
        close_code, 0,
        "and close adds no redness of its own where check is \
         silent:\n{closed}"
    );
    assert!(
        closed.contains("battery:"),
        "and here the battery does run -- the wave does not make the \
         court cheaper by skipping it:\n{closed}"
    );
}
