//! Scenario test of wave 0043: a red test holds the wave open.
//!
//! The closing court runs the battery three times (§7.13), watches a
//! test fail, names it -- and, before this wave, closed the wave
//! anyway with rc 0. Measured identically in rust, ruby and elixir,
//! which is why the probe plays all three: the hole is in the court,
//! not in an adapter.
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

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
}

/// A wave with one chore transform and no promises: it closes light
/// (§2.11), which is exactly the state that used to swallow the red.
fn wave_file(dir: &Path) {
    let mut decisions = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        decisions.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"робота без обіцянок\"\n    files:\n      - lib\n{decisions}---\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("keel/reviews/0001-a-wave.md"), "# Рецензія\n\nok\n").unwrap();
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "init"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// The whole scenario, in the three tongues this release leads.
///
/// proves: a-red-test-holds-the-wave-open@80e2b5
#[test]
fn a_red_test_holds_the_wave_open() {
    // --- rust ---
    let dir = keel_sandbox("redrust");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(
        dir.join("tests/toy_test.rs"),
        "#[test]\nfn nobody_claims_me() {\n    assert!(false, \"red and unclaimed\");\n}\n",
    )
    .unwrap();
    wave_file(&dir);
    settle(&dir);
    let (said, code) = closing(&dir);
    assert!(
        said.contains("червоний тест") && said.contains("nobody_claims_me"),
        "the court names what it watched fail:\n{said}"
    );
    assert_ne!(
        code, 0,
        "and a court that saw red does not close the wave:\n{said}"
    );
    assert!(
        !said.contains("закрита"),
        "no verdict above reads as closure:\n{said}"
    );

    // --- ruby ---
    if have("ruby") {
        let dir = keel_sandbox("redruby");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
        std::fs::write(dir.join("lib/toy.rb"), "class Toy\n  def works\n    true\n  end\nend\n")
            .unwrap();
        std::fs::write(
            dir.join("test/toy_test.rb"),
            "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_nobody_claims_me\n    assert false, \"red and unclaimed\"\n  end\nend\n",
        )
        .unwrap();
        wave_file(&dir);
        settle(&dir);
        let (said, code) = closing(&dir);
        assert!(
            said.contains("червоний тест") && said.contains("test_nobody_claims_me"),
            "ruby's red is named too:\n{said}"
        );
        assert_ne!(code, 0, "and holds the wave open in ruby:\n{said}");
    }

    // --- elixir ---
    if have("mix") {
        let dir = keel_sandbox("redelixir");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"elixir\"\n").unwrap();
        std::fs::write(
            dir.join("mix.exs"),
            "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\n  def application, do: []\nend\n",
        )
        .unwrap();
        std::fs::write(dir.join("lib/toy.ex"), "defmodule Toy do\n  def works, do: true\nend\n")
            .unwrap();
        std::fs::write(dir.join("test/test_helper.exs"), "ExUnit.start()\n").unwrap();
        std::fs::write(
            dir.join("test/toy_test.exs"),
            "defmodule ToyTest do\n  use ExUnit.Case\n\n  test \"nobody claims me\" do\n    assert 1 == 2\n  end\nend\n",
        )
        .unwrap();
        wave_file(&dir);
        settle(&dir);
        let (said, code) = closing(&dir);
        assert!(
            said.contains("червоний тест") && said.contains("nobody claims me"),
            "elixir's red is named too:\n{said}"
        );
        assert_ne!(code, 0, "and holds the wave open in elixir:\n{said}");
    }
}
