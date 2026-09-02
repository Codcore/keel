//! Scenario tests of wave 0012-loop-stages, transform step-hand:
//! `keel next` hands exactly one self-sufficient step -- from the
//! birth of a test through the transform package to the review and
//! the PR -- and, off a wave branch, the readiness overview.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0012n-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    dir
}

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

fn keel(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// A crate with git on the given branch and the cargo adapter.
fn project(name: &str, branch: &str) -> PathBuf {
    let dir = sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    git(&dir, &["init", "-q", "-b", branch]);
    dir
}

fn commit_all(dir: &Path) {
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "state"]);
}

/// proves: next-hands-one-step@82fe98 -- holds §9.2/§9.10/§8.4: on a
/// wave branch every run hands exactly one step and the package is
/// self-sufficient -- the birth of a test with the scenario body
/// verbatim, revision and tag shape; the transform with its files,
/// section body and commit grammar; then "time for the review" and
/// "time for the PR"; off a wave branch, ready waves by branch name
/// and the honest all-closed word.
#[test]
fn next_hands_one_step() {
    let dir = project("stages", "0070-w");
    write(
        &dir,
        "keel/waves/0070-w.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t\n",
    );
    commit_all(&dir);

    // Stage one: the scenario has no tag -- the step is the birth of
    // its test, with the body verbatim and the red grammar.
    let s_rev = keel::rev::text_rev("body of s\n");
    let (out, err, code) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a step is an answer, not a finding:\n{out}");
    assert!(
        out.contains("write the test") && out.contains("red: s"),
        "the first step is the birth of the test (§7.12/§8.4):\n{out}"
    );
    assert!(
        out.contains("body of s") && out.contains(&s_rev),
        "the package carries the scenario body verbatim and its revision (§9.10):\n{out}"
    );
    assert!(
        out.contains(&format!("proves: s@{s_rev}")),
        "the tag shape rides in the package:\n{out}"
    );

    // Stage two: the tag is on -- the step is the transform, its
    // package self-sufficient.
    write(
        &dir,
        "tests/s_test.rs",
        &format!("/// proves: s@{s_rev}\n#[test]\nfn holds_s() {{}}\n"),
    );
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("transform \"t\"") && out.contains("src/lib.rs"),
        "the second step is the transform with its files:\n{out}"
    );
    assert!(
        out.contains("the work of t"),
        "the transform's own section body rides in the package:\n{out}"
    );
    assert!(
        out.contains("t: "),
        "the commit grammar of §8.4 is spelled out:\n{out}"
    );

    // Stage three: the files are touched, the review is missing.
    write(&dir, "src/lib.rs", "pub fn grown() {}\n");
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("time for the review") && out.contains("keel review"),
        "the assembled wave asks for its reviewer (§9.9):\n{out}"
    );

    // Stage four: the review lies next to the wave -- time for the PR.
    write(&dir, "keel/reviews/0070-w.md", "# Рецензія\n\nok\n");
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("time for the PR"),
        "the reviewed wave heads to its merge (§8.7):\n{out}"
    );

    // Off the wave branch with everything closed: the honest word is
    // to plan a new wave.
    git(&dir, &["checkout", "-q", "-b", "elsewhere"]);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("plan a new wave"),
        "all closed and nothing ready -- planning is the step:\n{out}"
    );

    // A ready plan off the wave branch is named with its branch.
    let dir = project("ready", "main");
    write(
        &dir,
        "keel/waves/0080-ready.md",
        "---\nscenarios:\n  r: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [r]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: r\n\nbody of r\n",
    );
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("start the branch \"0080-ready\""),
        "the approved and unstarted wave is the step, by branch name (§8.2):\n{out}"
    );

    // A stale tag is a step of its own: the recorded and the current
    // revision by name.
    let dir = project("stale", "0090-x");
    write(
        &dir,
        "keel/waves/0090-x.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t\n",
    );
    write(
        &dir,
        "tests/s_test.rs",
        "/// proves: s@abcdef\n#[test]\nfn holds_s() {}\n",
    );
    commit_all(&dir);
    let s_rev = keel::rev::text_rev("body of s\n");
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("drifted") && out.contains("abcdef") && out.contains(&s_rev),
        "the drifted revision is a step with both revisions by name (§5.5):\n{out}"
    );
}
