//! Scenario test of wave 0046: javascript tests are read and run.
//!
//! These run a real `node --test` against real projects; where node
//! is not on the machine the probe stops aloud with the hand of wave
//! 0044. TypeScript rides the same runner (node 22 strips types), so
//! a `.test.ts` file is judged here too.
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

/// A node project: package.json, a module in src/, and a test file
/// under test/ as given.
fn project(name: &str, test_name: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"javascript\"\n",
    )
    .unwrap();
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
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/toy.js\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("test").join(test_name), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn test_file(rev: &str) -> String {
    format!(
        "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('it works', () => {{\n  assert.ok(works());\n}});\n"
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

/// proves: javascript-tests-are-read-and-run@fbbe41
#[test]
fn javascript_tests_are_read_and_run() {
    if !common::machine_has("node").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // The tag is read from test/*.test.js, and the project is judged
    // whole -- adapter and all.
    let dir = project("jsread", "toy.test.js", &test_file(&rev));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a node project is judged whole:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "and the tag over `test('…')` is read:\n{said}"
    );

    // The gate runs exactly that test by its name, and reads the
    // verdict from TAP -- never from the exit code, which node hands
    // out the same for a pass and for a name that matched nothing.
    let (said, code) = gate(&dir);
    assert_eq!(
        code, 0,
        "the work passes over a test node ran green:\n{said}"
    );
    assert!(said.contains("робота проходить"), "and says so:\n{said}");

    // A GREEN close: the tag and the battery meet on one key, and the
    // wave closes -- asserted, because review 0045 found no python
    // probe had ever asserted it.
    let dir = project("jsgreen", "toy.test.js", &test_file(&rev));
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "a proven node wave closes:\n{said}");
    assert!(
        said.contains("0001-a-wave: закрита"),
        "and says so:\n{said}"
    );
    let written: Vec<String> = walk(&dir)
        .into_iter()
        .filter(|p| {
            p.contains("node_modules") || p.contains(".nyc_output") || p.contains("coverage")
        })
        .collect();
    assert!(
        written.is_empty(),
        "close leaves the project as it found it: {written:?}"
    );

    // The battery: the roll AND the verdicts from TAP. A second test
    // the reader did not tag still exists for the court, together
    // with its failure; a test inside `describe` is named by its
    // bare name, as node names it; and a `.ts` file is read and run
    // by the same runner.
    // A skipped test, an `it(` with a backtick name, and a file with
    // no tests in it (node prints the FILE as one passed test there)
    // ride along: none of the three is a test that ran green.
    let two = format!(
        "import {{ test, it, describe }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('it works', () => {{\n  assert.ok(works());\n}});\n\ntest('nobody claims me', () => {{\n  assert.strictEqual(1, 2);\n}});\n\ntest('not now', {{ skip: true }}, () => {{\n  assert.ok(false);\n}});\n\nit(`spoken with it`, () => {{\n  assert.ok(true);\n}});\n\ndescribe('grouped', () => {{\n  test('inside', () => {{\n    assert.ok(true);\n  }});\n}});\n"
    );
    let dir = project("jsbattery", "toy.test.js", &two);
    std::fs::write(
        dir.join("test/typed.test.ts"),
        "import { test } from 'node:test';\nimport assert from 'node:assert';\n\ntest('typed', () => {\n  const x: number = 1;\n  assert.strictEqual(x, 1);\n});\n",
    )
    .unwrap();
    std::fs::write(dir.join("test/empty.test.js"), "// nothing here yet\n").unwrap();
    // Before the battery: the reader sees `it(` and the backtick name
    // as declarations, so a tag over each is read.
    let tagged_it = format!(
        "import {{ it }} from 'node:test';\nimport assert from 'node:assert';\n\n// proves: it-works@{rev}\nit(`spoken with it`, () => {{\n  assert.ok(true);\n}});\n"
    );
    let read = keel::tags::scan_text(Path::new("t/x.test.js"), &tagged_it).unwrap();
    assert_eq!(read.len(), 1, "a tag over it(`…`) is read");
    assert_eq!(read[0].test, "spoken with it");
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 5 тестів"),
        "the battery counts what node RAN -- the grouped one, the `it`, \
         and the typed one included; not the skipped one, not the \
         describe block, not the empty file's own line:\n{said}"
    );
    assert!(
        !said.contains("not now"),
        "a skipped test is neither green nor red, so it is not named:\n{said}"
    );
    assert!(
        said.contains("червоний тест") && said.contains("nobody claims me"),
        "and names the red one by node's own name:\n{said}"
    );
    assert_ne!(code, 0, "so a red battery does not close:\n{said}");
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
