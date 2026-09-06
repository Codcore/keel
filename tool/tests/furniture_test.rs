//! Scenario test of wave 0052: furniture is known by its digest.
//!
//! The global review of 2026-09-06 (methodology R-6, R-7; wave 0050)
//! measured the scope court blind to the digest: `keel update` on a
//! work branch made drift out of `.claude/settings.json` and
//! `keel.toml`, files in the very form the tool leaves; a plan branch
//! of two commits, merged, had `keel review` call the second commit's
//! file "added after the anchor"; and a recorded digest stood foreign
//! for six waves while the text matched the release, and `keel check`
//! said nothing.
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

/// The generated files of a rust project, as `keel update` writes
/// them in this release: what the scope court must know as
/// furniture whenever their digest is the one the tool recorded.
const FURNITURE: [&str; 4] = [
    ".claude/settings.json",
    ".claude/skills/keel/SKILL.md",
    ".github/workflows/keel.yml",
    "AGENTS.md",
];

fn drift_line(file: &str) -> String {
    format!("гілка чіпає \"{file}\", якого жодна трансформа хвилі не називає")
}

/// The wave of one transform over src/lib.rs AND the probe file, so
/// the branch is lawful on every line but the one under test.
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

fn birth_and_work(dir: &Path) {
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    panic!(\"red\");\n}}\n"
        ),
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "--no-verify", "-m", "red: it-works"]);
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
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "--no-verify", "-m", "work: done"]);
}

fn hand_edit(dir: &Path) {
    let path = dir.join(".claude/settings.json");
    let mut text = std::fs::read_to_string(&path).unwrap();
    text.push_str("\n// edited by a hand\n");
    std::fs::write(&path, text).unwrap();
}

