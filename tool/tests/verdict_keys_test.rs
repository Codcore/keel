//! Scenario test of wave 0050: every verdict keeps its own key.
//!
//! The global review of 2026-09-06 measured four roads by which a
//! red verdict never reached the closing court: cargo's battery keyed
//! by the STEM of a target (`unittests src/lib.rs` and `unittests
//! src/main.rs` share one, and the second overwrote the first);
//! pytest's second word about one node (`PASSED` in the body, `ERROR`
//! at teardown) dropped by a first-wins roll; a node test named as
//! its own file thrown out by a filter meant for another node; and
//! minitest without `minitest/autorun`, which prints nothing, leaves
//! with 0, and was read as green. Each is played here against the
//! real runner, and the court must say red -- or "did not run" --
//! never closed.
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

/// One wave, one scenario `it-works`, one transform `work` over the
/// file named -- and the review beside it, so the only thing that
/// can hold the wave open is the battery.
fn wave(dir: &Path, adapter: &str, file: &str) {
    std::fs::create_dir_all(dir.join("keel/waves")).unwrap();
    std::fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    std::fs::create_dir_all(dir.join("keel/reviews")).unwrap();
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
}

/// git around it, and the wave's own branch checked out.
fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// proves: every-verdict-keeps-its-own-key@5ab84a
#[test]
fn every_verdict_keeps_its_own_key() {
    let rev = keel::rev::text_rev(BODY);

    // --- rust: a library and a binary with unit tests of one name,
    // the library's red; and a red test that prints a green verdict
    // line and a block opener into cargo's failures section ---
    let dir = keel_sandbox("keyrust");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn smoke() {\n        assert_eq!(1, 2, \"the library's red\");\n    }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/main.rs"),
        "fn main() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn smoke() {}\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/toy_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(toy::works());\n}}\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/loud_test.rs"),
        "#[test]\nfn loud() {\n    println!(\"test quiet ... ok\");\n    println!(\"running 1 test\");\n    panic!(\"loud and red\");\n}\n\n#[test]\nfn quiet() {\n    panic!(\"quiet and red\");\n}\n",
    )
    .unwrap();
    wave(&dir, "rust", "src/lib.rs");
    settle(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("tests::smoke") && said.contains("unittests src/lib.rs"),
        "the library's red keeps its own key -- the target as cargo \
         announces it, not a stem shared with the binary:\n{said}"
    );
    assert!(
        said.contains("червоний тест: loud") && said.contains("червоний тест: quiet"),
        "a red test that prints `test quiet ... ok` into the failures \
         section paints nothing green, and the printed `running 1 test` \
         opens no block:\n{said}"
    );
    assert!(
        !said.contains("зшивка не сходиться"),
        "the stitch of targets and blocks still meets:\n{said}"
    );
    assert_ne!(code, 0, "and a court that saw red does not close:\n{said}");
    assert!(
        !said.contains("закрита"),
        "no verdict reads as closure:\n{said}"
    );

    // --- rust: two targets announced by one path (a workspace with
    // `a/tests/basic.rs` and `b/tests/basic.rs`) -- nothing a tag
    // could tell apart, so a refusal aloud, never a merged verdict ---
    let dir = keel_sandbox("keyalike");
    std::fs::write(
        dir.join("Cargo.toml"),
        "[workspace]\nmembers = [\"a\", \"b\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    for (member, body) in [
        ("a", "#[test]\nfn smoke() {\n    panic!(\"a is red\");\n}\n"),
        ("b", "#[test]\nfn smoke() {}\n"),
    ] {
        std::fs::create_dir_all(dir.join(member).join("src")).unwrap();
        std::fs::create_dir_all(dir.join(member).join("tests")).unwrap();
        std::fs::write(
            dir.join(member).join("Cargo.toml"),
            format!("[package]\nname = \"{member}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .unwrap();
        std::fs::write(dir.join(member).join("src/lib.rs"), "pub fn f() {}\n").unwrap();
        std::fs::write(dir.join(member).join("tests/basic.rs"), body).unwrap();
    }
    wave(&dir, "rust", "a/src/lib.rs");
    settle(&dir);
    let (said, code) = keel(&dir, &["close"]);
    // cargo announces the members' library targets first, and both
    // are `unittests src/lib.rs`: the first pair of one name is the
    // one the refusal names.
    assert!(
        said.contains("оголошує ціль") && said.contains("двічі"),
        "two targets of one name are refused by that name:\n{said}"
    );
    assert_ne!(code, 0, "and the refusal is not a closure:\n{said}");
    assert!(
        !said.contains("закрита"),
        "no verdict reads as closure:\n{said}"
    );

    // --- python: green in the body, ERROR at teardown -- pytest says
    // two words about one node, and the second is red ---
    if common::machine_has("pytest").ready() {
        let dir = keel_sandbox("keypy");
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
            format!(
                "import pytest\nfrom toy import works\n\n\n@pytest.fixture\ndef broken():\n    yield\n    assert False, \"teardown broke\"\n\n\n# proves: it-works@{rev}\ndef test_it_works(broken):\n    assert works()\n"
            ),
        )
        .unwrap();
        wave(&dir, "python", "src/toy/__init__.py");
        settle(&dir);
        let (said, code) = gate(&dir);
        assert_ne!(code, 0, "the gate reads pytest's exit code: red:\n{said}");
        assert!(said.contains("падає"), "and says so:\n{said}");
        let (said, code) = keel(&dir, &["close"]);
        assert!(
            said.contains("червоний тест: test_it_works"),
            "the battery reads BOTH words about the node, and ERROR at \
             teardown is red:\n{said}"
        );
        assert_ne!(code, 0, "the two courts agree on one tree:\n{said}");
        assert!(
            !said.contains("закрита"),
            "no verdict reads as closure:\n{said}"
        );
    }

    // --- javascript: a red test named as its own file ---
    if common::machine_has("node").ready() {
        let dir = keel_sandbox("keyjs");
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
                "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('test/toy.test.js', () => {{\n  assert.strictEqual(works(), false);\n}});\n"
            ),
        )
        .unwrap();
        wave(&dir, "javascript", "src/toy.js");
        settle(&dir);
        let (said, code) = gate(&dir);
        assert_ne!(code, 0, "the gate sees the red test by its name:\n{said}");
        assert!(said.contains("падає"), "and says so:\n{said}");
        let (said, code) = keel(&dir, &["close"]);
        assert!(
            said.contains("червоний тест: test/toy.test.js"),
            "a test named as its file is a test with a verdict of its own, \
             not a file-level line to drop:\n{said}"
        );
        assert!(
            !said.contains("не виконала"),
            "and the battery did not lose it as `not run`:\n{said}"
        );
        assert_ne!(code, 0, "a court that saw red does not close:\n{said}");
        assert!(
            !said.contains("закрита"),
            "no verdict reads as closure:\n{said}"
        );
    }

    // --- ruby: minitest required without `minitest/autorun` -- ruby
    // prints nothing and leaves with 0, and nothing ran ---
    if common::machine_has("ruby").ready() {
        let dir = keel_sandbox("keyrb");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy_test.rb"),
            format!(
                "require \"minitest\"\nrequire_relative \"../lib/toy\"\n\nclass ToyTest < Minitest::Test\n  # proves: it-works@{rev}\n  def test_it_works\n    raise \"never runs, and would be red\"\n  end\nend\n"
            ),
        )
        .unwrap();
        wave(&dir, "ruby", "lib/toy.rb");
        settle(&dir);
        let (said, code) = gate(&dir);
        assert_ne!(
            code, 0,
            "a run that printed no summary is not a green run:\n{said}"
        );
        assert!(
            said.contains("біг не виконав жодного тесту"),
            "the gate says nothing ran, in the words it has for that:\n{said}"
        );
        assert!(
            !said.contains("робота проходить"),
            "and never passes the work over silence:\n{said}"
        );
        let (said, code) = keel(&dir, &["close"]);
        assert!(
            said.contains("minitest/autorun"),
            "the battery names what minitest lacked to run at all:\n{said}"
        );
        assert_ne!(code, 0, "and does not close over it:\n{said}");
        assert!(
            !said.contains("закрита"),
            "no verdict reads as closure:\n{said}"
        );
    }
}
