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
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"python\"\n",
    )
    .unwrap();
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
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
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

    // A tag over a METHOD is read with the class in front, which is
    // the only name pytest will select it by -- and a docstring
    // between them, holding a `def test_` of its own, is text: it
    // declares nothing and orphans nothing.
    // And a module docstring that DOCUMENTS the tag syntax -- the
    // most natural place for a `# proves:` line that is not a tag.
    // Read as one, it bound a promise to a `def` that exists only as
    // an example, and the gate then ran a test pytest does not have.
    let grouped = format!(
        "\"\"\"How a test claims a promise here:\n\n# proves: it-works@{rev}\ndef test_example():\n    pass\n\"\"\"\nfrom toy import works\n\nclass TestGrouped:\n    \"\"\"A group.\n\n    def test_example():\n        pass\n    \"\"\"\n\n    # proves: it-works@{rev}\n    def test_inside(self):\n        assert works()\n\ndef test_after():\n    assert True\n"
    );
    let dir = project("pygrouped", &grouped);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "a tag over a method is read, past the docstring:\n{said}"
    );
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "exactly the one tag, and the docstring's `def test_example` \
         is not a declaration:\n{said}"
    );
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
        "and the method really runs under `TestGrouped::test_inside` -- \
         a bare `test_inside` would have been \"not found\", never \
         green:\n{said}"
    );

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

