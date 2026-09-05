//! Scenario test of wave 0037: every wave has its reviewer.

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

fn keel(dir: &Path, command: &str) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args([command, dir.to_str().unwrap()])
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

fn decided() -> String {
    let mut out = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        out.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    out
}

/// A light wave by §6.8: one transform, no contract, nothing
/// withdrawn -- and a promise proven by its tag, so the only thing
/// that can hold it open is the reviewer's report.
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
    // The wave, its proof and its work live on the wave's own branch,
    // so `next` sees the work done and the report missing.
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{d}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    let rev = keel::rev::text_rev("тіло обіцянки\n");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{}}\n"),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "red: it-holds"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "work: the work of it",
        ],
    );
    dir
}

/// proves: every-wave-has-its-reviewer@193d9f -- the operator's
/// decision of 2026-09-04. §9.9 said "a FULL wave without this file
/// does not merge", and review 0036 measured the consequence once
/// the weight was counted by §6.8 as written: a wave with one
/// transform and a promise would ride one PR with nobody reading it.
/// The barrier no longer depends on the weight.
#[test]
fn every_wave_has_its_reviewer() {
    // A light wave, everything else in order, no report: not closed.
    let dir = project("noreport");
    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага легка"),
        "the fixture is a LIGHT wave, so the weight cannot be what \
         holds it open (§6.8):\n{said}"
    );
    let (said, code) = keel(&dir, "close");
    assert_eq!(
        code, 1,
        "a wave without its reviewer's report is not closed \
         (§9.9):\n{said}"
    );
    assert!(
        said.contains("0001-a-wave") && said.contains("рецензі"),
        "and the reason is named:\n{said}"
    );

    // The step the tool hands out is the review, not the PR.
    let (said, _) = keel(&dir, "next");
    assert!(
        said.contains("keel review"),
        "and the next step is the reviewer, whatever the weight:\n{said}"
    );

    // An EMPTY file is not a review: review 0037 R-2 measured `: >
    // file` passing the gate, with the verdict then claiming the
    // report was beside the wave -- more than the machine ever
    // looked at.
    std::fs::write(dir.join("keel/reviews/0001-a-wave.md"), "   \n\n").unwrap();
    let (said, code) = keel(&dir, "close");
    assert_eq!(code, 1, "an empty file is not a review (§9.9):\n{said}");
    assert!(
        said.contains("порожній"),
        "and the reason says exactly that:\n{said}"
    );

    // With the report beside the wave, it closes.
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nвсе гаразд\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, "close");
    assert_eq!(code, 0, "with the report it closes:\n{said}");
    assert!(
        said.contains("машина його не читала"),
        "and the verdict says only what it measured -- that the FILE \
         is there, not that a review happened (review 0037 \
         R-2):\n{said}"
    );

    // A wave with NO promises at all is read too: merging is still
    // its closure, but a person reads it first.
    let dir = keel_sandbox("chorewave");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0002-b-wave.md"),
        format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0002-b-wave"]);
    let (said, code) = keel(&dir, "close");
    assert_eq!(
        code, 1,
        "a wave with no promises is read by a person too \
         (§9.9):\n{said}"
    );
    assert!(
        said.contains("0002-b-wave") && said.contains("рецензі"),
        "and the reason is named:\n{said}"
    );
}
