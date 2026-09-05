//! Scenario test of wave 0045: python tests are read and run.
//!
//! These run a real `pytest` against real projects. Where pytest is
//! not on the machine the probe says so and stops -- with the hand
//! wave 0044 built for exactly that, so a stop is a stop and not a
//! red.
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

/// A pytest project: pyproject.toml with the src layout, a package,
/// and a test file as given.
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
    std::fs::write(
        dir.join("src/toy/__init__.py"),
        "def works():\n    return True\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/toy/bar.py"),
        "def works(a: int, b: int) -> int:\n    return a + b\n",
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

fn test_file(rev: &str) -> String {
    format!(
        "from toy import works\n\n# proves: it-works@{rev}\ndef test_it_works():\n    assert works()\n"
    )
}

/// proves: python-tests-are-read-and-run@e1fb34
#[test]
fn python_tests_are_read_and_run() {
    if !common::machine_has("pytest").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // The tag is read from tests/test_*.py, and the project is
    // judged whole -- adapter and all.
    let dir = project("pyread", &test_file(&rev));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a python project is judged whole:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "and the tag over `def test_…` is read:\n{said}"
    );

    // The gate runs exactly that test by its node id, and reads the
    // verdict from pytest's exit code: 0 is green.
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
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "the work passes over a test pytest ran green:\n{said}"
    );
    assert!(said.contains("робота проходить"), "and says so:\n{said}");

    // The battery: the roll AND the verdicts from pytest's own voice.
    // A second test the reader did not tag still exists for the
    // court, together with its failure; a test inside a class is
    // named with its class, exactly as pytest names it.
    let two = format!(
        "from toy import works\n\n# proves: it-works@{rev}\ndef test_it_works():\n    assert works()\n\ndef test_nobody_claims_me():\n    assert 1 == 2\n\nclass TestGrouped:\n    def test_inside(self):\n        assert True\n"
    );
    let dir = project("pybattery", &two);
    std::fs::write(dir.join("keel/reviews/0001-a-wave.md"), "# Рецензія\n\nok\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "review"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 3 тестів"),
        "the battery counts what pytest ran -- all three, the grouped \
         one included:\n{said}"
    );
    assert!(
        said.contains("червоний тест") && said.contains("test_nobody_claims_me"),
        "and names the red one by pytest's own name:\n{said}"
    );
    assert_ne!(code, 0, "so a red battery does not close:\n{said}");

    // The adapter writes NOTHING into the project: no cache, no
    // bytecode. Measured, because pytest does both by default.
    let written: Vec<String> = walk(&dir)
        .into_iter()
        .filter(|p| p.contains("__pycache__") || p.contains(".pytest_cache"))
        .collect();
    assert!(
        written.is_empty(),
        "the adapter leaves the project as it found it: {written:?}"
    );
}

fn walk(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            out.push(path.display().to_string());
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    out
}
