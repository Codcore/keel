//! Scenario tests of wave 0052: a transform is closed by its commit,
//! and two waves with one number are red.
//!
//! The global review of 2026-09-06 (methodology R-9, R-11) measured
//! §6.2 unread -- a branch with all its work in one `wip:` commit was
//! "time for the PR" -- and §8.8 judged by `keel plan` alone, so two
//! wave files with one number were zero findings in `keel check`.
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

/// A rust crate: Cargo.toml, src/lib.rs, tests/ -- and the frame of
/// the methodology with the wave text given.
fn crate_with(name: &str, adapter: &str, wave_text: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), wave_text).unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    dir
}

/// The plain full wave: scenario `it-works`, transform `work` over
/// src/lib.rs.
fn plain_wave() -> String {
    format!(
        "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    )
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

fn wave_over(files: &[&str]) -> String {
    let mut list = String::new();
    for f in files {
        list.push_str(&format!("      - {f}\n"));
    }
    format!(
        "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n{list}{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    )
}

/// proves: a-transform-is-closed-by-its-commit@a57982
#[test]
fn a_transform_is_closed_by_its_commit() {
    let dir = crate_with(
        "slugwip",
        "rust",
        &wave_over(&["src/lib.rs", "tests/w_test.rs"]),
    );
    settle(&dir);
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    panic!(\"red\");\n}}\n"
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "red: it-works"],
    );
    // All the work of the transform, under a subject that is not
    // its slug.
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// work\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(toy::works());\n}}\n"),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "wip: everything at once",
        ],
    );

    let (said, _) = keel(&dir, &["next"]);
    assert!(
        said.contains("`work: <слова>`") && said.contains("§6.2"),
        "the step is the commit of the transform under its slug (§6.2):\n{said}"
    );
    assert!(
        !said.contains("час рецензії") && !said.contains("час PR"),
        "and not the review or the PR:\n{said}"
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains("§6.2") && said.contains("\"work\""),
        "the transform without a commit under its slug is a finding by the paragraph:\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");

    // The commit under the slug closes the transform, and several of
    // them are allowed (§2.4).
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// work\n// more\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "work: the rest of it"],
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        !said.contains("§6.2"),
        "a commit under the slug closes it:\n{said}"
    );
    assert_eq!(code, 0, "and the branch is green:\n{said}");
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        said.contains("час PR"),
        "and the step moves on -- the review record stands, so the PR:\n{said}"
    );

    // On main the discipline of the branch is not read (§6.5):
    // history is judged by its consequences.
    git(&dir, &["reset", "-q", "--hard", "HEAD~1"]);
    git(&dir, &["checkout", "-q", "main"]);
    git(
        &dir,
        &[
            "merge",
            "-q",
            "--no-ff",
            "--no-verify",
            "-m",
            "merge",
            "0001-a-wave",
        ],
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        !said.contains("§6.2"),
        "on main a missing slug commit is nobody's finding:\n{said}"
    );
    assert_eq!(code, 0, "main is green:\n{said}");
}

/// proves: two-waves-with-one-number-are-red@60bf75
#[test]
fn two_waves_with_one_number_are_red() {
    let dir = crate_with("twonumbers", "rust", &plain_wave());
    std::fs::write(
        dir.join("keel/waves/0001-b-wave.md"),
        plain_wave()
            .replace("it-works", "it-also-works")
            .replace("  work:", "  more:")
            .replace("## transform: work", "## transform: more"),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-b-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains("§8.8") && said.contains("0001-a-wave") && said.contains("0001-b-wave"),
        "two waves with one number are a finding naming both files (§8.8):\n{said}"
    );
    assert!(
        said.contains("0002"),
        "and the next free number, by the hand of `keel plan`:\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");
    let (said, code) = keel(&dir, &["plan", "0001-c-wave"]);
    assert_ne!(code, 0, "`keel plan` refuses the taken number:\n{said}");
    assert!(
        said.contains("0002"),
        "with the same next free number:\n{said}"
    );
    // The branches count for the next free number too (review 0052
    // R-12, M19): a branch named 0002-other moves it to 0003.
    git(&dir, &["branch", "0002-other"]);
    let (said, _) = keel(&dir, &["check"]);
    assert!(
        said.contains("0003"),
        "the next free number reads the branches, as `keel plan` does:\n{said}"
    );
}
