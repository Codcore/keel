//! Scenario test of wave 0051: a test is selected as its runner
//! selects it.
//!
//! The global review of 2026-09-06 (bugs cut R-11, R-14, R-17, R-19,
//! R-26) measured tests the courts could not run by their names alone:
//! an ExUnit test with letters beyond ASCII, which `mix test --only`
//! excludes entirely; a minitest method with such letters, which
//! `ruby -n` under a locale without UTF-8 does not select; a cargo
//! target renamed by `[[test]] name`, which `--test <stem>` refuses
//! while the battery reads it fine; a pytest file in a directory with
//! a space, cut in two by the roll's `split_whitespace`. And rspec's
//! JSON lay under a predictable name in the shared /tmp.
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

/// Both courts over one tree: the gate passes the work, the closing
/// court closes the wave.
fn both_courts_green(dir: &Path, what: &str) {
    let (said, code) = gate(dir);
    assert_eq!(
        code, 0,
        "{what}: the gate runs the very test and passes:\n{said}"
    );
    assert!(
        !said.contains("не виконав"),
        "{what}: and did not lose it as `not run`:\n{said}"
    );
    let (said, code) = keel(dir, &["close"]);
    assert!(
        said.contains("0001-a-wave: закрита"),
        "{what}: the battery reads its verdict and the wave closes:\n{said}"
    );
    assert_eq!(code, 0, "{what}: nothing red:\n{said}");
}

