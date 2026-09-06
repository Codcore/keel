//! Scenario test of wave 0053: a court stricter than the norm says so.
//!
//! The global review of 2026-09-06 (methodology R-17, R-18) measured
//! six courts judging beyond the letter of the norm in silence --
//! `graph-scenario-twice`, `graph-name-taken`, `graph-double-cover`,
//! `gate-case`, `rev-nearmiss`, `close-no-room` -- and the red birth's
//! word "truly fails" said alike for ruby, where a fall and a broken
//! build leave with one code.
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

fn gate(dir: &Path, message: &str) -> (String, i32) {
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, message).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
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

fn vocabulary(tongue: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("i18n")
            .join(format!("{tongue}.ftl")),
    )
    .unwrap()
}

fn entry<'a>(text: &'a str, key: &str) -> &'a str {
    text.lines()
        .find(|l| l.starts_with(&format!("{key} =")))
        .unwrap_or_else(|| panic!("the vocabulary carries {key}"))
}

/// proves: a-court-stricter-than-the-norm-says-so@fca33b
#[test]
fn a_court_stricter_than_the_norm_says_so() {
    // --- the six words say they are stricter than the letter, and
    // why, in both tongues ---
    let uk = vocabulary("uk");
    let en = vocabulary("en");
    for key in [
        "graph-scenario-twice",
        "graph-name-taken",
        "graph-double-cover",
        "gate-case",
        "rev-nearmiss",
        "close-no-room",
    ] {
        let word = entry(&uk, key);
        assert!(
            word.contains("суворіше за букву") && word.contains("бо"),
            "{key} (uk) says it is stricter than the letter, and why:\n{word}"
        );
        let word = entry(&en, key);
        assert!(
            word.contains("stricter than the letter") && word.contains("because"),
            "{key} (en) says it is stricter than the letter, and why:\n{word}"
        );
    }

    // --- the red birth's word carries the tongue's border: over
    // ruby a fall is not told from a broken build by the exit code
    // (§7.12); over rust it "truly fails", as before ---
    let rev = keel::rev::text_rev(BODY);
    let dir = crate_with("stricterrust", "rust", &plain_wave());
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    panic!(\"red\");\n}}\n"
        ),
    )
    .unwrap();
    settle(&dir);
    let (said, code) = gate(&dir, "red: it-works\n");
    assert_eq!(code, 0, "a red birth over rust passes:\n{said}");
    assert!(
        said.contains("справді падає") && !said.contains("не розрізняє"),
        "and rust's word is the plain one:\n{said}"
    );
    if common::machine_has("ruby").ready() {
        let dir = keel_sandbox("stricterruby");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy_test.rb"),
            format!("require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    flunk \"red on purpose\"\n  end\nend\n"),
        )
        .unwrap();
        std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
        std::fs::write(
            dir.join("keel/waves/0001-a-wave.md"),
            plain_wave().replace("      - src/lib.rs\n", "      - lib/toy.rb\n"),
        )
        .unwrap();
        std::fs::write(
            dir.join("keel/reviews/0001-a-wave.md"),
            "# Рецензія\n\nok\n",
        )
        .unwrap();
        settle(&dir);
        let (said, code) = gate(&dir, "red: it-works\n");
        assert_eq!(code, 0, "a red birth over ruby passes:\n{said}");
        assert!(
            said.contains("не розрізняє") && said.contains("§7.12"),
            "and ruby's word carries the border of its exit code:\n{said}"
        );
    }
}
