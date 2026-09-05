//! Scenario tests of wave 0012-loop-stages, transform step-hand:
//! `keel next` hands exactly one self-sufficient step -- from the
//! birth of a test through the transform package to the review and
//! the PR -- and, off a wave branch, the readiness overview.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

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
fn project(name: &str, branch: &str) -> Sandbox {
    let dir = keel_sandbox(name);
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
        // A withdrawn promise makes this wave FULL by §6.8 -- the
        // death of a promise is the risk the paragraph buys two
        // human looks for -- so the review step below is the one the
        // norm actually asks for. Before wave 0036 the weight was
        // counted by a rule of `close`'s own, which called any
        // one-transform wave with a scenario full whether §6.8 does
        // or not (review 0036 R-1).
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\n  gone: {covers: [performance.capacity], withdrawn: \"folded\"}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## scenario: gone\n\nold body\n\n## transform: t\n\nthe work of t\n",
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
        out.contains("time for the PR") && out.contains("the review lies next to the wave"),
        "the reviewed FULL wave hears its own words exactly, not a shared substring (0016 R-4):\n{out}"
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

/// proves: next-hands-one-step@82fe98 -- the second birth out of
/// review 0012 (R-1/R-3/R-4/R-5/R-7/R-8): a namesake's foreign proof
/// is never "drifted" -- the step is this wave's own test, no tag
/// rewriting advised; a header transform whose body section is
/// missing is said by name in the package, not handed out silently
/// incomplete; a CRLF wave still hands the body verbatim; a light
/// wave is not driven through the review §9.9 never asked of it; an
/// empty test-run list is a word, not a bare label; and `one new in`
/// is satisfied by exactly one file, not by any.
#[test]
fn next_hands_one_step_second_birth() {
    // R-1: the namesake. The old wave's tag legally proves the old
    // wave; the plan's step is its own birth, never a tag rewrite.
    let dir = project("namesake", "0201-new");
    write(
        &dir,
        "keel/waves/0200-old.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n",
    );
    let old_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/s_test.rs",
        &format!("/// proves: s@{old_rev}\n#[test]\nfn holds_s() {{}}\n"),
    );
    write(&dir, "keel/reviews/0200-old.md", "# Рецензія\n\nok\n");
    write(
        &dir,
        "keel/waves/0201-new.md",
        "---\ndepends_on: [0200-old]\nscenarios:\n  s: {covers: [functional.completeness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\na different body of s\n\n## transform: t\n\nthe work of t\n",
    );
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("write the test") && out.contains("red: s"),
        "the namesake plan's step is its own birth (R-1, §5.6):\n{out}"
    );
    assert!(
        !out.contains("rewrite the tag") && !out.contains("drifted"),
        "no advice to rewrite the closed wave's proof (R-1):\n{out}"
    );

    // R-3: a transform declared in the header with no body section
    // is named in the package, never silently incomplete.
    let dir = project("headless", "0300-h");
    write(
        &dir,
        "keel/waves/0300-h.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  headless:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n",
    );
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/s_test.rs",
        &format!("/// proves: s@{s_rev}\n#[test]\nfn holds_s() {{}}\n"),
    );
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("transform \"headless\"") && out.contains("has no body section"),
        "the missing section is a word in the package, not silence (R-3, §7.7):\n{out}"
    );

    // R-4: a CRLF wave file still hands the scenario body verbatim
    // under the verbatim label (0009 R-3 school).
    let dir = project("crlf", "0400-c");
    write(
        &dir,
        "keel/waves/0400-c.md",
        "---\r\nscenarios:\r\n  s: {covers: [functional.correctness]}\r\ntransforms:\r\n  t:\r\n    implements: [s]\r\n    files: [src/lib.rs]\r\n---\r\n\r\n## Why\r\n\r\nwhy words\r\n\r\n## scenario: s\r\n\r\nbody of s rides crlf\r\n",
    );
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("body of s rides crlf"),
        "the verbatim label carries the body even over CRLF (R-4):\n{out}"
    );

    // R-5 as the norm now stands: a light wave is read by a person
    // too (§9.9, the operator's decision of 2026-09-04), and what
    // weight decides is the number of pull requests -- one, not two.
    // The 0012 R-5 finding this guards is still guarded: the step
    // after the report must not be the two-PR word.
    let dir = project("light", "0500-l");
    write(
        &dir,
        "keel/waves/0500-l.md",
        "---\ntransforms:\n  tidy:\n    chore: \"a tidy-up without a promise\"\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n",
    );
    commit_all(&dir);
    write(&dir, "src/lib.rs", "pub fn grown() {}\n");
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("time for the review"),
        "a light wave is read by a person too (§9.9):\n{out}"
    );
    write(&dir, "keel/reviews/0500-l.md", "# Рецензія\n\nok\n");
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("time for the PR") && out.contains("one PR"),
        "and then heads to its ONE pull request, not two (R-5, \
         §6.8):\n{out}"
    );

    // R-7: a transform whose only scenario is withdrawn -- the run
    // list is a word, never a bare label over nothing.
    let dir = project("norun", "0600-w");
    write(
        &dir,
        "keel/waves/0600-w.md",
        "---\nscenarios:\n  s:\n    covers: [functional.correctness]\n    withdrawn: \"retired in review\"\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t\n",
    );
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("tests of its scenarios do not exist"),
        "the empty run list is a word, not a bare label (R-7):\n{out}"
    );

    // R-8: `one new in` promises exactly one file -- two added files
    // do not satisfy it, the transform is still the step.
    let dir = project("onenew", "0700-n");
    write(
        &dir,
        "keel/waves/0700-n.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [one new in migrations/]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t\n",
    );
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/s_test.rs",
        &format!("/// proves: s@{s_rev}\n#[test]\nfn holds_s() {{}}\n"),
    );
    commit_all(&dir);
    write(&dir, "migrations/one.sql", "select 1;\n");
    write(&dir, "migrations/two.sql", "select 2;\n");
    commit_all(&dir);
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("transform \"t\""),
        "two new files do not satisfy the promise of one (R-8, §4.1):\n{out}"
    );
    assert!(
        !out.contains("time for the review"),
        "the wave is not called assembled over a red scope (R-8):\n{out}"
    );
}

/// proves: light-pr-words-honest@6fa167 -- holds §6.8/§9.9 and the
/// debt named by the 0015 dogfood: a light wave's PR step speaks its
/// own words -- ONE pull request, closed by the fact of merge -- while
/// a full wave hears about its two. Since the operator's decision of
/// 2026-09-04 the reviewer is asked of every wave, so the report lies
/// beside both before either hears a PR word: weight decides how many
/// pull requests and nothing else.
#[test]
fn light_pr_words_honest() {
    let dir = project("lightpr", "0800-l");
    write(
        &dir,
        "keel/waves/0800-l.md",
        "---\ntransforms:\n  tidy:\n    chore: \"a tidy-up without a promise\"\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n",
    );
    commit_all(&dir);
    write(&dir, "src/lib.rs", "pub fn grown() {}\n");
    commit_all(&dir);

    // First the reviewer, as for every wave (§9.9).
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("time for the review"),
        "a light wave is read by a person too (§9.9, the operator's \
         decision of 2026-09-04):\n{out}"
    );

    write(&dir, "keel/reviews/0800-l.md", "# Рецензія\n\nok\n");
    let (out, err, _) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("time for the PR") && out.contains("one PR"),
        "and then it hears its own PR words -- one, not two \
         (§6.8):\n{out}"
    );
}
