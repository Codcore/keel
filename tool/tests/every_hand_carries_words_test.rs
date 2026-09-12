//! Scenario test of wave 0071: a red battery carries the words that
//! made it red -- on every road keel drives a foreign runner down.
//!
//! Issues #49 and #52. The author of #52 re-ran a flaky test about
//! twenty times by hand and never once saw the failure keel had seen:
//! the court names the test and keeps nothing of what it said.
//!
//! One probe per road, on purpose. This session found two class
//! defects at exactly these seams -- latin letters in elixir under an
//! empty locale, ASCII escaping in pytest -- and a change made in the
//! shared part breaks, in silence, whichever road nobody measured.

mod common;

use common::{keel_sandbox, machine_has};
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
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

fn closing(dir: &Path) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
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

/// A chore wave with no promises, its branch having done the work.
fn frame(dir: &Path, adapter: &str, touched: &str) {
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        d.push_str(&format!("  {cut}: \"не про цю пісочницю, вона грає інше\"\n"));
    }
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"робота без обіцянок\"\n    files:\n      - {touched}\n{d}---\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    common::did_the_work(dir);
}

/// The words a failing test said, which the court must carry: a
/// marker no keel line could produce by itself.
const MARK: &str = "the-assertion-that-fell";

/// proves: a-red-battery-carries-the-words-that-made-it-red@PLACEHOLDER
#[test]
fn a_red_battery_carries_the_words_that_made_it_red() {
    // --- cargo ----------------------------------------------------
    let dir = keel_sandbox("wordscargo");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        format!("#[test]\nfn it_falls() {{\n    assert!(false, \"{MARK}\");\n}}\n"),
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_ne!(code, 0, "a red battery holds the wave open:\n{said}");
    assert!(
        said.contains("it_falls"),
        "cargo: the court names the test (it has since v1.0.0):\n{said}"
    );
    assert!(
        said.contains(MARK),
        "cargo: and carries the words that made it red. Naming the \
         test without them is issue #52: its author re-ran the test \
         about twenty times and never saw the failure keel saw:\n{said}"
    );

    // --- a green battery still says nothing extra -----------------
    let dir = keel_sandbox("wordsquiet");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        "#[test]\nfn it_stands() {\n    assert!(true);\n}\n",
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_eq!(code, 0, "a green battery closes:\n{said}");
    assert!(
        !said.contains("stdout") && !said.contains("панік"),
        "and carries no quotation at all -- the words belong to a red \
         verdict, not to every verdict:\n{said}"
    );

    // --- minitest -------------------------------------------------
    if machine_has("ruby").ready() {
        let dir = keel_sandbox("wordsminitest");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("test")).unwrap();
        fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        fs::write(
            dir.join("test/toy_test.rb"),
            format!(
                "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_it_falls\n    flunk \"{MARK}\"\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "minitest: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "minitest: and carries the words. This is the road issue \
             #49 was reported from -- rails and minitest:\n{said}"
        );
    }

    // --- rspec ----------------------------------------------------
    if machine_has("rspec").ready() {
        let dir = keel_sandbox("wordsrspec");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("spec")).unwrap();
        fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        fs::write(
            dir.join("spec/toy_spec.rb"),
            format!(
                "RSpec.describe \"Toy\" do\n  it \"falls\" do\n    raise \"{MARK}\"\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "rspec: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "rspec: and carries the words -- from its own JSON, which \
             is the one road where the raw voice does not exist:\n{said}"
        );
    }

    // --- elixir ---------------------------------------------------
    if machine_has("mix").ready() {
        let dir = keel_sandbox("wordselixir");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("test")).unwrap();
        fs::write(
            dir.join("mix.exs"),
            "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\nend\n",
        )
        .unwrap();
        fs::write(dir.join("lib/toy.ex"), "defmodule Toy do\nend\n").unwrap();
        fs::write(dir.join("test/test_helper.exs"), "ExUnit.start()\n").unwrap();
        fs::write(
            dir.join("test/toy_test.exs"),
            format!(
                "defmodule ToyTest do\n  use ExUnit.Case\n\n  test \"it falls\" do\n    flunk(\"{MARK}\")\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "elixir", "lib/toy.ex");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "elixir: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "elixir: and carries the words:\n{said}"
        );
    }

    // --- javascript -----------------------------------------------
    if machine_has("node").ready() {
        let dir = keel_sandbox("wordsnode");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("test")).unwrap();
        fs::write(dir.join("package.json"), "{\n  \"name\": \"toy\"\n}\n").unwrap();
        fs::write(dir.join("src/toy.js"), "module.exports = {};\n").unwrap();
        fs::write(
            dir.join("test/toy.test.js"),
            format!(
                "const {{ test }} = require('node:test');\nconst assert = require('node:assert');\n\ntest('it falls', () => {{\n  assert.fail('{MARK}');\n}});\n"
            ),
        )
        .unwrap();
        frame(&dir, "javascript", "src/toy.js");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "node: a red battery holds the wave:\n{said}");
        assert!(said.contains(MARK), "node: and carries the words:\n{said}");
    }

    // --- python ---------------------------------------------------
    if machine_has("pytest").ready() {
        let dir = keel_sandbox("wordspytest");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("tests")).unwrap();
        fs::write(dir.join("src/toy.py"), "def a():\n    return True\n").unwrap();
        fs::write(
            dir.join("tests/test_toy.py"),
            format!("def test_it_falls():\n    assert False, \"{MARK}\"\n"),
        )
        .unwrap();
        frame(&dir, "python", "src/toy.py");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "pytest: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "pytest: and carries the words:\n{said}"
        );
    }
}
