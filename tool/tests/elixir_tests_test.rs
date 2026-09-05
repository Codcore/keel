//! Scenario test of wave 0042: elixir tests are read and run.
//!
//! These run a real `mix` against real projects. Where mix is not on
//! the machine the probe says so and stops rather than pretending --
//! a green test over a language nobody ran is exactly the fixture
//! more convenient than reality that review 0041 named.
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

/// A mix project: mix.exs, a module, and a test file as given.
fn project(name: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"elixir\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("lib/toy")).unwrap();
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
    std::fs::write(
        dir.join("lib/toy/bar.ex"),
        "defmodule Toy.Bar do\n  def works(a, b), do: a + b\nend\n",
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
    std::fs::write(dir.join("test/toy_test.exs"), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn test_file(rev: &str) -> String {
    format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert Toy.works()\n  end\nend\n"
    )
}

/// proves: elixir-tests-are-read-and-run@257ce5 -- the concept named
/// four starting adapters and called Elixir the main consumer. Two of
/// the four were built.
#[test]
fn elixir_tests_are_read_and_run() {
    if !common::machine_has("mix").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);
    let dir = project("extags", &test_file(&rev));

    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "an elixir project is judged whole:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "the proves tag is read from an elixir test file, and the \
         test's name is the STRING the declaration carries (§5.5):\n{said}"
    );

    // A stale revision is stale here as it is everywhere: the court
    // is the same, only the reading differs.
    let dir = project("exstale", &test_file("beef00"));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "a stale elixir tag is a finding:\n{said}");
    assert!(
        said.contains("beef00") && said.contains("it-works"),
        "and it names the tag and the promise:\n{said}"
    );

    // The battery runs, and the closing court names what fell by the
    // name ExUnit gives it.
    let falls = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert Toy.works()\n  end\n\n  test \"it falls\" do\n    assert 1 == 2\n  end\nend\n"
    );
    let dir = project("exbattery", &falls);
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("it falls"),
        "the closing court names the elixir test it watched fall:\n{said}"
    );
    assert!(
        !said.lines().any(|line| line.contains("червоний тест")
            && line.contains("it works")
            && !line.contains("it falls")),
        "and the green one is not dragged in with it:\n{said}"
    );
}
