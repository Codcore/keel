//! Scenario test of wave 0053: the word is the court's word.
//!
//! The global review of 2026-09-06 (methodology R-12, R-13, R-15,
//! R-16) measured the courts saying otherwise than they judge: review
//! and map assembling for a cancelled wave without the word; an empty
//! review file "time for the PR" in `next` and "not a review" in
//! `close`; the blockers of a light wave called a full wave's; the
//! Ukrainian usage line without `--for`; refusals in English past
//! the vocabulary; a closing price measured in another generation.
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

fn chore_wave() -> String {
    format!(
        "---\ntransforms:\n  tidy:\n    chore: \"прибирання\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
        decisions_except(&[])
    )
}

fn source(rel: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(rel),
    )
    .unwrap()
}

/// proves: the-word-is-the-courts-word@bbd6da
#[test]
fn the_word_is_the_courts_word() {
    // --- the branch of a cancelled wave: review and map say so, as
    // check does (§6.3-а) ---
    let cancelled = plain_wave().replacen(
        "---\nscenarios:",
        "---\ncancelled: \"передумали: обіцянку закриє інша хвиля\"\nscenarios:",
        1,
    );
    let dir = crate_with("wordcancelled", "rust", &cancelled);
    settle(&dir);
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(
        code, 0,
        "a cancelled wave is judged no more, not refused:\n{said}"
    );
    assert!(
        said.contains("скасован") && said.contains("§6.3"),
        "review says the wave is cancelled, with the paragraph:\n{said}"
    );
    let (said, _) = keel(&dir, &["map"]);
    assert!(
        said.contains("скасован"),
        "and the map of its branch says so too:\n{said}"
    );

    // --- an empty review file is one word in every court (§9.9) ---
    let dir = crate_with("wordempty", "rust", &chore_wave());
    std::fs::write(dir.join("keel/reviews/0001-a-wave.md"), "").unwrap();
    settle(&dir);
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// tidy\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "tidy: work"]);
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        !said.contains("час PR") && said.contains("порожній"),
        "`next` does not lead an empty report to the PR -- it names the empty file:\n{said}"
    );
    let (said, _) = keel(&dir, &["status"]);
    assert!(
        said.contains("порожній файл не рецензія"),
        "and status says the same word:\n{said}"
    );

    // --- the blockers of a light wave are called by its weight ---
    let dir = crate_with("wordlight", "rust", &chore_wave());
    std::fs::remove_file(dir.join("keel/reviews/0001-a-wave.md")).unwrap();
    settle(&dir);
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// tidy\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "tidy: work"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 1,
        "the missing report blocks the light wave's own branch:\n{said}"
    );
    assert!(
        said.contains("блокер") && !said.contains("повна хвиля"),
        "and the blockers line does not call it a full wave:\n{said}"
    );

    // --- the usage line has one shape in both tongues ---
    let uk = source("tool/i18n/uk.ftl");
    let en = source("tool/i18n/en.ftl");
    let usage = |text: &str| {
        text.lines()
            .find(|l| l.starts_with("main-usage ="))
            .map(|l| l.matches("--for").count())
            .unwrap_or(0)
    };
    assert_eq!(
        usage(&uk),
        usage(&en),
        "both tongues name `keel next --for` alike"
    );
    assert!(usage(&uk) > 0, "and the usage line names `--for` at all");

    // --- no refusal speaks past the vocabulary: every reason and
    // instead in the sources goes through i18n. The one named border
    // is config.rs, where the tongue is not read yet. ---
    let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut literal: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&src).unwrap().flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.extension().is_none_or(|e| e != "rs") || name == "config.rs" {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        for (at, line) in text.lines().enumerate() {
            let trimmed = line.trim();
            let field = trimmed
                .strip_prefix("reason: ")
                .or_else(|| trimmed.strip_prefix("instead: "));
            if let Some(value) = field
                && (value.starts_with('"') || value.starts_with("format!("))
            {
                literal.push(format!("{name}:{}", at + 1));
            }
        }
    }
    assert!(
        literal.is_empty(),
        "refusals past the vocabulary (review 0051 R-16): {literal:?}"
    );

    // --- the closing price is the measured one: the word carries the
    // constant, and the constant is what a closure leaves ---
    for (tongue, text) in [("uk", &uk), ("en", &en)] {
        let price = text
            .lines()
            .find(|l| l.starts_with("close-price ="))
            .expect("the price word exists");
        assert!(
            !price.contains("1,26") && !price.contains("1.26"),
            "{tongue}: the price word carries no number of another generation:\n{price}"
        );
        assert!(
            price.contains("{ $needed }"),
            "{tongue}: the price word names the constant"
        );
    }
    let dir = crate_with("wordprice", "rust", &chore_wave());
    settle(&dir);
    let (said, _) = keel(&dir, &["close"]);
    let price = said
        .lines()
        .find(|l| l.starts_with("ціна цього суду"))
        .expect("close names its price first");
    assert!(
        price.contains("~3 ГіБ"),
        "the constant is the measured three gibibytes:\n{price}"
    );
}
