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
    std::fs::write(dir.join("test/toy.test.js"), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// The same project with its tests under `tests/` and no tag yet --
/// the shape `keel next` speaks to.
fn project_in_tests(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"javascript\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::create_dir_all(dir.join("keel/contracts")).unwrap();
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
    std::fs::write(
        dir.join("tests/toy.test.js"),
        "import { test } from 'node:test';\n\ntest('untagged', () => {});\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
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
    std::fs::write(
        dir.join("src/toy.js"),
        "export function works( {\n  return true;\n}\n",
    )
    .unwrap();
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "a broken module is not a green test:\n{said}");
    assert!(
        said.contains("SyntaxError") && said.contains("toy.js"),
        "and the refusal carries node's own words, and the place they \
         point at:\n{said}"
    );
    assert!(
        !said.contains("червоний тест"),
        "a build that broke is a REFUSAL, not a red test:\n{said}"
    );

    // The same broken module seen by the BATTERY: close refuses with
    // node's words, rather than reading the file's own `not ok` as a
    // red test or skipping the file in silence.
    let dir = project("jsbrokenclose", &tagged(&rev, "it works"));
    std::fs::write(
        dir.join("src/toy.js"),
        "export function works( {\n  return true;\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "review"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a battery over a file that did not load does not close:\n{said}"
    );
    assert!(
        said.contains("SyntaxError") && !said.contains("червоний тест"),
        "and it is a refusal with node's words, not a red test:\n{said}"
    );

    // A tagged test that FAILS: node leaves with 1, the same code as
    // the SyntaxError above, and the verdict is the `not ok` line --
    // "падає", not "broken", not "did not run". Review 0046 asked
    // where the probe was that ran the gate over a red test, and
    // found none: four mutations of `classify` (every named line
    // green; a skip green) survived the scenario probes.
    let dir = project(
        "jsred",
        &format!(
            "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\nimport {{ works }} from '../src/toy.js';\n\n// proves: it-works@{rev}\ntest('it works', () => {{\n  assert.strictEqual(works(), false);\n}});\n"
        ),
    );
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "work over a red test does not pass:\n{said}");
    assert!(
        said.contains("падає") && said.contains("it works"),
        "and it is called a failing test, by name:\n{said}"
    );
    assert!(
        !said.contains("SyntaxError") && !said.contains("не виконав"),
        "not a broken build, not \"did not run\":\n{said}"
    );

    // A tagged test node SKIPPED -- by option and by method alike --
    // is "did not run" (§7.12): `ok … # SKIP` is not green.
    for (name, line) in [
        ("jsskipopt", "test('it works', { skip: true }, () => {"),
        ("jsskipmethod", "test.skip('it works', () => {"),
        ("jstodo", "test.todo('it works', () => {"),
    ] {
        let dir = project(
            name,
            &format!(
                "import {{ test }} from 'node:test';\nimport assert from 'node:assert';\n\n// proves: it-works@{rev}\n{line}\n  assert.ok(false);\n}});\n"
            ),
        );
        let (said, code) = gate(&dir);
        assert_ne!(code, 0, "{name}: a skipped test proves nothing:\n{said}");
        assert!(
            said.contains("не виконав жодного тесту"),
            "{name}: and it is \"did not run\", not green and not red:\n{said}"
        );
    }

    // Two tests of one name in two describes: `--test-name-pattern`
    // runs both, the tag cannot say which it meant, and the SAFE
    // reading is the battery's -- any red among them is red. The
    // gate used to take the first line (review 0046 R-2): green
    // under `a`, and `work:` blessed over the tagged red under `b`.
    let dir = project(
        "jssame",
        &format!(
            "import {{ test, describe }} from 'node:test';\nimport assert from 'node:assert';\n\ndescribe('a', () => {{\n  test('it works', () => {{\n    assert.ok(true);\n  }});\n}});\n\ndescribe('b', () => {{\n  // proves: it-works@{rev}\n  test('it works', () => {{\n    assert.ok(false);\n  }});\n}});\n"
        ),
    );
    let (said, code) = gate(&dir);
    assert_ne!(
        code, 0,
        "a red among two tests of one name is red, whichever came first:\n{said}"
    );
    assert!(said.contains("падає"), "and said so:\n{said}");

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
    assert_ne!(
        code, 0,
        "work over a test that never ran does not pass:\n{said}"
    );
    assert!(
        !said.contains("робота проходить"),
        "and \"did not run\" is not read as green:\n{said}"
    );

    // A name with regex metacharacters goes into --test-name-pattern
    // escaped, so `a.b (x)` selects `a.b (x)` and nothing else.
    let dir = project("jsregex", &tagged(&rev, "a.b (x)"));
    let (said, code) = gate(&dir);
    assert_eq!(
        code, 0,
        "a name with metacharacters is selected as itself:\n{said}"
    );

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
    assert!(
        said.contains("helper.js"),
        "and names the file in test/ it did not read:\n{said}"
    );

    // The frame's own row about this tongue: what npm LEAVES, named
    // by the adapter -- said, not guessed (review 0046 R-11 asked
    // where the javascript case of that probe was). Until wave 0057
    // the row said "this tongue builds nothing", which was true of
    // the build directory and false of the tree: `npm install`
    // writes node_modules/ (queue after 0055, bugs R-21).
    let (said, _) = keel(&dir, &["init"]);
    assert!(
        said.contains("node_modules/"),
        "the ignore row names what npm leaves in the tree:\n{said}"
    );

    // And `keel next` names the directory the project KEEPS its
    // tests in: `tests/` where that is the one that exists, with the
    // tag in node's own comment mark (review 0046 R-10 -- the hint
    // said `test/` to a project of `tests/`, and `///` to every
    // tongue).
    let dir = project_in_tests("jsnexttests");
    let (said, code) = keel(&dir, &["next"]);
    assert_eq!(code, 0, "next has a step to name:\n{said}");
    assert!(
        said.contains("читає тести в tests/"),
        "the hint names the directory that exists:\n{said}"
    );
    assert!(
        said.contains("// proves: it-works@") && !said.contains("/// proves"),
        "and writes the tag as node would read it:\n{said}"
    );

    // The line a person is told to type runs exactly the tagged
    // test -- with an apostrophe in the name, which left the pasted
    // line waiting for a closing quote (review 0046 R-5). The line
    // is handed to a real shell here, and node's TAP names the test.
    let dir = project("jsnextline", &tagged(&rev, "it\\'s fine (a.b)"));
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["next"]);
    let line = said
        .lines()
        .find(|line| line.contains("node --test"))
        .unwrap_or_else(|| panic!("next hands the tongue's own run line:\n{said}"))
        .trim();
    assert!(
        line.contains("--test-name-pattern=") && line.ends_with("test/toy.test.js"),
        "the line runs one file by one pattern:\n{line}"
    );
    let out = Command::new("sh")
        .args(["-c", line])
        .current_dir(&dir)
        .output()
        .unwrap();
    let ran = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        out.status.success() && ran.contains("ok 1 - it's fine (a.b)") && ran.contains("# tests 1"),
        "pasted into a shell, the line runs that one test and no other:\n{line}\n{ran}"
    );
}

