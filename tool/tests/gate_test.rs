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
            // The dead cover of `gone` does not answer its cut
            // (§2.12), so the decisions block still carries it -- the
            // fixture leaves no side finding to blur exit codes
            // (review R-6e).
            all_decided_except(&["functional.correctness"])
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

    // Second birth (review R-3): the capitalized twin of a birth --
    // the likeliest field typo, git convention loves a capital -- must
    // not walk past as "outside the judgement".
    let dir = project("redcapital", "", "assert!(true);");
    let (out, code) = gate(&dir, "Red: s");
    assert_eq!(code, 1, "a capitalized twin does not slip past:\n{out}");
    assert!(
        out.contains("lowercase"),
        "the refusal teaches the shape:\n{out}"
    );

    // A birth claimed with no tagged test refuses (review R-6d).
    let dir = project("reduntagged", "", "assert!(true);");
    write(&dir, "tests/t_test.rs", "#[test]\nfn plain() {}\n");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 1, "an untagged birth refuses:\n{out}");
    assert!(
        out.contains("no test carries"),
        "the missing tag named:\n{out}"
    );
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

    // All tags of a scenario run, whichever file they live in -- the
    // second, failing one blocks the work (review R-6c).
    let dir = project("worktwotags", "", "assert!(true);");
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/second_test.rs",
        &format!("/// proves: s@{s_rev}\n#[test]\nfn holds_s_again() {{ assert!(false); }}\n"),
    );
    let (out, code) = gate(&dir, "t: does the declared work");
    assert_eq!(code, 1, "every tag of the scenario runs:\n{out}");
    assert!(
        out.contains("holds_s_again"),
        "the second, failing test named:\n{out}"
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

    // Second birth (review R-10): a transform whose every implements
    // scenario is withdrawn passes on a vacuum -- the pass must say
    // no live scenario was judged (§2.12), not count zero quietly.
    let dir = project("workvacuum", "", "assert!(true);");
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\n  gone:\n    covers: [performance.capacity]\n    withdrawn: \"folded\"\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n  empty-t:\n    implements: [gone]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody of s\n\n## scenario: gone\n\nold body\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "empty-t: declared"]);
    let (out, code) = gate(&dir, "empty-t: only the dead remain");
    assert_eq!(code, 0, "a vacuum is not a refusal:\n{out}");
    assert!(
        out.contains("no live scenario"),
        "the vacuum said aloud (§2.12):\n{out}"
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

    // A branch named as no wave -- nothing to judge, in every mode
    // (review R-6b: soft and manual proven too, not only strict).
    let dir = project("modenotwave", "mode = \"strict\"\n", "assert!(true);");
    git(&dir, &["checkout", "-q", "-b", "just-work"]);
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 0, "no wave, nothing to judge:\n{out}");
    assert!(
        out.contains("nothing to judge"),
        "the pass carries its word:\n{out}"
    );
    let dir = project("modenotwavesoft", "mode = \"soft\"\n", "assert!(true);");
    git(&dir, &["checkout", "-q", "-b", "just-work"]);
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 0, "soft passes a non-wave branch too:\n{out}");
    assert!(out.contains("nothing to judge"), "with the word:\n{out}");
    let dir = project("modenotwavemanual", "mode = \"manual\"\n", "assert!(true);");
    git(&dir, &["checkout", "-q", "-b", "just-work"]);
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 0, "manual passes everywhere:\n{out}");
    assert!(
        out.contains("judgement is off"),
        "manual's own word stands:\n{out}"
    );

    // A mode outside the three refuses with the list (review R-6d).
    let dir = project("modebad", "mode = \"STRICT\"\n", "assert!(true);");
    let (out, code) = gate(&dir, "red: s");
    assert_eq!(code, 2, "an unknown mode is a machine refusal:\n{out}");
    assert!(
        out.contains("strict, soft, manual"),
        "the three named:\n{out}"
    );
}

