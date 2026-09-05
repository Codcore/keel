//! Scenario test of wave 0042: a tongue that tells the two apart
//! says so.
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

fn have_mix() -> bool {
    Command::new("mix")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
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

/// proves: a-tongue-that-tells-the-two-apart-says-so@4a6eee -- sec. 7.12
/// was written for the case where an adapter CAN tell a failure from
/// a broken build. Ruby cannot: both exit 1. Elixir can: 2 on a
/// failure, 1 on a compilation error, 0 green. Dragging ruby's border
/// along would be as untrue as leaving one unsaid.
#[test]
fn a_tongue_that_tells_the_two_apart_says_so() {
    assert!(have_mix(), "this probe runs a real mix; it is not on PATH");
    let rev = keel::rev::text_rev(BODY);

    // A file that does not compile is a refusal aloud, not a red test
    // and not a green battery.
    let broken = "defmodule ToyTest do\n  use ExUnit.Case\n  test \"it works\" do\n";
    let dir = project("exbroken", broken);
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a file that does not compile never closes:\n{said}"
    );
    assert!(
        !said.contains("закрита"),
        "and the verdict does not read as closure:\n{said}"
    );

    // And the border ruby needed is NOT printed here, because it is
    // not true of this project.
    let dir = project("exborder", &test_file(&rev));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a whole elixir project is judged:\n{said}");
    assert!(
        !said.contains("не зібрався» кодом виходу"),
        "the check does not carry ruby's border into a tongue that \
         does not have it:\n{said}"
    );
    assert!(
        said.contains("elixir") && said.contains("розрізняє"),
        "it says the opposite, which is what is true here:\n{said}"
    );

    // A tag over a name ExUnit does not know is "not run", never
    // "work passes".
    let unknown = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"a name no run knows\" do\n    assert Toy.works()\n  end\nend\n"
    );
    let dir = project("exnotrun", &unknown);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, "work: тіло\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "work over a test that never ran does not pass the gate:\n{said}"
    );
}
