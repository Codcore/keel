//! Scenario test of wave 0047: minitest and rspec live in one project.
//!
//! One adapter, two readings: `test/**/*_test.rb` and
//! `spec/**/*_spec.rb`, both read, both run, one battery with keys
//! that do not meet, red from either blocking the close -- and a
//! project of `test/` alone judged as it was before the wave.
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

const BODY: &str = "тіло обіцянки\n\n";

/// A ruby project with a `lib/`, and whichever of `test/` and `spec/`
/// the bodies ask for: minitest in the first, rspec in the second.
fn project(name: &str, test_body: Option<&str>, spec_body: Option<&str>) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
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
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - lib/toy.rb\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    if let Some(body) = test_body {
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(dir.join("test/toy_test.rb"), body).unwrap();
    }
    if let Some(body) = spec_body {
        std::fs::create_dir_all(dir.join("spec")).unwrap();
        std::fs::write(dir.join(".rspec"), "--require spec_helper\n").unwrap();
        std::fs::write(dir.join("spec/spec_helper.rb"), "require \"toy\"\n").unwrap();
        std::fs::write(dir.join("spec/toy_spec.rb"), body).unwrap();
    }
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn minitest(rev: &str, extra: &str) -> String {
    format!(
        "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    assert Toy.works\n  end\n{extra}end\n"
    )
}

fn rspec(rev: &str, extra: &str) -> String {
    format!(
        "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n    expect(Toy.works).to be(true)\n  end\n{extra}end\n"
    )
}

