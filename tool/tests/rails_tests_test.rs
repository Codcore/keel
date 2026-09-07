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

/// Every line the project's own `bin/rails` wrote down about how it
/// was called -- the proof that the road taken was the Rails one.
fn argv(dir: &Path) -> String {
    std::fs::read_to_string(dir.join("rails-argv.txt")).unwrap_or_default()
}

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
    // The shim RECORDS what it was called with, and that record is
    // the only thing telling the two roads apart: review 0059 R-2
    // measured a transparent shim -- `exec ruby …` and nothing else
    // -- letting mutants that put `run_all` and `run_test` back on
    // `ruby -Itest` walk the whole battery, because both roads then
    // printed the same bytes. The second reading of this tongue
    // solved it the same way (a shim that writes argv), and the
    // contract says so.
    std::fs::write(
        dir.join("bin/rails"),
        "#!/bin/sh\n\
         echo \"RAILSARGV: $*\" >> \"$(dirname \"$0\")/../rails-argv.txt\"\n\
         echo \"RUBYOPT=${RUBYOPT-}\" >> \"$(dirname \"$0\")/../rails-argv.txt\"\n\
         if [ \"$1\" != \"test\" ]; then echo \"unknown command: $1\" >&2; exit 1; fi\n\
         shift\n\
         exec ruby -Itest \"$@\"\n",
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
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
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
    // The ROAD, not only the verdict (review 0059 R-2): the project's
    // own runner was called, with the file and the method, and it was
    // told the encoding of its arguments (R-1).
    let seen = argv(&dir);
    assert!(
        seen.contains("RAILSARGV: test test/models/toy_test.rb -n test_greets_a_user"),
        "the one test went through the project's own bin/rails, by file \
         and by method:\n{seen}"
    );
    assert!(
        seen.contains("RUBYOPT=-EUTF-8"),
        "and carried the word about encoding this road can only say \
         through RUBYOPT (§0051's lesson, review 0059 R-1):\n{seen}"
    );

    // The same word, MEASURED where its loss shows: a name past ASCII
    // under a locale that is not UTF-8. Before R-1 this road ran
    // nothing at all here while the plain one ran it green.
    let unicode = format!(
        "require_relative \"../test_helper\"\nrequire_relative \"../../lib/toy\"\n\n\
         class ToyTest < AppTest\n  \
         # proves: it-works@{rev}\n  \
         test \"вітає ünïcode\" do\n    \
         assert Toy.works\n  \
         end\nend\n"
    );
    let dir = project("railsunicode", &unicode);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(dir.join("msg.txt"), "work: the toy grows\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args([
            "gate",
            dir.join("msg.txt").to_str().unwrap(),
            dir.to_str().unwrap(),
        ])
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env_remove("RUBYOPT")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "a name past ASCII runs on the Rails road too, under a locale \
         that is not UTF-8:\n{said}"
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
    let seen = argv(&dir);
    assert!(
        seen.contains("RAILSARGV: test test/models/toy_test.rb -v"),
        "and the battery itself went through bin/rails, one process per \
         file, asking minitest for its own verdicts (§7.13 runs it three \
         times, so the record holds three such lines):\n{seen}"
    );
    assert_eq!(
        seen.matches("RAILSARGV: test").count(),
        3,
        "three runs of the closing court, three calls of the project's \
         runner -- and none of them a `ruby -Itest` behind its back:\n{seen}"
    );

    // --- the advice a person is handed is the line Rails answers
    // to, and the generated CI runs the project's own battery ---
    // The wave's work is not done yet, so the step is the one that
    // hands a person the run line of the promise's own test.
    let dir = project("railsadvice", &test_file(&rev));
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        said.contains("bin/rails test") && said.contains("test_greets_a_user"),
        "the step hands the line a person in Rails actually types, with \
         the method ActiveSupport built:\n{said}"
    );
    // No mirror assert here: `dead_assert_test` is right that a
    // negative one over a line nothing prints can never fail. What is
    // asked is asked above -- the line IS the Rails one, and one line
    // is handed per tag.
    let (said, _) = keel(&dir, &["update"]);
    let workflow = std::fs::read_to_string(dir.join(".github/workflows/keel.yml")).unwrap_or(said);
    assert!(
        workflow.contains("bin/rails test"),
        "and the generated workflow runs the battery Rails runs:\n{workflow}"
    );

    // --- every spelling ActiveSupport takes, and the name it
    // builds from each (review 0059 R-10; single quotes measured by
    // the author) ---
    for (name, decl, method) in [
        (
            "railssingle",
            "test 'greets a user' do",
            "test_greets_a_user",
        ),
        (
            "railsparens",
            "test(\"greets a user\") do",
            "test_greets_a_user",
        ),
        (
            "railsruns",
            "test \"greets   a    user\" do",
            "test_greets_a_user",
        ),
        ("railstab", "test \"tab\\tname\" do", "test_tab_name"),
    ] {
        let body = format!(
            "require_relative \"../test_helper\"\nrequire_relative \"../../lib/toy\"\n\n\
             class ToyTest < AppTest\n  \
             # proves: it-works@{rev}\n  \
             {decl}\n    \
             assert Toy.works\n  \
             end\nend\n"
        );
        let dir = project(name, &body);
        let (said, code) = keel(&dir, &["check"]);
        assert_eq!(code, 0, "{name}: `{decl}` is a test:\n{said}");
        assert!(
            said.contains("тегів тестів звірено: 1"),
            "{name}: and the tag over it is read:\n{said}"
        );
        git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
        let (said, _) = keel(&dir, &["next"]);
        assert!(
            said.contains(method),
            "{name}: with the method ActiveSupport builds ({method}):\n{said}"
        );
        // And the runner really answers to that name -- asked of the
        // project's own minitest, not of the reader.
        std::fs::write(dir.join("msg.txt"), "work: the toy grows\n").unwrap();
        let (said, code) = keel(&dir, &["gate", dir.join("msg.txt").to_str().unwrap()]);
        assert_eq!(
            code, 0,
            "{name}: and the test really runs under it:\n{said}"
        );
    }

    // --- a name built at RUN TIME is refused aloud, as the second
    // reading refuses it (review 0059 R-8) ---
    let dynamic = format!(
        "require_relative \"../test_helper\"\nrequire_relative \"../../lib/toy\"\n\n\
         class ToyTest < AppTest\n  \
         # proves: it-works@{rev}\n  \
         test \"greets #{{Toy.name}}\" do\n    \
         assert Toy.works\n  \
         end\nend\n"
    );
    let dir = project("railsdynamic", &dynamic);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "an interpolated name is a refusal:\n{said}");
    assert!(
        said.contains("будується під час бігу"),
        "and it names the REASON -- the name does not exist before the \
         run -- instead of sending the person to check the tag:\n{said}"
    );

    // --- the line handed over is one a shell will take whole
    // (review 0059 R-5) ---
    let quoted = format!(
        "require_relative \"../test_helper\"\nrequire_relative \"../../lib/toy\"\n\n\
         class ToyTest < AppTest\n  \
         # proves: it-works@{rev}\n  \
         test \"it's fine\" do\n    \
         assert Toy.works\n  \
         end\nend\n"
    );
    let dir = project("railsquote", &quoted);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["next"]);
    let line = said
        .lines()
        .find(|line| line.contains("bin/rails test"))
        .unwrap_or_else(|| panic!("the step hands a run line:\n{said}"))
        .trim();
    let out = Command::new("sh")
        .args(["-c", &format!("{line} 2>&1 | tail -3")])
        .current_dir(&*dir)
        .output()
        .unwrap();
    let ran = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        out.status.success() && ran.contains("1 runs"),
        "pasted into a shell, the line runs that one test -- an \
         apostrophe in a Rails name must not leave the shell waiting \
         for a closing quote (review 0046 R-5, measured again as 0059 \
         R-5):\n{line}\n{ran}"
    );

    // --- the CI step runs what the closing court runs, system tests
    // included (review 0059 R-6) ---
    let dir = project("railssystem", &test_file(&rev));
    std::fs::create_dir_all(dir.join("test/system")).unwrap();
    std::fs::write(
        dir.join("test/system/thing_test.rb"),
        "require_relative \"../test_helper\"\n\nclass ThingTest < AppTest\n  \
         test \"drives\" do\n    assert true\n  end\nend\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "a system test"]);
    let (said, _) = keel(&dir, &["update"]);
    let workflow = std::fs::read_to_string(dir.join(".github/workflows/keel.yml")).unwrap_or(said);
    assert!(
        workflow.contains("bin/rails test:system"),
        "`bin/rails test` leaves test/system out on purpose, and the \
         closing court runs it by name -- so the step says both:\n{workflow}"
    );

    // --- TWO marks, and neither alone (review 0059 R-3): the
    // negative side is asked by the LINE the tool hands over and by
    // the shim's silence, not by an exit code that is 0 either way ---
    for (name, missing) in [
        ("railsnorails", "bin/rails"),
        ("railsnoconfig", "config/application.rb"),
    ] {
        let dir = project(name, &test_file(&rev));
        std::fs::remove_file(dir.join(missing)).unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "one mark only"]);
        git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
        let (said, _) = keel(&dir, &["next"]);
        assert!(
            said.contains("ruby -Itest"),
            "{name}: with {missing} gone this is a plain ruby project, and \
             the line says so:\n{said}"
        );
        assert!(
            !said.contains("bin/rails test"),
            "{name}: one mark is not two:\n{said}"
        );
        let (said, _) = keel(&dir, &["close"]);
        assert!(
            argv(&dir).is_empty(),
            "{name}: and the battery never called the project's rails \
             either -- the record is empty:\n{said}\n{}",
            argv(&dir)
        );
    }
}
