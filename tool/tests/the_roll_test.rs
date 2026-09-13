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
use std::fs;
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

/// The same project laid out the way Rails lays one out: `bin/rails`
/// and `config/application.rb` are what keel looks for, and the shim
/// hands the arguments to plain ruby -- so the ROAD is the Rails one
/// while the runner underneath stays the minitest this machine has.
fn rails_project(name: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    fs::create_dir_all(dir.join("lib")).unwrap();
    fs::create_dir_all(dir.join("test")).unwrap();
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::create_dir_all(dir.join("config")).unwrap();
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
    fs::write(dir.join("config/application.rb"), "# rails\n").unwrap();
    fs::write(dir.join("config/environment.rb"), "# rails\n").unwrap();
    fs::write(
        dir.join("bin/rails"),
        "#!/bin/sh\nif [ \"$1\" != \"test\" ]; then exit 1; fi\nshift\nexec ruby -Itest \"$@\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(dir.join("bin/rails"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    fs::write(dir.join("test/toy_test.rb"), test_body).unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        d.push_str(&format!(
            "  {cut}: \"не про цю пісочницю, вона грає інше\"\n"
        ));
    }
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"робота без обіцянок\"\n    files:\n      - lib/toy.rb\n{d}---\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
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

/// proves: the-roll-matches-what-the-runner-ran@f7813d
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
        said.contains("червоний тест: test_it_forges_its_own_green"),
        "and it is named RED, which is what it is. A forged timing no \
         longer matters at all: verdicts come from the report \
         minitest writes after the run, and a test cannot print into \
         a report written once it has finished:\n{said}"
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

    // --- the forged tail, hidden behind one character -------------
    //
    // The guard against a self-closing green was worth exactly one
    // `#` (review R2-2): `tail_mark` refused any tail carrying a
    // hash, so a test printing `issue #55` before its real timing
    // walked straight past it. A hash is a thing tests print. What
    // tells a tail from a whole verdict is minitest's own shape --
    // a second ` = ` -- and nothing else.
    let hidden = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_it_hides_behind_a_hash
    puts ""
    puts "0.00 s = ."
    print "issue #55"
    flunk "this test really fell"
  end

  def test_quiet
    assert true
  end
end
"#;
    let dir = project("rollhash", hidden);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "one character must not buy a test a green it did not \
         earn:\n{said}"
    );
    assert!(
        said.contains("червоний тест: test_it_hides_behind_a_hash"),
        "and it is named RED. Three rounds each defeated the \
         character chosen to tell a real verdict from a printed one \
         -- a `#`, then a second ` = `. The answer was not a better \
         character: the verdicts left that stream:\n{said}"
    );

    // --- a verdict-shaped line on STDERR --------------------------
    //
    // The reader used to be handed stdout and stderr spliced
    // together, so a line printed to stderr took a place in the
    // sequence that was never its own (review R2-5). minitest writes
    // its verdicts to stdout; the roll is read from there alone.
    let on_stderr = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_prints_to_stderr
    $stderr.puts "GhostTest#test_that_never_was = 0.00 s = ."
    assert true
  end

  def test_quiet
    assert true
  end
end
"#;
    let dir = project("rollstderr", on_stderr);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a line on stderr is not a verdict, and must not disturb the \
         roll:\n{said}"
    );
    assert!(
        said.contains("батарея: 2 тестів"),
        "both tests, and no ghost:\n{said}"
    );

    // --- a test printing minitest's own summary line --------------
    //
    // `runs_said` took the FIRST line carrying ` runs,`, so a test
    // could choose the number the court compared its roll against
    // (review R2-4). minitest's summary is the LAST such line, and it
    // carries its own neighbours.
    let summary = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_prints_a_summary
    puts ""
    puts "5 runs, 5 assertions, 0 failures, 0 errors, 0 skips"
    assert true
  end