/// proves: hook-installed-aloud@25a179 -- holds §9.7 and the wave's
/// install honesty: keel hook writes an executable commit-msg that
/// calls keel gate; a second run is quietly the same file; a foreign
/// hook is never overwritten -- a refusal aloud, the file untouched.
#[test]
fn hook_installed_aloud() {
    // A repo without a hook gains one.
    let dir = project("hookfresh", "", "assert!(true);");
    let (out, _err, code) = keel(&["hook", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "installation passes:\n{out}");
    let hook = dir.join(".git/hooks/commit-msg");
    assert!(hook.is_file(), "the hook file exists");
    let text = fs::read_to_string(&hook).unwrap();
    assert!(
        text.contains("keel gate"),
        "the hook calls keel gate:\n{text}"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(
            fs::metadata(&hook).unwrap().permissions().mode() & 0o111 != 0,
            "the hook is executable"
        );
    }

    // The second run: quietly the same file.
    let (out2, _err, code) = keel(&["hook", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "idempotent:\n{out2}");
    assert_eq!(
        text,
        fs::read_to_string(&hook).unwrap(),
        "the file is unchanged"
    );

    // A foreign hook is not overwritten.
    let dir = project("hookforeign", "", "assert!(true);");
    let foreign = dir.join(".git/hooks/commit-msg");
    fs::create_dir_all(foreign.parent().unwrap()).unwrap();
    fs::write(&foreign, "#!/bin/sh\necho custom\n").unwrap();
    let (out, err, code) = keel(&["hook", dir.to_str().unwrap()]);
    assert_ne!(code, 0, "a foreign hook refuses:\n{out}{err}");
    assert_eq!(
        fs::read_to_string(&foreign).unwrap(),
        "#!/bin/sh\necho custom\n",
        "the foreign file stays untouched"
    );

    // Second birth (review R-1): in a git worktree the hook must land
    // where git actually reads it -- the shared hooks of the common
    // dir -- and a real commit through it must be judged; "installed"
    // about a file git never reads is the lie this wave was built
    // against.
    let dir = project("hookwt", "", "assert!(true);");
    git(&dir, &["checkout", "-q", "-b", "parking"]);
    let wt = std::env::temp_dir().join(format!("keel-0005g-{}-hookwt-wt", std::process::id()));
    let _ = fs::remove_dir_all(&wt);
    git(
        &dir,
        &["worktree", "add", "-q", wt.to_str().unwrap(), "0009-w"],
    );

    let (out, _err, code) = keel(&["hook", wt.to_str().unwrap()]);
    assert_eq!(code, 0, "installation from a worktree passes:\n{out}");
    assert!(
        dir.join(".git/hooks/commit-msg").is_file(),
        "the hook lands in the shared hooks git reads:\n{out}"
    );

    // A real commit in the worktree: green test, a claimed birth --
    // the gate must block it.
    write(&wt, "src/lib.rs", "// touched\n");
    let bin_dir = Path::new(env!("CARGO_BIN_EXE_keel"))
        .parent()
        .unwrap()
        .to_path_buf();
    let path_env = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let commit = Command::new("git")
        .arg("-C")
        .arg(&wt)
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(["commit", "-am", "red: s"])
        .env("PATH", path_env)
        .output()
        .unwrap();
    assert!(
        !commit.status.success(),
        "the unearned red is blocked through the worktree too:\n{}{}",
        String::from_utf8_lossy(&commit.stdout),
        String::from_utf8_lossy(&commit.stderr)
    );
    git(
        &dir,
        &["worktree", "remove", "--force", wt.to_str().unwrap()],
    );
}

/// proves: battery-isolated@715790 -- holds §7.12 and §6.7 (0008
/// review R-8): the adapter's cargo runs ignore an inherited
/// CARGO_TARGET_DIR -- the judged project builds into its own
/// target directory and nothing leaks into the shared cache, so a
/// shared cache cannot shift verdicts.
#[test]
fn battery_isolated() {
    let dir = project("isolated", "", "assert!(false);");
    let poison = std::env::temp_dir().join(format!("keel-0009-poison-{}", std::process::id()));
    let _ = fs::remove_dir_all(&poison);
    fs::create_dir_all(&poison).unwrap();
    let msg = dir.join("COMMIT_EDITMSG");
    fs::write(&msg, "red: s").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
        .env("CARGO_TARGET_DIR", &poison)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "the red verdict stays honest under a poisoned environment:\n{text}"
    );
    assert!(
        dir.join("target").exists(),
        "the judged project builds into its own target directory:\n{text}"
    );
    assert_eq!(
        fs::read_dir(&poison).unwrap().count(),
        0,
        "nothing leaks into the inherited CARGO_TARGET_DIR:\n{text}"
    );

    // Second birth (review 0009 R-2): cargo's own alias for the same
    // knob -- CARGO_BUILD_TARGET_DIR -- must be dropped too, or the
    // shared cache walks back in through the side door.
    let dir = project("isolated-alias", "", "assert!(false);");
    let poison = std::env::temp_dir().join(format!("keel-0009-poison-b-{}", std::process::id()));
    let _ = fs::remove_dir_all(&poison);
    fs::create_dir_all(&poison).unwrap();
    let msg = dir.join("COMMIT_EDITMSG");
    fs::write(&msg, "red: s").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
        .env("CARGO_BUILD_TARGET_DIR", &poison)
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "the red verdict stays honest under the alias too:\n{text}"
    );
    assert!(
        dir.join("target").exists(),
        "the judged project builds into its own target under the alias (R-2):\n{text}"
    );
    assert_eq!(
        fs::read_dir(&poison).unwrap().count(),
        0,
        "nothing leaks through CARGO_BUILD_TARGET_DIR:\n{text}"
    );
}