/// proves: furniture-is-known-by-its-digest@472b93
#[test]
fn furniture_is_known_by_its_digest() {
    // --- a work branch where `keel update` rewrote the generated
    // files and keel.toml: in the form the tool left, outside scope
    // (§4.8) ---
    let dir = crate_with(
        "furnupdate",
        "rust",
        &wave_over(&["src/lib.rs", "tests/w_test.rs"]),
    );
    settle(&dir);
    birth_and_work(&dir);
    let (said, code) = keel(&dir, &["update"]);
    assert_eq!(code, 0, "update writes the furniture:\n{said}");
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "work: keel update"],
    );
    let (said, code) = keel(&dir, &["check"]);
    for file in FURNITURE.iter().chain(["keel.toml"].iter()) {
        assert!(
            !said.contains(&drift_line(file)),
            "a file in the form the tool left is furniture, not drift (§4.8): {file}\n{said}"
        );
    }
    assert_eq!(code, 0, "and the branch is green:\n{said}");

    // --- the same file edited by a hand: code, and drift ---
    hand_edit(&dir);
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "work: a hand in the furniture",
        ],
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains(&drift_line(".claude/settings.json")),
        "a generated file edited by a hand is code, and undeclared code is drift:\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");
    assert!(
        said.contains("правлений рукою"),
        "and the stale record says the text is a hand's (review 0052 R-12):\n{said}"
    );
    // The court judges what the branch COMMITTED (§4.5): the tree put
    // back to the release's text does not clear the committed hand,
    // and a hand only in the tree is not the branch's drift (review
    // 0052 R-4).
    git(&dir, &["show", "HEAD~1:.claude/settings.json"]);
    let out = Command::new("git")
        .args(["show", "HEAD~1:.claude/settings.json"])
        .current_dir(&*dir)
        .output()
        .unwrap();
    std::fs::write(dir.join(".claude/settings.json"), out.stdout).unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains(&drift_line(".claude/settings.json")),
        "the committed hand is drift whatever the tree says now:\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");
    git(&dir, &["reset", "-q", "--hard", "HEAD~1"]);
    hand_edit(&dir);
    let (said, _) = keel(&dir, &["check"]);
    assert!(
        !said.contains(&drift_line(".claude/settings.json")),
        "a hand only in the tree is not the branch's drift:\n{said}"
    );
    git(&dir, &["checkout", "-q", "--", ".claude/settings.json"]);

    // --- the plan branch judges by the same digest: `keel update`
    // there is furniture, a hand there is code (§4.8, §4.9) ---
    let dir = crate_with("furnplan", "rust", &plain_wave());
    std::fs::remove_file(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    std::fs::remove_file(dir.join("keel/reviews/0001-a-wave.md")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), plain_wave()).unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "plan: wave 0001"],
    );
    let (said, code) = keel(&dir, &["update"]);
    assert_eq!(code, 0, "update writes the furniture:\n{said}");
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "plan: keel update"],
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        !said.contains("план-гілка несе план, а не код"),
        "furniture on the plan branch is not code:\n{said}"
    );
    assert_eq!(code, 0, "and the plan branch is green:\n{said}");
    hand_edit(&dir);
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "plan: a hand in the furniture",
        ],
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains(".claude/settings.json — план-гілка несе план, а не код"),
        "a generated file edited by a hand is code on the plan branch too:\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");

    // --- the anchor of a full wave is the wave file at the fork
    // point with main: a plan branch of two commits, merged, is one
    // plan (§4.6). Full by §6.8 -- two transforms -- so it rides a
    // plan branch and a plan PR ---
    let full = |files: &[&str]| {
        wave_over(files).replacen(
            "transforms:\n",
            "transforms:\n  tidy:\n    chore: \"прибирання\"\n    files:\n      - README.md\n",
            1,
        ) + "## transform: tidy\nтіло\n"
    };
    let dir = crate_with("furnanchor", "rust", &plain_wave());
    std::fs::remove_file(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), full(&["src/lib.rs"])).unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "plan: wave 0001"],
    );
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        full(&["src/lib.rs", "Cargo.toml"]),
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
            "plan: Cargo.toml added before the merge",
        ],
    );
    git(&dir, &["checkout", "-q", "main"]);
    git(
        &dir,
        &[
            "merge",
            "-q",
            "--no-ff",
            "--no-verify",
            "-m",
            "merge plan",
            "plan/0001-a-wave",
        ],
    );
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// work\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "work: done"]);
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(code, 0, "the package assembles:\n{said}");
    assert!(
        !said.contains("Cargo.toml — дописаний після якоря"),
        "a file planned before the plan merged is no drift:\n{said}"
    );
    assert!(
        said.contains("точці розгалуження"),
        "and the package names the anchor as the fork point with main:\n{said}"
    );

    // --- a light wave has no plan PR: its anchor is its first commit ---
    let dir = crate_with("furnlight", "rust", &plain_wave());
    std::fs::remove_file(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let light = |files: &str| {
        format!(
            "---\ntransforms:\n  tidy:\n    chore: \"прибирання\"\n    files:\n{files}{}---\n\n## transform: tidy\nтіло\n",
            decisions_except(&[])
        )
    };
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        light("      - src/lib.rs\n"),
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
            "tidy: the wave is born light",
        ],
    );
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        light("      - src/lib.rs\n      - Cargo.toml\n"),
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
            "tidy: Cargo.toml added after the birth",
        ],
    );
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(code, 0, "the package assembles:\n{said}");
    assert!(
        said.contains("Cargo.toml — дописаний після якоря"),
        "on a light wave the anchor is its first commit, and growth after it is drift:\n{said}"
    );

    // --- a recorded digest gone stale while the text matches the
    // release: `keel check` names the file ---
    let dir = crate_with("furnstale", "rust", &plain_wave());
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    let (said, code) = keel(&dir, &["update"]);
    assert_eq!(code, 0, "update writes the furniture:\n{said}");
    let toml = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    let recorded = toml
        .lines()
        .find(|l| l.contains("\".claude/settings.json\""))
        .and_then(|l| l.split('"').nth(3))
        .expect("the digest of settings.json is recorded")
        .to_string();
    std::fs::write(
        dir.join("keel.toml"),
        toml.replacen(&recorded, "000000000000", 1),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "the record goes stale"],
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains(".claude/settings.json") && said.contains("000000000000"),
        "a stale recorded digest is named with the file, even when the text matches the release:\n{said}"
    );
    assert!(
        said.contains("keel update") && said.contains("реліз"),
        "and its instead says the text is the release's, to be re-recorded (review 0052 R-12):\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");

    // --- a light wave keeps its first commit even where its file
    // changed on main before the branch: the fork point would hide
    // that growth (review 0052 R-12, M25) ---
    let dir = crate_with("furnlightmain", "rust", &plain_wave());
    std::fs::remove_file(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    let light = |files: &str| {
        format!(
            "---\ntransforms:\n  tidy:\n    chore: \"прибирання\"\n    files:\n{files}{}---\n\n## transform: tidy\nтіло\n",
            decisions_except(&[])
        )
    };
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        light("      - src/lib.rs\n"),
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
            "the light wave, on main",
        ],
    );
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        light("      - src/lib.rs\n      - Cargo.toml\n"),
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
            "Cargo.toml added on main",
        ],
    );
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// tidy\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "tidy: work"]);
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(code, 0, "the package assembles:\n{said}");
    assert!(
        said.contains("Cargo.toml — дописаний після якоря") && !said.contains("точці розгалуження"),
        "a light wave's anchor is its first commit, not the fork point:\n{said}"
    );
}