fn gate(dir: &Path) -> (String, i32) {
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, "work: тіло\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
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

fn reviewed(dir: &Path) {
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "review"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// proves: minitest-and-rspec-live-in-one-project@0e8ee7
#[test]
fn minitest_and_rspec_live_in_one_project() {
    if !common::machine_has("rspec").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // Both readings read: a tag in test/ and a tag in spec/ are two
    // tags, and the gate runs both -- one by method, one by id.
    let dir = project("rsboth", Some(&minitest(&rev, "")), Some(&rspec(&rev, "")));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "a project of both readings is judged whole:\n{said}"
    );
    assert!(
        said.contains("тегів тестів звірено: 2"),
        "a tag from test/ and a tag from spec/ are both read:\n{said}"
    );
    assert!(
        said.contains("minitest") && said.contains("rspec"),
        "and the report names both readings and each one's border:\n{said}"
    );
    let (said, code) = gate(&dir);
    assert_eq!(code, 0, "the work passes over both green:\n{said}");
    assert!(said.contains("2 тестів сценаріїв зелені"), "{said}");

    // One battery out of two: the keys `toy_test` and `spec/toy_spec`
    // do not meet, both rolls are counted, and a red from EITHER
    // reading is named and blocks the close.
    let dir = project(
        "rsbothbattery",
        Some(&minitest(
            &rev,
            "\n  def test_it_falls\n    flunk \"навмисно\"\n  end\n",
        )),
        Some(&rspec(
            &rev,
            "\n  it \"falls\" do\n    expect(1).to eq(2)\n  end\n",
        )),
    );
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 4 тестів"),
        "two minitest tests and two rspec examples are four:\n{said}"
    );
    assert!(
        said.contains("test_it_falls") && said.contains("Toy falls"),
        "the red from each reading is named in its own tongue's name:\n{said}"
    );
    assert_ne!(code, 0, "and a red battery does not close:\n{said}");

    // Red only in spec/, green in test/: still blocked -- the second
    // reading is not a lesser reading.
    let dir = project(
        "rsspecred",
        Some(&minitest(&rev, "")),
        Some(&rspec(
            &rev,
            "\n  it \"falls\" do\n    expect(1).to eq(2)\n  end\n",
        )),
    );
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "a red example blocks the close:\n{said}");
    assert!(said.contains("Toy falls"), "{said}");

    // A project of test/ alone is judged as it was before the wave:
    // green, closing, and nothing of spec/ asked of it.
    let dir = project("rsonlytest", Some(&minitest(&rev, "")), None);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a minitest project is judged as before:\n{said}");
    assert!(said.contains("тегів тестів звірено: 1"), "{said}");
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "and closes as before:\n{said}");
    assert!(said.contains("батарея: 1 тестів"), "{said}");

    // Borders said aloud: a helper in spec/support/ is named as not
    // read, and the one-liner is named as the example no tag can hold.
    let dir = project("rsborders", None, Some(&rspec(&rev, "")));
    std::fs::create_dir_all(dir.join("spec/support")).unwrap();
    std::fs::write(dir.join("spec/support/helper.rb"), "# a helper\n").unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "{said}");
    assert!(
        said.contains("spec/support/helper.rb"),
        "the file in spec/ the reading walked past is named:\n{said}"
    );
    assert!(
        said.contains("it {"),
        "and the example no tag can name is named as the border:\n{said}"
    );
    let unread = said
        .lines()
        .find(|line| line.contains("адаптер не читає"))
        .unwrap_or_default();
    assert!(
        unread.contains("spec/support/helper.rb") && !unread.contains("spec_helper.rb"),
        "the helper rspec requires itself is not called unread (review 0047 \
         R-8, mutation M14):\n{unread}"
    );

    // A SyntaxError in the SPEC FILE itself: ruby paints it, rspec's
    // `messages` carry the paint, and the refusal shows a person
    // plain words -- no escape codes (review 0047 R-3).
    let dir = project(
        "rspainted",
        None,
        Some(
            "RSpec.describe Toy do\n  # proves: it-works@aaaaaa\n  it \"works\" do\n    expect(Toy.works).to be(true\n  end\nend\n",
        ),
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a spec that does not parse does not close:\n{said}"
    );
    assert!(
        said.contains("SyntaxError") || said.contains("syntax error"),
        "and the refusal carries ruby's words:\n{said}"
    );
    assert!(
        !said.contains('\u{1b}'),
        "with no colour codes in them:\n{}",
        said.replace('\u{1b}', "<ESC>")
    );

    // minitest's own speed line: `995.6450 runs/s`, and one run in ten
    // `1000.0000 runs/s` -- a substring `0 runs` read that as "nothing
    // ran" and the gate refused a green test one time in ten (review
    // 0047 R-4). Only the summary line says how many ran.
    let fast = "Run options: -n test_it_works --seed 1\n\n# Running:\n\n.\n\nFinished in 0.001000s, 1000.0000 runs/s, 1000.0000 assertions/s.\n\n1 runs, 1 assertions, 0 failures, 0 errors, 0 skips\n";
    assert!(
        matches!(
            keel::ruby::classify(fast, true),
            keel::adapter::Outcome::Green
        ),
        "a green run that happened to be fast is green"
    );
    let none = "Run options: -n test_nothing --seed 1\n\n# Running:\n\n\n\nFinished in 0.000500s, 0.0000 runs/s, 0.0000 assertions/s.\n\n0 runs, 0 assertions, 0 failures, 0 errors, 0 skips\n";
    assert!(
        matches!(
            keel::ruby::classify(none, true),
            keel::adapter::Outcome::NotRun
        ),
        "and a run of nothing is still nothing"
    );

    // `keel next` speaks rspec to a spec: the directory the project
    // keeps its examples in, and the line a person would type --
    // `rspec <file> -e '<full description>'`, with `-e` said to be a
    // substring match.
    let dir = project("rsnext", None, Some(&rspec(&rev, "")));
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        said.contains("rspec spec/toy_spec.rb -e 'Toy works'"),
        "next hands rspec's own run line for a spec:\n{said}"
    );
    assert!(
        said.contains("підрядк"),
        "and says beside it that -e is a substring match (review 0047 \
         R-7):\n{said}"
    );
    assert!(
        !said.contains("ruby -Itest"),
        "and not minitest's for an example:\n{said}"
    );

    // And a project of spec/ alone, with no tag yet, is told the
    // directory it keeps its examples in (review 0047 R-8, N1).
    let dir = project(
        "rsnextdir",
        None,
        Some("RSpec.describe Toy do\n  it \"untagged\" do\n  end\nend\n"),
    );
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        said.contains("читає тести в spec/"),
        "the hint names spec/ where that is what exists:\n{said}"
    );
    assert!(
        said.contains("# proves: it-works@"),
        "and writes the tag in ruby's own mark:\n{said}"
    );
}
