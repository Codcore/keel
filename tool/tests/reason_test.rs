//! Scenario test of wave 0066: an answer without a reason is a
//! finding.
//!
//! §10.3 asks for a REASON -- "does not apply, BECAUSE…" -- and the
//! machine only ever asked whether an answer was there at all.
//! Measured across all 64 waves of this tree before the wave: 2247
//! answers, of which 1222 explain after a colon, 194 carry "бо", and
//! **831 are the bare formula**. Nothing lies between the two, so the
//! question here is exact and needs no threshold and no word list --
//! the first attempt at it (the plan package of wave 0064) asked
//! about length and words, and a reviewer measured it catching honest
//! short reasons while missing long empty ones.

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

/// A wave whose answers take the forms this tree actually holds: the
/// bare formula, the formula with an explanation after a colon, and
/// the formula with "бо".
fn project(name: &str, bare_cut: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut == "functional.correctness" {
            continue;
        }
        let said = if *cut == bare_cut {
            // The bare formula: mechanically an answer, empty of one.
            "не застосовується".to_string()
        } else if cut.starts_with("security") {
            // A reason after a colon -- the form 1222 answers of this
            // tree take.
            "названо: ця пісочниця не має чого захищати".to_string()
        } else {
            // A reason with "бо" -- the form §10.3 spells out.
            "не застосовується, бо ця пісочниця грає один розріз".to_string()
        };
        decided.push_str(&format!("  {cut}: \"{said}\"\n"));
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: it-works\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: an-answer-without-a-reason-is-a-finding@9b1dce
#[test]
fn an_answer_without_a_reason_is_a_finding() {
    // --- the bare formula is a finding, and the finding says WHAT is
    // missing ---
    let dir = project("reasonbare", "performance.capacity");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "a bare formula reddens the check:\n{said}");
    assert!(
        said.contains("performance.capacity"),
        "and names the cut whose answer says nothing:\n{said}"
    );
    assert!(
        said.contains("§10.3"),
        "citing the paragraph that asks for a reason, so a person can \
         read what is wanted:\n{said}"
    );
    // Not the OTHER finding: silence is a different fault, and telling
    // them apart is the whole point (graph-silence already reddens
    // over a cut with no answer at all).
    assert!(
        !said.contains("розрізи без відповіді"),
        "and it is not confused with silence -- the answer is there, it \
         is empty:\n{said}"
    );

    // --- an answer that explains is silent, in both forms this tree
    // actually holds ---
    let dir = project("reasongiven", "");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "answers that carry a reason -- after a colon or with «бо» -- \
         leave the court silent:\n{said}"
    );

    // --- a CLOSED wave with the same formula is not touched: history
    // is not rewritten (§8.5 -- the number is a prefix, not an order,
    // so the border is open/closed, not old/new) ---
    let dir = project("reasonclosed", "performance.capacity");
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the wave is in the trunk"]);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "a wave whose file stands in the trunk is history, and 831 bare \
         answers of this tree's own past are not rewritten by a new \
         rule:\n{said}"
    );
}
