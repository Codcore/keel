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

/// proves: the-roll-matches-what-the-runner-ran@cafb06
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

    // --- a print that ends a line with somebody's timing and an F --
    //
    // The strict timing is what keeps a test's own print from
    // becoming its own verdict. `waited long s` is not minitest's
    // shape -- a number and ` s` -- so the line is output, and the
    // test keeps its green. A reader that took any `<words> s = F`
    // for a verdict would let a test name itself fallen, and worse,
    // name its neighbours.
    let tailnoise = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_one
    puts "ToyTest#test_one = waited long s = F"
    assert true
  end
end
"#;
    let dir = project("rolltailnoise", tailnoise);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a timing that is not minitest's own shape is not a \
         verdict:\n{said}"
    );
    assert!(
        said.contains("батарея: 1 тестів"),
        "and the one real test keeps its green:\n{said}"
    );
}

/// proves: the-roll-matches-what-the-runner-ran@cafb06 -- the same
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
    // The COUNT of the battery is not stable on this road, and that is
    // said here rather than asserted away: under `parallelize_me!` the
    // interleaved stream can put another test's `F` tail on a
    // neighbour's name -- a false red, the one direction §7.12 calls
    // affordable, and the join takes red from ANY reading. A skipped
    // test named red by a stolen tail stands in the battery of a seed
    // that merged the lines badly. The skip semantics themselves are
    // held by the serial probe above (`rollskip`); here the stable
    // facts are the two real reds and an open wave.

    // --- two names on ONE line, and the tail belongs to the NEARER
    //     one ---------------------------------------------------------
    //
    // Under `parallelize_me!` two tests' verdicts land on one line --
    // `ToyTest#test_three = ToyTest#test_two = 0.00 s = F` -- and the
    // timing and mark belong to the NEARER name, not the first. This
    // is the same shape printed by hand: reading the FIRST name would
    // put the fallen tail on the innocent test that merely ran
    // alongside.
    let nearest = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_ran_alongside
    puts ""
    puts "ToyTest#test_ran_alongside = ToyTest#test_the_liar = 0.00 s = F"
    assert true
  end

  def test_the_liar
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rollnearest", nearest);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the test that fell holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_the_liar"),
        "and the fallen tail is attributed to the name nearest the \
         timing, which is the test that really fell:\n{said}"
    );
    assert!(
        !said.contains("червоний тест: test_ran_alongside"),
        "and the innocent neighbour keeps its green -- the mark \
         belongs to the NEARER name, never to the first on the \
         line:\n{said}"
    );
}

