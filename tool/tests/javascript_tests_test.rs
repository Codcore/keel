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

fn keel_with(dir: &Path, args: &[&str], envs: &[(&str, String)]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
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
    // The branch does its work: a wave that declares a file and
    // never touches it is unfinished, and since wave 0068 the
    // closing court says so before it spends a battery.
    let touched = dir.join("src/toy.js");
    let mut body = std::fs::read_to_string(&touched).unwrap_or_default();
    body.push_str("\n// touched by the branch\n");
    std::fs::write(&touched, body).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "work: the declared file"]);
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
    // The branch does its work: a wave that declares a file and
    // never touches it is unfinished, and since wave 0068 the
    // closing court says so before it spends a battery.
    let touched = dir.join("src/toy.js");
    let mut body = std::fs::read_to_string(&touched).unwrap_or_default();
    body.push_str("\n// touched by the branch\n");
    std::fs::write(&touched, body).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "work: the declared file"]);
}

/// proves: javascript-tests-are-read-and-run@fbbe41
#[test]
fn javascript_tests_are_read_and_run() {
    if !common::machine_has("node").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // The tag is read from test/*.test.js, and the project is judged
    // whole -- adapter and all. And from EVERY name this hand vouches
    // for: a typed file, a module file, a commonjs file, a typed
    // module -- review 0046 R-4 found no probe had ever put a tag in
    // a `.ts`, and the reader's `ts`/`mts` could go missing with the
    // whole battery green.
    let dir = project("jsread", "toy.test.js", &test_file(&rev));
    for (name, body) in [
        (
            "typed.test.ts",
            format!(
                "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\n\n// proves: it-works@{rev}\ntest('typed', () => {{\n  const x: number = 1;\n  assert.strictEqual(x, 1);\n}});\n"
            ),
        ),
        (
            "m.test.mjs",
            format!(
                "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\n\n// proves: it-works@{rev}\ntest('module', () => {{\n  assert.ok(true);\n}});\n"
            ),
        ),
        (
            "c.test.cjs",
            format!(
                "const {{ test }} = require('node:test');\nconst assert = require('node:assert');\n\n// proves: it-works@{rev}\ntest('commonjs', () => {{\n  assert.ok(true);\n}});\n"
            ),
        ),
        (
            "mt.test.mts",
            format!(
                "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\n\n// proves: it-works@{rev}\ntest('typed module', () => {{\n  const x: string = 'x';\n  assert.strictEqual(x, 'x');\n}});\n"
            ),
        ),
    ] {
        std::fs::write(dir.join("test").join(name), body).unwrap();
    }
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "more"]);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a node project is judged whole:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 5"),
        "and the tag over `test('…')` is read from .js, .ts, .mjs, .cjs \
         and .mts alike:\n{said}"
    );

    // The gate runs exactly those tests by name, and reads the
    // verdict from TAP -- never from the exit code, which node hands
    // out the same for a pass and for a name that matched nothing.
    // Five files, five runners' worth of syntax, one command.
    let (said, code) = gate(&dir);
    assert_eq!(
        code, 0,
        "the work passes over tests node ran green:\n{said}"
    );
    assert!(
        said.contains("5 тестів сценаріїв зелені") && said.contains("робота проходить"),
        "and says so, for every one of the five:\n{said}"
    );

    // Names node ESCAPES in TAP: `#` goes out as `\#`, `\` as `\\`,
    // and a trailing space is kept. Review 0046 R-1: `fixes #12` --
    // the most ordinary name there is -- was one no verdict ever
    // named, and the gate held `work:` over a green test.
    let dir = project(
        "jsnames",
        "toy.test.js",
        &format!(
            "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('fixes #12', () => {{\n  assert.ok(works());\n}});\n\n// proves: it-works@{rev}\ntest('back\\\\slash', () => {{\n  assert.ok(works());\n}});\n\n// proves: it-works@{rev}\ntest('trail ', () => {{\n  assert.ok(works());\n}});\n"
        ),
    );
    let (said, code) = gate(&dir);
    assert_eq!(
        code, 0,
        "a name with `#`, a backslash, or a trailing space is read back \
         from TAP as itself:\n{said}"
    );
    assert!(
        said.contains("3 тестів сценаріїв зелені"),
        "all three, by their own names:\n{said}"
    );

    // The forms node's own documentation writes (review 0046 R-3):
    // `test.only(`, `it.only(`, `await test(`, a bound call, and any
    // width of space before the parenthesis -- each a declaration
    // the tag holds, and each one node runs.
    let dir = project(
        "jsforms",
        "toy.test.js",
        &format!(
            "import {{ test, it }} from 'node:test';\nimport assert from 'node:assert';\n\n// proves: it-works@{rev}\ntest.only('only one', () => {{\n  assert.ok(true);\n}});\n\n// proves: it-works@{rev}\nit.only('it only', () => {{\n  assert.ok(true);\n}});\n\n// proves: it-works@{rev}\nawait test('awaited', () => {{\n  assert.ok(true);\n}});\n\n// proves: it-works@{rev}\nconst bound = test('bound', () => {{\n  assert.ok(true);\n}});\n\n// proves: it-works@{rev}\ntest  (\"spaced\", () => {{\n  assert.ok(true);\n}});\n"
        ),
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "every form is a declaration:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 5"),
        "and each tag is read:\n{said}"
    );
    let (said, code) = gate(&dir);
    assert_eq!(
        code, 0,
        "and node runs each of them under its name:\n{said}"
    );
    assert!(said.contains("5 тестів сценаріїв зелені"), "{said}");

    // Two lines a tag cannot hold, refused by NAME and not as "no
    // test function": a subtest, which node runs only through its
    // parent, and a name node builds at run time.
    let subtest = format!(
        "import {{ test }} from 'node:test';\n\ntest('parent', async (t) => {{\n  // proves: it-works@{rev}\n  await t.test('inner', () => {{}});\n}});\n"
    );
    let Err(err) = keel::tags::scan_text(Path::new("t/x.test.js"), &subtest) else {
        panic!("a tag over a subtest holds nothing on its own");
    };
    assert!(
        err.reason.contains("t.test") && err.instead.contains("test("),
        "a tag over a subtest is refused as one, with the parent named as the \
         place for it: {} / {}",
        err.reason,
        err.instead
    );
    let dynamic = format!(
        "import {{ test }} from 'node:test';\n\nconst n = 1;\n// proves: it-works@{rev}\ntest(`case ${{n}}`, () => {{}});\n"
    );
    let Err(err) = keel::tags::scan_text(Path::new("t/x.test.js"), &dynamic) else {
        panic!("a name built at run time cannot be read from the source");
    };
    assert!(
        err.reason.contains("${"),
        "a name built at run time is refused as one: {}",
        err.reason
    );

    // A GREEN close: the tag and the battery meet on one key, and the
    // wave closes -- asserted, because review 0045 found no python
    // probe had ever asserted it.
    let dir = project("jsgreen", "toy.test.js", &test_file(&rev));
    reviewed(&dir);
    // With the environment pointing node's compile cache INTO the
    // project -- review 0046 R-6 found `.ncc/` left behind after
    // `close` this way, while every measurement of "writes nothing"
    // had been taken in a silent environment.
    let cache = dir.join(".ncc").display().to_string();
    // Every path of the tree before the court, to be compared whole
    // after it: the first cut of this probe looked for three names
    // only, and any other file would have passed (global review
    // 2026-09-06, tests cut R-11).
    let mut before = walk(&dir);
    before.sort();
    let (said, code) = keel_with(&dir, &["close"], &[("NODE_COMPILE_CACHE", cache)]);
    assert_eq!(code, 0, "a proven node wave closes:\n{said}");
    assert!(
        !dir.join(".ncc").exists(),
        "and node's compile cache does not land in the project, whatever \
         the environment says:\n{said}"
    );
    assert!(
        said.contains("0001-a-wave: закрита"),
        "and says so:\n{said}"
    );
    let mut after = walk(&dir);
    after.sort();
    let written: Vec<String> = after.into_iter().filter(|p| !before.contains(p)).collect();
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
        "import {{ test, it, describe }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('it works', () => {{\n  assert.ok(works());\n}});\n\ntest('nobody claims me', () => {{\n  assert.strictEqual(1, 2);\n}});\n\ntest('not now', {{ skip: true }}, () => {{\n  assert.ok(false);\n}});\n\nit(`spoken with it`, () => {{\n  assert.ok(true);\n}});\n\ndescribe('grouped', () => {{\n  test('inside', () => {{\n    assert.ok(true);\n  }});\n}});\n\ndescribe('a', () => {{\n  test('same', () => {{\n    assert.ok(false);\n  }});\n}});\n\ndescribe('b', () => {{\n  test('same', () => {{\n    assert.ok(true);\n  }});\n}});\n"
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
        said.contains("батарея: 6 тестів"),
        "the battery counts what node RAN -- the grouped one, the `it`, \
         the typed one, and the two named `same` as ONE included; not \
         the skipped one, not the describe block, not the empty file's \
         own line:\n{said}"
    );
    // Two tests of one name in two describes are one key, and the
    // key is red if EITHER is: the failing one comes first here, so
    // a court that let the last line win would call it green (review
    // 0046 R-2, mutation M14).
    assert!(
        said.contains("червоний тест") && said.contains("same"),
        "one red among the two named `same` is a red `same`:\n{said}"
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

/// What node is HANDED: the pattern anchored `^…$` so `it works` does
/// not select `it works too`, the TAP reporter, the file, one argument
/// each and never through a shell -- read off a shim named `node`
/// that logs its argv. Review 0046 R-8: the anchors were promised by
/// the contract and held by no probe (mutation M08 survived).
#[test]
fn what_node_is_handed() {
    // No `machine_has("sh")` here: that hand asks `sh --version`,
    // which dash answers with 2, and the probe skipped itself on the
    // very machine it was written on -- the shape review 0044 named
    // (a skip that fires where everything is installed judges
    // nothing). The shim below is `#!/bin/sh`; a machine without a
    // POSIX shell cannot run this battery at all.
    let rev = keel::rev::text_rev(BODY);
    // A name with a dot and parentheses, so the escaping is in the
    // argv too (mutations M09, M10): `a.b (x)` must reach node as
    // `^a\\.b \\(x\\)$`.
    let dir = project(
        "jsshim",
        "toy.test.js",
        &format!(
            "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\n\n// proves: it-works@{rev}\ntest('a.b (x)', () => {{\n  assert.ok(true);\n}});\n"
        ),
    );
    let bin = dir.join("shim");
    std::fs::create_dir_all(&bin).unwrap();
    let log = dir.join("argv.log");
    std::fs::write(
        bin.join("node"),
        format!(
            "#!/bin/sh\npwd > '{}'\nfor a in \"$@\"; do printf '[%s]\\n' \"$a\" >> '{}'; done\nprintf 'TAP version 13\\nok 1 - a.b (x)\\n  ---\\n  type: %s\\n  ...\\n1..1\\n' \"'test'\"\n",
            log.display(),
            log.display()
        ),
    )
    .unwrap();
    let mut perms = std::fs::metadata(bin.join("node")).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
    std::fs::set_permissions(bin.join("node"), perms).unwrap();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    // The branch does its work: a wave that declares a file and
    // never touches it is unfinished, and since wave 0068 the
    // closing court says so before it spends a battery.
    let touched = dir.join("src/toy.js");
    let mut body = std::fs::read_to_string(&touched).unwrap_or_default();
    body.push_str("\n// touched by the branch\n");
    std::fs::write(&touched, body).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: the declared file"]);
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, "work: тіло\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
        .env("PATH", path)
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
        "the shim's TAP is read as green:\n{said}"
    );
    let logged = std::fs::read_to_string(&log).unwrap();
    let mut lines = logged.lines();
    let cwd = lines.next().unwrap_or_default();
    assert_eq!(
        std::fs::canonicalize(cwd).unwrap(),
        std::fs::canonicalize(&dir).unwrap(),
        "node runs in the project's own root"
    );
    let argv: Vec<&str> = lines.collect();
    assert_eq!(
        argv,
        vec![
            "[--test]",
            "[--test-reporter=tap]",
            "[--test-name-pattern=^a\\.b \\(x\\)$]",
            "[test/toy.test.js]",
        ],
        "one argument each, the pattern anchored at both ends, and nothing \
         a shell could have split:\n{logged}"
    );
}
