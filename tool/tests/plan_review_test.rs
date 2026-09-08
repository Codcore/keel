//! Scenario test of wave 0064: the plan meets a reader before the
//! merge.
//!
//! Measured before the plan: `keel review` on a plan branch refuses --
//! "the branch is not named as a wave (§8.2)" -- so the package a
//! fresh reader gets is assembled for the WORK branch alone. The forty
//! answers to the cuts are written at planning; the reader of §9.9
//! arrives at closing, when all the work is already done under that
//! plan. Between the two stands the approval of §6.6, held by nothing
//! but a person's reading.
//!
//! The tool catches ABSENCE well -- `graph-silence` shouts over a cut
//! with no answer. It does not catch UNTRUTH: the answer is there, it
//! is wrong, and the machine says nothing. This probe holds the
//! narrowed package that shows a person WHERE to look.

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

/// A project whose plan is FULL by every mechanical measure -- every
/// cut answered, `keel check` green -- and whose answers a person
/// would call wrong on sight: one cut closed by a promise that does
/// not prove it, one decided with a shrug.
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
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut == "functional.correctness" || *cut == "compatibility.interoperability" {
            continue;
        }
        if *cut == "performance.time-behaviour" {
            // A shrug where a reason belongs: mechanically an answer,
            // and empty of one.
            decided.push_str(&format!("  {cut}: \"не застосовується\"\n"));
        } else {
            decided.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  sqlite-is-the-database:\n    covers: [functional.correctness, compatibility.interoperability]\ntransforms:\n  work:\n    implements:\n      - sqlite-is-the-database\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: sqlite-is-the-database\nтіло обіцянки: база даних — sqlite\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: the-plan-meets-a-reader-before-the-merge@ba3e95
#[test]
fn the_plan_meets_a_reader_before_the_merge() {
    let dir = project("planreview");
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);

    // The plan is full by every mechanical measure.
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "the plan is green by the machine:\n{said}");

    // And the package for a reader assembles HERE, on the plan
    // branch, where it was a refusal before this wave.
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(
        code, 0,
        "`keel review` on a plan branch gives a package, not a refusal \
         -- the cheapest review is the one nobody can get:\n{said}"
    );
    assert!(
        !said.contains("не зветься як хвиля"),
        "and does not send the reader to a branch that does not exist \
         yet:\n{said}"
    );

    // What the package narrows to: the cut closed by a promise, with
    // the question a person answers yes or no.
    assert!(
        said.contains("compatibility.interoperability") && said.contains("sqlite-is-the-database"),
        "a cut closed by a promise is named beside the promise that \
         closes it -- the reader's one question is whether that promise \
         proves THIS cut:\n{said}"
    );
    // And the decided one whose reason is a shrug.
    assert!(
        said.contains("performance.time-behaviour"),
        "a cut decided with a bare «не застосовується» is named too: \
         mechanically an answer, and empty of a reason:\n{said}"
    );
    // The line it points at, verbatim -- not a retelling.
    assert!(
        said.contains("не застосовується"),
        "each question carries the line it points at, word for word, so \
         the reader judges the text and not a summary:\n{said}"
    );

    // On the WORK branch the package is what it always was.
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(code, 0, "the work branch still gets its package:\n{said}");
    assert!(
        said.contains("дрейф") || said.contains("мапа"),
        "with the three lists §9.9 asks for:\n{said}"
    );
}
