//! Scenario tests of wave 0005-commit-gate, transform gate-command:
//! the commit judged by the machine (journal A3; §7.12, §8.4). The
//! sandboxes are real projects -- a cargo crate, a git branch named
//! as its wave, tagged tests -- and the gate runs through the binary.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0005g-{}-{name}", std::process::id()));
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

/// A real project on a branch named as its wave: keel.toml (the mode
/// line as given), a toy crate, one wave with scenario `s` and
/// transform `t`, and a tagged test whose body is the caller's.
fn project(name: &str, mode_line: &str, test_fn_body: &str) -> PathBuf {
    let dir = sandbox(name);
    write(
        &dir,
        "keel.toml",
        &format!("adapter = \"cargo\"\n{mode_line}"),
    );
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/t_test.rs",
        &format!("/// proves: s@{s_rev}\n#[test]\nfn holds_s() {{ {test_fn_body} }}\n"),
    );
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\n  gone:\n    covers: [performance.capacity]\n    withdrawn: \"folded\"\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody of s\n\n## scenario: gone\n\nold body\n",
            all_decided_except(&["functional.correctness", "performance.capacity"])
        ),
    );
    git(&dir, &["init", "-q", "-b", "0009-w"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the wave rides its branch"]);
    dir
}

fn gate(dir: &Path, message: &str) -> (String, i32) {
    let msg = dir.join("COMMIT_EDITMSG");
    fs::write(&msg, message).unwrap();
    let (out, err, code) = keel(&["gate", msg.to_str().unwrap(), dir.to_str().unwrap()]);
    (format!("{out}{err}"), code)
}

/// proves: red-commit-needs-failing-test@249885 -- holds §7.12/A3: a
/// claimed red birth passes only when the named scenario's test truly
/// fails; a green test is a refusal aloud; a withdrawn scenario is
/// not born; several tags of one scenario at birth refuse.
#[test]
fn red_commit_needs_failing_test() {
    // The test truly fails -- the birth passes.
    let dir = project("redfails", "", "assert!(false, \"born failing\");");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 0, "a failing test earns the red commit:\n{out}");
    assert!(
        out.contains("truly fails"),
        "the pass says what it saw:\n{out}"
    );

    // The test is green -- the birth refuses.
    let dir = project("redgreen", "", "assert!(true);");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 1, "an unearned red does not enter history:\n{out}");
    assert!(
        out.contains("is green"),
        "the refusal names the green test:\n{out}"
    );

    // A withdrawn scenario is not born (§2.12).
    let dir = project("redgone", "", "assert!(false);");
    let (out, code) = gate(&dir, "red: gone");
    assert_eq!(code, 1, "a dead promise is not born:\n{out}");
    assert!(out.contains("withdrawn"), "the refusal says why:\n{out}");

    // Two tags of one scenario at birth -- no guessing which is new.
    let dir = project("redtwotags", "", "assert!(false);");
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/second_test.rs",
        &format!("/// proves: s@{s_rev}\n#[test]\nfn holds_s_again() {{ assert!(false); }}\n"),
    );
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 1, "several tags at birth refuse:\n{out}");
    assert!(out.contains("tags"), "the refusal counts the tags:\n{out}");
}

