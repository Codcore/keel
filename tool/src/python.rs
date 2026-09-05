//! The python adapter (contract tool-adapter-python): the one place
//! that knows how a pytest project keeps its tests and its modules.
//! It runs `pytest` as a command of the system, exactly as a person
//! would in a terminal, and writes nothing anywhere -- which pytest
//! does not do by itself, so the adapter tells it not to.
//!
//! pytest, and said so aloud: unittest without pytest is another
//! wave.

use crate::docs::Refusal;
use crate::i18n::{t, ta};
use crate::tags::TestTag;
use crate::targs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `tests/**/test_*.py` and `tests/**/*_test.py` -- where the proves
/// tags live, in the two names pytest collects by. A project without
/// a tests directory has none, and that is not a refusal.
pub fn test_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let dir = root.join("tests");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let mut stack = vec![dir];
    while let Some(here) = stack.pop() {
        let entries = std::fs::read_dir(&here).map_err(|e| Refusal {
            file: here.clone(),
            reason: ta("docs-unreadable", targs!("error" => e.to_string())),
            instead: t("docs-unreadable-instead"),
        })?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(is_test_file)
            {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

fn is_test_file(name: &str) -> bool {
    name.ends_with(".py") && (name.starts_with("test_") || name.ends_with("_test.py"))
}

/// The `.py` files in `tests/` this adapter does NOT read: pytest
/// collects tests only from `test_*.py` and `*_test.py`, so a
/// `conftest.py` or a helper is walked past -- and named, as the
/// ruby and elixir hands name theirs (review 0042 R-6).
pub fn unread_files(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    let mut stack = vec![root.join("tests")];
    while let Some(here) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&here) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if path.file_name().is_some_and(|n| n == "__pycache__") {
                    continue;
                }
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "py")
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| is_test_file(n) || n == "__init__.py")
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Where a module's source may live -- every layout python keeps one
/// in, because a project chooses and the court must not: `toy.bar`
/// is `src/toy/bar.py` or `toy/bar.py`; the bare `toy` is a package
/// (`…/toy/__init__.py`) or a file (`…/toy.py`), under `src/` or at
/// the root. The name is the name: python makes no snake_case out of
/// anything.
pub fn module_paths(root: &Path, module: &str) -> Vec<PathBuf> {
    let joined = module.split('.').collect::<Vec<_>>().join("/");
    let mut out = Vec::with_capacity(4);
    for base in [root.join("src"), root.to_path_buf()] {
        out.push(base.join(format!("{joined}.py")));
        out.push(base.join(&joined).join("__init__.py"));
    }
    out
}

/// Runs exactly the tagged test, by the node id pytest itself uses:
/// `tests/test_toy.py::test_it_works`, or with the class in front
/// for a method. The exit code alone says what came of it.
pub fn run_test(root: &Path, tag: &TestTag) -> Result<crate::adapter::Outcome, Refusal> {
    let relative = tag.file.strip_prefix(root).unwrap_or(&tag.file);
    let node = format!("{}::{}", relative.display(), tag.test);
    let (said, code) = pytest(root, &[node])?;
    Ok(classify(&said, code))
}

/// The whole battery, verdicts per test, **in pytest's own words**:
/// one `-v` line per test, node id and verdict on it. So the roll
/// and the verdicts come from the runner from the first day -- the
/// lesson ruby paid for in review 0038 and elixir walked back in
/// review 0042, in the plan this time rather than the review.
///
/// A collection that broke (code 2) is a refusal with python's own
/// words: without a collection there is no verdict for anyone.
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    let (said, code) = pytest(root, &["-v".to_string()])?;
    if code == 2 {
        return Err(Refusal {
            file: root.to_path_buf(),
            reason: ta(
                "adapter-python-broken",
                targs!("error" => first_error(&said)),
            ),
            instead: t("adapter-python-broken-instead"),
        });
    }
    let mut out: BTreeMap<(String, String), bool> = BTreeMap::new();
    for (file, name, verdict) in ran(&said) {
        let stem = Path::new(&file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        // `PASSED` is green; `FAILED` and `ERROR` are not; a
        // `SKIPPED` or `XFAIL` did not run, and a test that did not
        // run is not a green one either.
        out.insert((stem, name), verdict == "PASSED");
    }
    Ok(out)
}