/// proves: a-test-is-selected-as-its-runner-selects-it@200446
#[test]
fn a_test_is_selected_as_its_runner_selects_it() {
    let rev = keel::rev::text_rev(BODY);

    // --- elixir: the line is exact -- a red one-liner right above the
    // tagged test and one right below, so a line off by one runs a red
    // test and the gate says so (review 0051 R-4; the closing court
    // is not asked here: it refuses a red test nobody claims, which is
    // its own court) ---
    if common::machine_has("mix").ready() {
        let dir = keel_sandbox("selectline");
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
        std::fs::write(
            dir.join("test/toy_test.exs"),
            format!("defmodule ToyTest do\n  use ExUnit.Case\n\n  test \"falls above\", do: assert(false)\n  # proves: it-works@{rev}\n  test \"ünïcode holds\", do: assert(Toy.works())\n  test \"falls below\", do: assert(false)\nend\n"),
        )
        .unwrap();
        frame(&dir, "elixir", "lib/toy.ex");
        let (said, code) = gate(&dir);
        assert_eq!(
            code, 0,
            "elixir: the gate runs the tagged line and no neighbour:\n{said}"
        );
        assert!(
            !said.contains("не виконав") && !said.contains("падає"),
            "elixir: neither `not run` nor a neighbour's red:\n{said}"
        );
    }
    // The rule under it: a line that names no test -- mix says "All
    // tests have been excluded" and leaves with 0 -- is `not run`, not
    // green (contract tool-adapter-elixir; review 0051 R-12: the
    // mutant without this branch survived every probe of the wave).
    assert!(
        matches!(
            keel::elixir::classify(
                "Excluding tags: [:test]\nIncluding tags: [line: \"1\"]\n\nAll tests have been excluded.\n\nFinished in 0.00s\n0 tests, 0 failures\n",
                0
            ),
            keel::adapter::Outcome::NotRun
        ),
        "a line that names no test is `not run`, whatever the exit code"
    );

    // --- elixir: a name beyond ASCII, selected by the file's line ---
    if common::machine_has("mix").ready() {
        let dir = keel_sandbox("selectelixir");
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
        std::fs::write(
            dir.join("test/toy_test.exs"),
            format!("defmodule ToyTest do\n  use ExUnit.Case\n\n  test \"plain first\" do\n    assert true\n  end\n\n  # proves: it-works@{rev}\n  test \"ünïcode holds\" do\n    assert Toy.works()\n  end\nend\n"),
        )
        .unwrap();
        frame(&dir, "elixir", "lib/toy.ex");
        both_courts_green(&dir, "elixir");
    }

    // --- ruby: a method beyond ASCII, under a locale without UTF-8 ---
    if common::machine_has("ruby").ready() {
        let dir = keel_sandbox("selectruby");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy_test.rb"),
            format!("require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_ünïcode\n    assert Toy.works\n  end\nend\n"),
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        // The gate under the C locale: the word to ruby is the
        // adapter's, not the environment's.
        let msg = dir.join("COMMIT_EDITMSG");
        std::fs::write(&msg, "work: тіло\n").unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
            .env("LANG", "C")
            .env("LC_ALL", "C")
            .output()
            .unwrap();
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.status.code(),
            Some(0),
            "ruby: `-n test_ünïcode` selects the method whatever the locale:\n{said}"
        );
        let (said, code) = keel(&dir, &["close"]);
        assert!(
            said.contains("0001-a-wave: закрита"),
            "ruby: and the wave closes:\n{said}"
        );
        assert_eq!(code, 0, "ruby: nothing red:\n{said}");
    }

    // --- cargo: a target renamed by [[test]] name ---
    let dir = keel_sandbox("selectrenamed");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[test]]\nname = \"renamed\"\npath = \"tests/w_test.rs\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(toy::works());\n}}\n"),
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    both_courts_green(&dir, "cargo");
    // The same target spelled `./tests/w_test.rs`, which cargo accepts
    // as the same file (review 0051 R-8).
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[[test]]\nname = \"renamed\"\npath = \"./tests/w_test.rs\"\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "work: the path spelled with ./",
        ],
    );
    let (said, code) = gate(&dir);
    assert_eq!(
        code, 0,
        "cargo: `./tests/w_test.rs` names the same target:\n{said}"
    );

    // The `-rA` summary read by node shape, not cut at the first ` - `:
    // a dash in the directory, a dash in a parametrize id, and the
    // message after (review 0051 R-14; a project `addopts = "-q"`
    // silences the progress line that used to save it).
    let ran = keel::python::ran(
        "PASSED tests/my - dir/test_x.py::test_a\nFAILED tests/x.py::test_b[c - d] - AssertionError: x - y\nERROR tests/x.py::TestK::test_c - boom - twice\n",
    );
    assert_eq!(
        ran,
        vec![
            (
                "tests/my - dir/test_x.py".to_string(),
                "test_a".to_string(),
                "PASSED".to_string()
            ),
            (
                "tests/x.py".to_string(),
                "test_b[c - d]".to_string(),
                "FAILED".to_string()
            ),
            (
                "tests/x.py".to_string(),
                "TestK::test_c".to_string(),
                "ERROR".to_string()
            ),
        ],
        "the node is read whole: the dash in a directory or an id is not the message"
    );

    // --- pytest: a directory with a space ---
    if common::machine_has("pytest").ready() {
        let dir = keel_sandbox("selectspace");
        std::fs::create_dir_all(dir.join("src/toy")).unwrap();
        std::fs::create_dir_all(dir.join("tests/my dir")).unwrap();
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
        std::fs::write(
            dir.join("tests/my dir/test_toy.py"),
            format!("from toy import works\n\n\n# proves: it-works@{rev}\ndef test_it_works():\n    assert works()\n"),
        )
        .unwrap();
        frame(&dir, "python", "src/toy/__init__.py");
        both_courts_green(&dir, "python");
    }

    // --- rspec: the directory of the run is gone on a refusal too
    // (review 0051 R-5), and a temp dir that cannot be written is
    // named for what it is (R-9) ---
    if common::machine_has("ruby").ready() {
        let dir = keel_sandbox("selectrspectmp");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("spec")).unwrap();
        std::fs::create_dir_all(dir.join("tmp")).unwrap();
        std::fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        std::fs::write(
            dir.join("spec/toy_spec.rb"),
            format!(
                "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let msg = dir.join("COMMIT_EDITMSG");
        std::fs::write(&msg, "work: тіло\n").unwrap();
        // A PATH with git on it and no rspec: the run is refused
        // before rspec speaks, and the court still knows its branch.
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        let git_path = String::from_utf8(
            Command::new("sh")
                .args(["-c", "command -v git"])
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(git_path.trim(), dir.join("bin/git")).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
            .env("PATH", dir.join("bin"))
            .env("TMPDIR", dir.join("tmp"))
            .output()
            .unwrap();
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_ne!(out.status.code(), Some(0), "rspec is not there:\n{said}");
        let left: Vec<_> = std::fs::read_dir(dir.join("tmp"))
            .unwrap()
            .flatten()
            .collect();
        assert!(
            left.is_empty(),
            "and the run's directory is gone with the refusal: {left:?}\n{said}"
        );
        // A temp dir that does not exist: the refusal says so, and the
        // instead speaks of TMPDIR, not of a name already taken.
        let out = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
            .env("TMPDIR", dir.join("nowhere"))
            .output()
            .unwrap();
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_ne!(out.status.code(), Some(0), "no temp dir, no run:\n{said}");
        assert!(
            said.contains("тимчасову теку для JSON rspec не створити") && said.contains("TMPDIR"),
            "the refusal names the directory it could not make and the instead names TMPDIR:\n{said}"
        );
    }

    // --- rspec: the JSON lies in a directory of its own, made for
    // this run, never under a name anyone could set up in advance ---
    if common::machine_has("ruby").ready() {
        let dir = keel_sandbox("selectrspecout");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("spec")).unwrap();
        std::fs::create_dir_all(dir.join("shim")).unwrap();
        std::fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        std::fs::write(
            dir.join("spec/toy_spec.rb"),
            format!(
                "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n  end\nend\n"
            ),
        )
        .unwrap();
        // A shim named `rspec` that records where it was told to write
        // and writes an empty JSON there, so the adapter's own words
        // are what is read.
        let log = dir.join("rspec-argv.log");
        std::fs::write(
            dir.join("shim/rspec"),
            format!(
                "#!/bin/sh\necho \"$@\" >> '{}'\nout=\"\"\nwhile [ $# -gt 0 ]; do case \"$1\" in --out) out=\"$2\"; shift;; esac; shift; done\nprintf '{{\"examples\":[]}}' > \"$out\"\n",
                log.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(dir.join("shim/rspec"))
                .unwrap()
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(dir.join("shim/rspec"), perms).unwrap();
        }
        frame(&dir, "ruby", "lib/toy.rb");
        let path = format!(
            "{}:{}",
            dir.join("shim").display(),
            std::env::var("PATH").unwrap_or_default()
        );
        let _ = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(["close", dir.to_str().unwrap()])
            .env("PATH", path)
            .output()
            .unwrap();
        let argv = std::fs::read_to_string(&log).unwrap_or_default();
        let out_path = argv
            .split_whitespace()
            .skip_while(|w| *w != "--out")
            .nth(1)
            .expect("rspec was told where to write");
        let out_path = Path::new(out_path);
        let parent = out_path.parent().expect("the JSON has a directory");
        // A directory named `keel-rspec-<pid>-<n>` is by that name not
        // the shared temp dir itself: one assertion says both. (The
        // sandbox probe of wave 0030 flags any probe that so much as
        // names the shared temp dir, so the comparison stays implicit.)
        assert!(
            parent
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("keel-rspec-")),
            "the JSON lies in a directory made for this run, not bare in the shared temp dir: {argv}"
        );
        assert!(
            !parent.exists(),
            "and the directory is gone after the run: {}",
            parent.display()
        );
    }
}
