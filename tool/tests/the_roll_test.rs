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

/// proves: the-roll-matches-what-the-runner-ran@86f99e
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
    // Measured after review R-2 changed the rule, and the answer got
    // BETTER than the one that was asked for: while a name is open
    // only its timing closes it, so a whole verdict printed by a
    // running test is its output and never enters the roll at all.
    // No ghost, no refusal, both real tests where they belong.
    assert_eq!(
        code, 0,
        "a forged verdict printed by a running test is that test's \
         own output, not a test:\n{said}"
    );
    assert!(
        said.contains("батарея: 2 тестів"),
        "the roll holds the two tests that ran, and not the one that \
         named itself:\n{said}"
    );
    assert!(
        !said.contains("test_that_never_was"),
        "and the ghost is nowhere in the verdict:\n{said}"
    );

    // --- a test that forges the TAIL of a verdict -----------------
    //
    // The sharpest shape, and the one the first cut of this wave made
    // WORSE than it found it (review R-2). A test printing `0.00 s =
    // .` on a line of its own closed ITSELF green, and its real
    // `0.00 s = F` was dropped without a word: every count agreed and
    // the wave closed. A false green is the one answer §4.10 calls
    // worse than a red.
    let forged_tail = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_it_forges_its_own_green
    puts ""
    puts "0.00 s = ."
    flunk "this test really fell"
  end

  def test_a_quiet_neighbour
    assert true
  end
end
"#;
    let dir = project("rolltail", forged_tail);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a test cannot close itself green by printing a timing: the \
         reader that let it was worse than the one that lost \
         tests:\n{said}"
    );
    assert!(
        !said.contains("закрита"),
        "and the wave is certainly not closed over it:\n{said}"
    );

    // --- an ordinary print that merely LOOKS like a name ----------
    //
    // The other side of the same rule, and the reading that got it
    // wrong failed the whole battery and blamed the project (review
    // R-3). `User#full_name = Jane` is a line a test may print; it is
    // not a second test starting. Minitest is serial, so between two
    // real names there is always a timing.
    let record = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_prints_a_record
    puts ""
    puts "User#full_name = Jane"
    assert true
  end

  def test_quiet
    assert true
  end
end
"#;
    let dir = project("rollrecord", record);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a test may print whatever it likes; the roll survives \
         it:\n{said}"
    );
    assert!(
        said.contains("батарея: 2 тестів"),
        "and both tests are in it:\n{said}"
    );

    // --- a file that does not load keeps its own words ------------
    //
    // The roll's courts must not shout over a truer refusal (review
    // R-4): "a test was named and never judged" over a LoadError
    // sends a person hunting a keel defect instead of a missing
    // require.
    let broken = r#"require "nothing_that_exists"
require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_quiet
    assert true
  end
end
"#;
    let dir = project("rollbroken", broken);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "a file that does not load is a refusal:\n{said}");
    assert!(
        said.contains("LoadError"),
        "and it carries ruby's own words, not the roll's:\n{said}"
    );

    // --- the roll against the runner's count ----------------------
    //
    // Measured as a mutant by the reviewer and held by nothing: with
    // the comparison removed the battery stayed green (review R-5).
    // A skip is the shape that tells them apart, because minitest
    // counts a skip among its runs and the battery does not hold it
    // -- so the comparison must count what was READ, skips included,
    // and a probe must stand on that.
    let skipping = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_skips
    skip "not today"
  end

  def test_runs
    assert true
  end
end
"#;
    let dir = project("rollskip", skipping);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a skip is neither green nor red (§7.12) and must not make \
         the roll disagree with the runner: minitest counts it among \
         its runs, the battery does not hold it, and the comparison \
         is between what was READ and what ran:\n{said}"
    );
    assert!(
        said.contains("батарея: 1 тестів"),
        "one test in the battery, the skip outside it:\n{said}"
    );

    // --- a mark nobody knows ---------------------------------------
    //
    // The mark is strict: `.`, `S`, `F`, `E`. The card says an
    // unknown one makes the roll short and the court says THAT. The
    // reviewer measured a loose mark turning a green test red and
    // held by nothing (review R-6, R-7). The shape that produces one
    // is a test printing a timing with a mark of its own -- which is
    // the orphan tail above, and this is its second face.
    let strange = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_prints_a_strange_mark
    puts ""
    puts "0.00 s = ?"
    assert true
  end
end
"#;
    let dir = project("rollmark", strange);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a mark the reader does not know is not a verdict at all, so \
         the line is the test's own output and the roll still holds \
         one test:\n{said}"
    );
    assert!(
        said.contains("батарея: 1 тестів"),
        "and that test is in it:\n{said}"
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