/// The shapes review 0045 found unjudged or wrong -- every one of
/// them ordinary pytest, and every one measured with the real
/// runner: a green close (the tag MATCHING the battery, which no
/// probe had asserted), two files of one stem, parametrize, skip
/// and xfail, a black-style decorator, `async def`, a project's own
/// `addopts = "-q"`, a comment in column 0 inside a class, nested
/// classes, `*_test.py`, a `'''` docstring, and the project left
/// untouched after `close`.
///
/// proves: python-tests-are-read-and-run@e1fb34
#[test]
fn the_shapes_pytest_comes_in() {
    if !common::machine_has("pytest").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);
    let reviewed = |dir: &Path| {
        std::fs::write(
            dir.join("keel/reviews/0001-a-wave.md"),
            "# Рецензія\n\nok\n",
        )
        .unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-q", "-m", "review"]);
        git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    };

    // M28: a GREEN close. The tag and the battery meet on the same
    // key, and the wave closes -- the one thing every earlier python
    // probe left unasserted, its only `close` being red.
    let dir = project("pygreen", &test_file(&rev));
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "a proven python wave closes:\n{said}");
    assert!(
        said.contains("0001-a-wave: закрита"),
        "and says so:\n{said}"
    );
    // R-15: and after the battery ran three times, nothing was
    // written into the project -- measured HERE, after close.
    let written: Vec<String> = walk(&dir)
        .into_iter()
        .filter(|p| p.contains("__pycache__") || p.contains(".pytest_cache"))
        .collect();
    assert!(
        written.is_empty(),
        "close leaves the project as it found it: {written:?}"
    );

    // R-1: two files of one stem in two subdirectories. pytest says
    // three tests, one failed; the court used to say two tests and
    // close the wave -- the second file overwrote the first, and the
    // verdict depended on collection order.
    let dir = project("pystem", &test_file(&rev));
    for sub in ["a", "b"] {
        std::fs::create_dir_all(dir.join("tests").join(sub)).unwrap();
        std::fs::write(dir.join("tests").join(sub).join("__init__.py"), "").unwrap();
    }
    std::fs::write(dir.join("tests/__init__.py"), "").unwrap();
    std::fs::write(
        dir.join("tests/a/test_x.py"),
        "def test_same():\n    assert 1 == 2\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/b/test_x.py"),
        "def test_same():\n    assert True\n",
    )
    .unwrap();
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 3 тестів"),
        "three tests, as pytest counted them -- two of one stem:\n{said}"
    );
    assert!(
        said.contains("червоний тест") && said.contains("test_same") && said.contains("a/test_x"),
        "and the red one is named with the directory that tells it \
         from its namesake:\n{said}"
    );
    assert_ne!(
        code, 0,
        "a red test in a subdirectory holds the wave open:\n{said}"
    );

    // R-2: a parametrized test is one test to its tag. pytest names
    // the instances `test_x[1-2]`; the tag names `test_x`; both the
    // gate and the close must find it.
    let dir = project(
        "pyparam",
        &format!(
            "import pytest\nfrom toy import works\n\n# proves: it-works@{rev}\n@pytest.mark.parametrize(\"a,b\", [(1, 2), (3, 4)])\ndef test_it_works(a, b):\n    assert works() and a < b\n"
        ),
    );
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "a parametrized test proves its scenario:\n{said}");
    assert!(
        said.contains("батарея: 1 тестів"),
        "and its instances fold into the one test the tag names:\n{said}"
    );

    // R-3: a SKIPPED test did not run. The gate must not bless work
    // over it (pytest leaves with 0, and 0 was read as green), and
    // the close must neither call it red nor let it hold an unclaimed
    // wave open -- it is not in the battery at all.
    let dir = project(
        "pyskip",
        &format!(
            "import pytest\nfrom toy import works\n\n# proves: it-works@{rev}\n@pytest.mark.skip(reason=\"not now\")\ndef test_it_works():\n    assert works()\n\n@pytest.mark.xfail\ndef test_expected():\n    assert False\n\ndef test_plain():\n    assert True\n"
        ),
    );
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
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "work over a SKIPPED test is not blessed -- pytest's 0 here \
         means \"did not run\", not green:\n{said}"
    );
    assert!(
        !said.contains("робота проходить"),
        "and it does not say it passes:\n{said}"
    );
    // Already on the wave's branch from the gate above: the review
    // file lands there, and the close judges it there.
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "review"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        !said.contains("червоний тест"),
        "a skipped or xfail test is not a RED test:\n{said}"
    );
    assert!(
        said.contains("батарея: 1 тестів"),
        "the battery holds only what ran -- the plain test:\n{said}"
    );
    assert!(
        said.contains("не виконала тесту \"test_it_works\""),
        "and the claimed, skipped test is a lack said as \"did not run\":\n{said}"
    );

    // R-4, R-5, R-9, R-10, T4: the reader and the shapes of a file.
    // A black-style decorator over several lines; `async def`; a
    // comment in column 0 inside a class; a nested class; a `'''`
    // docstring holding a `def test_` of its own.
    let shapes = format!(
        "import pytest\nfrom toy import works\n\n'''How a test claims a promise:\n\n# proves: it-works@{rev}\ndef test_in_a_docstring():\n    pass\n'''\n\n# proves: it-works@{rev}\n@pytest.mark.parametrize(\n    \"a,b,expected\", [(1, 2, 3), (10, 20, 30), (100, 200, 300), (1000, 2000, 3000)]\n)\ndef test_it_works(a, b, expected):\n    assert works() and a + b == expected\n\nasync def test_async():\n    assert True\n\nclass TestOuter:\n# a comment in column 0, which python allows\n    class TestInner:\n        def test_deep(self):\n            assert True\n\n    def test_after_inner(self):\n        assert True\n"
    );
    let dir = project("pyshapes", &shapes);
    std::fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n\n[tool.pytest.ini_options]\npythonpath = [\"src\"]\ntestpaths = [\"tests\"]\nasyncio_mode = \"auto\"\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "every one of those shapes is read, none refused:\n{said}"
    );
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "exactly the one tag -- the docstring's def is text:\n{said}"
    );
    let tags = keel::tags::scan_text(
        Path::new("tests/test_toy.py"),
        &std::fs::read_to_string(dir.join("tests/test_toy.py")).unwrap(),
    )
    .unwrap();
    assert_eq!(tags.len(), 1);
    assert_eq!(
        tags[0].test, "test_it_works",
        "the tag rides over the whole decorator"
    );
    // The names the reader would give the others, checked without a
    // tag: pytest's own names, class by class.
    let named = |body: &str| -> Vec<String> {
        keel::tags::scan_text(Path::new("t/test_x.py"), body)
            .unwrap()
            .into_iter()
            .map(|t| t.test)
            .collect()
    };
    assert_eq!(
        named(&format!(
            "class TestOuter:\n# comment\n    class TestInner:\n        # proves: it-works@{rev}\n        def test_deep(self):\n            pass\n"
        )),
        vec!["TestOuter::TestInner::test_deep".to_string()],
        "nested classes stack, and a column-0 comment closes nothing"
    );
    assert_eq!(
        named(&format!(
            "class TestOuter:\n    class TestInner:\n        def test_deep(self):\n            pass\n\n    # proves: it-works@{rev}\n    def test_after(self):\n        pass\n"
        )),
        vec!["TestOuter::test_after".to_string()],
        "a method after the inner class belongs to the outer one"
    );
    assert_eq!(
        named(&format!(
            "# proves: it-works@{rev}\nasync def test_async():\n    pass\n"
        )),
        vec!["test_async".to_string()],
        "async def is a declaration"
    );

    // R-6: the project's own `addopts = "-q"` used to silence the
    // voice the battery lives by -- "0 tests" over a green run.
    // M3: a `*_test.py` file is read too.
    let dir = project("pyaddq", &test_file(&rev));
    std::fs::write(
        dir.join("pyproject.toml"),
        "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n\n[tool.pytest.ini_options]\npythonpath = [\"src\"]\ntestpaths = [\"tests\"]\naddopts = \"-qq\"\n",
    )
    .unwrap();
    // `-qq`, which even `-vv` does not outshout: only the `-rA`
    // summary survives it. And the `*_test.py` file carries a tag of
    // its own, so the reader's second name is judged, not only the
    // runner's.
    std::fs::write(
        dir.join("tests/extra_test.py"),
        format!("# proves: it-works@{rev}\ndef test_extra():\n    assert True\n"),
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a tag in a *_test.py file is read:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 2"),
        "both names pytest collects by are read for tags:\n{said}"
    );
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 2 тестів"),
        "the roll survives the project's -qq, and *_test.py is in it:\n{said}"
    );
    assert_eq!(code, 0, "and the wave closes:\n{said}");
}