end
"#;
    let dir = project("rollsummary", summary);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a test does not get to choose the number its roll is \
         measured against:\n{said}"
    );
    assert!(
        said.contains("батарея: 1 тестів"),
        "and the roll is what really ran:\n{said}"
    );

    // --- a line lost WHOLE, which only the count can see ----------
    //
    // Two reviews argued about whether the comparison with minitest's
    // own `N runs,` is a dead court. Round two disproved the first
    // "unreachable" by building two shapes; those two are now caught
    // earlier and more precisely, by the stdout split and by reading
    // the LAST summary line -- which put the question back.
    //
    // This is the shape that answers it. A test name carrying a
    // newline makes minitest print a verdict line in two halves, and
    // NEITHER half is anything the reader can use: the first has no
    // ` = `, the second has no `#`. No name is opened, so the
    // abandoned-name guard sees nothing; the line simply is not
    // there. Only the runner's own count knows a test went missing --
    // which is issue #55's symptom exactly, and the belt is for it.
    let split_name = "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  define_method(\"test_a_name_with_a\\nnewline_in_it\") { assert true }\n\n  def test_quiet\n    assert true\n  end\nend\n";
    let dir = project("rolllost", split_name);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a name that breaks the `-v` line in two is read all the \
         same: the roll comes from the file, not from that \
         stream:\n{said}"
    );
    assert!(
        said.contains("батарея: 2 тестів"),
        "and both tests are in the battery:\n{said}"
    );
}

/// proves: the-roll-matches-what-the-runner-ran@f7813d -- the same
/// scenario, standing on its own feet.
///
/// One `#[test]` per group, and not one long one, because a probe
/// that stops at the first assert hides every court after it: with
/// the whole scenario in one body, three mutants died at the SAME
/// assert and the courts they actually broke were never reached
/// (review 0074 R5-7 asks each court for a red of its own).
///
/// This group: the verdicts come from the report minitest writes
/// AFTER the run, not from the stream it writes during it.
#[test]
fn the_report_is_read_and_not_the_stream() {
    // --- a forged report BLOCK, which is the only forgery left ----
    //
    // A test cannot print into a report written after it finished --
    // but it can print something that LOOKS like one while it runs.
    // Then the three sources disagree: two failure blocks against
    // minitest's own count of one. The reader says it cannot account
    // for the run, and names the file; it does not pick a winner.
    let forged_block = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_it_forges
    puts ""
    puts "  1) Failure:"
    puts "ToyTest#test_quiet [x:1]:"
    puts "a lie"
    flunk "this really fell"
  end

  def test_quiet
    assert true
  end
end
"#;
    let dir = project("rollblock", forged_block);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a tree whose run cannot be accounted for does not close:\n{said}"
    );
    assert!(
        said.contains("не сходиться сам із собою"),
        "and the court says the report does not agree with its own \
         totals, rather than choosing between a real block and a \
         printed one:\n{said}"
    );

    // --- tests running in PARALLEL, which is where #55 came from --
    //
    // Rails turns `parallelize_me!` on by default, and under it the
    // `-v` stream is not merely forgeable, it is physically
    // interleaved: one test's name and another's timing land on one
    // line. No line-ordered reader survives that, and no cleverness
    // will -- which is why the verdicts come from the report
    // minitest writes after the run instead.
    let parallel = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  parallelize_me!

  def test_one
    assert true
  end

  def test_two
    flunk "it fell"
  end

  def test_three
    skip "not today"
  end

  def test_four
    raise "it errored"
  end
end
"#;
    let dir = project("rollparallel", parallel);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the failures hold the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_two") && said.contains("червоний тест: test_four"),
        "a failure and an error are both red, and both named:\n{said}"
    );
    assert!(
        said.contains("батарея: 3 тестів"),
        "the skip is neither green nor red (§7.12), so three tests \
         stand in the battery of four declared:\n{said}"
    );
}

