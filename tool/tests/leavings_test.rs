//! Scenario test of wave 0057: the tongue names what its runner
//! leaves.
//!
//! Measured before the plan, in sandboxes, on keel 1.0.0: `keel init`
//! told python, javascript and ruby alike that "this language builds
//! nothing, so there is no build directory worth ignoring" -- while
//! pytest was writing `__pycache__/` beside every module it imports
//! and `.pytest_cache/` at the root. A stranger, told there was
//! nothing to ignore, committed them, and the scope court called
//! pytest's own `.pyc` files drift (queue after 0055, bugs R-21).
//!
//! What each runner leaves was measured, not guessed: pytest leaves
//! the two above; `npm install` writes `node_modules/` as soon as
//! there is one dependency; ruby's minitest and rspec left nothing at
//! all in a bare project, and the empty list is said aloud.
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

fn decisions_except(covered: &[&str]) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// A project of the given tongue, with keel.toml alone: what a
/// stranger has in the minute before `keel init`.
fn project(name: &str, adapter: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// proves: the-tongue-names-what-its-runner-leaves@eabda7
#[test]
fn the_tongue_names_what_its_runner_leaves() {
    // --- python: two paths, each with the exact line for .gitignore
    let dir = project("leav-py", "python");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("tests/test_a.py"), "def test_ok():\n    assert True\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the project"]);
    let (said, code) = keel(&dir, &["init", "--no-ask"]);
    assert_eq!(code, 0, "init answers:\n{said}");
    assert!(
        said.contains("__pycache__/") && said.contains(".pytest_cache/"),
        "the frame names what pytest leaves, both of them:\n{said}"
    );
    assert!(
        !said.contains("нічого не збирає"),
        "and never says this tongue builds nothing while pytest fills \
         the tree:\n{said}"
    );

    // --- javascript: npm's own directory
    let dir = project("leav-js", "javascript");
    std::fs::write(dir.join("package.json"), "{\"name\":\"toy\"}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the project"]);
    let (said, _) = keel(&dir, &["init", "--no-ask"]);
    assert!(
        said.contains("node_modules/"),
        "the frame names what npm leaves:\n{said}"
    );

    // --- ruby: nothing was measured, and the empty list is said
    let dir = project("leav-rb", "ruby");
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(dir.join("test/a_test.rb"), "# a test\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the project"]);
    let (said, _) = keel(&dir, &["init", "--no-ask"]);
    assert!(
        said.contains("не лишає в дереві нічого свого"),
        "a tongue whose runner leaves nothing says exactly that -- not \
         that it builds nothing:\n{said}"
    );

    // --- and the scope court knows the leavings as furniture
    let dir = keel_sandbox("leav-scope");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"python\"\n").unwrap();
    std::fs::write(dir.join("pkg.py"), "def add(a, b):\n    return a + b\n").unwrap();
    std::fs::write(
        dir.join("tests/test_a.py"),
        "def test_ok():\n    assert True\n",
    )
    .unwrap();
    let wave = format!(
        "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - pkg.py\n      - tests/test_a.py\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    );
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), wave).unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    // The work, and beside it exactly what pytest leaves -- committed,
    // because the frame had told this person there was nothing to
    // ignore.
    std::fs::write(dir.join("pkg.py"), "def add(a, b):\n    return a + b\n# work\n").unwrap();
    std::fs::create_dir_all(dir.join("tests/__pycache__")).unwrap();
    std::fs::create_dir_all(dir.join(".pytest_cache/v")).unwrap();
    std::fs::write(dir.join("tests/__pycache__/test_a.pyc"), "compiled\n").unwrap();
    std::fs::write(dir.join(".pytest_cache/v/lastfailed"), "{}\n").unwrap();
    std::fs::write(
        dir.join("tests/test_a.py"),
        "def test_ok():\n    assert True\n# work\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "work: done"]);
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        !said.contains("__pycache__"),
        "what the runner left is furniture, not drift:\n{said}"
    );
    assert!(
        !said.contains(".pytest_cache"),
        "both of them:\n{said}"
    );
    assert_eq!(code, 0, "so the branch is green:\n{said}");
}