/// proves: the-roll-matches-what-the-runner-ran@cafb06 -- the same
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
        said.contains("вироку не зібрано"),
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

    // --- a name must be ONE word, and only this road pays for it --
    //
    // Without a roll, a line that merely looks like the OPENING of a
    // verdict opens one. A file printing `Foo#bar baz = 1` while it
    // loads -- before any verdict line -- puts a phantom name in the
    // queue, and the next timing closes THAT instead of the test it
    // belonged to: measured, `bar baz` stands in the battery as a red
    // and one real test is gone from it. The name has to be one word,
    // as minitest writes them.
    let dir = rails_project(
        "rollrailswordy",
        "require \"minitest/autorun\"\n\nputs \"Foo#bar baz = 1\"\n\nclass ToyTest < Minitest::Test\n  def test_one_falls\n    flunk \"THE-FIRST-FELL\"\n  end\n\n  def test_two_falls\n    flunk \"THE-SECOND-FELL\"\n  end\nend\n",
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "both tests fell:\n{said}");
    assert!(
        said.contains("червоний тест: test_one_falls")
            && said.contains("червоний тест: test_two_falls"),
        "and both are named by their own names:\n{said}"
    );
    assert!(
        !said.contains("bar baz"),
        "and no phantom stands among them, holding a verdict that \
         belonged to a test:\n{said}"
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

/// proves: the-roll-matches-what-the-runner-ran@cafb06 -- the same
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

    // --- two names, one of them a decoy -------------------------
    //
    // The roll used to keep the METHOD alone, so a block's line had
    // to be matched by substring -- and a file declaring both
    // `test_boom` and `test_boom:` made that guess choose between two
    // names that BOTH stood in it lawfully. It chose the innocent
    // one: the test that threw went into the battery green while its
    // decoy was named red, and every count agreed (review 0074 round
    // six, `atk_decoy`). Whole, `Class#method`, the match is an
    // equality and there is nothing to choose.
    let decoy = "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  define_method(\"test_boom:\") { assert true }\n\n  def test_boom\n    raise \"THE-REAL-ONE-THREW\"\n  end\nend\n";
    let dir = project("rolldecoy", decoy);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the test that threw holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_boom (") && said.contains("батарея: 2 тестів"),
        "and it is THE one named -- the decoy beside it is green, and \
         naming the innocent is as wrong as a false green:\n{said}"
    );

    // --- the pen minitest hands back after its own report ---------
    //
    // `Minitest.after_run` is minitest's own documented hook, called
    // from `autorun`'s at_exit AFTER the report is printed. A test
    // that registers one can write a second summary that agrees with
    // a block it forged while running -- three perfectly consistent
    // sources, and a failing tree closing green (review 0074 round
    // six, `atk_afterrun`). keel registers its own hook LAST, so it
    // is called FIRST, and everything past its mark belongs to
    // somebody else.
    let after_run = r#"require "minitest/autorun"

Minitest.after_run { puts "2 runs, 2 assertions, 1 failures, 0 errors, 1 skips" }

class ToyTest < Minitest::Test
  def test_red
    puts ""
    puts "  1) Skipped:"
    puts "ToyTest#test_red [x:1]:"
    puts "skipped for reasons"
    puts ""
    flunk "this promise is NOT kept"
  end

  def test_green
    assert true
  end
end
"#;
    let dir = project("rollafterrun", after_run);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the failing test holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_red"),
        "and it is named: a block a test printed about itself stands \
         BEFORE the one minitest writes, so the later of the two is \
         minitest's own:\n{said}"
    );

    // --- two names, and one of them wears a bracket ---------------
    //
    // The error form `Class#method:` is an equality, but the failure
    // form carries a location -- `Class#method [file:11]:` -- and it
    // has to be a prefix. A file declaring `test_x` and
    // `test_x [foo]` puts both roll names at that prefix lawfully,
    // and the SHORTER one used to win: the innocent test was named
    // red and the one that fell went into the battery green, with
    // every count agreeing (review 0074 round seven).
    let bracket = "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  define_method(\"test_x [foo]\") { flunk \"THE-BRACKET-ONE-FELL\" }\n\n  def test_x\n    assert true\n  end\nend\n";
    let dir = project("rolldecoy2", bracket);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the test that fell holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_x [foo]") && said.contains("батарея: 2 тестів"),
        "and the LONGER name is the one the block is about; naming \
         the innocent is as wrong as a false green:\n{said}"
    );

    // --- the longest carrier, where only the blocks can decide ----
    //
    // The `-v` stream names the guilty test on its own line whenever
    // the line survived -- so here the line does not survive: the
    // falling test prints without a newline, its own print sits
    // between its name and minitest's timing, and the stream says
    // nothing about anybody. The blocks are all that is left, and the
    // block line carries BOTH roll names lawfully: the LONGEST is the
    // one it is about. A reader that took the shorter one would send
    // the innocent test red and let the one that fell go green.
    let mangled = "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  define_method(\"test_y [foo]\") { print \"cameraman: rolling \"; flunk \"THE-BRACKET-ONE-FELL\" }\n\n  def test_y\n    assert true\n  end\nend\n";
    let dir = project("rollmangled", mangled);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the test that fell holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_y [foo]"),
        "and the LONGEST name in the block line is the one named red \
         -- the stream said nothing, so the blocks decide, and the \
         block named the test with the bracket:\n{said}"
    );
    assert!(
        !said.contains("червоний тест: test_y ("),
        "and the innocent `test_y` keeps its green:\n{said}"
    );

    // --- the hook a TEST BODY registers ---------------------------
    //
    // Round six put keel's mark in an `after_run` hook, and round
    // seven broke it: `@@after_run` is a list called in reverse, and
    // a hook registered from inside a test body is appended AFTER
    // keel's, so it is called BEFORE it -- its forged summary landed
    // inside the region and closed a failing tree green. A test body
    // runs later than any registration keel can make, so no place in
    // that queue is safe. The mark is printed from inside
    // `Minitest.run` now, which is already running by then.
    let hook_in_body = r##"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_red
    Minitest.after_run { puts "2 runs, 2 assertions, 1 failures, 0 errors, 1 skips" }
    puts ""
    puts "  1) Skipped:"
    puts "ToyTest#test_red [x:1]:"
    puts "skipped for reasons"
    puts ""
    flunk "this promise is NOT kept"
  end

  def test_green
    assert true
  end
