//! Scenario test of wave 0072: the trunk is the one git names.
//!
//! Issue #51, from a live project on keel 1.3.0 that moved to
//! `development` as its default branch: `keel check` reddened every
//! plan branch on §4.9, naming files the branch never touched --
//! exactly the files the default branch has and `main` does not.
//!
//! git already knew the answer. `scope::trunk` asked it LAST: it
//! walked `main`, `master`, `origin/main`, `origin/master` first, so
//! in any repository where one of those resolves, `refs/…/HEAD` was
//! never asked at all.

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

/// A project whose default branch is NOT `main`, arranged the way
/// issue #51 describes: the trunk carries a file `main` does not, and
/// the plan branch is cut from the trunk, touching only `keel/`.
///
/// The remote is a real bare repository, because `refs/…/HEAD` is a
/// remote-tracking symref and a sandbox that fakes it would be
/// measuring the fixture, not the tool.
fn project(name: &str, default_branch: &str, remote: &str) -> Sandbox {
    let home = keel_sandbox(name);
    let bare = home.join("origin.git");
    let work = home.join("work");
    fs::create_dir_all(&bare).unwrap();
    fs::create_dir_all(&work).unwrap();

    write(&work, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(
        &work,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&work, "src/lib.rs", "pub fn a() {}\n");
    fs::create_dir_all(work.join("keel/contracts")).unwrap();

    git(&work, &["init", "-q", "-b", "main"]);
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "base"]);

    // The trunk goes ahead of `main` by one file -- the shape of a
    // project that promotes `main` from its default branch on purpose.
    git(&work, &["checkout", "-q", "-b", default_branch]);
    write(&work, "DESIGN.md", "what the default branch carries\n");
    git(&work, &["add", "-A"]);
    git(
        &work,
        &["commit", "-q", "-m", "the default branch moves on"],
    );

    Command::new("git")
        .args(["init", "--bare", "-q"])
        .arg(&bare)
        .output()
        .unwrap();
    git(&work, &["remote", "add", remote, bare.to_str().unwrap()]);
    git(&work, &["push", "-q", remote, "main", default_branch]);
    git(
        &work,
        &[
            "symbolic-ref",
            &format!("refs/remotes/{remote}/HEAD"),
            &format!("refs/remotes/{remote}/{default_branch}"),
        ],
    );

    // A plan branch cut from the trunk, carrying the plan and nothing
    // else.
    git(&work, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    write(
        &work,
        "keel/waves/0001-a-wave.md",
        &format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: work\nтіло\n",
            all_decided()
        ),
    );
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "the plan"]);
    home
}

/// proves: the-trunk-is-the-one-git-names@dcb236
#[test]
fn the_trunk_is_the_one_git_names() {
    // --- issue #51: the trunk is what git names, not what sorts
    // first in a list of names ---
    let home = project("trunkdefault", "development", "origin");
    let work = home.join("work");
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("DESIGN.md"),
        "the plan branch is judged against the DEFAULT branch, not \
         against main: DESIGN.md belongs to the trunk and this branch \
         never touched it\n{out}"
    );
    assert_eq!(code, 0, "and nothing else reddens either:\n{out}");

    // --- the remote is not spelled `origin`: review 0031 R-8 put
    // that into `check`, and merging the hands must not lose it ---
    let home = project("trunkupstream", "development", "upstream");
    let work = home.join("work");
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("DESIGN.md"),
        "and the remote may be called anything -- a clone pushed to \
         `upstream` gets the same answer:\n{out}"
    );

    // --- a project whose default branch IS main sees no change:
    // this is the population the wave must leave alone, and the local
    // ref must win over the remote-tracking one ---
    let home = project("trunkmain", "feature-x", "origin");
    let work = home.join("work");
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    // A commit on local main that was never pushed: with `origin/main`
    // as the trunk this file would look like the branch's own.
    git(&work, &["checkout", "-q", "main"]);
    write(&work, "LOCAL.md", "committed locally, never pushed\n");
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "local only"]);
    git(&work, &["checkout", "-q", "plan/0001-a-wave"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("LOCAL.md"),
        "the LOCAL main is the trunk when it exists -- resolving the \
         remote HEAD to a NAME and looking for the local ref first is \
         what keeps this population still:\n{out}"
    );
}
