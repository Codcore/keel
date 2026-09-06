//! Scenario test of wave 0055: a skipped test proves nothing.
//!
//! These run a real `ruby` and a real `mix`. Where a tongue is not on
//! the machine the probe says so and stops; under a declared runner
//! (`CI` set) a missing tongue is a fall, not a skip (wave 0053).
//!
//! Python and node learned this in waves 0045 and 0046: a runner that
//! leaves with 0 over a test it did not run is saying "did not run",
//! not "green". Ruby's minitest and Elixir's ExUnit were still read
//! by the exit code alone -- the final review of 2026-09-06 (bugs
//! R-1, R-2, R-23, R-25) measured a `skip` blessing work at the gate
//! and a `@tag :skip` closing a wave with a phantom third test.
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

/// `keel gate` over a commit message, the way the hook calls it.
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

/// The one wave every fixture here carries: a promise, and a
/// transform over the module the test exercises.
fn wave(dir: &Path, file: &str) {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - {file}\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
}

/// A ruby project with minitest: the tagged test is SKIPPED, a plain
/// one runs green.
fn ruby_project(name: &str, rev: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works\n    true\n  end\nend\n",
    )
    .unwrap();
    wave(&dir, "lib/toy.rb");
    std::fs::write(
        dir.join("test/toy_test.rb"),
        format!(
            "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    skip \"later\"\n    assert Toy.works\n  end\n\n  def test_plain\n    assert true\n  end\nend\n"
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    dir
}

/// A mix project: the tagged test carries the mark given -- `@tag
/// :skip`, or a tag the project excludes in its own test_helper --
/// and a plain one runs green.
fn elixir_project(name: &str, rev: &str) -> common::Sandbox {
    elixir_project_marked(name, rev, "@tag :skip", "ExUnit.start()\n")
}

fn elixir_project_marked(name: &str, rev: &str, mark: &str, helper: &str) -> common::Sandbox {
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
    std::fs::write(dir.join("test/test_helper.exs"), helper).unwrap();
    wave(&dir, "lib/toy.ex");
    std::fs::write(
        dir.join("test/toy_test.exs"),
        format!(
            "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  {mark}\n  test \"it works\" do\n    assert Toy.works()\n  end\n\n  test \"plain\" do\n    assert true\n  end\nend\n"
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    dir
}

/// The review file, committed on the wave's branch, so the closing
/// court has everything but a proven promise.
fn reviewed(dir: &Path) {
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "review"]);
}

/// What both courts must say over a tongue whose tagged test was
/// skipped: the same word -- "did not run" -- and the same count.
fn both_courts_say_not_run(dir: &Path, test: &str, tongue: &str) {
    // The gate over work: the runner leaves with 0, and 0 here means
    // "did not run", not green.
    let (said, code) = gate(dir, "work: тіло\n");
    assert_ne!(
        code, 0,
        "{tongue}: work over a SKIPPED test is not blessed -- the \
         runner's 0 means \"did not run\", not green:\n{said}"
    );
    assert!(
        !said.contains("робота проходить"),
        "{tongue}: and it does not say the work passes:\n{said}"
    );
    assert!(
        said.contains(&format!("не виконав жодного тесту \"{test}\"")),
        "{tongue}: the gate says the test did not run, in the word \
         python and node already use:\n{said}"
    );

    // The closing court: the skipped test is neither red nor green,
    // the battery holds only what ran, and the promise is a lack
    // said as "did not run" -- the same word as the gate's.
    reviewed(dir);
    let (said, code) = keel(dir, &["close"]);
    assert_ne!(
        code, 0,
        "{tongue}: a wave whose promise did not run does not close:\n{said}"
    );
    assert!(
        !said.contains("закрита"),
        "{tongue}: and is not called closed:\n{said}"
    );
    assert!(
        !said.contains("червоний тест") && !said.contains("падав"),
        "{tongue}: a skipped test is not a RED test -- it did not run \
         at all:\n{said}"
    );
    assert!(
        said.contains("батарея: 1 тестів"),
        "{tongue}: the battery holds only what ran -- the plain test, \
         and no phantom:\n{said}"
    );
    assert!(
        !said.contains("(skipped)"),
        "{tongue}: the runner's own mark of a skip is not a test's \
         name:\n{said}"
    );
    assert!(
        said.contains(&format!("не виконала тесту \"{test}\"")),
        "{tongue}: and the claimed, skipped test is a lack said as \
         \"did not run\":\n{said}"
    );
}

/// proves: a-skipped-test-proves-nothing@5e0fe7 -- ruby `skip` and
/// elixir `@tag :skip` left with 0, and both courts read the 0 as a
/// verdict: the gate blessed work over a test that never ran, and
/// the close either called the skip red (minitest's `S` read as "not
/// green") or counted a phantom `it works (skipped)` and closed the
/// wave (final review 2026-09-06, bugs R-1, R-2, R-23, R-25).
#[test]
fn a_skipped_test_proves_nothing() {
    let rev = keel::rev::text_rev(BODY);

    if common::machine_has("ruby").ready() {
        let dir = ruby_project("rbskip", &rev);
        both_courts_say_not_run(&dir, "test_it_works", "ruby");
    }

    if common::machine_has("mix").ready() {
        let dir = elixir_project("exskip", &rev);
        both_courts_say_not_run(&dir, "it works", "elixir");

        // A test EXCLUDED by a tag, which is how a live elixir project
        // keeps its slow tests out of the ordinary run
        // (`ExUnit.start(exclude: [:integration])` and `@tag
        // :integration`): ExUnit prints the same pair of lines, with
        // `(excluded)` where a duration would stand. The first cut of
        // this wave knew only the word `skipped`, so the start line
        // counted as a run and the state line as a third, phantom test
        // -- and `keel close` closed a wave over a test the gate had
        // just called red (review 0055 R-2).
        let dir = elixir_project_marked(
            "exexcluded",
            &rev,
            "@tag :integration",
            "ExUnit.start(exclude: [:integration])\n",
        );
        // The gate is not asked here, and the border is named: `mix
        // test <file>:<line>` INCLUDES a test the project excludes --
        // measured -- so the gate runs it and judges what it saw.
        // The battery is the project's own ordinary run, and there
        // the test is not run at all: the closing court must say that
        // and hold the wave open, never count a phantom green.
        reviewed(&dir);
        let (said, code) = keel(&dir, &["close"]);
        assert_ne!(
            code, 0,
            "elixir excluded by a tag: a promise whose test the \
             project's own battery excludes is not proven:\n{said}"
        );
        assert!(
            said.contains("батарея: 1 тестів"),
            "the battery holds only what ran -- the plain test, and no \
             phantom `(excluded)` beside it:\n{said}"
        );
        assert!(
            !said.contains("(excluded)") && !said.contains("закрита"),
            "the runner's own mark is not a test's name, and the wave \
             does not close over it:\n{said}"
        );
        assert!(
            said.contains("не виконала тесту \"it works\""),
            "and the lack is said in the same word as every other \
             tongue's:\n{said}"
        );
    }
}
