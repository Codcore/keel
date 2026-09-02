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
    assert!(out.contains("in progress"), "the states still told:\n{out}");

    // Second birth (review R-2): the branch named as a wave still in
    // its plan state -- a plan PR merges as a plan (§6.6), so the
    // exit stays 0, but the footer must say that truth, not "named
    // as no unclosed wave".
    git(&dir, &["checkout", "-q", "-b", "0011-plan"]);
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a plan PR merges as a plan (§6.6):\n{out}");
    assert!(
        out.contains("merges as a plan"),
        "the footer tells the plan truth, not a wrong word:\n{out}"
    );
    assert!(
        !out.contains("named as no unclosed wave"),
        "the old footer would lie here:\n{out}"
    );

    // Second birth (review R-3): a scenario namesake in two waves,
    // each with its own matching tag and green test -- both waves
    // close; a tag whose revision belongs to the other wave is not
    // this wave's lack.
    let dir = project("namesakes", "just-work");
    write(&dir, "keel/waves/0020-first.md", &wave_text("s"));
    write(
        &dir,
        "keel/waves/0021-second.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nanother body of s\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let first_rev = keel::rev::text_rev("body of s\n");
    let second_rev = keel::rev::text_rev("another body of s\n");
    write(
        &dir,
        "tests/first_test.rs",
        &format!("/// proves: s@{first_rev}\n#[test]\nfn holds_s_first() {{}}\n"),
    );
    write(
        &dir,
        "tests/second_test.rs",
        &format!("/// proves: s@{second_rev}\n#[test]\nfn holds_s_second() {{}}\n"),
    );
    write(&dir, "keel/reviews/0020-first.md", "# Рецензія\n\nok\n");
    write(&dir, "keel/reviews/0021-second.md", "# Рецензія\n\nok\n");
    commit_all(&dir);
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "namesakes do not deadlock each other:\n{out}");
    assert!(
        out.contains("0020-first: closed") && out.contains("0021-second: closed"),
        "each wave closed by its own revision (R-3):\n{out}"
    );

    // Second birth (review R-4): where history cannot testify, the
    // closed line must not claim the references converged -- the
    // unjudged are named, green is not painted over them.
    let dir = project("unjudgedrefs", "just-work");
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: A\nexports: [\"one()\"]\n---\n\nwords\n",
    );
    write(
        &dir,
        "keel/waves/0022-w.md",
        &format!(
            "---\nscenarios:\n  a:\n    proves: anchor@bbbbbb\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [a]\n    files: [src/lib.rs]\n{}---\n\n## scenario: a\n\nbody of a\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let a_rev = keel::rev::text_rev("body of a\n");
    write(
        &dir,
        "tests/a_test.rs",
        &format!("/// proves: a@{a_rev}\n#[test]\nfn holds_a() {{}}\n"),
    );
    write(&dir, "keel/reviews/0022-w.md", "# Рецензія\n\nok\n");
    // No git at all: the fabricated reference cannot be judged.
    fs::remove_dir_all(dir.join(".git")).unwrap();
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "unjudgeable is not a lack:\n{out}");
    assert!(
        out.contains("not judged"),
        "the closed line does not claim converged references (R-4):\n{out}"
    );
}

/// proves: closure-needs-review-file@d1cb7a -- holds §9.9 as
/// mechanics: a full wave with everything proven but no review file
/// next to it is not closed; with the report it is; a light chore
/// wave closes by the fact of merge and needs no report (§6.5).
#[test]
fn closure_needs_review_file() {
    let dir = project("reviewgate", "just-work");
    write(&dir, "keel/waves/0010-full.md", &wave_text("a"));
    let a_rev = keel::rev::text_rev("body of a\n");
    write(
        &dir,
        "tests/a_test.rs",
        &format!("/// proves: a@{a_rev}\n#[test]\nfn holds_a() {{}}\n"),
    );
    // A light chore wave, no report anywhere near it.
    write(
        &dir,
        "keel/waves/0013-tidy.md",
        &format!(
            "---\ntransforms:\n  tidy: {{chore: \"lad\", files: [README.md]}}\n{}---\n",
            all_decided_except(&[])
        ),
    );
    commit_all(&dir);

    // Everything proven, yet the review file is missing.
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "no blocker off the wave branch:\n{out}");
    assert!(
        out.contains("in progress") && out.contains("review"),
        "a full wave without its review is not closed (§9.9):\n{out}"
    );
    assert!(
        out.contains("0013-tidy: closed"),
        "a chore wave closes by the fact of merge:\n{out}"
    );

    // The report lands next to the wave -- closed.
    write(&dir, "keel/reviews/0010-full.md", "# Рецензія\n\nok\n");
    let (out, err, _code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("0010-full: closed"),
        "with the report the wave closes:\n{out}"
    );

    // Second birth (review R-5): light is §6.8's word, not "chores
    // only" -- one transform, nothing withdrawn, no contracts. Two
    // chore transforms make a full wave that wants its review; a
    // chore wave that withdraws a scenario is full too: the death of
    // a promise gets two human looks.
    write(
        &dir,
        "keel/waves/0014-two-chores.md",
        &format!(
            "---\ntransforms:\n  one: {{chore: \"lad\", files: [README.md]}}\n  two: {{chore: \"dust\", files: [README.md]}}\n{}---\n",
            all_decided_except(&[])
        ),
    );
    write(
        &dir,
        "keel/waves/0015-withdrawing.md",
        &format!(
            "---\nscenarios:\n  gone:\n    covers: [performance.capacity]\n    withdrawn: \"folded\"\ntransforms:\n  tidy: {{chore: \"lad\", files: [README.md]}}\n{}---\n\n## scenario: gone\n\nold body\n",
            all_decided_except(&[])
        ),
    );
    let (out, err, _code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("0014-two-chores: in progress"),
        "two chores are not light (§6.8) -- the review is wanted:\n{out}"
    );
    assert!(
        out.contains("0015-withdrawing: in progress"),
        "withdrawing makes a wave full (§6.8):\n{out}"
    );
    assert!(
        out.contains("0013-tidy: closed"),
        "the one-chore wave stays light:\n{out}"
    );
}

