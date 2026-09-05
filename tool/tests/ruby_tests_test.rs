//! Scenario test of wave 0038: ruby tests are read and run.

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

/// A ruby project with minitest: a promise, its test carrying the
/// tag, and a second test that fails on purpose.
fn project(name: &str, body: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works\n    true\n  end\nend\n",
    )
    .unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - lib/toy.rb\n{d}---\n\n## scenario: it-works\n{body}\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("test/toy_test.rb"), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

const BODY: &str = "тіло обіцянки\n\n";

fn test_file(rev: &str) -> String {
    format!(
        "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\nend\n"
    )
}

/// proves: ruby-tests-are-read-and-run@efd41a -- the concept named
/// four starting adapters, the operator's own decision, and none was
/// built: the two language-shaped courts ran for Rust alone. Ruby is
/// the second tongue, chosen by the operator on 2026-09-05.
#[test]
fn ruby_tests_are_read_and_run() {
    let rev = keel::rev::text_rev(BODY);
    let dir = project("rubytags", BODY, &test_file(&rev));

    // The tags are read from test/**/*_test.rb, and a matching one
    // is counted rather than skipped.
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a ruby project is judged whole:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "the proves tag is read from a ruby test file (§5.5):\n{said}"
    );

    // A tag whose revision no longer matches is stale, exactly as in
    // Rust: the court is the same, only the reading differs.
    let dir = project("rubystale", BODY, &test_file("beef00"));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "a stale ruby tag is a finding:\n{said}");
    assert!(
        said.contains("beef00") && said.contains("it-works"),
        "and it names the tag and the promise:\n{said}"
    );

    // The battery runs, and the closing court names what failed by
    // its ruby name -- Class#method.
    let falls = format!(
        "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\n\n  def test_it_falls\n    flunk \"навмисно\"\n  end\nend\n"
    );
    let dir = project("rubybattery", BODY, &falls);
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("test_it_falls"),
        "the closing court names the ruby test it watched fail:\n{said}"
    );
    assert!(
        said.contains("toy_test"),
        "with the file it lives in -- the same shape the rust courts \
         get, since nothing above the adapter knows the language:\n{said}"
    );
    assert!(
        !said.contains("test_it_works"),
        "and the green one is not dragged in with it:\n{said}"
    );
}
