//! Scenario test of wave 0067: a name survives its runner.
//!
//! Measured in the field, release 1.2.0: `mix test --trace` prints a
//! test's name in whatever encoding Erlang takes from the locale.
//! Under `LANG=en_US.UTF-8` the bytes are UTF-8; with the locale
//! stripped they are latin1 -- and `String::from_utf8_lossy` turns
//! them into U+FFFD. The key of the battery then no longer matches
//! the name in the file, and the closing court says the battery did
//! not run a test it ran and passed.
//!
//! The hostile setting is put on the CHILD this probe launches, never
//! on the whole battery: otherwise on a machine that has a locale the
//! probe is born green and §6.3 has nothing to satisfy.

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

/// keel, with the locale taken away from THIS child alone.
fn keel_without_a_locale(dir: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
        .env_remove("LANG")
        .env_remove("LC_ALL")
        .env_remove("LC_CTYPE")
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
/// Above U+00FF on purpose: latin1 carries every character below it,
/// so a name of umlauts alone would come back whole from the wrong
/// decoding and prove nothing.
const NAME: &str = "імʼя живе";
/// python names a test by its FUNCTION, not by a docstring -- review
/// 0067 measured the first reading of this side putting the name
/// where pytest never prints it. A python identifier may carry these
/// letters, so the name goes there.
const NAME_PY: &str = "імʼя_живе";

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"elixir\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("mix.exs"),
        "defmodule Toy.MixProject do\n  use Mix.Project\n  def project do\n    [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\n  end\n  def application, do: []\nend\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("lib/toy.ex"),
        "defmodule Toy do\n  def works, do: true\nend\n",
    )
    .unwrap();
    std::fs::write(dir.join("test/test_helper.exs"), "ExUnit.start()\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - lib/toy.ex\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("test/toy_test.exs"),
        format!(
            "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"{NAME}\" do\n    assert Toy.works()\n  end\nend\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: a-non-ascii-name-survives-its-runner@724e9e
#[test]
fn a_non_ascii_name_survives_its_runner() {
    if !common::machine_has("mix").ready() {
        // Said aloud, not passed in silence: on a machine without the
        // runner this probe proves nothing, and the wave says so in
        // its own words rather than leaving a green mark behind.
        eprintln!("mix is not on this machine: the elixir road was not walked");
        return;
    }
    let dir = project("nameelixir");
    let (said, code) = keel_without_a_locale(&dir, &["close"]);
    let _ = code;
    // The wound, in the court's own words: the battery RAN the test
    // and it passed, and the verdict said it did not run it. Measured
    // before the work, in this very sandbox, with the locale taken
    // away from the child:
    //
    //   0001-a-wave: in progress -- the missing, by name:
    //     scenario "it-works": the battery did not run the test "імʼя живе"
    assert!(
        !said.contains("не виконала"),
        "the name goes from the source to the verdict whole, so the \
         battery's key matches and the court sees what it ran:\n{said}"
    );
    assert!(
        !said.contains("\\x{"),
        "and no escape of the runner's own making stands where a name \
         belongs:\n{said}"
    );
}

/// proves: a-non-ascii-name-survives-its-runner@724e9e
///
/// The elixir road is where §6.3 is satisfied; the others are green
/// from birth, and that is said aloud rather than hidden behind the
/// one red. What holds them is not a red birth but this probe and the
/// mutants of the work's review: break a hand's reading and one of
/// these sides falls.
#[test]
fn every_hand_carries_a_name() {
    // The name goes above U+00FF on every road, for the same reason
    // as above: latin1 carries everything below it whole.
    let roads: [(&str, &str, &str, &str, &str); 4] = [
        (
            "cargo",
            "cargo",
            "Cargo.toml",
            "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
            "tests/toy_test.rs",
        ),
        (
            "python",
            "pytest",
            "pyproject.toml",
            "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n",
            "tests/test_toy.py",
        ),
        (
            "javascript",
            "node",
            "package.json",
            "{\n  \"name\": \"toy\",\n  \"version\": \"1.0.0\"\n}\n",
            "test/toy.test.js",
        ),
        (
            "ruby",
            "rspec",
            "Gemfile",
            "source \"https://rubygems.org\"\ngem \"rspec\"\n",
            "spec/toy_spec.rb",
        ),
    ];
    for (tongue, runner, manifest, manifest_body, test_path) in roads {
        if !common::machine_has(runner).ready() {
            // Said aloud, never passed in silence: on a machine
            // without the runner this road was not walked, and the
            // wave names that as its sharpest limit.
            eprintln!("{runner} is not on this machine: the {tongue} road was not walked");
            continue;
        }
        let dir = keel_sandbox(&format!("name{tongue}"));
        std::fs::write(
            dir.join("keel.toml"),
            format!("lang = \"uk\"\nadapter = \"{tongue}\"\n"),
        )
        .unwrap();
        std::fs::write(dir.join(manifest), manifest_body).unwrap();
        let rev = keel::rev::text_rev(BODY);
        let name = if tongue == "python" || tongue == "cargo" {
            NAME_PY
        } else {
            NAME
        };
        if tongue == "cargo" {
            std::fs::create_dir_all(dir.join("src")).unwrap();
            std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
        }
        let body = match tongue {
            // A rust test function may carry these letters in its own
            // identifier, and that identifier IS the name the hand
            // reads.
            "cargo" => format!(
                "/// proves: it-works@{rev}\n#[test]\nfn {name}() {{\n    assert!(toy::works());\n}}\n"
            ),
            "python" => format!("# proves: it-works@{rev}\ndef test_{name}():\n    assert True\n"),
            "javascript" => format!(
                "const {{ test }} = require(\"node:test\");\nconst assert = require(\"node:assert\");\n\n// proves: it-works@{rev}\ntest(\"{name}\", () => {{\n  assert.ok(true);\n}});\n"
            ),
            _ => format!(
                "RSpec.describe \"toy\" do\n  # proves: it-works@{rev}\n  it \"{name}\" do\n    expect(true).to be true\n  end\nend\n"
            ),
        };
        let path = dir.join(test_path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, body).unwrap();
        let mut d = String::from("decisions:\n");
        for cut in keel::graph::cuts() {
            if *cut != "functional.correctness" {
                d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
            }
        }
        std::fs::write(
            dir.join("keel/waves/0001-a-wave.md"),
            format!(
                "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - {manifest}\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join("keel/reviews/0001-a-wave.md"),
            "# Рецензія\n\nok\n",
        )
        .unwrap();
        git(&dir, &["init", "-q", "-b", "main"]);
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "base"]);

        let (said, _) = keel_without_a_locale(&dir, &["close"]);
        // The wave holds a promise and the test carries its tag, so
        // the court has a key to match. A hand that reads the name
        // wrongly misses that key and the court says the battery did
        // not run a test it ran -- which is the whole wound, on every
        // road. Review 0067 measured the first reading of this side
        // passing with a hand deliberately broken: it asserted over
        // a green court, which prints no names at all.
        assert!(
            !said.contains("не виконала"),
            "the {tongue} hand carries a name above U+00FF whole, so \
             the battery's key matches the promise:\n{said}"
        );
        assert!(
            !said.contains('\u{FFFD}'),
            "and no replacement character stands in a name on the \
             {tongue} road:\n{said}"
        );
    }
}