end
"##;
    let dir = project("rollafterrunbody", hook_in_body);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the failing test holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_red"),
        "and it is named: keel's mark is printed by the method that \
         writes the report, not by a hook a test can get in front \
         of:\n{said}"
    );

    // ...and the same body calling the registered hooks itself, to
    // make the mark arrive early. It cannot: the mark is not in that
    // list any more.
    let call_hooks = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_red
    Minitest.class_variable_get(:@@after_run).each(&:call)
    flunk "this promise is NOT kept"
  end

  def test_green
    assert true
  end
end
"#;
    let dir = project("rollcallhooks", call_hooks);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the failing test holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_red"),
        "and calling minitest's own hooks from a test body does not \
         bring keel's mark forward:\n{said}"
    );

    // --- a roll without minitest's totals is not a run ------------
    //
    // A file that puts a delegate on STDOUT and drops the summary
    // line leaves the roll and the verdicts standing and the numbers
    // gone. Reading the roll on its own would then call every name
    // green -- over a test that fell.
    let eater = r#"require "minitest/autorun"

class Eater
  def initialize(io) @io = io end
  def write(*args)
    kept = args.reject { |s| s.to_s.include?(" runs, ") }
    return 0 if kept.empty?
    @io.write(*kept)
  end
  def puts(*args)
    @io.puts(*args.reject { |s| s.to_s.include?(" runs, ") })
  end
  def print(*args)
    @io.print(*args.reject { |s| s.to_s.include?(" runs, ") })
  end
  def method_missing(m, *a, &b) @io.send(m, *a, &b) end
  def respond_to_missing?(*) true end
end

$stdout = Eater.new(STDOUT)

class ToyTest < Minitest::Test
  def test_red
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rolleatsummary", eater);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a roll with no totals beside it is not a run, and calling \
         its names green would be a false green over a test that \
         fell:\n{said}"
    );
    assert!(
        said.contains("не має жодного підсумку"),
        "and the refusal says minitest's own summary is nowhere in \
         the voice: without it the blocks have no count to answer \
         to, and the blocks are what a test can write:\n{said}"
    );

    // --- minitest's number is the LAST it writes -------------------
    //
    // A test printing a summary of its own does it WHILE it runs, so
    // minitest's own stands after it. Reading the first would compare
    // the roll against a number the test chose.
    let early = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_aaa_falls
    puts ""
    puts "2 runs, 2 assertions, 0 failures, 0 errors, 1 skips"
    flunk "THE-REAL-ONE-FELL"
  end

  def test_bbb
    assert true
  end
end
"#;
    let dir = project("rollearlysummary", early);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the test that fell holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_aaa_falls"),
        "and it is named: the numbers are minitest's own, which it \
         writes AFTER every body has finished -- the summary a test \
         printed while it ran stands earlier, and says a skip where \
         there was a failure:\n{said}"
    );

    // --- a method name carrying a hash ----------------------------
    //
    // The class is on the left of the FIRST `#` and the method on its
    // right, and a method name may hold hashes of its own: minitest's
    // spec style makes them out of `it "#add works"`. Splitting from
    // the right put that test in the battery under `add works`, which
    // is not a name anything selects by.
    let hashed = r##"require "minitest/autorun"

