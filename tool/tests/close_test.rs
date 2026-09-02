//! Scenario tests of wave 0006-wave-closure, transform close-command:
//! the closure of waves judged by consequences (§6.5, journal A2) --
//! sandboxes are real projects with git, cargo and reviews, and the
//! court runs through the binary.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0006c-{}-{name}", std::process::id()));
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

/// A wave file with one live scenario named `sc` and one transform.
fn wave_text(sc: &str) -> String {
    format!(
        "---\nscenarios:\n  {sc}: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [{sc}]\n    files: [src/lib.rs]\n{}---\n\n## scenario: {sc}\n\nbody of {sc}\n",
        all_decided_except(&["functional.correctness"])
    )
}

/// The base of every closure sandbox: a crate, a git repo on the
/// given branch, keel.toml with the cargo adapter.
fn project(name: &str, branch: &str) -> PathBuf {
    let dir = sandbox(name);
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
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

/// proves: wave-closure-judged@ecaf9f -- holds §6.5/§6.3 and journal
/// A2: every wave gets one of three states -- closed, approved and
/// not started (not red), or in progress with the missing named --
/// and only the branch's own wave blocks the exit.
#[test]
fn wave_closure_judged() {
    let dir = project("threestates", "0012-progress");
    // 0010-closed: the live scenario is proven and the review lies
    // next to the wave.
    write(&dir, "keel/waves/0010-closed.md", &wave_text("a"));
    let a_rev = keel::rev::text_rev("body of a\n");
    write(
        &dir,
        "tests/a_test.rs",
        &format!("/// proves: a@{a_rev}\n#[test]\nfn holds_a() {{}}\n"),
    );
    write(&dir, "keel/reviews/0010-closed.md", "# Рецензія\n\nok\n");
    // 0011-plan: a plan without a single tag.
    write(&dir, "keel/waves/0011-plan.md", &wave_text("b"));
    // 0012-progress: two live scenarios, one proven, one untagged.
    write(
        &dir,
        "keel/waves/0012-progress.md",
        &format!(
            "---\nscenarios:\n  c: {{covers: [functional.correctness]}}\n  d: {{covers: [performance.capacity]}}\ntransforms:\n  t:\n    implements: [c, d]\n    files: [src/lib.rs]\n{}---\n\n## scenario: c\n\nbody of c\n\n## scenario: d\n\nbody of d\n",
            all_decided_except(&["functional.correctness", "performance.capacity"])
        ),
    );
    let c_rev = keel::rev::text_rev("body of c\n");
    write(
        &dir,
        "tests/c_test.rs",
        &format!("/// proves: c@{c_rev}\n#[test]\nfn holds_c() {{}}\n"),
    );
    write(&dir, "keel/reviews/0012-progress.md", "# Рецензія\n\nok\n");
    commit_all(&dir);

    // The branch is named as the unclosed wave -- its lacks block.
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "the branch's own wave blocks the exit:\n{out}");
    assert!(
        out.contains("0010-closed: closed"),
        "the proven wave is closed:\n{out}"
    );
    assert!(
        out.contains("approved, not started") && out.contains("0011-plan"),
        "a plan without tests is not red (§6.5):\n{out}"
    );
    assert!(
        out.contains("in progress") && out.contains("\"d\""),
        "the missing scenario named:\n{out}"
    );
    assert!(
        out.contains("no proves tag"),
        "the lack named by its kind:\n{out}"
    );

    // The same project from a branch named as no wave: the same
    // states, nothing blocks.
    git(&dir, &["checkout", "-q", "-b", "just-work"]);
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "other waves inform, they do not punish:\n{out}");
    assert!(
        out.contains("in progress"),
        "the states still told:\n{out}"
    );
}
