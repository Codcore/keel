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

/// Whether a path of the tree, relative to the root, is one
/// `test_files` would read: under `tests/`, named as pytest collects.
/// The §7.15 court asks this of a tree that is not on disk (wave
/// 0050).
pub fn is_test_path(rel: &str) -> bool {
    rel.strip_prefix("tests/")
        .and_then(|rest| rest.rsplit('/').next())
        .is_some_and(is_test_file)
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
/// the root. The PACKAGE comes first, because that is what python
/// itself imports when both exist (review 0045 R-8). The name is the
/// name: python makes no snake_case out of anything.
///
/// A name with an empty segment -- `.usr.share.x`, `toy..bar`,
/// `toy.` -- is no module at all, and is not looked for: joined, it
/// began with `/` and `Path::join` then REPLACED the root, so a
/// contract could hold a file anywhere on the machine (review 0045
/// R-7).
pub fn module_paths(root: &Path, module: &str) -> Vec<PathBuf> {
    let segments: Vec<&str> = module.split('.').collect();
    if segments
        .iter()
        .any(|part| part.is_empty() || part.contains('/') || part.contains('\\'))
    {
        return Vec::new();
    }
    let joined = segments.join("/");
    let mut out = Vec::with_capacity(4);
    for base in [root.join("src"), root.to_path_buf()] {
        out.push(base.join(&joined).join("__init__.py"));
        out.push(base.join(format!("{joined}.py")));
    }
    out
}

/// Runs exactly the tagged test, by the node id pytest itself uses:
/// `tests/test_toy.py::test_it_works`, or with the classes in front
/// for a method. A parametrized test is selected whole by that same
/// bare node -- pytest runs every instance under it.
pub fn run_test(root: &Path, tag: &TestTag) -> Result<crate::adapter::Outcome, Refusal> {
    let relative = tag.file.strip_prefix(root).unwrap_or(&tag.file);
    let node = format!("{}::{}", relative.display(), tag.test);
    let (said, code) = pytest(root, &[node])?;
    Ok(classify(&said, code))
}

/// The whole battery, verdicts per test, **in pytest's own words**.
///
/// The roll is read from the `-rA` summary -- `PASSED <file>::<node>`
/// one per test -- and not from the `-v` lines, because a project's
/// own `addopts = "-q"` silences `-v` and the battery then read
/// "0 tests" over a green run (review 0045 R-6); the summary survives
/// any quietness. Both shapes are read, and either is enough.
///
/// A parametrized test is one test to a tag, whatever pytest calls
/// its instances (`test_x[1-2]`, `test_x[3-4]`): the instances fold
/// into the bare name, green only when every one of them passed
/// (review 0045 R-2). A SKIPPED, XFAIL or XPASS test did not run as a
/// proof of anything: it is not in the map at all -- not green, and
/// not a red that would hold a wave open (review 0045 R-3, §7.12).
///
/// A collection that broke (code 2) is a refusal with python's own
/// words: without a collection there is no verdict for anyone.
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    let (said, code) = pytest(root, &["-vv".to_string()])?;
    if code == 2 || (code == 4 && usage_error(&said)) {
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
        let green = match verdict.as_str() {
            "PASSED" => true,
            "FAILED" | "ERROR" => false,
            _ => continue,
        };
        let key = crate::adapter::battery_key(root, &root.join(&file));
        let name = bare_name(&name);
        out.entry((key, name))
            .and_modify(|was| *was = *was && green)
            .or_insert(green);
    }
    Ok(out)
}

/// `test_x[1-2]` is an instance of `test_x`; the tag names the test.
fn bare_name(name: &str) -> String {
    match name.split_once('[') {
        Some((head, _)) => head.to_string(),
        None => name.to_string(),
    }
}

/// Every test pytest says it ran, from its own lines, in both shapes
/// it prints them: the `-rA` summary (`PASSED tests/test_toy.py::x`,
/// `FAILED tests/test_toy.py::y - assert …`) and the `-v` progress
/// line (`tests/test_toy.py::x PASSED [ 33%]`). The file, the node
/// after the first `::` (classes and all), and the verdict word --
/// every distinct verdict word of a node, since a node may get two.
pub fn ran(said: &str) -> Vec<(String, String, String)> {
    const VERDICTS: [&str; 6] = ["PASSED", "FAILED", "ERROR", "SKIPPED", "XFAIL", "XPASS"];
    let mut out: Vec<(String, String, String)> = Vec::new();
    // One node is spoken of twice by shape (the progress line and
    // the -rA summary) and may be spoken of twice by VERDICT: `PASSED`
    // for the body and `ERROR` for its teardown. The first roll kept
    // the first word only, and a test whose teardown broke came out
    // green in the battery while the gate saw pytest leave with 1
    // (global review 2026-09-06, bugs cut R-3). Every distinct word
    // is kept, and the battery folds them -- red wins.
    let mut seen: std::collections::BTreeSet<(String, String, String)> =
        std::collections::BTreeSet::new();
    let mut take = |file: &str, name: &str, verdict: &str| {
        if !file.ends_with(".py") || name.is_empty() {
            return;
        }
        if seen.insert((file.to_string(), name.to_string(), verdict.to_string())) {
            out.push((file.to_string(), name.to_string(), verdict.to_string()));
        }
    };
    // A node is read as a WHOLE: a directory with a space in it cut
    // the node in two words and the battery lost the test (global
    // review 2026-09-06 R-17; wave 0051).
    for line in said.lines() {
        let trimmed = line.trim();
        // The summary shape: the verdict first, then the node, then
        // maybe ` - <message>`. A SKIPPED line carries `file:line`
        // and no node, and is left out here by its shape.
        if let Some((first, rest)) = trimmed.split_once(' ')
            && VERDICTS.contains(&first)
        {
            let node = rest.split(" - ").next().unwrap_or(rest).trim();
            if let Some((file, name)) = node.split_once("::") {
                take(file, name, first);
            }
            continue;
        }
        // The progress shape: the node, then the verdict, then the
        // percentage -- the verdict is the LAST verdict word in it.
        let Some((cut, verdict)) = VERDICTS
            .iter()
            .filter_map(|v| {
                trimmed
                    .rfind(&format!(" {v}"))
                    .filter(|&at| {
                        let after = &trimmed[at + 1 + v.len()..];
                        after.is_empty() || after.starts_with(' ')
                    })
                    .map(|at| (at, *v))
            })
            .max()
        else {
            continue;
        };
        let node = trimmed[..cut].trim();
        if let Some((file, name)) = node.split_once("::") {
            take(file, name, verdict);
        }
    }
    out
}