describe "Calc" do
  it "#add works" do
    _(1).must_equal 1
  end

  it "#sub fails" do
    flunk "this promise is NOT kept"
  end
end
"##;
    let dir = project("rollhashname", hashed);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "the failing example holds the wave open:\n{said}");
    assert!(
        said.contains("червоний тест: test_0002_#sub fails"),
        "and its name is whole, hash and all:\n{said}"
    );

    // --- a file that declares no test at all ----------------------
    //
    // Not a fault, and not a missing roll either: minitest says `0
    // runs` and means it. The court that refuses a run with no roll
    // has to ask whether anything ran at all.
    let empty = "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\nend\n";
    let dir = project("rollnotests", empty);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "a file with no tests is not a fault:\n{said}");
    assert!(
        said.contains("батарея: 0 тестів"),
        "and it holds no tests:\n{said}"
    );

    // --- a summary of its own, and out by `exit!` ------------------
    //
    // `exit!` runs no `at_exit` at all -- and it is called at LOAD
    // time, so it does not even let keel's own listing reach the
    // stream: the process dies in the middle of the load, the roll is
    // never printed, and the file's own summary is all there is. A
    // summary keel can read with no roll beside it is the oldest
    // refusal of this wave, and it still says the truest thing: there
    // is no roll, and minitest says it ran one.
    let unmarked = "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_one\n    assert true\n  end\nend\n\nputs \"1 runs, 1 assertions, 0 failures, 0 errors, 0 skips\"\n$stdout.flush\nexit!(0)\n";
    let dir = project("rollunmarked", unmarked);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a summary with no run beside it is refused:\n{said}"
    );
    assert!(
        said.contains("переліку нема"),
        "and the refusal says there is no roll: the process died \
         before keel's own listing was printed, and a forged summary \
         buys nobody a green:\n{said}"
    );

    // --- a red exit keel could read nothing out of -----------------
    //
    // A file that muffles STDOUT while it loads and never puts it
    // back takes the roll, the verdicts and the summary with it. The
    // battery used to say "0 tests" and the wave closed -- over a
    // test that fell. The same shape covers a file that raises after
    // declaring its tests (review 0074 R5-13).
    let muffled = r#"require "minitest/autorun"

$stdout = StringIO.new

class ToyTest < Minitest::Test
  def test_red
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rollmute", muffled);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "a red exit keel cannot read is refused:\n{said}");
    assert!(
        said.contains("вийшов із помилкою і не сказав нічого"),
        "and says so: nought tests over a red exit is a quiet \
         untruth:\n{said}"
    );

    // ...and the same with a summary-shaped line on STDERR, which is
    // not minitest's stream. Read from the splice, that line bought
    // the tree its quiet nought back.
    let muffled_stderr = r#"require "minitest/autorun"

$stdout = StringIO.new
warn "1 runs, 1 assertions, 0 failures, 0 errors, 0 skips"

