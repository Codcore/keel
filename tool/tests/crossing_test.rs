//! Scenario test of wave 0054: two open waves do not share a file.
//!
//! §8.8: two waves may run in parallel "if there is no depends_on edge
//! between them and their scopes do not cross; a crossing of files
//! between independent waves is a question at planning time".
//! Measured before the plan: two approved, unstarted waves both
//! declaring `src/lib.rs` -- `keel check` on main and on the plan
//! branch, "2 documents, 0 findings" (the operator's decision of
//! 2026-09-06: a finding).
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

/// A wave promising `it-works-<n>` over one file, with the header
/// lines given (depends_on, cancelled) in front of its scenarios.
fn wave(n: &str, file: &str, extra: &str) -> String {
    format!(
        "---\n{extra}scenarios:\n  it-works-{n}:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works-{n}\n    files:\n      - {file}\n{}---\n\n## scenario: it-works-{n}\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    )
}

/// A rust project with two waves, committed on main.
fn project(name: &str, first: &str, second: &str) -> common::Sandbox {
    project_in(name, "uk", first, second)
}

fn project_in(name: &str, lang: &str, first: &str, second: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"{lang}\"\nadapter = \"rust\"\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(dir.join("src/other.rs"), "pub fn b() {}\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-first.md"), first).unwrap();
    std::fs::write(dir.join("keel/waves/0002-second.md"), second).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn crossing_named(said: &str) -> bool {
    said.contains("0001-first")
        && said.contains("0002-second")
        && said.contains("src/lib.rs")
        && said.contains("§8.8")
}

/// proves: two-open-waves-do-not-share-a-file@498f78
#[test]
fn two_open_waves_do_not_share_a_file() {
    // --- two open waves, one file, no edge: a finding on main, on
    // the plan branch and on the work branch alike ---
    let dir = project(
        "crossopen",
        &wave("0001", "src/lib.rs", ""),
        &wave("0002", "src/lib.rs", ""),
    );
    for branch in ["main", "plan/0002-second", "0002-second"] {
        if branch != "main" {
            git(&dir, &["checkout", "-q", "-b", branch]);
        }
        let (said, code) = keel(&dir, &["check"]);
        assert_eq!(
            code, 1,
            "on {branch} the crossing is a finding (§8.8):\n{said}"
        );
        assert!(
            crossing_named(&said),
            "on {branch} it names the line and both waves:\n{said}"
        );
        // The finding lands on the LATER wave's file, and its instead
        // names both ways out (review 0054 R-4, R-5).
        let row = said
            .lines()
            .find(|l| l.contains("червоне") && l.contains("оголошують"))
            .expect("the red row of the crossing");
        assert!(
            row.contains("keel/waves/0002-second.md"),
            "the finding stands on the later wave's file:\n{row}"
        );
        let instead = said
            .lines()
            .find(|l| l.contains("натомість") && l.contains("§8.8"))
            .expect("the instead line of the crossing");
        assert!(
            instead.contains("назви залежність depends_on") && instead.contains("поділи файл"),
            "the instead names both ways out:\n{instead}"
        );
    }

    // --- the same in English (review 0054 R-4) ---
    let dir = project_in(
        "crossen",
        "en",
        &wave("0001", "src/lib.rs", ""),
        &wave("0002", "src/lib.rs", ""),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "the crossing is a finding in English too:\n{said}");
    assert!(
        said.contains("is declared by two open waves")
            && said.contains("§8.8")
            && said.contains("name a depends_on")
            && said.contains("divide the file"),
        "the English reason and instead say the same:\n{said}"
    );

    // --- the edge the other way round (review 0054 R-5): no finding ---
    let dir = project(
        "crossreverse",
        &wave("0001", "src/lib.rs", "depends_on: [0002-second]\n"),
        &wave("0002", "src/lib.rs", ""),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "an edge from the first to the second joins them too:\n{said}"
    );
    assert!(!crossing_named(&said), "and no crossing is named:\n{said}");

    // --- a chain through a third wave (review 0054 R-5): the first
    // and the third share a file, joined only through the second ---
    let dir = project(
        "crosschain",
        &wave("0001", "src/lib.rs", ""),
        &wave("0002", "src/other.rs", "depends_on: [0001-first]\n"),
    );
    std::fs::write(
        dir.join("keel/waves/0003-third.md"),
        wave("0003", "src/lib.rs", "depends_on: [0002-second]\n"),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "a third wave"]);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "waves joined through a chain do not cross:\n{said}"
    );
    assert!(
        !said.contains("0003-third.md — рядок"),
        "and no crossing is named on the third:\n{said}"
    );

    // --- two `one new in` over one directory cross; a `one new in`
    // beside a path in that directory does not (review 0054 R-5) ---
    let dir = project(
        "crossnewin",
        &wave("0001", "one new in src/", ""),
        &wave("0002", "one new in src/", ""),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 1,
        "two `one new in` over one directory cross:\n{said}"
    );
    assert!(
        said.contains("рядок scope \"one new in src/\""),
        "and the finding calls the line a scope line, not a file:\n{said}"
    );
    let dir = project(
        "crossbeside",
        &wave("0001", "one new in src/", ""),
        &wave("0002", "src/lib.rs", ""),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "a `one new in` beside a path is a different line:\n{said}"
    );

    // --- an edge between them: no finding ---
    let dir = project(
        "crossedge",
        &wave("0001", "src/lib.rs", ""),
        &wave("0002", "src/lib.rs", "depends_on: [0001-first]\n"),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "waves joined by depends_on may share a file:\n{said}"
    );
    assert!(!crossing_named(&said), "and no crossing is named:\n{said}");

    // --- different files: no finding ---
    let dir = project(
        "crossapart",
        &wave("0001", "src/lib.rs", ""),
        &wave("0002", "src/other.rs", ""),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "waves over different files do not cross:\n{said}");

    // --- the first wave closed structurally (its promise proven):
    // a closed wave's scope is history, not a crossing ---
    let dir = project(
        "crossclosed",
        &wave("0001", "src/lib.rs", ""),
        &wave("0002", "src/lib.rs", ""),
    );
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works-0001@{rev}\n#[test]\nfn it_works() {{}}\n"),
    )
    .unwrap();
    std::fs::write(dir.join("keel/reviews/0001-first.md"), "# Рецензія\n\nok\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the first is proven"]);
    let (said, _) = keel(&dir, &["status"]);
    assert!(
        said.contains("0001-first — закрита"),
        "the fixture's first wave is closed structurally:\n{said}"
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a closed wave does not cross an open one:\n{said}");
    assert!(!crossing_named(&said), "and no crossing is named:\n{said}");

    // --- the first wave cancelled: outside judgement (§6.3-а) ---
    let dir = project(
        "crosscancelled",
        &wave("0001", "src/lib.rs", "cancelled: \"передумали\"\n"),
        &wave("0002", "src/lib.rs", ""),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a cancelled wave does not cross anything:\n{said}");
    assert!(!crossing_named(&said), "and no crossing is named:\n{said}");
}
