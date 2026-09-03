//! Scenario tests of wave 0005-commit-gate, transform tag-floor:
//! proves tags in test files judged by the machine (§5.5, §7.5) --
//! the last hand-held revision check moves into keel check.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
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

/// proves: stale-tag-found@6fb3c5 -- holds §5.5/§7.5: a tag whose
/// revision no longer matches the scenario's text, and a tag proving
/// a scenario no wave knows, are findings by name; matching tags are
/// counted aloud; tags of withdrawn scenarios are not judged (§2.12).
#[test]
fn stale_tag_found() {
    let dir = keel_sandbox("staletag");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\n  ok: {{covers: [performance.capacity]}}\n  gone:\n    covers: [security.integrity]\n    withdrawn: \"folded\"\ntransforms:\n  t:\n    implements: [s, ok]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody of s\n\n## scenario: ok\n\nbody of ok\n\n## scenario: gone\n\nold body\n",
            // gone's dead cover does not answer security.integrity
            // (§2.12), so the decisions block still carries it -- no
            // side finding blurs the exit codes (review R-6e).
            all_decided_except(&["functional.correctness", "performance.capacity"])
        ),
    );
    let ok_rev = keel::rev::text_rev("body of ok\n");
    write(
        &dir,
        "tests/x_test.rs",
        &format!(
            "/// proves: s@aaaaaa -- stale on purpose\n#[test]\nfn holds_s() {{}}\n\n/// proves: ok@{ok_rev}\n#[test]\nfn holds_ok() {{}}\n\n/// proves: ghost@ab12cd\n#[test]\nfn holds_ghost() {{}}\n\n/// proves: gone@ffffff\n#[test]\nfn holds_gone() {{}}\n"
        ),
    );

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "a stale tag and an orphan tag are findings:\n{out}"
    );
    let s_rev = keel::rev::text_rev("body of s\n");
    assert!(
        out.contains("s@aaaaaa") && out.contains(&s_rev),
        "the stale tag named with recorded and current revisions:\n{out}"
    );
    assert!(
        out.contains("tests/x_test.rs"),
        "the test file named:\n{out}"
    );
    assert!(
        out.contains("ghost"),
        "the orphan tag named -- no wave knows the scenario:\n{out}"
    );
    assert!(
        !out.contains("gone@ffffff"),
        "a withdrawn scenario's tag is not judged (§2.12):\n{out}"
    );
    assert!(
        out.contains("test tags checked: 1"),
        "the one matching tag counted aloud:\n{out}"
    );

    // Second birth (review R-8): a revision written outside the §5.2
    // shape (4-6 hex) is a crooked record, and the refusal must say
    // that -- not dress it up as a stale comparison.
    let dir = keel_sandbox("badrev");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody of s\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    write(
        &dir,
        "tests/x_test.rs",
        "/// proves: s@abc\n#[test]\nfn holds_s() {}\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a crooked revision record is a finding:\n{out}");
    assert!(
        out.contains("4") && out.contains("hex") && out.contains("§5.2"),
        "the refusal names the shape, not a fake staleness:\n{out}"
    );

    // A dangling tag -- no test function right after it -- refuses by
    // name, e2e (review R-6d holds the tool-tags promise by run).
    let dir = keel_sandbox("dangling");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody of s\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    write(
        &dir,
        "tests/x_test.rs",
        "/// proves: s@abcdef\nconst NOT_A_TEST: u8 = 0;\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a dangling tag is a finding:\n{out}");
    assert!(
        out.contains("no test function right after"),
        "the dangling tag named:\n{out}"
    );
}

/// proves: vanished-tag-is-red@a68294 -- holds §7.15: a tag that was
/// present at the fork point and is gone at HEAD while the scenario
/// is alive is a finding where the disarming happened; a withdrawn
/// scenario's vanished tag is silence (§2.12).
#[test]
fn vanished_tag_is_red() {
    let git = |dir: &Path, args: &[&str]| {
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
    };
    let dir = keel_sandbox("vanish");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "keel/waves/0015-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\n  gone:\n    covers: [performance.capacity]\n    withdrawn: \"folded\"\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody of s\n\n## scenario: gone\n\nold body\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/t_test.rs",
        &format!(
            "/// proves: s@{s_rev}\n#[test]\nfn holds_s() {{}}\n\n/// proves: gone@ffffff\n#[test]\nfn held_gone() {{}}\n"
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "base with both tags"]);

    // On a branch both tags vanish; only the live scenario's loss is
    // a finding, named where the disarming happened.
    git(&dir, &["checkout", "-q", "-b", "feature"]);
    write(
        &dir,
        "tests/t_test.rs",
        "#[test]\nfn holds_s() {}\n\n#[test]\nfn held_gone() {}\n",
    );
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "tags quietly disarmed"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a vanished tag of a live scenario is red:\n{out}");
    assert!(
        out.contains("\"s\"") && out.contains("t_test.rs"),
        "the scenario and the file named (§7.15):\n{out}"
    );
    assert!(
        !out.contains("\"gone\""),
        "a withdrawn scenario's vanished tag is silence (§2.12):\n{out}"
    );

    // Second birth (review R-6): the tag vanishing together with the
    // whole wave file is the quiet destruction of a promise -- still
    // red, but the words must say the scenario is gone with its
    // wave, not call it alive.
    git(&dir, &["checkout", "-q", "-b", "erasing"]);
    fs::remove_file(dir.join("keel/waves/0015-w.md")).unwrap();
    fs::remove_file(dir.join("tests/t_test.rs")).unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the promise erased whole"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "erasing the wave with its tag is red:\n{out}");
    assert!(
        out.contains("gone with its wave"),
        "the words name the destruction, not a living scenario:\n{out}"
    );
}
