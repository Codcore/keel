//! Scenario test of wave 0051: the readers hold their mutants.
//!
//! The test-completeness cut of the global review (2026-09-06, R-4 to
//! R-7, R-13 to R-15) replayed mutations that survived the whole
//! battery -- four rules of the readers held by code alone -- and
//! named three promises of the javascript hand no probe plays. Each is
//! played here with a real project, so the mutant reddens a probe: a
//! dangling tag with code between it and the declaration; a signature
//! found only past a word boundary; the §6.5 window held by a plan AND
//! a started wave; a `#` inside a ruby string on the declaration line;
//! the adapter's synonyms; a file before `index`; the four borders
//! `keel check` names for node.
//!
//! Born green by the named exception of §6.3: this is a court over our
//! own battery, and the commit that births it records each mutant.
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

/// A project of the given tongue: keel.toml, one wave `0001-a-wave`
/// with scenario `it-works` and transform `work` over `file`, the
/// review, git on main.
fn project(name: &str, adapter: &str, file: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - {file}\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
            decisions_except(&["functional.correctness"])
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    dir
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
}

/// proves: the-readers-hold-their-mutants@cb30ef
#[test]
fn the_readers_hold_their_mutants() {
    let rev = keel::rev::text_rev(BODY);

    // --- T2: code between a tag and its declaration is a dangling
    // tag, not a tag over the next fn ---
    let text = format!(
        "/// proves: it-works@{rev}\nconst BETWEEN: u8 = 0;\n#[test]\nfn it_works() {{}}\n"
    );
    let refused = keel::tags::scan_text(Path::new("tests/w_test.rs"), &text)
        .err()
        .expect("code between the tag and the fn is refused (tests R-4)");
    assert!(
        refused.reason.contains("одразу за собою") || refused.reason.contains("right after"),
        "and the refusal says the tag holds nothing right after it: {}",
        refused.reason
    );

    // --- H1: a signature is found at a word boundary BEFORE it too:
    // `xpub fn works` does not hold `pub fn works` ---
    let dir = project("mutbound", "rust", "src/lib.rs");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    // The form court reads TEXT, and this file is never built: the
    // promised signature stands here only glued to a letter before
    // it -- `mypub fn works() -> bool` -- which a reader without the
    // boundary before the match takes for the promise.
    std::fs::write(
        dir.join("src/lib.rs"),
        "mypub fn works() -> bool { true }\npub fn works_not() -> bool { true }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: toy\nexports:\n  - \"pub fn works() -> bool\"\n---\n\nформа\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{}}\n"),
    )
    .unwrap();
    settle(&dir);
    let (said, code) = check(&dir);
    assert!(
        said.contains("обіцяє \"works\"") || said.contains("обіцяє \"pub fn works() -> bool\""),
        "a signature glued to letters before it is not held (tests R-5):\n{said}"
    );
    assert_eq!(code, 1, "and the form court is red:\n{said}");

    // --- H8: the window of §6.5 forgives only a contract held by
    // plans alone -- a plan AND a started wave together are judged ---
    let dir = project("mutwindow", "rust", "src/lib.rs");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(
        dir.join("keel/contracts/grown.md"),
        "---\nmodule: toy\nexports:\n  - \"pub fn later() -> bool\"\n---\n\nобіцянка\n",
    )
    .unwrap();
    let grown_rev = keel::rev::contract_rev(&dir.join("keel/contracts/grown.md")).unwrap();
    // The started wave 0001 holds the contract by its scenario's
    // `proves`, and its tag exists; the plan 0002 holds it too.
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    proves: grown@{grown_rev}\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
            decisions_except(&["functional.correctness"])
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0002-plan.md"),
        format!(
            "---\nscenarios:\n  later:\n    proves: grown@{grown_rev}\n    covers: [functional.correctness]\ntransforms:\n  work2:\n    implements:\n      - later\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: later\n\nтіло пізнішої\n\n## transform: work2\nтіло\n",
            decisions_except(&["functional.correctness"])
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{}}\n"),
    )
    .unwrap();
    settle(&dir);
    let (said, code) = check(&dir);
    assert!(
        said.contains("обіцяє \"later\""),
        "a contract a started wave holds is judged, plan or no plan (tests R-6):\n{said}"
    );
    assert!(
        !said.contains("форма не судиться"),
        "and the window is not opened for it:\n{said}"
    );
    assert_eq!(code, 1, "the form court is red:\n{said}");

    // --- H9: `#` inside a ruby string on the declaration line is not
    // a comment, and the signature after it still holds ---
    let dir = project("mutrubyhash", "ruby", "lib/toy.rb");
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works(url = \"http://x#anchor\", verbose = false)\n    true\n  end\nend\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: Toy\nexports:\n  - \"def self.works(url = \\\"http://x#anchor\\\", verbose = false)\"\n---\n\nформа\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("test/toy_test.rb"),
        format!("require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\nend\n"),
    )
    .unwrap();
    settle(&dir);
    let (said, code) = check(&dir);
    assert!(
        !said.contains("обіцяє"),
        "a `#` inside the string keeps the rest of the line (tests R-7):\n{said}"
    );
    assert_eq!(code, 0, "and the form holds:\n{said}");

    // --- javascript: the adapter's synonyms name the same hand ---
    for synonym in ["typescript", "node", "js", "ts"] {
        let dir = project(&format!("mutsyn{synonym}"), synonym, "src/toy.js");
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("package.json"),
            "{ \"name\": \"toy\", \"version\": \"0.1.0\", \"type\": \"module\" }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("src/toy.js"),
            "export function works() {\n  return true;\n}\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy.test.js"),
            format!("import {{ test }} from 'node:test';\n\n// proves: it-works@{rev}\ntest('it works', () => {{}});\n"),
        )
        .unwrap();
        settle(&dir);
        let (said, code) = check(&dir);
        assert_eq!(
            code, 0,
            "{synonym}: the project is judged as javascript:\n{said}"
        );
        assert!(
            said.contains("тегів тестів звірено: 1"),
            "{synonym}: and its tag is read (tests R-13):\n{said}"
        );
    }

    // --- javascript: `src/toy.js` stands before `src/toy/index.js` ---
    let dir = project("mutindex", "javascript", "src/toy.js");
    std::fs::create_dir_all(dir.join("src/toy")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("package.json"),
        "{ \"name\": \"toy\", \"version\": \"0.1.0\", \"type\": \"module\" }\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/toy.js"),
        "export function works() {\n  return true;\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/toy/index.js"),
        "export function other() {\n  return false;\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: toy\nexports:\n  - \"export function works()\"\n---\n\nформа\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("test/toy.test.js"),
        format!("import {{ test }} from 'node:test';\n\n// proves: it-works@{rev}\ntest('it works', () => {{}});\n"),
    )
    .unwrap();
    settle(&dir);
    let (said, code) = check(&dir);
    assert_eq!(
        code, 0,
        "the file before index holds the form (tests R-14):\n{said}"
    );
    assert!(
        said.contains("сигнатур звірено: 1"),
        "and one signature was compared:\n{said}"
    );

    // --- javascript: the four borders `keel check` names ---
    let (said, _) = check(&dir);
    for border in [
        "код не питається ніколи",
        "node збирає ширше",
        "два тести з одним іменем у різних describe",
        "jest/vitest/mocha не читаються",
    ] {
        assert!(
            said.contains(border),
            "the border «{border}» is said aloud (tests R-15):\n{said}"
        );
    }
}
