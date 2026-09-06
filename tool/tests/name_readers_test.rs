//! Scenario test of wave 0051: a name is read as its tongue writes it.
//!
//! The global review of 2026-09-06 (bugs cut R-9, R-10, R-19, R-20,
//! R-22) measured the readers of names reading a file otherwise than
//! its runner does: rspec's group closed by the word `end` inside a
//! string, elixir's `describe` closed by a line starting with `end…`
//! and opened by a comment ending in ` do`, `def test_ünïcode` cut to
//! `test_` in python and ruby, a decorator's parentheses counted
//! inside string literals, and the word `LoadError` in a failure
//! message read as a broken build. Each case is read here by the tag
//! reader itself -- pure, no runner -- and judged by the gate against
//! the real runner where one stands.
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

/// The commit court over a `work:` message on the wave's branch.
fn gate(dir: &Path) -> (String, i32) {
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

const BODY: &str = "тіло обіцянки\n\n";

/// One wave, one scenario `it-works`, one transform `work`, the review
/// beside it, git around it, the wave's branch out.
fn frame(dir: &Path, adapter: &str, file: &str) {
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
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
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - {file}\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// The names the reader gives the tags of one file text.
fn names(file: &str, text: &str) -> Vec<String> {
    keel::tags::scan_text(Path::new(file), text)
        .unwrap_or_else(|e| panic!("{file}: the reader refused: {}", e.reason))
        .into_iter()
        .map(|t| t.test)
        .collect()
}

/// proves: a-name-is-read-as-its-tongue-writes-it@23460a
#[test]
fn a_name_is_read_as_its_tongue_writes_it() {
    let rev = keel::rev::text_rev(BODY);

    // --- rspec: the word `end` inside strings before the tagged
    // example closes no group ---
    let spec = format!(
        "RSpec.describe Toy do\n  it \"syncs end-to-end\" do\n    expect(\"the end of it\").to include(\"end\")\n  end\n\n  # proves: it-works@{rev}\n  it \"holds\" do\n    expect(Toy.works).to be(true)\n  end\nend\n"
    );
    assert_eq!(
        names("spec/toy_spec.rb", &spec),
        vec!["Toy holds".to_string()],
        "the group survives an `end` inside a string (bugs R-9)"
    );

    // --- elixir: a line starting with `end…` inside a test, and a
    // comment ending in ` do` -- neither is an `end` or a `do` ---
    let endpoint = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  describe \"group\" do\n    test \"first\" do\n      endpoint = \"http://x\"\n      assert endpoint != nil\n    end\n\n    # proves: it-works@{rev}\n    test \"holds\" do\n      assert Toy.works()\n    end\n  end\nend\n"
    );
    assert_eq!(
        names("test/toy_test.exs", &endpoint),
        vec!["group holds".to_string()],
        "`endpoint = …` is not an `end` (bugs R-10)"
    );
    let comment = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  describe \"group\" do\n    # TODO: decide what to do\n    test \"inside\" do\n      assert true\n    end\n  end\n\n  # proves: it-works@{rev}\n  test \"holds\" do\n    assert Toy.works()\n  end\nend\n"
    );
    assert_eq!(
        names("test/toy_test.exs", &comment),
        vec!["holds".to_string()],
        "a comment ending in ` do` opens nothing, and the test outside the \
         describe carries no group (bugs R-10)"
    );

    // --- python and ruby: a name with letters beyond ASCII is read
    // whole ---
    let py = format!(
        "from toy import works\n\n\n# proves: it-works@{rev}\ndef test_ünïcode():\n    assert works()\n"
    );
    assert_eq!(
        names("tests/test_toy.py", &py),
        vec!["test_ünïcode".to_string()],
        "python: the identifier is read whole (bugs R-19)"
    );
    let rb = format!(
        "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_ünïcode\n    assert true\n  end\nend\n"
    );
    assert_eq!(
        names("test/toy_test.rb", &rb),
        vec!["test_ünïcode".to_string()],
        "ruby: the identifier is read whole (bugs R-19)"
    );

    // --- python: parentheses inside a decorator's strings are text ---
    let param = format!(
        "import pytest\nfrom toy import works\n\n\n# proves: it-works@{rev}\n@pytest.mark.parametrize(\"expr\", [\"f(x\", \"g(y\"])\ndef test_it_works(expr):\n    assert works() and expr\n"
    );
    assert_eq!(
        names("tests/test_toy.py", &param),
        vec!["test_it_works".to_string()],
        "the decorator ends where its parentheses end in CODE (bugs R-20)"
    );

    // --- minitest: the word `LoadError` in a failure message is a
    // failure; a broken build is known by ruby's own line shape ---
    assert!(
        matches!(
            keel::ruby::classify(
                "Minitest::Assertion: this is not a LoadError, but says the word\n1 runs, 1 assertions, 1 failures, 0 errors, 0 skips\n",
                false
            ),
            keel::adapter::Outcome::Failed
        ),
        "a failure that says the word is a failure (bugs R-22)"
    );
    assert!(
        matches!(
            keel::ruby::classify(
                "test/toy_test.rb:2:in 'require_relative': cannot load such file -- /x/lib/toy (LoadError)\n",
                false
            ),
            keel::adapter::Outcome::BuildBroken(_)
        ),
        "and ruby's own LoadError line is a broken build"
    );

    // --- and the gate agrees with the runners, where they stand ---
    if common::machine_has("rspec").ready() {
        let dir = keel_sandbox("namerspec");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("spec")).unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(dir.join(".rspec"), "--require spec_helper\n").unwrap();
        std::fs::write(dir.join("spec/spec_helper.rb"), "require \"toy\"\n").unwrap();
        std::fs::write(dir.join("spec/toy_spec.rb"), &spec).unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = gate(&dir);
        assert_eq!(
            code, 0,
            "rspec runs `Toy holds` and the work passes:\n{said}"
        );
    }
    if common::machine_has("mix").ready() {
        let dir = keel_sandbox("nameelixir");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("mix.exs"),
            "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\n  def application, do: []\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("lib/toy.ex"),
            "defmodule Toy do\n  def works, do: true\nend\n",
        )
        .unwrap();
        std::fs::write(dir.join("test/test_helper.exs"), "ExUnit.start()\n").unwrap();
        std::fs::write(dir.join("test/toy_test.exs"), &endpoint).unwrap();
        frame(&dir, "elixir", "lib/toy.ex");
        let (said, code) = gate(&dir);
        assert_eq!(
            code, 0,
            "mix runs `group holds` and the work passes:\n{said}"
        );
    }
    if common::machine_has("pytest").ready() {
        let dir = keel_sandbox("nameparam");
        std::fs::create_dir_all(dir.join("src/toy")).unwrap();
        std::fs::create_dir_all(dir.join("tests")).unwrap();
        std::fs::write(
            dir.join("pyproject.toml"),
            "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n\n[tool.pytest.ini_options]\npythonpath = [\"src\"]\ntestpaths = [\"tests\"]\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("src/toy/__init__.py"),
            "def works():\n    return True\n",
        )
        .unwrap();
        std::fs::write(dir.join("tests/test_toy.py"), &param).unwrap();
        frame(&dir, "python", "src/toy/__init__.py");
        let (said, code) = gate(&dir);
        assert_eq!(
            code, 0,
            "pytest runs the parametrized test and the work passes:\n{said}"
        );
    }
    if common::machine_has("ruby").ready() {
        let dir = keel_sandbox("nameloaderr");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy_test.rb"),
            format!("require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    flunk \"this is not a LoadError, but says the word\"\n  end\nend\n"),
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = gate(&dir);
        assert_ne!(code, 0, "the failing test does not pass the work:\n{said}");
        assert!(
            said.contains("падає") && !said.contains("не збираються"),
            "and it is a failure, not a broken build:\n{said}"
        );
    }
}
