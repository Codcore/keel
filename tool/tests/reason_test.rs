//! Scenario test of wave 0066: an answer without a reason is a
//! finding.
//!
//! §10.3 asks an answer to carry a REASON -- "does not apply,
//! BECAUSE…" -- and the machine only ever asked whether an answer was
//! there at all. Measured across all 64 waves of this tree before the
//! wave: 2247 answers, of which 1222 explain after a colon, 194 carry
//! "бо", and **831 are the bare formula**, with nothing in between --
//! so the question is exact and needs neither a length threshold nor
//! a word list.
//!
//! The court is asked HERE, not through a whole project: a sandbox
//! grown to satisfy every other court costs more than the thing it
//! measures, and the answer to this question does not depend on any
//! of them.

mod common;

use common::keel_sandbox;

fn wave_with(decisions: &[(&str, &str)]) -> keel::docs::Wave {
    let dir = keel_sandbox("reasonwave");
    let mut said = String::from("decisions:\n");
    for (cut, text) in decisions {
        said.push_str(&format!("  {cut}: \"{text}\"\n"));
    }
    for cut in keel::graph::cuts() {
        if decisions.iter().any(|(c, _)| c == cut) {
            continue;
        }
        said.push_str(&format!("  {cut}: \"не про цю хвилю, бо вона про інше\"\n"));
    }
    let path = dir.join("keel/waves/0001-a-wave.md");
    std::fs::write(
        &path,
        format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{said}---\n\n## transform: work\nтіло\n"
        ),
    )
    .unwrap();
    let wave = keel::docs::read_wave(&path).expect("the fixture reads");
    std::mem::forget(dir); // the sandbox outlives the borrow of its file
    wave
}

/// proves: an-answer-without-a-reason-is-a-finding@9b1dce
#[test]
fn an_answer_without_a_reason_is_a_finding() {
    // --- the bare formula, in every spelling this tree and its
    // neighbours write ---
    for bare in [
        "не застосовується",
        "не застосовується.",
        "НЕ ЗАСТОСОВУЄТЬСЯ",
        "not applicable",
        "n/a",
        "-",
    ] {
        let wave = wave_with(&[("performance.capacity", bare)]);
        let found = keel::graph::reason_findings(&wave);
        assert_eq!(
            found.len(),
            1,
            "«{bare}» is the formula and nothing else -- mechanically an \
             answer, and empty of one (§10.3)"
        );
        assert!(
            found[0].0.contains("performance.capacity"),
            "and the finding names the cut: {}",
            found[0].0
        );
        assert!(
            found[0].0.contains("§10.3"),
            "citing the paragraph that asks for the reason: {}",
            found[0].0
        );
    }

    // --- an answer that EXPLAINS is silent, in both forms this tree
    // actually holds: after a colon (1222 of them) and with «бо» (194) ---
    for given in [
        "названо: ця хвиля не чіпає мережі",
        "не застосовується, бо тут нема UI",
        "не застосовується: тут нема чого міряти",
        "свідомо не робимо, бо ціна більша за користь",
    ] {
        let wave = wave_with(&[("performance.capacity", given)]);
        assert!(
            keel::graph::reason_findings(&wave).is_empty(),
            "«{given}» carries a reason, and the court says nothing"
        );
    }

    // --- several empty answers are ONE finding that names them all:
    // twenty-nine separate rows would be the noise this project
    // already paid for once (wave 0064's plan package) ---
    let wave = wave_with(&[
        ("performance.capacity", "не застосовується"),
        ("security.integrity", "n/a"),
        ("flexibility.scalability", "не застосовується"),
    ]);
    let found = keel::graph::reason_findings(&wave);
    assert_eq!(found.len(), 1, "one finding, not one per cut");
    for cut in [
        "performance.capacity",
        "security.integrity",
        "flexibility.scalability",
    ] {
        assert!(
            found[0].0.contains(cut),
            "and it names {cut}: {}",
            found[0].0
        );
    }

    // --- the advice says what to do, and that history is not touched ---
    assert!(
        found[0].1.contains("closed waves are not touched"),
        "the advice says the border aloud -- closed waves are history, \
         and the 831 bare answers already merged are not rewritten by a \
         new rule: {}",
        found[0].1
    );
}
