//! Scenario test of wave 0036: work without a proof is red.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

/// git with an identity of its own: review 0036 R-11 measured all
/// five probes of this wave failing on a machine with no global
/// git config -- a fresh CI container, that is -- while the
/// twenty-one older ones, which pass `-c user.email`, held.
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

fn check(dir: &Path) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
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

fn wave(extra_cut: &str) -> String {
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != extra_cut {
            decided.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    format!(
        "---\nscenarios:\n  it-holds:\n    covers: [{extra_cut}]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
    )
}

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: work-without-a-proof-is-red@d30a6f -- the conformance
/// audit (ВАЖКА-7) measured a branch with work commits and NOT ONE
/// test getting zero findings: §7.5 says every live scenario has a
/// green test of its own name, and on a branch nothing judged it.
#[test]
fn work_without_a_proof_is_red() {
    // Work committed, no tag anywhere: the promise has no proof, and
    // that is a finding by name.
    let dir = project("untested");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        wave("functional.correctness"),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: no test at all"]);

    let (said, code) = check(&dir);
    assert_eq!(code, 1, "work with no proof is red (§7.5):\n{said}");
    assert!(
        said.contains("it-holds") && said.contains("не має жодного тегу"),
        "and the finding names the promise left unproven:\n{said}"
    );

    // An approved-not-started wave is NOT red: §7.5 says so in as
    // many words, and there is no work commit on this branch.
    let dir = project("notstarted");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        wave("functional.correctness"),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "plan: wave 0001"]);

    let (said, _) = check(&dir);
    assert!(
        !said.contains("не має жодного тегу"),
        "a wave approved and not started is not red (§7.5):\n{said}"
    );

    // And a promise with its tag is silence.
    let dir = project("tagged");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let text = wave("functional.correctness");
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), &text).unwrap();
    let rev = keel::rev::text_rev("тіло обіцянки\n");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{}}\n"),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: with its proof"]);

    let (said, _) = check(&dir);
    assert!(
        !said.contains("не має жодного тегу"),
        "a promise with its tag is silence:\n{said}"
    );

    // A WITHDRAWN promise is outside judgement (§2.12) -- review 0036
    // R-3 (M13) measured this rule held by nothing: accusing the
    // withdrawn left the battery green.
    let dir = project("withdrawnpromise");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        decided.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\n    withdrawn: \"згорнуто\"\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "work: on a withdrawn promise"],
    );
    let (said, _) = check(&dir);
    assert!(
        !said.contains("не має жодного тегу"),
        "a withdrawn promise is outside judgement (§2.12):\n{said}"
    );

    // And a project the tags were never read for accuses nobody:
    // without the rust adapter there is nothing to read them from.
    let dir = project("noadapter");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "chore: no adapter"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        wave("functional.correctness"),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: no tags to read"]);
    let (said, _) = check(&dir);
    assert!(
        !said.contains("не має жодного тегу"),
        "where the tags were never read, nothing is accused:\n{said}"
    );
}
