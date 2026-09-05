//! Scenario test of wave 0045: a tongue with five answers says them.
//!
//! pytest tells its states apart by exit code alone -- measured
//! before the plan: 0 green, 1 a test failed, 2 collection broke, 4
//! no such node, 5 nothing collected. These run a real pytest; where
//! it is not on the machine the probe stops aloud.
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

fn project(name: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"python\"\n").unwrap();
    std::fs::create_dir_all(dir.join("src/toy")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n\n[tool.pytest.ini_options]\npythonpath = [\"src\"]\ntestpaths = [\"tests\"]\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/toy/__init__.py"), "def works():\n    return True\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/toy/__init__.py\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("tests/test_toy.py"), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
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

/// proves: a-tongue-with-five-answers-says-them@303382
#[test]
fn a_tongue_with_five_answers_says_them() {
    if !common::machine_has("pytest").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // Code 2: collection broke. A SyntaxError in the MODULE is the
    // ordinary way this happens, and the refusal carries python's
    // own words -- not "something went wrong".
    let dir = project(
        "pybroken",
        &format!("from toy import works\n\n# proves: it-works@{rev}\ndef test_it_works():\n    assert works()\n"),
    );
    std::fs::write(dir.join("src/toy/__init__.py"), "def works(:\n").unwrap();
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "a broken build is not a green test:\n{said}");
    assert!(
        said.contains("SyntaxError"),
        "and the refusal carries python's own words:\n{said}"
    );
    assert!(
        !said.contains("червоний тест"),
        "a build that broke is a REFUSAL, not a red test:\n{said}"
    );

    // Code 4: a node pytest does not know is "did not run" (§7.12) --
    // never green, and not red either. `if False:` is the honest way
    // to build it: the reader sees the declaration and takes the
    // name, pytest collects nothing under it.
    let dir = project(
        "pynotrun",
        &format!("from toy import works\n\nif False:\n    # proves: it-works@{rev}\n    def test_it_works():\n        assert works()\n\ndef test_other():\n    assert True\n"),
    );
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "work over a test that never ran does not pass:\n{said}");
    assert!(
        !said.contains("робота проходить"),
        "and \"did not run\" is not read as green:\n{said}"
    );

    // And `keel check` prints python's OWN border: five states told
    // apart, so sec. 7.12's "where the adapter cannot tell" does not
    // stand here; what it reads and does not read, said aloud.
    let dir = project(
        "pyborder",
        &format!("from toy import works\n\n# proves: it-works@{rev}\ndef test_it_works():\n    assert works()\n"),
    );
    std::fs::write(dir.join("tests/conftest.py"), "# proves: it-works@aaaaaa\n").unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a whole python project is judged:\n{said}");
    assert!(
        said.contains("пʼять") || said.contains("5 стан") || said.contains("пять"),
        "it says the five states are told apart:\n{said}"
    );
    assert!(
        !said.contains("ruby не відрізняє") && !said.contains("ця мова відрізняє «впав» від «не зібрався» кодом виходу (0 зелене, 2 падіння"),
        "and carries neither ruby's nor elixir's border into a tongue \
         that has its own:\n{said}"
    );
    assert!(
        said.contains("conftest.py"),
        "and names the file in tests/ it did not read:\n{said}"
    );
}
