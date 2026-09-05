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

/// The battery's verdicts come from minitest's own voice, not from
/// the file's text (review 0038 R-1, R-2, R-6, R-20).
#[test]
fn the_ruby_battery_believes_only_what_ran() {
    let rev = keel::rev::text_rev(BODY);

    // R-1: a test file that does not parse is not a green file. The
    // adapter refuses aloud, exactly as the cargo one does over
    // "could not compile" -- without a run there is no verdict.
    let broken = format!(
        "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\n"
    );
    let dir = project("rubybroken", BODY, &broken);
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a file that does not parse never closes a wave:\n{said}"
    );
    assert!(
        !said.contains("закрита"),
        "and the verdict does not read as closure:\n{said}"
    );

    // R-6: one test name is a prefix of another. Only the one that
    // truly failed is named.
    let pair = format!(
        "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\n\n  def test_it_works_more\n    flunk \"навмисно\"\n  end\nend\n"
    );
    let dir = project("rubyprefix", BODY, &pair);
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("test_it_works_more"),
        "the failing test is named:\n{said}"
    );
    assert!(
        !said
            .lines()
            .any(|line| line.contains("test_it_works") && !line.contains("test_it_works_more")),
        "and the green one whose name it merely begins with is not \
         dragged in with it (review 0038 R-6):\n{said}"
    );

    // R-20: a `def test...` in a class minitest never runs is not a
    // test, and the battery's count does not swell with it.
    let helpers = format!(
        "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass Helper\n  def testify\n    1\n  end\nend\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\nend\n"
    );
    let dir = project("rubycount", BODY, &helpers);
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 1 тестів"),
        "the battery counts what minitest ran, not what looked like a \
         test in the file (review 0038 R-20):\n{said}"
    );
}

/// A tag over a method minitest never runs is "not run", not "work
/// passes" (review 0038 R-2): minitest leaves with 0 over an unknown
/// -n, so the courts must read its words and not its exit code.
#[test]
fn a_tag_over_a_non_test_is_not_a_green_run() {
    let rev = keel::rev::text_rev(BODY);
    let body = format!(
        "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def helper_is_not_a_test\n    assert Toy.works\n  end\nend\n"
    );
    let dir = project("rubynotrun", BODY, &body);
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
    assert!(
        !said.contains("робота проходить"),
        "and the gate does not say it did:\n{said}"
    );
}

/// The border §7.12 names is said by the tool, not only by the wave
/// (review 0038 R-7).
#[test]
fn the_ruby_border_is_said_aloud() {
    let rev = keel::rev::text_rev(BODY);
    let dir = project("rubyborder", BODY, &test_file(&rev));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a whole ruby project is judged:\n{said}");
    assert!(
        said.contains("ruby") && said.contains("не зібрався"),
        "the check names the tongue's own border -- failed and did not \
         build are not told apart by an exit code here (§7.12):\n{said}"
    );
}

/// The shapes `classify` must tell apart, played without a project
/// on disk (review 0038 R-10): the function was made public for
/// exactly this probe and the probe was never written, so three
/// mutations of it -- "never BuildBroken", "never NotRun", and the
/// whole body replaced by Green -- left the battery green.
#[test]
fn what_ruby_said_is_read_before_how_it_left() {
    use keel::adapter::Outcome;

    // Nothing loaded: a broken build is not a failure (journal A3),
    // whatever the exit code says.
    assert!(matches!(
        keel::ruby::classify("test.rb:4: syntax error, SyntaxError", false),
        Outcome::BuildBroken(_)
    ));
    assert!(matches!(
        keel::ruby::classify("cannot load such file -- lib/gone (LoadError)", false),
        Outcome::BuildBroken(_)
    ));

    // Minitest leaves with 0 over an unknown -n. Asking the exit
    // code first turned that into green and let work through a gate.
    assert!(matches!(
        keel::ruby::classify("0 runs, 0 assertions, 0 failures, 0 errors, 0 skips", true),
        Outcome::NotRun
    ));

    // The two plain answers.
    assert!(matches!(
        keel::ruby::classify("1 runs, 1 assertions, 0 failures, 0 errors, 0 skips", true),
        Outcome::Green
    ));
    assert!(matches!(
        keel::ruby::classify("1 runs, 1 assertions, 1 failures, 0 errors, 0 skips", false),
        Outcome::Failed
    ));

    // And where ruby's words do not say which it was, the failure is
    // taken as a failure -- the direction that cannot turn red into
    // green (§7.12).
    assert!(matches!(
        keel::ruby::classify("some words ruby chose not to explain", false),
        Outcome::Failed
    ));
}

/// A `.rb` file in test/ that this adapter walks past is named, not
/// skipped in silence (review 0038 R-19).
#[test]
fn what_the_ruby_adapter_does_not_read_is_named() {
    let rev = keel::rev::text_rev(BODY);
    let dir = project("rubyspec", BODY, &test_file(&rev));
    std::fs::write(dir.join("test/toy_spec.rb"), "RSpec.describe Toy do\nend\n").unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "an unread file is a limit, not a finding:\n{said}");
    assert!(
        said.contains("toy_spec.rb") && said.contains("не перевірено"),
        "and the verdict names the file it did not read:\n{said}"
    );
}
