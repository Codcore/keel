//! Scenario test of wave 0053: the old revisions count once.
//!
//! The global review of 2026-09-06 (methodology R-14) measured the
//! line "old revisions, true in the file's history" counting rows,
//! not revisions: one `wave: contract@revision` repeated for every
//! reference of the header (169 rows on this tree, 88 unique at the
//! review's commit).
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn keel(dir: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
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

const BODY: &str = "тіло обіцянки\n\n";

fn decisions_except(covered: &[&str]) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// proves: the-old-revisions-count-once@583e63
#[test]
fn the_old_revisions_count_once() {
    // A closed wave whose header references one contract at one old
    // revision three times: the proves of two scenarios and the
    // contracts of the transform.
    let dir = keel_sandbox("oldonce");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn one() {}\n\npub fn two() {}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn one()\"]\n---\n\nold words\n",
    )
    .unwrap();
    let old_rev = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "old contract"]);
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: first@{rev}\n#[test]\nfn first() {{}}\n\n/// proves: second@{rev}\n#[test]\nfn second() {{}}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  first:\n    proves: anchor@{old_rev}\n    covers: [functional.correctness]\n  second:\n    proves: anchor@{old_rev}\n    covers: [functional.completeness]\ntransforms:\n  work:\n    implements:\n      - first\n      - second\n    contracts: [anchor@{old_rev}]\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: first\n{BODY}## scenario: second\n{BODY}## transform: work\nтіло роботи\n",
            decisions_except(&["functional.correctness", "functional.completeness"])
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn one()\", \"pub fn two()\"]\n---\n\nnew words\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "the wave and the newer contract"],
    );

    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "an old revision true in history is lawful (§5.6):\n{said}"
    );
    let count: u64 = said
        .lines()
        .find(|l| l.starts_with("старих редакцій"))
        .and_then(|l| l.split(": ").nth(1))
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(|word| word.parse().ok())
        .expect("the line counts the old revisions");
    assert_eq!(
        count, 1,
        "one contract at one old revision is ONE old revision, however many references hold it:\n{said}"
    );
    let rows = said
        .lines()
        .filter(|l| l.contains(&format!("anchor@{old_rev}")) && l.contains("стара"))
        .count();
    assert_eq!(rows, 1, "and it is listed once:\n{said}");
}
