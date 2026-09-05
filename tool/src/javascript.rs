//! The javascript adapter (contract tool-adapter-javascript): the one
//! place that knows how a node project keeps its tests and its
//! modules. It runs `node --test` as a command of the system, exactly
//! as a person would in a terminal, and writes nothing anywhere.
//! TypeScript rides the same runner -- node 22 strips types itself --
//! so a `.test.ts` file is judged by this hand too.
//!
//! `node --test`, and said so aloud: jest, vitest and mocha are other
//! runners and other waves.
//!
//! This is the first tongue whose runner does NOT tell its states
//! apart by exit code -- a failed test and a SyntaxError both leave
//! with 1, and a name that matched nothing leaves with 0 while the
//! FILE is counted as one passed test. So every verdict here is read
//! from TAP, and the exit code is never asked.

use crate::docs::Refusal;
use crate::i18n::{t, ta};
use crate::tags::TestTag;
use crate::targs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The names a test file may carry for this hand: `*.test.js` and its
/// module and typed cousins. node collects far wider (`*-test.*`,
/// `*_test.*`, `test-*.*`, anything under `test/`); this reads the
/// one name it can vouch for and says so.
const TEST_SUFFIXES: [&str; 5] = [
    ".test.js",
    ".test.mjs",
    ".test.cjs",
    ".test.ts",
    ".test.mts",
];

fn is_test_file(name: &str) -> bool {
    TEST_SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
}

fn is_source(name: &str) -> bool {
    [".js", ".mjs", ".cjs", ".ts", ".mts"]
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

/// `test/**` and `tests/**`, files named `*.test.{js,mjs,cjs,ts,mts}`
/// -- where the proves tags live. A project without either directory
/// has none, and that is not a refusal.
pub fn test_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let mut out: Vec<PathBuf> = Vec::new();
    for dir in ["test", "tests"] {
        let dir = root.join(dir);
        if !dir.is_dir() {
            continue;
        }
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
                    if path.file_name().is_some_and(|n| n == "node_modules") {
                        continue;
                    }
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
    }
    out.sort();
    Ok(out)
}

/// The source files under `test/` and `tests/` this hand does NOT
/// read -- helpers, fixtures, a `setup.js` -- named aloud, as every
/// other hand names its own.
pub fn unread_files(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for dir in ["test", "tests"] {
        let mut stack = vec![root.join(dir)];
        while let Some(here) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&here) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if path.file_name().is_some_and(|n| n == "node_modules") {
                        continue;
                    }
                    stack.push(path);
                } else if path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| is_source(n) && !is_test_file(n))
                {
                    out.push(path);
                }
            }
        }
    }
    out.sort();
    out
}

/// Where a module's source may live: `toy.bar` is `src/toy/bar.ts`,
/// `src/toy/bar.js`, `src/toy/bar/index.*`, or the same at the root;
/// typed before plain, a file before its `index`. Dots make
/// directories, as they do for python -- a `/` in a name is refused
/// by the form court as pointing outside, and that stays.
pub fn module_paths(root: &Path, module: &str) -> Vec<PathBuf> {
    let segments: Vec<&str> = module.split('.').collect();
    if segments
        .iter()
        .any(|part| part.is_empty() || part.contains('/') || part.contains('\\'))
    {
        return Vec::new();
    }
    let joined = segments.join("/");
    let mut out = Vec::with_capacity(12);
    for base in [root.join("src"), root.to_path_buf()] {
        for ext in ["ts", "js", "mjs", "cjs", "mts"] {
            out.push(base.join(format!("{joined}.{ext}")));
        }
        for ext in ["ts", "js"] {
            out.push(base.join(&joined).join(format!("index.{ext}")));
        }
    }
    out
}

/// Runs exactly the tagged test: the file, and the name escaped into
/// a pattern that matches that name and nothing else. The name goes
/// into a REGULAR EXPRESSION, so `a.b (x)` becomes `^a\.b \(x\)$`;
/// unescaped, `a.b` would select `axb` too and a `(` would break the
/// runner (wave 0044's lesson about a string handed to another
/// program).
pub fn run_test(root: &Path, tag: &TestTag) -> Result<crate::adapter::Outcome, Refusal> {
    let relative = tag.file.strip_prefix(root).unwrap_or(&tag.file);
    let said = node(
        root,
        &[
            format!("--test-name-pattern=^{}$", escape_regex(&tag.test)),
            relative.display().to_string(),
        ],
    )?;
    Ok(classify(&said, &tag.test))
}

/// Every regex metacharacter, backslashed.
pub fn escape_regex(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 8);
    for ch in name.chars() {
        if matches!(
            ch,
            '\\' | '^'
                | '$'
                | '.'
                | '|'
                | '?'
                | '*'
                | '+'
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '/'
        ) {
            out.push('\\');
        }
        out.push(ch);
    }
    out
}