/// What a run came to, read from how pytest left -- and, where a
/// code carries more than one meaning, from what it said. Five
/// codes, measured before the plan: 0 green, 1 a test failed, 2
/// collection broke, 4 no such node, 5 nothing collected.
///
/// The plan said the text would never be asked; the measurements
/// after it corrected the plan twice. Code 0 is pytest's answer for
/// a SKIPPED or XFAIL test too -- `1 skipped`, exit 0 -- and a test
/// that did not run is not a green one, so 0 is green only where the
/// summary says something PASSED (review 0045 R-3). And code 4 means
/// three things: no such node; one node asked for in a file whose
/// import broke (the SyntaxError printed above it); and a usage
/// error, where pytest did not start at all (review 0045 R-12).
pub fn classify(said: &str, code: i32) -> crate::adapter::Outcome {
    match code {
        0 if something_passed(said) => crate::adapter::Outcome::Green,
        0 => crate::adapter::Outcome::NotRun,
        1 => crate::adapter::Outcome::Failed,
        2 => crate::adapter::Outcome::BuildBroken(first_error(said)),
        4 if collection_broke(said) || usage_error(said) => {
            crate::adapter::Outcome::BuildBroken(first_error(said))
        }
        4 | 5 => crate::adapter::Outcome::NotRun,
        _ => crate::adapter::Outcome::Failed,
    }
}

/// pytest's closing line names what happened to the tests it ran:
/// `2 passed in 0.01s`, `1 skipped in 0.01s`, `1 xfailed in …`.
fn something_passed(said: &str) -> bool {
    said.lines().any(|line| {
        let trimmed = line.trim_matches(|c: char| c == '=' || c == ' ');
        trimmed.contains(" passed") && trimmed.contains(" in ")
    })
}

/// pytest's traceback lines begin with `E `; one that names an error
/// is a collection that broke, whatever code came after it.
fn collection_broke(said: &str) -> bool {
    said.lines()
        .map(str::trim)
        .any(|line| line.starts_with("E ") && line.contains("Error"))
}

/// `pytest: error: unrecognized arguments: …` -- pytest did not start,
/// and code 4 is what it leaves with then too.
fn usage_error(said: &str) -> bool {
    said.lines()
        .any(|line| line.trim().starts_with("pytest: error:"))
}

/// Python's own words for what broke, in this order: the `E   …Error`
/// line pytest prints under a traceback, with the file and line above
/// it when they are there; the `pytest: error:` line of a usage
/// error; and, where neither stands, the `ERROR collecting <file>`
/// banner stripped of its underscores together with the line that
/// follows it -- which is where pytest puts words like `import file
/// mismatch:` (review 0045 R-11).
fn first_error(said: &str) -> String {
    let lines: Vec<&str> = said.lines().map(str::trim).collect();
    if let Some(at) = lines
        .iter()
        .position(|line| line.starts_with("E ") && line.contains("Error"))
    {
        let words = lines[at].trim_start_matches('E').trim();
        let place = lines[..at]
            .iter()
            .rev()
            .find(|line| line.starts_with("E ") && line.contains("File \""))
            .map(|line| line.trim_start_matches('E').trim());
        return match place {
            Some(place) => format!("{words} ({place})"),
            None => words.to_string(),
        };
    }
    if let Some(usage) = lines.iter().find(|line| line.starts_with("pytest: error:")) {
        return usage.to_string();
    }
    if let Some(at) = lines
        .iter()
        .position(|line| line.contains("ERROR collecting"))
    {
        let banner = lines[at].trim_matches(|c: char| c == '_' || c == ' ');
        let next = lines[at + 1..]
            .iter()
            .find(|line| !line.is_empty() && !line.starts_with('_') && !line.starts_with('='));
        return match next {
            Some(words) => format!("{banner}: {words}"),
            None => banner.to_string(),
        };
    }
    lines
        .iter()
        .find(|line| line.contains("Interrupted"))
        .map(|line| line.to_string())
        .unwrap_or_else(|| "pytest could not collect this project".to_string())
}

/// pytest as a command of the system, in the project's own world --
/// and told to leave that world as it found it: no cache directory,
/// no bytecode. Measured: with both, `find` after a run comes back
/// empty; without them, pytest writes `.pytest_cache` and
/// `__pycache__` into the project it was asked only to judge.
fn pytest(root: &Path, args: &[String]) -> Result<(String, i32), Refusal> {
    let mut command = Command::new("pytest");
    command
        .args(["-p", "no:cacheprovider", "-rA"])
        .args(args)
        .env("PYTHONDONTWRITEBYTECODE", "1")
        // The environment's own options never reach the run: a
        // `--deselect` in PYTEST_ADDOPTS took the tagged test out of
        // the battery and the wave closed over its red (global review
        // 2026-09-06, bugs cut R-6). The project's own addopts in its
        // config are read as before -- they are the project's word.
        .env_remove("PYTEST_ADDOPTS")
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
