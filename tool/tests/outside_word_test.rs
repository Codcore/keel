//! Scenario test of wave 0050: the battery hears no word from
//! outside.
//!
//! The global review of 2026-09-06 (bugs cut R-6, R-8, R-18) measured
//! the environment talking the verdict over: `PYTEST_ADDOPTS` with a
//! `--deselect` took the red test out of the battery, `NODE_OPTIONS`
//! with `--test-skip-pattern` did the same for node, a `~/.rspec` (or
//! a `.rspec-local`) saying `--dry-run` made every example "passed";
//! `KEEL_RUNNING_REF` -- a word the launcher says -- turned the pin
//! court off for whoever set it, and reached the project's tests; and
//! the verify of a contract ran with the hook's `GIT_DIR` and the
//! shared `CARGO_TARGET_DIR` the battery itself refuses. The verdict
//! must be the verdict of a silent environment.
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

/// One run of keel -- this binary, or another copy of it -- with the
/// environment given on top of the probe's own.
fn run(binary: &Path, dir: &Path, args: &[&str], envs: &[(&str, String)]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(binary)
        .args(&all)
        .envs(envs.iter().map(|(k, v)| (*k, v.as_str())))
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

fn keel(dir: &Path, args: &[&str], envs: &[(&str, String)]) -> (String, i32) {
    run(Path::new(env!("CARGO_BIN_EXE_keel")), dir, args, envs)
}

/// The commit court over a `work:` message, with the environment.
fn gate(dir: &Path, envs: &[(&str, String)]) -> (String, i32) {
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, "work: тіло\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
        .envs(envs.iter().map(|(k, v)| (*k, v.as_str())))
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

/// The methodology's frame over a project: keel.toml, one wave with
/// one scenario `it-works` and one transform `work`, its review.
fn frame(dir: &Path, config: &str, file: &str) {
    std::fs::create_dir_all(dir.join("keel/waves")).unwrap();
    std::fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    std::fs::create_dir_all(dir.join("keel/reviews")).unwrap();
    std::fs::write(dir.join("keel.toml"), config).unwrap();
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
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// proves: the-battery-hears-no-word-from-outside@5901d4
#[test]
fn the_battery_hears_no_word_from_outside() {
    let rev = keel::rev::text_rev(BODY);

    // --- python: PYTEST_ADDOPTS deselects the very test the tag
    // names -- and the battery must still see it red ---
    if common::machine_has("pytest").ready() {
        let dir = keel_sandbox("wordpy");
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
        std::fs::write(
            dir.join("tests/test_toy.py"),
            format!("from toy import works\n\n\n# proves: it-works@{rev}\ndef test_red():\n    assert not works()\n"),
        )
        .unwrap();
        frame(
            &dir,
            "lang = \"uk\"\nadapter = \"python\"\n",
            "src/toy/__init__.py",
        );
        settle(&dir);
        let env = [(
            "PYTEST_ADDOPTS",
            "--deselect tests/test_toy.py::test_red".to_string(),
        )];
        let (said, code) = gate(&dir, &env);
        assert!(
            said.contains("падає") && code != 0,
            "the gate runs the test the tag names, whatever PYTEST_ADDOPTS \
             says:\n{said}"
        );
        let (said, code) = keel(&dir, &["close"], &env);
        assert!(
            said.contains("червоний тест: test_red"),
            "the battery sees the deselected test red:\n{said}"
        );
        assert!(
            !said.contains("не виконала"),
            "and never as `not run`:\n{said}"
        );
        assert_ne!(code, 0, "a court that saw red does not close:\n{said}");
    }

    // --- javascript: NODE_OPTIONS skips the red test by pattern ---
    if common::machine_has("node").ready() {
        let dir = keel_sandbox("wordjs");
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("package.json"),
            "{ \"name\": \"toy\", \"version\": \"0.1.0\", \"type\": \"module\" }\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("src/toy.js"),
            "export function works() {\n  return true;\n}\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy.test.js"),
            format!(
                "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('red one', () => {{\n  assert.strictEqual(works(), false);\n}});\n"
            ),
        )
        .unwrap();
        frame(
            &dir,
            "lang = \"uk\"\nadapter = \"javascript\"\n",
            "src/toy.js",
        );
        settle(&dir);
        let env = [("NODE_OPTIONS", "--test-skip-pattern=red".to_string())];
        let (said, code) = gate(&dir, &env);
        assert!(
            said.contains("падає") && code != 0,
            "the gate runs the test the tag names, whatever NODE_OPTIONS \
             says:\n{said}"
        );
        let (said, code) = keel(&dir, &["close"], &env);
        assert!(
            said.contains("червоний тест: red one"),
            "the battery sees the skipped-by-pattern test red:\n{said}"
        );
        assert!(
            !said.contains("не виконала"),
            "and never as `not run`:\n{said}"
        );
        assert_ne!(code, 0, "a court that saw red does not close:\n{said}");
    }

    // --- ruby: ~/.rspec and .rspec-local say --dry-run, under which
    // rspec reports every example passed without running one ---
    if common::machine_has("rspec").ready() {
        let dir = keel_sandbox("wordrb");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("spec")).unwrap();
        std::fs::create_dir_all(dir.join("home")).unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(dir.join(".rspec"), "--require spec_helper\n").unwrap();
        std::fs::write(dir.join(".rspec-local"), "--dry-run\n").unwrap();
        std::fs::write(dir.join("home/.rspec"), "--dry-run\n").unwrap();
        std::fs::write(
            dir.join("spec/spec_helper.rb"),
            "require \"toy\"\n\nRSpec.configure do |config|\n  config.color = false\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("spec/toy_spec.rb"),
            format!(
                "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n    expect(Toy.works).to be(false)\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "lang = \"uk\"\nadapter = \"ruby\"\n", "lib/toy.rb");
        settle(&dir);
        let env = [("HOME", dir.join("home").display().to_string())];
        let (said, code) = gate(&dir, &env);
        assert!(
            said.contains("падає") && code != 0,
            "the gate reads the project's own .rspec and no one else's:\n{said}"
        );
        let (said, code) = keel(&dir, &["close"], &env);
        assert!(
            said.contains("червоний тест: Toy works"),
            "the battery sees the example red under a dry-run asked for \
             by the machine, not the project:\n{said}"
        );
        assert_ne!(code, 0, "a court that saw red does not close:\n{said}");
        assert!(
            !said.contains("закрита"),
            "no verdict reads as closure:\n{said}"
        );
    }

    // --- ruby: neither reading hears the words either -- a minitest
    // test and an rspec example both tagged, both asking for a silent
    // environment (review 0050 R-5, mutation M18) ---
    if common::machine_has("rspec").ready() {
        let dir = keel_sandbox("wordrbenv");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::create_dir_all(dir.join("spec")).unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy_test.rb"),
            format!(
                "require \"minitest/autorun\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    %w[GIT_DIR GIT_WORK_TREE KEEL_BRANCH KEEL_RUNNING_REF].each do |name|\n      assert_nil ENV[name], \"#{{name}} reached the battery\"\n    end\n    assert Toy.works\n  end\nend\n"
            ),
        )
        .unwrap();
        std::fs::write(dir.join(".rspec"), "--require spec_helper\n").unwrap();
        std::fs::write(
            dir.join("spec/spec_helper.rb"),
            "require \"toy\"\n\nRSpec.configure do |config|\n  config.color = false\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("spec/toy_spec.rb"),
            format!(
                "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n    %w[GIT_DIR GIT_WORK_TREE KEEL_BRANCH KEEL_RUNNING_REF].each do |name|\n      expect(ENV[name]).to be_nil\n    end\n    expect(Toy.works).to be(true)\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "lang = \"uk\"\nadapter = \"ruby\"\n", "lib/toy.rb");
        settle(&dir);
        let env = [
            ("KEEL_RUNNING_REF", "a word of the launcher".to_string()),
            ("KEEL_BRANCH", "0001-a-wave".to_string()),
            ("GIT_DIR", dir.join(".git").display().to_string()),
            ("GIT_WORK_TREE", dir.display().to_string()),
        ];
        let (said, code) = keel(&dir, &["close"], &env);
        assert!(
            said.contains("0001-a-wave: закрита"),
            "minitest and rspec children hear none of the words:\n{said}"
        );
        assert_eq!(
            code, 0,
            "nothing red under a loud environment, in ruby:\n{said}"
        );
    }

    // --- the pin court: a word in the air sways nothing; the file
    // the installer wrote beside the binary does ---
    let dir = keel_sandbox("wordpin");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    frame(
        &dir,
        "lang = \"uk\"\nadapter = \"rust\"\nversion = \"zzz-not-this\"\n",
        "src/lib.rs",
    );
    settle(&dir);
    let (said, _) = keel(
        &dir,
        &["check"],
        &[("KEEL_RUNNING_REF", "zzz-not-this".to_string())],
    );
    assert!(
        said.contains("pins version"),
        "an environment variable does not pass the pin court:\n{said}"
    );
    // The same binary, standing where the launcher installs a version
    // -- `versions/<ref>/keel` with `.keel-ref` beside it -- answers
    // for that ref by what lies on the disk.
    let home = dir.join("versions/zzz-not-this");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::copy(env!("CARGO_BIN_EXE_keel"), home.join("keel")).unwrap();
    std::fs::write(home.join(".keel-ref"), "zzz-not-this\n").unwrap();
    let (said, _) = run(&home.join("keel"), &dir, &["version"], &[]);
    assert!(
        said.contains("тримається"),
        "the ref recorded beside the binary holds the pin:\n{said}"
    );
    let (said, _) = run(&home.join("keel"), &dir, &["check"], &[]);
    assert!(
        !said.contains("pins version"),
        "and the courts judge with it:\n{said}"
    );

    // --- none of the words reaches the tests, and verify runs as
    // clean as the battery ---
    let dir = keel_sandbox("wordenv");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(
        dir.join("tests/env_test.rs"),
        format!(
            "/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    for name in [\"KEEL_RUNNING_REF\", \"KEEL_BRANCH\", \"GIT_DIR\"] {{\n        assert!(std::env::var_os(name).is_none(), \"{{name}} reached the battery\");\n    }}\n}}\n"
        ),
    )
    .unwrap();
    let verify =
        "test -z \"$CARGO_TARGET_DIR\" && test -z \"$GIT_DIR\" && test -z \"$KEEL_RUNNING_REF\"";
    let fingerprint = keel::trust::fingerprint(verify);
    // The project's own ci runs through the same hand (review 0050
    // R-8): the same command, trusted once, as the merge gate.
    frame(
        &dir,
        &format!(
            "lang = \"uk\"\nadapter = \"rust\"\nci = '{verify}'\n\n[trust]\n'{verify}' = \"{fingerprint}\"\n"
        ),
        "src/lib.rs",
    );
    std::fs::write(
        dir.join("keel/contracts/clean.md"),
        format!("---\nverify: '{verify}'\n---\n\nЧужа обіцянка, яку судять у тиші (§2.8).\n"),
    )
    .unwrap();
    settle(&dir);
    let env = [
        ("KEEL_RUNNING_REF", "a word of the launcher".to_string()),
        ("KEEL_BRANCH", "0001-a-wave".to_string()),
        ("GIT_DIR", dir.join(".git").display().to_string()),
        ("CARGO_TARGET_DIR", dir.join("shared").display().to_string()),
    ];
    let (said, code) = keel(&dir, &["close"], &env);
    assert!(
        said.contains("контракту clean — пройшла"),
        "verify runs without the hook's git and without the shared \
         target:\n{said}"
    );
    assert!(
        said.contains("пройшов: власний gate проєкту зелений"),
        "and so does the project's ci:\n{said}"
    );
    assert!(
        said.contains("0001-a-wave: закрита"),
        "and the test that asks for a silent environment is green:\n{said}"
    );
    assert_eq!(code, 0, "nothing red under a loud environment:\n{said}");
}