/// proves: the-roll-matches-what-the-runner-ran@f7813d -- the same
/// scenario on the other road.
///
/// Rails boots the application and owns the run, so there is no roll
/// there and the reader has only the shape of the `-v` line. That
/// road had no probe at all before this (review 0074 R5-7).
#[test]
fn the_rails_road_keeps_its_own_guard() {
    // --- the Rails road, which reads the older way -----------------
    //
    // Rails boots the application and owns the run, so no listing is
    // available there and the reader falls back to the shape of the
    // `-v` line. That fallback keeps its own guard -- a timing with
    // no name open means a real verdict has already been eaten --
    // and review 0074 R4-2 measured what dropping it cost: a wave
    // closed over a failing test, where the trunk had said "the
    // battery ran no test of that name". Nothing held that guard;
    // this does.
    let dir = rails_project(
        "rollrails",
        "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_it_forges\n    puts \"\"\n    puts \"0.00 s = .\"\n    flunk \"this really fell\"\n  end\n\n  def test_quiet\n    assert true\n  end\nend\n",
    );
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    common::did_the_work(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "on the Rails road a test that prints a timing of its own \
         must not close itself green -- the fallback reader keeps \
         its guard:\n{said}"
    );
    assert!(
        said.contains("вирок розбито"),
        "and it says which road it is on and why it can say nothing \
         about this file, rather than borrowing the words of the \
         road that HAS a roll:\n{said}"
    );

    // --- issue #55 itself, on the Rails road ----------------------
    //
    // Four of five tests print while they run. This is the shape the
    // issue was reported on, and on this road the reader has only
    // the `-v` stream -- so the probe stands here as well as on the
    // plain road, and the number is the whole of it.
    let dir = rails_project(
        "rollrailsfive",
        "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_quiet\n    assert true\n  end\n\n  def test_newline\n    puts \"capybara: a warning\"\n    assert true\n  end\n\n  def test_equals\n    print \"expected = actual\"\n    assert true\n  end\n\n  def test_plain\n    print \"selenium: waiting\"\n    assert true\n  end\n\n  def test_five\n    assert true\n  end\nend\n",
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "five green tests close:\n{said}");
    assert!(
        said.contains("батарея: 5 тестів"),
        "and all five are in the battery -- three of them print, and \
         the released 1.4.0 counted three:\n{said}"
    );

    // --- the strict mark, which only this road still pays for -----
    //
    // On the plain road the mark decides nothing any more: verdicts
    // come from the report, not from the stream. Here it decides
    // everything. A test printing `0.00 s = ?` offers a timing with
    // a mark nobody knows; a LOOSE reading would take it for a
    // fallen test, find no name open, and refuse the whole file.
    // Strict, it is not a verdict at all -- so it is the test's own
    // output, and the one real test keeps its green.
    let dir = rails_project(
        "rollrailsmark",
        "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_prints_a_strange_mark\n    puts \"\"\n    puts \"0.00 s = ?\"\n    assert true\n  end\nend\n",
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a mark the reader does not know is not a verdict, so the \
         line is output and the file still closes:\n{said}"
    );
    assert!(
        said.contains("батарея: 1 тестів"),
        "and the one real test is in the battery:\n{said}"
    );
}