class ToyTest < Minitest::Test
  def test_red
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rollmutestderr", muffled_stderr);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a summary a test wrote to stderr is not minitest's, and buys \
         nothing:\n{said}"
    );
    assert!(
        said.contains("вийшов із помилкою і не сказав нічого"),
        "the refusal is the same one:\n{said}"
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

/// proves: the-roll-matches-what-the-runner-ran@cafb06 -- the two
/// readings of one voice, joined in the safe direction.
///
/// Round eight found three roads to a clean green over a `flunk`,
/// and all three went through the same wall: keel read only what
/// stood before its own end mark, and the mark did not own the
/// region it closed. A file can register inside `Minitest.run` (a
/// `prepend` made at LOAD time stands inside keel's own and writes
/// between the real report and the mark); a reporter APPENDED to
/// minitest's prints from inside `Minitest.run` too; and the mark
/// itself could be read back out of the process. So there is no
/// region any more, and no ordering rule can hold -- a test writes
/// both before and after the real report.
///
/// What holds is the direction of the join: the roll gives the
/// names; the report blocks give verdicts; the `-v` stream gives a
/// second verdict on the same names; a name is RED if any reading
/// says so, and green only where no reading says otherwise. A forged
/// skip is beaten by the stream's own `F` on the same name. The
/// price runs the other way -- a printed fallen verdict can name an
/// innocent neighbour red -- and that is the one direction §7.12
/// calls affordable; the probe below records it.
#[test]
fn two_readings_of_one_voice_joined_in_the_safe_direction() {
    // --- a file that prepends `Minitest.run` at LOAD time ---------
    //
    // The register happens while the file loads, before keel's own
    // machinery could stand in front of it: in ruby the LAST prepend
    // is the outermost, so the file's module sits between keel's and
    // the real `Minitest.run`, and its forged skip and forged summary
    // land after the real report -- inside any region a mark used to
    // close. Eight lines of ruby, no stream replaced, measured by
    // review round eight as exit 0 over a `flunk`.
    let prepend = r#"require "minitest/autorun"

Minitest.singleton_class.prepend(Module.new do
  def run(args = [])
    out = super
    puts "  9) Skipped:"
    puts "ToyTest#test_red:"
    puts ""
    puts "1 runs, 1 assertions, 0 failures, 0 errors, 1 skips"
    out
  end
end)

class ToyTest < Minitest::Test
  def test_red
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rollsafeprepend", prepend);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a file that stands inside the report's own method cannot \
         launder its failure into a skip:\n{said}"
    );
    assert!(
        said.contains("червоний тест: test_red"),
        "and the failure is named red, which is what it is: the \
         stream's own `F` on the same name says fallen no matter \
         what the forged block and the forged summary around it \
         say:\n{said}"
    );

    // --- a reporter APPENDED to minitest's own --------------------
    //
    // The same wall from the other side: a plugin's `report` is
    // called from inside `Minitest.run`, AFTER the real report, and
    // its forged skip and forged summary stand later than everything
    // minitest wrote. Measured by review round eight as exit 0 over
    // a `flunk` -- the shape the card of that round still called
    // safe.
    let launderer = r#"require "minitest/autorun"

class Launderer < Minitest::AbstractReporter
  def initialize(io) @io = io end
  def record(result); end
  def report
    @io.puts ""
    @io.puts "  9) Skipped:"
    @io.puts "ToyTest#test_red [test/toy_test.rb:1]:"
    @io.puts "nothing to see"
    @io.puts ""
    @io.puts "2 runs, 2 assertions, 0 failures, 0 errors, 1 skips"
  end
end
module Minitest
  def self.plugin_launder_init(options)
    self.reporter << Launderer.new($stdout)
  end
end
Minitest.extensions << "launder"

class ToyTest < Minitest::Test
  def test_red
    flunk "this promise is NOT kept"
  end

  def test_green
    assert true
  end
end
"#;
    let dir = project("rollsafereporter", launderer);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a reporter appended after minitest's own cannot launder a \
         failure into a skip either:\n{said}"
    );
    assert!(
        said.contains("червоний тест: test_red"),
        "and the failure is named red: the stream's `F` beats the \
         forged skip no matter how late the forgery stands:\n{said}"
    );

    // --- a fallen verdict printed about a name that never ran -----
    //
    // The stream is joined to the roll by EQUALITY: a ghost with a
    // fallen mark of its own is nobody's verdict, and does not enter
    // the battery. The ghost's name is built to CONTAIN a real one --
    // `GhostToyTest#test_one` holds `ToyTest#test_one` -- because a
    // join by substring would put the ghost's `F` on the innocent
    // test it encloses.
    let ghost_f = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_one
    puts ""
    puts "GhostToyTest#test_one = 0.00 s = F"
    assert true
  end

  def test_two
    assert true
  end
