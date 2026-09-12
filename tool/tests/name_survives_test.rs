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
        .env("LANG", "C")
        .env("LC_ALL", "C")
        .env("LC_CTYPE", "C")
        // The locale alone no longer breaks it: Erlang/OTP 29 prints
        // UTF-8 whatever the locale says, and the field report of
        // release 1.2.0 was measured on an older one. The hostile
        // state that is alive TODAY is the runner's own printing
        // flag, and it is the same wound: with `+pc latin1` mix
        // writes the name as `\x{456}\x{43C}…`, which matches
        // nothing in the file, and the closing court says the battery
        // did not run a test it ran and passed.
        .env("ERL_FLAGS", "+pc latin1")
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

/// proves: a-non-ascii-name-survives-its-runner@0000000
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
    // and it passed, and the verdict says it did not run it. Measured
    // before the work, in this very sandbox:
    //
    //   0001-a-wave: in progress -- the missing, by name:
    //     scenario "it-works": the battery did not run the test "імʼя живе"
    assert!(
        !said.contains("не виконала"),
        "a name goes from the source to the verdict whole: the runner \
         is told what encoding to print in, instead of taking it from \
         a setting that may say anything\n{said}"
    );
    assert!(
        !said.contains("\\x{"),
        "and no escape of the runner's own making is read as a \
         name:\n{said}"
    );
}
