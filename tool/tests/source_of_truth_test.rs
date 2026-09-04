//! Scenario test of wave 0032: the source of truth is a numbered
//! rule, not a block quote above the first one.

mod common;

use common::keel_sandbox;
use std::process::Command;

/// proves: the-source-of-truth-is-a-numbered-rule@f620d1 -- the open
/// question the reviewer of wave 0029 put to the operator (P-2): the
/// most important rule about the two texts was the only one with no
/// number, so `keel method §1.1` served half of it and nothing could
/// cite it.
#[test]
fn the_source_of_truth_is_a_numbered_rule() {
    let dir = keel_sandbox("truth");

    for (lang, wanted) in [("uk", "джерело правди"), ("en", "source of truth")] {
        std::fs::write(dir.join("keel.toml"), format!("lang = \"{lang}\"\n")).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(["method", "§1.8", dir.to_str().unwrap()])
            .output()
            .unwrap();
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            said.contains(wanted),
            "`keel method §1.8` in {lang} serves the rule about the two \
             texts:\n{}",
            &said[..said.len().min(400)]
        );
        assert!(
            said.contains("§1.8"),
            "and serves it under its own number:\n{}",
            &said[..said.len().min(200)]
        );
    }

    // And the skeleton court still sees one skeleton: the paragraph
    // arrives in BOTH texts or in neither.
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let verdict = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        !verdict.contains("skeleton") && !verdict.contains("кістяк"),
        "the two texts still carry one skeleton:\n{verdict}"
    );
}