/// Every test pytest says it ran, from its `-v` lines:
/// `tests/test_toy.py::TestGrouped::test_inside PASSED  [ 66%]`.
/// The file, the name after the first `::` (class and all), and the
/// verdict word.
pub fn ran(said: &str) -> Vec<(String, String, String)> {
    let mut out: Vec<(String, String, String)> = Vec::new();
    for line in said.lines() {
        let trimmed = line.trim();
        let Some((node, rest)) = trimmed.split_once(' ') else {
            continue;
        };
        let Some((file, name)) = node.split_once("::") else {
            continue;
        };
        let verdict = rest.split_whitespace().next().unwrap_or_default();
        if !matches!(
            verdict,
            "PASSED" | "FAILED" | "ERROR" | "SKIPPED" | "XFAIL" | "XPASS"
        ) {
            continue;
        }
        if !file.ends_with(".py") || name.is_empty() {
            continue;
        }
        out.push((file.to_string(), name.to_string(), verdict.to_string()));
    }
    out
}

/// What a run came to, read from how pytest left. This tongue
/// answers with five distinct codes, measured before the plan: 0
/// green, 1 a test failed, 2 collection broke, 4 no such node, 5
/// nothing collected.
///
/// The plan said the text would never be asked, and the measurement
/// after it corrected the plan by one case: asked for ONE node in a
/// file whose import broke, pytest leaves with 4 ("not found") and
/// prints the SyntaxError above it -- so code 4 carries two meanings,
/// exactly as elixir's 1 does, and the text tells them apart.
pub fn classify(said: &str, code: i32) -> crate::adapter::Outcome {
    match code {
        0 => crate::adapter::Outcome::Green,
        1 => crate::adapter::Outcome::Failed,
        2 => crate::adapter::Outcome::BuildBroken(first_error(said)),
        4 if collection_broke(said) => crate::adapter::Outcome::BuildBroken(first_error(said)),
        4 | 5 => crate::adapter::Outcome::NotRun,
        _ => crate::adapter::Outcome::Failed,
    }
}

/// pytest's traceback lines begin with `E `; one that names an error
/// is a collection that broke, whatever code came after it.
fn collection_broke(said: &str) -> bool {
    said.lines()
        .map(str::trim)
        .any(|line| line.starts_with("E ") && line.contains("Error"))
}

/// Python's own words for what broke: the `E   SyntaxError: …` line
/// pytest prints under the traceback, with the file and line above
/// it when they are there.
fn first_error(said: &str) -> String {
    let lines: Vec<&str> = said.lines().map(str::trim).collect();
    let error = lines
        .iter()
        .position(|line| line.starts_with("E ") && line.contains("Error"));
    match error {
        Some(at) => {
            let words = lines[at].trim_start_matches('E').trim();
            let place = lines[..at]
                .iter()
                .rev()
                .find(|line| line.starts_with("E ") && line.contains("File \""))
                .map(|line| line.trim_start_matches('E').trim());
            match place {
                Some(place) => format!("{words} ({place})"),
                None => words.to_string(),
            }
        }
        None => lines
            .iter()
            .find(|line| line.contains("ERROR collecting") || line.contains("Interrupted"))
            .map(|line| line.to_string())
            .unwrap_or_else(|| "pytest could not collect this project".to_string()),
    }
}

/// pytest as a command of the system, in the project's own world --
/// and told to leave that world as it found it: no cache directory,
/// no bytecode. Measured: with both, `find` after a run comes back
/// empty; without them, pytest writes `.pytest_cache` and
/// `__pycache__` into the project it was asked only to judge.
fn pytest(root: &Path, args: &[String]) -> Result<(String, i32), Refusal> {
    let mut command = Command::new("pytest");
    command
        .args(["-p", "no:cacheprovider"])
        .args(args)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .current_dir(root);
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: ta("adapter-python-failed", targs!("error" => e.to_string())),
        instead: t("adapter-python-failed-instead"),
    })?;
    Ok((
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    ))
}
