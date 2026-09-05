//! Scenario test of wave 0046: a tongue that cannot tell says so.
//!
//! node leaves with 1 for a failed test and for a SyntaxError alike,
//! and with 0 for a name that matched nothing -- counting the file
//! as one passed test. So the verdict is read from TAP and never from
//! the code, and keel check says so about this tongue.
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
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
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
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"javascript\"\n").unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("package.json"),
        "{ \"name\": \"toy\", \"version\": \"0.1.0\", \"type\": \"module\" }\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/toy.js"), "export function works() {\n  return true;\n}\n").unwrap();
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
    std::fs::write(dir.join("test/toy.test.js"), test_body).unwrap();
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

fn tagged(rev: &str, name: &str) -> String {
    format!(
        "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('{name}', () => {{\n  assert.ok(works());\n}});\n"
    )
}

/// proves: a-tongue-that-cannot-tell-says-so@3c0ff7
#[test]
fn a_tongue_that_cannot_tell_says_so() {
    if !common::machine_has("node").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // A SyntaxError in the MODULE: node leaves with 1, the same code
    // as a failed test. The court must still say "broken", with
    // node's own words, and never "red test".
    let dir = project("jsbroken", &tagged(&rev, "it works"));
    std::fs::write(dir.join("src/toy.js"), "export function works( {\n  return true;\n}\n").unwrap();
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "a broken module is not a green test:\n{said}");
    assert!(said.contains("SyntaxError"), "and the refusal carries node's own words:\n{said}");
    assert!(!said.contains("червоний тест"), "a build that broke is a REFUSAL, not a red test:\n{said}");

    // A name that matches nothing: node leaves with 0 and counts the
    // FILE as one passed test. That is "did not run" (§7.12) -- never
    // green. `if (false)` is the honest way to build it: the reader
    // sees the declaration, node registers nothing under that name.
    let dir = project(
        "jsnotrun",
        &format!(
            "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\nif (false) {{\n  // proves: it-works@{rev}\n  test('it works', () => {{\n    assert.ok(works());\n  }});\n}}\n\ntest('other', () => {{\n  assert.ok(true);\n}});\n"
        ),
    );
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "work over a test that never ran does not pass:\n{said}");
    assert!(!said.contains("робота проходить"), "and \"did not run\" is not read as green:\n{said}");

    // A name with regex metacharacters goes into --test-name-pattern
    // escaped, so `a.b (x)` selects `a.b (x)` and nothing else.
    let dir = project("jsregex", &tagged(&rev, "a.b (x)"));
    let (said, code) = gate(&dir);
    assert_eq!(code, 0, "a name with metacharacters is selected as itself:\n{said}");

    // And keel check prints node's OWN border.
    let dir = project("jsborder", &tagged(&rev, "it works"));
    std::fs::write(dir.join("test/helper.js"), "// proves: it-works@aaaaaa\n").unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a whole node project is judged:\n{said}");
    assert!(
        said.contains("TAP") && said.contains("кодом виходу"),
        "it says this tongue does not tell its states apart by exit \
         code, and that the verdict is read from TAP:\n{said}"
    );
    assert!(
        !said.contains("ruby не відрізняє") && !said.contains("пʼять станів"),
        "and carries no other tongue's border:\n{said}"
    );
    assert!(said.contains("helper.js"), "and names the file in test/ it did not read:\n{said}");
}