/// proves: the-roll-matches-what-the-runner-ran@f7813d -- the same
/// scenario: what the reader does when the roll is not what it
/// should be, and what it says about it.
#[test]
fn a_roll_that_is_not_the_run_is_said_aloud() {
    // --- the roll swallowed whole, which is never answered quietly-
    //
    // A file may replace `$stdout` while it loads -- the ordinary way
    // to quieten a noisy boot -- and put it back in an `at_exit`,
    // which by LIFO runs before minitest's own. keel's roll goes into
    // the StringIO; the verdicts go to the real stream. Falling back
    // to the older reader here would put the forgery of three review
    // rounds back in business on the road that was fixed (review
    // 0074 R5-2), so the reader says it has no roll instead.
    let swallowed = r#"require "minitest/autorun"

$keel_real = $stdout
$stdout = StringIO.new
at_exit { $stdout = $keel_real }

class ToyTest < Minitest::Test
  def test_forges_its_own_green
    flunk "this promise is NOT kept"
  end

  def test_quiet
    assert true
  end
end
"#;
    let dir = project("rollnone", swallowed);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a run keel has no roll of is refused, not read another \
         way:\n{said}"
    );
    assert!(
        said.contains("переліку нема"),
        "and the refusal says that is what happened:\n{said}"
    );

    // --- a name ADDED to the roll, which only the count can see ----
    //
    // The roll is printed by keel's own preamble, on the same stream
    // the file may write to while it LOADS -- so a file can put a
    // name into it. It cannot make minitest run that name, and the
    // count is minitest's own: three named, two run, and the court
    // says so. This is the court the reviewer's mutant M3 removed
    // with the battery staying green (review 0074 R5-7).
    let padded = r#"puts "KEEL-ROLL ToyTest#test_a_ghost"
require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_one
    assert true
  end

  def test_two
    assert true
  end
end
"#;
    let dir = project("rollpad", padded);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "a roll longer than the run is refused:\n{said}");
    assert!(
        said.contains("прогнав 2 тест") && said.contains("налічує 3"),
        "and both numbers are said, minitest's and the reader's, so \
         a person can see which way the difference runs:\n{said}"
    );

    // --- minitest's number is minitest's, not a test's -------------
    //
    // stderr is appended after all of stdout, so the LAST summary
    // line of the two streams spliced together is always the one a
    // test wrote there. Measured (review 0074 R5-3): one `warn "99
    // runs, …"` and the count court compared the roll against 99.
    // The count is asked of stdout alone.
    let on_stderr_summary = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_one
    warn "99 runs, 99 assertions, 0 failures, 0 errors, 0 skips"
    assert true
  end

  def test_two
    assert true
  end
end
"#;
    let dir = project("rollstderrsum", on_stderr_summary);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a summary line a test wrote to stderr is not minitest's \
         count:\n{said}"
    );
    assert!(
        said.contains("батарея: 2 тестів"),
        "and the two real tests stand:\n{said}"
    );

    // --- minitest's own spec style, whose names carry spaces -------
    //
    // `describe`/`it` is minitest's native DSL, not an exotic one,
    // and it declares `a user of the shop#test_0002_refuses an empty
    // basket`. Any rule about where such a name ends is a rule a
    // test can print, so the name is taken from the ROLL and the
    // report only has to say which of them it is (review 0074 R5-5).
    let spec = r#"require "minitest/autorun"

describe "a user of the shop" do
  it "greets by name" do
    _(1).must_equal 1
  end

  it "refuses an empty basket" do
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rollspec", spec);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the failing example holds the wave open:\n{said}");
    assert!(
        said.contains("батарея: 2 тестів"),
        "both examples are in the battery, spaces in their names and \
         all:\n{said}"
    );
    assert!(
        said.contains("test_0002_refuses an empty basket"),
        "and the red one is named whole -- the space is part of the \
         name, not the end of it:\n{said}"
    );

    // --- ruby's words about the file, not about keel's preamble ----
    //
    // The roll is taken by `ruby -e "… load ARGV.shift …"`, so ruby
    // blames the frame it was in: `-e:2:in 'load': test/toy_test.rb:5:
    // syntax error…`. The person reading the refusal came for their
    // own file (review 0074 R5-11).
    let unfinished = "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_one\n    assert true\n  end\n";
    let dir = project("rollsyntax", unfinished);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "a file that does not parse is refused:\n{said}");
    assert!(
        said.contains("test/toy_test.rb:") && !said.contains("-e:"),
        "and the refusal carries the project's own file and line, \
         with keel's preamble cut off the front:\n{said}"
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
