//! Scenario tests of wave 0005-commit-gate, transform tag-floor:
//! proves tags in test files judged by the machine (§5.5, §7.5) --
//! the last hand-held revision check moves into keel check.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0005t-{}-{name}", std::process::id()));
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
    let dir = sandbox("staletag");
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
            all_decided_except(&[
                "functional.correctness",
                "performance.capacity",
                "security.integrity"
            ])
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
    let dir = sandbox("badrev");
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
}
