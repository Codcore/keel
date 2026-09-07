//! Scenario test of wave 0059: a Rails project is judged out of the
//! box.
//!
//! Measured by the author on a REAL application (`rails new`, Rails
//! 8.1.3.1) before the plan: the tag reader saw no test at all under
//! `test "greets" do`, so in a Rails project a promise could not be
//! proven -- not a corner, but the framework's ordinary style.
//!
//! The sandbox here holds REAL minitest under that style: a
//! `test_helper` that builds the method from the string exactly as
//! ActiveSupport::TestCase does, and a `bin/rails` that translates
//! the command into the plain runner. What keel is told is what a
//! Rails project tells it -- two marks in the root -- and what keel
//! runs is the project's own `bin/rails`.

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

fn executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

const BODY: &str = "тіло обіцянки Rails\n\n";

/// A Rails-SHAPED project: the two marks the adapter asks for, a
/// `bin/rails` that runs the tests the way Rails does (its own
/// minitest, `-v` and `-n` alike), and a test declared by a string.
fn project(name: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test/models")).unwrap();
    std::fs::create_dir_all(dir.join("bin")).unwrap();
    std::fs::create_dir_all(dir.join("config")).unwrap();
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works\n    true\n  end\nend\n",
    )
    .unwrap();
    // The first mark. Its content is never read -- only that it is
    // there, beside the second.
    std::fs::write(
        dir.join("config/application.rb"),
        "# the application of this sandbox\n",
    )
    .unwrap();
    // The second mark, and the runner itself: `bin/rails test <file>
    // <options>` is what Rails offers a person, and here it leads to
    // the same minitest underneath.
    std::fs::write(
        dir.join("bin/rails"),
        "#!/bin/sh\n\
         if [ \"$1\" != \"test\" ]; then echo \"unknown command: $1\" >&2; exit 1; fi\n\
         shift\n\
         exec ruby -E UTF-8 -Itest \"$@\"\n",
    )
    .unwrap();
    executable(&dir.join("bin/rails"));
    // ActiveSupport::TestCase in five lines: `test "a b" do` becomes
    // the method `test_a_b`, which is the name minitest reports and
    // the name `-n` selects.
    std::fs::write(
        dir.join("test/test_helper.rb"),
        "require \"minitest/autorun\"\n\n\
         class AppTest < Minitest::Test\n  \
         def self.test(name, &block)\n    \
         define_method(\"test_#{name.gsub(/\\s+/, '_')}\", block)\n  \
         end\nend\n",
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
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - lib/toy.rb\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("keel/reviews/0001-a-wave.md"), "# Рецензія\n\nok\n").unwrap();
    std::fs::write(dir.join("test/models/toy_test.rb"), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn test_file(rev: &str) -> String {
    format!(
        "require_relative \"../test_helper\"\nrequire_relative \"../../lib/toy\"\n\n\
         class ToyTest < AppTest\n  \
         # proves: it-works@{rev}\n  \
         test \"greets a user\" do\n    \
         assert Toy.works\n  \
         end\nend\n"
    )
}

/// proves: a-rails-project-is-judged-out-of-the-box@89e259
#[test]
fn a_rails_project_is_judged_out_of_the_box() {
    if !common::machine_has("ruby").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // --- the reader sees the test Rails declares ---
    let dir = project("railstags", &test_file(&rev));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a Rails project is judged whole:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "the tag over `test \"…\" do` is read -- before this wave the \
         reader knew only `def test_…`, and a Rails promise could not \
         be proven at all (§5.5):\n{said}"
    );

    // --- and the tag holds the name minitest reports, so the ONE
    // test really runs: the gate over the work commit is the court
    // that runs it ---
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works\n    true\n  end\n\n  def self.more\n    1\n  end\nend\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    std::fs::write(dir.join("msg.txt"), "work: the toy grows\n").unwrap();
    let (said, code) = keel(&dir, &["gate", dir.join("msg.txt").to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "the gate runs the very test the tag names, through the \
         project's own bin/rails:\n{said}"
    );
    assert!(
        !said.contains("не виконав") && !said.contains("did not run"),
        "and does not lose it as `not run` -- the name it selects by is \
         the one ActiveSupport builds (test_greets_a_user):\n{said}"
    );

    // --- the battery is the project's own, and the closing court
    // names what it watched fail, by the ruby name ---
    let falls = format!(
        "require_relative \"../test_helper\"\nrequire_relative \"../../lib/toy\"\n\n\
         class ToyTest < AppTest\n  \
         # proves: it-works@{rev}\n  \
         test \"greets a user\" do\n    \
         assert Toy.works\n  \
         end\n\n  \
         test \"falls on purpose\" do\n    \
         flunk \"навмисно\"\n  \
         end\nend\n"
    );
    let dir = project("railsbattery", &falls);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("test_falls_on_purpose"),
        "the closing court names the Rails test it watched fail:\n{said}"
    );
    assert!(
        !said.contains("test_greets_a_user"),
        "and the green one is not dragged in with it:\n{said}"
    );

    // --- the advice a person is handed is the line Rails answers
    // to, and the generated CI runs the project's own battery ---
    // The wave's work is not done yet, so the step is the one that
    // hands a person the run line of the promise's own test.
    let dir = project("railsadvice", &test_file(&rev));
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        said.contains("bin/rails test") && said.contains("-n test_greets_a_user"),
        "the step hands the line a person in Rails actually types, with \
         the method ActiveSupport built:\n{said}"
    );
    assert!(
        !said.contains("ruby -Itest test/models"),
        "and not the line that boots no application:\n{said}"
    );
    let (said, _) = keel(&dir, &["update"]);
    let workflow = std::fs::read_to_string(dir.join(".github/workflows/keel.yml")).unwrap_or(said);
    assert!(
        workflow.contains("bin/rails test"),
        "and the generated workflow runs the battery Rails runs:\n{workflow}"
    );

    // --- a project WITHOUT the two marks stays on the first reading ---
    let dir = project("railsnomarks", &test_file(&rev));
    std::fs::remove_file(dir.join("bin/rails")).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "no rails here"]);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "without bin/rails the project is a plain ruby project, and \
         the reader still reads the tag:\n{said}"
    );
}
