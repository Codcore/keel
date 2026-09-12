//! Scenario test of wave 0074: the roll calls every test.
//!
//! Measured on the released 1.4.0 binary before any of this was
//! written: a minitest file holding five tests, and
//!
//!     battery: 3 tests × 3 runs (§7.13)
//!
//! Two vanished -- not from the run (minitest said `5 runs, 5
//! assertions, 0 failures`) but from what keel could read out of its
//! `-v` voice. A test that prints while it runs lands in the middle
//! of its own verdict line, and what happens then depends on WHAT it
//! printed.

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

/// A ruby project whose test file is given whole.
fn project(name: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        d.push_str(&format!(
            "  {cut}: \"не про цю пісочницю, вона грає інше\"\n"
        ));
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"робота без обіцянок\"\n    files:\n      - lib/toy.rb\n{d}---\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    std::fs::write(dir.join("test/toy_test.rb"), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// Five tests, and four of them print something while they run --
/// every shape the noise comes in. All five must be in the roll.
const NOISY: &str = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_quiet_one
    assert true
  end

  def test_print_plain
    print "selenium: waiting"
    assert true
  end

  def test_print_with_a_newline
    puts "capybara: a warning"
    assert true
  end

  def test_print_with_an_equals
    print "expected = actual"
    assert true
  end

  def test_print_with_a_trailing_space
    print "driver started "
    assert true
  end
end
"#;

/// proves: the-roll-matches-what-the-runner-ran@c1026d
#[test]
fn the_roll_matches_what_the_runner_ran() {
    // --- every test is in the roll, whatever it printed -----------
    let dir = project("rollnoisy", NOISY);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 5 тестів"),
        "minitest ran five and said so; the roll keel builds from its \
         voice must hold five too. A test that prints lands inside \
         its own verdict line, and the reader must survive that -- \
         four shapes of noise stand in this file, and two of them \
         used to delete the test from the battery (issue #55):\n{said}"
    );

    // --- and a roll that does not match is a REFUSAL, aloud -------
    //
    // The reader alone is half a fix: the next shape of noise nobody
    // has met yet would be as silent as these were. Minitest says how
    // many it ran; keel must compare, and say so when it cannot.
    //
    // The forged line is the sharpest form of it: a test body can
    // print something shaped exactly like a verdict, and no reader
    // can tell that apart from the real thing. The runner's own count
    // can.
    // Measured, and the first guess was wrong: a forged line printed
    // mid-verdict is swallowed by the real test's own line and makes
    // no ghost at all. The shape that DOES bite starts a line of its
    // own -- and then the reader closes a test that never ran and
    // abandons the one that did, ONE FOR ONE. Every count still
    // agrees. Only the abandoned name gives it away.
    let forged = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_it_forges
    puts ""
    puts "GhostTest#test_that_never_was = 0.00 s = ."
    assert true
  end

  def test_a_quiet_neighbour
    assert true
  end
end
"#;
    let dir = project("rollforged", forged);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a name opened and never judged is a refusal: the reader \
         cannot say what that test came to, and a count against the \
         runner's own total cannot see a one-for-one swap:\n{said}"
    );
    assert!(
        said.contains("test") && said.contains("toy_test.rb"),
        "and it names the file, so a person can go and look:\n{said}"
    );

    // --- a quiet tree stays quiet ---------------------------------
    let quiet = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_one
    assert true
  end

  def test_two
    assert true
  end
end
"#;
    let dir = project("rollquiet", quiet);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "a quiet tree closes:\n{said}");
    assert!(
        said.contains("батарея: 2 тестів"),
        "and its roll is its own length:\n{said}"
    );
}