/// proves: work-commit-needs-green@12c44c -- holds §8.4/§2.4: a work
/// commit named by a transform's slug passes only when every
/// implements scenario has a tagged test with a matching revision
/// and it runs green; a failing test, a stale tag and a missing tag
/// refuse by name; a slug-shaped stranger refuses; merge messages
/// and decision records pass with a word.
#[test]
fn work_commit_needs_green() {
    // Green test, matching tag -- the work passes.
    let dir = project("workgreen", "", "assert!(true);");
    let (out, code) = gate(&dir, "t: does the declared work");
    assert_eq!(code, 0, "green scenarios let the work in:\n{out}");

    // A failing scenario test refuses by name.
    let dir = project("workred", "", "assert!(false);");
    let (out, code) = gate(&dir, "t: does the declared work");
    assert_eq!(code, 1, "red scenarios block the work:\n{out}");
    assert!(out.contains("\"s\""), "the failing scenario named:\n{out}");

    // A stale tag revision refuses -- the record no longer holds.
    let dir = project("workstale", "", "assert!(true);");
    write(
        &dir,
        "tests/t_test.rs",
        "/// proves: s@aaaaaa\n#[test]\nfn holds_s() { assert!(true); }\n",
    );
    let (out, code) = gate(&dir, "t: does the declared work");
    assert_eq!(code, 1, "a stale tag blocks the work:\n{out}");
    assert!(out.contains("aaaaaa"), "the stale record named:\n{out}");

    // No tag at all -- the scenario is not held by any test.
    let dir = project("workuntagged", "", "assert!(true);");
    write(&dir, "tests/t_test.rs", "#[test]\nfn plain() {}\n");
    let (out, code) = gate(&dir, "t: does the declared work");
    assert_eq!(code, 1, "an untagged scenario blocks the work:\n{out}");
    assert!(
        out.contains("no proves tag"),
        "the missing tag named:\n{out}"
    );

    // A slug-shaped stranger is a typo, not \"outside the judgement\".
    let dir = project("workstranger", "", "assert!(true);");
    let (out, code) = gate(&dir, "typo-slug: something");
    assert_eq!(code, 1, "a stranger slug refuses:\n{out}");
    assert!(
        out.contains("neither"),
        "the refusal says it is neither red nor a transform:\n{out}"
    );

    // Merge messages and decision records pass with a word.
    let dir = project("workoutside", "", "assert!(true);");
    let (out, code) = gate(&dir, "Merge branch 'main' into 0009-w");
    assert_eq!(code, 0, "a merge message is outside the judgement:\n{out}");
    assert!(
        out.contains("outside the judgement"),
        "the pass carries its word:\n{out}"
    );
    let (out, code) = gate(&dir, "Journal: a decision recorded");
    assert_eq!(
        code, 0,
        "a decision record is outside the judgement:\n{out}"
    );
    assert!(
        out.contains("outside the judgement"),
        "the pass carries its word:\n{out}"
    );
}

/// proves: build-break-is-not-red@b34ba9 -- holds journal A3: "did
/// not compile" and "did not run" are not "failed". A red claimed on
/// a test that does not build, and on a test the run never executes,
/// both refuse with the reason and cargo's words.
#[test]
fn build_break_is_not_red() {
    // The test file does not compile -- no birth.
    let dir = project(
        "redbroken",
        "",
        "let broken_on_purpose: i32 = \"not a number\";",
    );
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 1, "a build break earns nothing:\n{out}");
    assert!(
        out.contains("do not compile"),
        "the refusal says the tests do not compile:\n{out}"
    );

    // The run executes no test -- the tag names a test compiled out.
    let dir = project("rednotrun", "", "assert!(false);");
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/t_test.rs",
        &format!(
            "/// proves: s@{s_rev}\n#[cfg(feature = \"never\")]\n#[test]\nfn holds_s() {{ assert!(false); }}\n"
        ),
    );
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 1, "zero executed tests earn nothing:\n{out}");
    assert!(
        out.contains("executed no test"),
        "the refusal says the run executed nothing:\n{out}"
    );
}

/// proves: gate-modes-obeyed@ba7150 -- holds journal A3 and the
/// config school: strict blocks with exit 1, soft says the same
/// words with exit 0, manual says the judgement is off; an absent
/// mode acts as strict and says it is the default; a branch named as
/// no wave passes with a word in every mode.
#[test]
fn gate_modes_obeyed() {
    // The same violation -- an unearned red -- under each mode.
    let dir = project("modestrict", "mode = \"strict\"\n", "assert!(true);");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 1, "strict blocks:\n{out}");
    assert!(out.contains("mode: strict"), "the mode named:\n{out}");

    let dir = project("modesoft", "mode = \"soft\"\n", "assert!(true);");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 0, "soft warns and lets through:\n{out}");
    assert!(
        out.contains("is green") && out.contains("mode: soft"),
        "the same words, as a warning:\n{out}"
    );

    let dir = project("modemanual", "mode = \"manual\"\n", "assert!(true);");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 0, "manual passes:\n{out}");
    assert!(
        out.contains("judgement is off"),
        "manual says the judgement is off:\n{out}"
    );

    // No mode field -- strict, and the default does not pass itself
    // off as read.
    let dir = project("modeabsent", "", "assert!(true);");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 1, "the absent mode acts as strict:\n{out}");
    assert!(
        out.contains("mode: strict (the default"),
        "the default named as a default:\n{out}"
    );

    // A branch named as no wave -- nothing to judge, in every mode.
    let dir = project("modenotwave", "mode = \"strict\"\n", "assert!(true);");
    git(&dir, &["checkout", "-q", "-b", "just-work"]);
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 0, "no wave, nothing to judge:\n{out}");
    assert!(
        out.contains("nothing to judge"),
        "the pass carries its word:\n{out}"
    );
}