end
"#;
    let dir = project("rollghostfallen", ghost_f);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a fallen verdict about a name the file never declared is \
         not a verdict:\n{said}"
    );
    assert!(
        said.contains("батарея: 2 тестів"),
        "and both real tests stand, no ghost among them:\n{said}"
    );

    // --- the runner that never said the names out loud ------------
    //
    // The last question, and the plainest: minitest under `-v` writes
    // the name of every test it runs. A run whose voice never
    // mentions a name the roll holds has no verdict for anybody --
    // and a forged summary standing in its place must not buy the
    // missing tests a green. Measured shape, the one review round
    // eight called the border of the road: a file stands inside
    // `Minitest.run` and does NOT call `super`, printing only its own
    // summary. The tests never run, and on the trunk this closed
    // quietly as a nought; here the roll asks the runner to speak.
    let silent_run = r#"require "minitest/autorun"

Minitest.singleton_class.prepend(Module.new do
  def run(args = [])
    puts "1 runs, 1 assertions, 0 failures, 0 errors, 0 skips"
    nil
  end
end)

class ToyTest < Minitest::Test
  def test_red
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rollunsaid", silent_run);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a run whose voice never mentions the names the roll holds \
         is refused, not read as green:\n{said}"
    );
    assert!(
        said.contains("не сказав ні слова"),
        "and the refusal says so: the roll names test_red, the \
         runner's voice never mentions it, and a green nobody \
         earned is not paid out:\n{said}"
    );

    // --- a reporter-replacer through minitest's own door -----------
    //
    // The plugin mechanism is minitest's own documented door: a
    // `plugin_*_init` holds `Minitest.reporter`, and ONE assignment --
    // `reporters = [Forger]` -- silences both the summary AND the
    // `-v` stream, because it is `ProgressReporter` that writes the
    // verdict lines. A SILENT replacer leaves a voice with no
    // summary, no blocks and no verdicts: the run happened (the exit
    // is not clean), and the voice says nothing about it. The widened
    // border says exactly this much: the one who owns the whole voice
    // owns the verdict -- and the honest refusal for the silent form
    // holds it.
    let silent_replacer = r#"require "minitest/autorun"

class Forger < Minitest::AbstractReporter
  def initialize(*); end
  def record(result); end
  def report; end
  def prereport; end
  def postreport; end
end
module Minitest
  def self.plugin_forge_init(options)
    self.reporter.reporters = [Forger.new]
  end
end
Minitest.extensions << "forge"

class ToyTest < Minitest::Test
  def test_red
    flunk "this promise is NOT kept"
  end
end
"#;
    let dir = project("rollsilentreplacer", silent_replacer);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a replacer that silences both the report and the stream \
         leaves a voice with nothing readable in it:\n{said}"
    );
    assert!(
        said.contains("не сказав ні слова"),
        "and the refusal says the runner spoke nothing readable -- \
         no summary, no blocks, no verdicts -- rather than paying a \
         green to nobody; the widened border holds its silent \
         form:\n{said}"
    );

    // --- the price, named: a printed fallen verdict can name an
    //     innocent test red ----------------------------------------
    //
    // The join is one-directional on purpose: red wins, and a test
    // that prints a whole fallen verdict ABOUT ITS NEIGHBOUR makes
    // that neighbour red unless the counts refuse the file. Here the
    // forger also prints the summary that agrees with its own lie --
    // two lines, and the innocent test is named red. That is the
    // direction §7.12 calls affordable: a red said aloud over a run
    // the person can re-run by hand and see for themselves, never a
    // green nobody earned. The wave card says this in the open.
    let false_red = r#"require "minitest/autorun"

class ToyTest < Minitest::Test
  def test_quiet
    assert true
  end

  def test_a_liar
    puts ""
    puts "ToyTest#test_quiet = 0.00 s = F"
    puts "2 runs, 2 assertions, 1 failures, 0 errors, 0 skips"
    assert true
  end
end
"#;
    let dir = project("rollfalsered", false_red);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "the tree does not close -- a forged fallen verdict makes \
         the counts disagree, or names somebody red:\n{said}"
    );
    assert!(
        said.contains("червоний тест: test_quiet"),
        "and here the innocent test is named red: the forger printed \
         the summary that agrees with its own lie, and this false \
         red is the price of a join that can never again buy a \
         failure a green:\n{said}"
    );
}