/// The whole battery, verdicts per test, **in node's own words**: one
/// run over the test files this hand reads, and the roll and the
/// verdicts alike from TAP. A test inside `describe` is named by its
/// bare name, as node names it; the describe's own line is a suite
/// and not in the map; a `# SKIP` or `# TODO` did not run and is not
/// in the map either -- not green, not a red that would hold a wave
/// open (§7.12).
///
/// One run per test file, so the file a verdict belongs to is known:
/// node's TAP names the test and not its file.
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    let mut out: BTreeMap<(String, String), bool> = BTreeMap::new();
    for file in test_files(root)? {
        let relative = file.strip_prefix(root).unwrap_or(&file);
        let said = node(root, &[relative.display().to_string()])?;
        let shown = relative.display().to_string();
        if let Some(words) = broken(&said, &shown) {
            return Err(Refusal {
                file: file.clone(),
                reason: ta("adapter-javascript-broken", targs!("error" => words)),
                instead: t("adapter-javascript-broken-instead"),
            });
        }
        let key = crate::adapter::battery_key(root, &file);
        for entry in tap(&said) {
            if entry.suite || entry.skipped || entry.name == shown {
                continue;
            }
            out.entry((key.clone(), entry.name))
                .and_modify(|was| *was = *was && entry.ok)
                .or_insert(entry.ok);
        }
    }
    Ok(out)
}

/// One `ok` / `not ok` line of TAP and what its YAML block said of it.
pub struct Entry {
    pub name: String,
    pub ok: bool,
    pub suite: bool,
    pub skipped: bool,
}

/// node's TAP, read as node writes it: `ok N - <name>` or `not ok N -
/// <name>`, indented four spaces per level of nesting, each followed
/// by a YAML block between `---` and `...` that says `type: 'test'`
/// or `type: 'suite'`. A `# SKIP` or `# TODO` after the name is a
/// directive, and the name stops before it.
pub fn tap(said: &str) -> Vec<Entry> {
    let lines: Vec<&str> = said.lines().collect();
    let mut out: Vec<Entry> = Vec::new();
    let mut at = 0usize;
    while at < lines.len() {
        let trimmed = lines[at].trim_start();
        let (ok, rest) = if let Some(rest) = trimmed.strip_prefix("not ok ") {
            (false, rest)
        } else if let Some(rest) = trimmed.strip_prefix("ok ") {
            (true, rest)
        } else {
            at += 1;
            continue;
        };
        // `N - name`, the number first.
        let Some((number, named)) = rest.split_once(" - ") else {
            at += 1;
            continue;
        };
        if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
            at += 1;
            continue;
        }
        let (name, skipped) = match named.find(" # ") {
            Some(cut) => (named[..cut].trim_end(), true),
            None => (named.trim_end(), false),
        };
        // The YAML block below it, if node wrote one.
        let mut suite = false;
        let mut look = at + 1;
        if lines.get(look).is_some_and(|l| l.trim() == "---") {
            look += 1;
            while let Some(line) = lines.get(look) {
                let body = line.trim();
                if body == "..." {
                    break;
                }
                if let Some(kind) = body.strip_prefix("type:") {
                    suite = kind.trim().trim_matches('\'') == "suite";
                }
                look += 1;
            }
        }
        out.push(Entry {
            name: name.to_string(),
            ok,
            suite,
            skipped,
        });
        at += 1;
    }
    out
}

/// What one run came to, read from TAP and from TAP alone: the line
/// that names THIS test decides -- `ok` green, `not ok` red. No line
/// naming it: a broken build if node printed an error for the file
/// (`# SyntaxError: …` above a `not ok` for the file itself), else
/// "did not run" -- whatever the exit code, which is 0 here while
/// node counts the file as one passed test.
pub fn classify(said: &str, test: &str) -> crate::adapter::Outcome {
    for entry in tap(said) {
        if entry.suite || entry.name != test {
            continue;
        }
        if entry.skipped {
            return crate::adapter::Outcome::NotRun;
        }
        return if entry.ok {
            crate::adapter::Outcome::Green
        } else {
            crate::adapter::Outcome::Failed
        };
    }
    match broken(said, "") {
        Some(words) => crate::adapter::Outcome::BuildBroken(words),
        None => crate::adapter::Outcome::NotRun,
    }
}

/// node's own words when a file could not even be loaded: the `#
/// SyntaxError: …` (or any `# …Error: …`) line it prints before the
/// file-level `not ok`, with the `# file:///…:LINE` above it where
/// there is one. `None` when the run loaded its files -- a failed
/// test is not a broken build.
fn broken(said: &str, file: &str) -> Option<String> {
    let lines: Vec<&str> = said.lines().map(str::trim).collect();
    let file_failed = lines.iter().any(|line| {
        line.starts_with("not ok ")
            && (file.is_empty() || line.ends_with(file) || line.contains(&format!("- {file}")))
            && (line.ends_with(".js")
                || line.ends_with(".ts")
                || line.ends_with(".mjs")
                || line.ends_with(".cjs")
                || line.ends_with(".mts"))
    });
    if !file_failed {
        return None;
    }
    let error = lines.iter().find(|line| {
        line.starts_with("# ")
            && line.contains("Error")
            && line.contains(':')
            && !line.contains(" at ")
    })?;
    let words = error.trim_start_matches('#').trim();
    let place = lines
        .iter()
        .find(|line| line.starts_with("# file://") || line.starts_with("# /"))
        .map(|line| line.trim_start_matches('#').trim());
    Some(match place {
        Some(place) => format!("{words} ({place})"),
        None => words.to_string(),
    })
}

/// node as a command of the system, in the project's own world, with
/// the TAP reporter so what it says is what this hand reads.
fn node(root: &Path, args: &[String]) -> Result<String, Refusal> {
    let mut command = Command::new("node");
    command
        .args(["--test", "--test-reporter=tap"])
        .args(args)
        .current_dir(root);
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: ta(
            "adapter-javascript-failed",
            targs!("error" => e.to_string()),
        ),
        instead: t("adapter-javascript-failed-instead"),
    })?;
    Ok(format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    ))
}
