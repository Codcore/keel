//! Scenario test of wave 0050: a vanished tag is red in every tongue.
//!
//! §7.15 -- a tag that stood at the fork point and is gone at HEAD
//! while its scenario lives -- was judged for rust alone: the court
//! asked `crate_root` for `<crate>/tests` and, refused in every other
//! tongue, returned nothing. The methodology cut of the global review
//! (R-1) and the bugs cut (R-12) measured python, ruby, elixir and
//! javascript deleting a proven test in silence while the summary
//! line still claimed the §7.15 court. No runner is needed here: the
//! court reads files out of git, and the tag readers are pure.
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

/// One tongue: the project's frame, a proven wave on main with its
/// tag in `test_path`, and a branch that deletes that file.
fn disarmed(name: &str, adapter: &str, source: (&str, &str), test_path: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    let (source_path, source_text) = source;
    std::fs::create_dir_all(dir.join(source_path).parent().unwrap()).unwrap();
    std::fs::write(dir.join(source_path), source_text).unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - {source_path}\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::create_dir_all(dir.join(test_path).parent().unwrap()).unwrap();
    std::fs::write(dir.join(test_path), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base with the tag"]);
    git(&dir, &["checkout", "-q", "-b", "feature"]);
    std::fs::remove_file(dir.join(test_path)).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the test quietly deleted"]);
    dir
}

/// proves: a-vanished-tag-is-red-in-every-tongue@e4b217
#[test]
fn a_vanished_tag_is_red_in_every_tongue() {
    let rev = keel::rev::text_rev(BODY);
    let cases: Vec<(&str, &str, (&str, &str), &str, String)> = vec![
        (
            "vanpy",
            "python",
            ("src/toy/__init__.py", "def works():\n    return True\n"),
            "tests/test_toy.py",
            format!("from toy import works\n\n\n# proves: it-works@{rev}\ndef test_it_works():\n    assert works()\n"),
        ),
        (
            "vanrb",
            "ruby",
            ("lib/toy.rb", "module Toy\n  def self.works\n    true\n  end\nend\n"),
            "test/toy_test.rb",
            format!("require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\nend\n"),
        ),
        (
            "vanex",
            "elixir",
            ("lib/toy.ex", "defmodule Toy do\n  def works, do: true\nend\n"),
            "test/toy_test.exs",
            format!("defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert Toy.works()\n  end\nend\n"),
        ),
        (
            "vanjs",
            "javascript",
            ("src/toy.js", "export function works() {\n  return true;\n}\n"),
            "test/toy.test.js",
            format!("import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('it works', () => {{\n  assert.ok(works());\n}});\n"),
        ),
    ];
    for (name, adapter, source, test_path, test_body) in cases {
        let dir = disarmed(name, adapter, source, test_path, &test_body);
        let (said, code) = check(&dir);
        assert_eq!(
            code, 1,
            "{adapter}: a vanished tag of a live scenario is red (§7.15):\n{said}"
        );
        assert!(
            said.contains("\"it-works\"") && said.contains("точці розгалуження"),
            "{adapter}: the scenario is named where the disarming happened:\n{said}"
        );
        assert!(
            said.contains(test_path),
            "{adapter}: and so is the file that carried the tag:\n{said}"
        );
    }
}