/// proves: battery-read-once@5643fa -- holds journal A3 and the
/// adapter contract: one cargo run gives every tag its own verdict;
/// a build that does not build is a refusal aloud with the
/// compiler's words -- no verdict for anyone.
#[test]
fn battery_read_once() {
    // Two tagged tests, one green and one red, judged from one run.
    let dir = project("battery", "just-work");
    write(
        &dir,
        "keel/waves/0014-two.md",
        &format!(
            "---\nscenarios:\n  green-one: {{covers: [functional.correctness]}}\n  red-one: {{covers: [performance.capacity]}}\ntransforms:\n  t:\n    implements: [green-one, red-one]\n    files: [src/lib.rs]\n{}---\n\n## scenario: green-one\n\nbody of green-one\n\n## scenario: red-one\n\nbody of red-one\n",
            all_decided_except(&["functional.correctness", "performance.capacity"])
        ),
    );
    let g = keel::rev::text_rev("body of green-one\n");
    let r = keel::rev::text_rev("body of red-one\n");
    write(
        &dir,
        "tests/two_test.rs",
        &format!(
            "/// proves: green-one@{g}\n#[test]\nfn holds_green() {{}}\n\n/// proves: red-one@{r}\n#[test]\nfn holds_red() {{ assert!(false); }}\n"
        ),
    );
    write(&dir, "keel/reviews/0014-two.md", "# Рецензія\n\nok\n");
    commit_all(&dir);

    // The library call itself: the whole battery, one map of verdicts.
    let verdicts = keel::adapter::run_all(&dir).unwrap();
    assert_eq!(
        verdicts.get(&("two_test".to_string(), "holds_green".to_string())),
        Some(&true),
        "the green test is green in the one run"
    );
    assert_eq!(
        verdicts.get(&("two_test".to_string(), "holds_red".to_string())),
        Some(&false),
        "the red test is red in the one run"
    );

    // Through the court: both verdicts land, each named.
    let (out, err, _code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("in progress") && out.contains("\"red-one\"") && out.contains("is red"),
        "the red scenario named with its kind:\n{out}"
    );

    // A build that does not build: a refusal aloud, no verdicts.
    write(
        &dir,
        "tests/broken_test.rs",
        "fn broken() { let x: i32 = \"no\"; }\n",
    );
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "no build -- no verdict for anyone:\n{out}");
    assert!(
        out.contains("error"),
        "the compiler's words carried:\n{out}"
    );

    // Second birth (review R-1): a target with harness = false prints
    // a "Running" line but no verdict block -- the stitch would shift
    // every later verdict onto the wrong stem, up to blessing a wave
    // with a red tagged test. The court refuses aloud instead of
    // judging by a seam that does not meet.
    let dir = project("harnessless", "just-work");
    write(&dir, "keel/waves/0014-two.md", &wave_text("a"));
    let a_rev = keel::rev::text_rev("body of a\n");
    write(
        &dir,
        "tests/a_test.rs",
        &format!("/// proves: a@{a_rev}\n#[test]\nfn holds_a() {{ assert!(false); }}\n"),
    );
    write(&dir, "keel/reviews/0014-two.md", "# Рецензія\n\nok\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n\n[[test]]\nname = \"a_test\"\n\n[[test]]\nname = \"bench_like\"\nharness = false\n",
    );
    write(&dir, "tests/bench_like.rs", "fn main() {}\n");
    commit_all(&dir);

    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 2,
        "a stitch that does not meet is a refusal, not a guess:\n{out}"
    );
    assert!(
        out.contains("does not meet"),
        "the mismatch named aloud:\n{out}"
    );
}

/// proves: verify-run-by-close@512a7f -- holds §7.6/§2.8: close
/// runs the verify of live contracts under the §7.16 trust court --
/// a trusted passing command is counted "passed" aloud; a trusted
/// failing one is a blocker carrying the command, because a broken
/// foreign promise does not merge.
#[test]
fn verify_run_by_close() {
    let dir = project("verify-pass", "just-work");
    let fp = keel::trust::fingerprint("true");
    write(
        &dir,
        "keel.toml",
        &format!("adapter = \"cargo\"\n\n[trust]\n\"true\" = \"{fp}\"\n"),
    );
    write(
        &dir,
        "keel/contracts/ext-up.md",
        "---\nverify: \"true\"\n---\n\nA foreign promise that stands (§2.8).\n",
    );
    commit_all(&dir);
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a passing verify closes nothing red:\n{out}");
    assert!(
        out.contains("verify commands judged: 1"),
        "the verify count is aloud:\n{out}"
    );
    assert!(
        out.contains("passed"),
        "the passing command is counted passed:\n{out}"
    );

    let dir = project("verify-fail", "just-work");
    let fp = keel::trust::fingerprint("false");
    write(
        &dir,
        "keel.toml",
        &format!("adapter = \"cargo\"\n\n[trust]\n\"false\" = \"{fp}\"\n"),
    );
    write(
        &dir,
        "keel/contracts/ext-down.md",
        "---\nverify: \"false\"\n---\n\nA foreign promise that broke (§2.8).\n",
    );
    commit_all(&dir);
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "a broken foreign promise does not merge:\n{out}");
    assert!(
        out.contains("\"false\"") && out.contains("FAILED"),
        "the failing verify named with its command:\n{out}"
    );
}