/// The TAP shapes, played without a project on disk: a test, a suite,
/// a skipped test, the file counted as a test, and node's error above
/// a file that did not load.
#[test]
fn what_node_said_and_never_how_it_left() {
    use keel::adapter::Outcome;
    let green = "TAP version 13\n# Subtest: it works\nok 1 - it works\n  ---\n  duration_ms: 0.9\n  type: 'test'\n  ...\n1..1\n# tests 1\n# pass 1\n";
    assert!(matches!(
        keel::javascript::classify(green, "it works"),
        Outcome::Green
    ));
    let red = "# Subtest: it falls\nnot ok 2 - it falls\n  ---\n  type: 'test'\n  ...\n";
    assert!(matches!(
        keel::javascript::classify(red, "it falls"),
        Outcome::Failed
    ));
    // The file counted as one passed test, no line naming the test:
    // exit code 0, and still NOT RUN.
    let nothing = "1..0\n# Subtest: test/toy.test.js\nok 1 - test/toy.test.js\n  ---\n  type: 'test'\n  ...\n# tests 1\n# pass 1\n";
    assert!(matches!(
        keel::javascript::classify(nothing, "it works"),
        Outcome::NotRun
    ));
    let skipped = "ok 1 - it works # SKIP\n  ---\n  type: 'test'\n  ...\n";
    assert!(matches!(
        keel::javascript::classify(skipped, "it works"),
        Outcome::NotRun
    ));
    let broken = "# file:///p/src/toy.js:1\n# export function works( {\n# SyntaxError: Unexpected token 'true'\n#     at compileSourceTextModule (node:internal/x:1:1)\n# Subtest: test/toy.test.js\nnot ok 1 - test/toy.test.js\n  ---\n  type: 'test'\n  ...\n";
    match keel::javascript::classify(broken, "it works") {
        Outcome::BuildBroken(words) => assert!(
            words.contains("SyntaxError") && words.contains("toy.js:1"),
            "node's words and its place: {words}"
        ),
        _ => panic!("a file that did not load is a broken build"),
    }
    // The roll: a suite is not a test, and its line is not in the map.
    let nested = "# Subtest: grouped\n    # Subtest: inside\n    ok 1 - inside\n      ---\n      type: 'test'\n      ...\n    1..1\nok 3 - grouped\n  ---\n  type: 'suite'\n  ...\n";
    let entries = keel::javascript::tap(nested);
    assert_eq!(entries.len(), 2);
    assert!(
        entries
            .iter()
            .any(|e| e.name == "inside" && !e.suite && e.ok)
    );
    assert!(entries.iter().any(|e| e.name == "grouped" && e.suite));
    // And the name escaped for the pattern.
    assert_eq!(
        keel::javascript::escape_regex("a.b (x) [y] $z"),
        "a\\.b \\(x\\) \\[y\\] \\$z"
    );

    // TAP's own escaping read back (review 0046 R-1): `\#` is `#`,
    // `\\` is `\`, a trailing space stays, and a directive is the
    // first UNESCAPED ` # `.
    let escaped = "ok 1 - fixes \\#12\n  ---\n  type: 'test'\n  ...\nok 2 - back\\\\slash\n  ---\n  type: 'test'\n  ...\nok 3 - trail \n  ---\n  type: 'test'\n  ...\nok 4 - a \\# b # SKIP\n  ---\n  type: 'test'\n  ...\n";
    let names: Vec<(String, bool)> = keel::javascript::tap(escaped)
        .into_iter()
        .map(|e| (e.name, e.skipped))
        .collect();
    assert_eq!(
        names,
        vec![
            ("fixes #12".to_string(), false),
            ("back\\slash".to_string(), false),
            ("trail ".to_string(), false),
            ("a # b".to_string(), true),
        ]
    );
    assert!(matches!(
        keel::javascript::classify(escaped, "fixes #12"),
        Outcome::Green
    ));

    // Two lines of one name (review 0046 R-2): a red among them is
    // red in EITHER order, and a suite that happens to carry the
    // test's name is not a line of the test (M22) -- node enters
    // every describe under a pattern, so `describe('it works')` with
    // a red inside prints `not ok - it works` as a SUITE.
    let red_first = "not ok 1 - same\n  ---\n  type: 'test'\n  ...\nok 2 - same\n  ---\n  type: 'test'\n  ...\n";
    let red_last = "ok 1 - same\n  ---\n  type: 'test'\n  ...\nnot ok 2 - same\n  ---\n  type: 'test'\n  ...\n";
    assert!(matches!(
        keel::javascript::classify(red_first, "same"),
        Outcome::Failed
    ));
    assert!(matches!(
        keel::javascript::classify(red_last, "same"),
        Outcome::Failed
    ));
    let suite_of_that_name = "    not ok 1 - inner fails\n      ---\n      type: 'test'\n      ...\nnot ok 1 - it works\n  ---\n  type: 'suite'\n  ...\nok 2 - it works\n  ---\n  type: 'test'\n  ...\n";
    assert!(matches!(
        keel::javascript::classify(suite_of_that_name, "it works"),
        Outcome::Green
    ));
    // A skipped twin beside a green one is green; two skipped ones
    // did not run.
    let one_skipped = "ok 1 - same # SKIP\n  ---\n  type: 'test'\n  ...\nok 2 - same\n  ---\n  type: 'test'\n  ...\n";
    assert!(matches!(
        keel::javascript::classify(one_skipped, "same"),
        Outcome::Green
    ));
    let both_skipped = "ok 1 - same # SKIP\n  ---\n  type: 'test'\n  ...\nok 2 - same # TODO\n  ---\n  type: 'test'\n  ...\n";
    assert!(matches!(
        keel::javascript::classify(both_skipped, "same"),
        Outcome::NotRun
    ));

    // The reader's forms (review 0046 R-3), without a project.
    for (line, name) in [
        ("test.only('only one', () => {", "only one"),
        ("test.skip(\"skipped\", () => {", "skipped"),
        ("test.todo('later')", "later"),
        ("it.only('it only', () => {", "it only"),
        ("it.skip('it skip', () => {", "it skip"),
        ("await test('awaited', async () => {", "awaited"),
        ("const p = test('bound', () => {", "bound"),
        (
            "let q = await it('bound and awaited', () => {",
            "bound and awaited",
        ),
        ("test  ('spaced', () => {", "spaced"),
        ("test('it\\'s fine', () => {", "it's fine"),
    ] {
        assert_eq!(
            keel::tags::js_test_name(line).as_deref(),
            Some(name),
            "the reader names {line}"
        );
    }
    for line in [
        "describe('a group', () => {",
        "tests.forEach((t) => {",
        "items.map(it => it)",
        "test.each([1, 2])('n', () => {",
        "await t.test('inner', () => {",
        "test(`case ${n}`, () => {",
    ] {
        assert_eq!(
            keel::tags::js_test_name(line),
            None,
            "the reader does not name {line}"
        );
    }
    assert!(matches!(
        keel::tags::js_call("await t.test('inner', () => {"),
        Some(keel::tags::JsCall::Subtest)
    ));
    assert!(matches!(
        keel::tags::js_call("test(`case ${n}`, () => {"),
        Some(keel::tags::JsCall::Dynamic)
    ));
}
