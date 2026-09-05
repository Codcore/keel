//! The ruby adapter (contract tool-adapter-ruby): the one place that
//! knows how a ruby project keeps its tests and its modules. It runs
//! `ruby` as a command of the system, exactly as a person would in a
//! terminal, and writes nothing anywhere.
//!
//! Minitest, and said so aloud: RSpec keeps its examples in `spec/`
//! and names them by string rather than by method, which is a second
//! reading and a second wave. Promising it here silently would be the
//! quiet narrowing §9.9 asks a reviewer to hunt.

use crate::docs::Refusal;
use crate::i18n::{t, ta};
use crate::tags::TestTag;
use crate::targs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `test/**/*_test.rb` -- where the proves tags live. A project
/// without a test directory has none, and that is not a refusal.
pub fn test_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let dir = root.join("test");
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
                .is_some_and(|n| n.ends_with("_test.rb"))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// The `.rb` files in `test/` this adapter does NOT read, because
/// minitest's own convention is `*_test.rb`. A border said aloud by
/// the tool rather than only by the README (review 0038 R-19): an
/// RSpec `_spec.rb` sitting here is skipped, and a skip nobody names
/// reads as "there was nothing to read".
pub fn unread_files(root: &Path) -> Vec<PathBuf> {
    let dir = root.join("test");
    let mut out: Vec<PathBuf> = Vec::new();
    let mut stack = vec![dir];
    while let Some(here) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&here) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rb")
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with("_test.rb"))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Where a module's source lives: `Toy::Bar` is `lib/toy/bar.rb`, and
/// the bare `Toy` is `lib/toy.rb`. Both layouts ruby itself uses.
pub fn module_paths(root: &Path, module: &str) -> Vec<PathBuf> {
    let parts: Vec<String> = module.split("::").map(snake_case).collect();
    let joined = parts.join("/");
    vec![
        root.join("lib").join(format!("{joined}.rb")),
        root.join("lib").join(&joined).join("init.rb"),
        root.join("app").join(format!("{joined}.rb")),
    ]
}

/// `SomeName` -> `some_name`, which is how ruby names the file of a
/// constant. An acronym stays one word -- `HTTPServer` is
/// `http_server` and not `h_t_t_p_server` (review 0038 R-15), which
/// is the rule ruby's own autoloaders read.
fn snake_case(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let mut out = String::with_capacity(word.len() + 4);
    for (at, ch) in chars.iter().enumerate() {
        // A break before an upper-case letter belongs where the word
        // itself breaks: after a lower-case letter or a digit, or at
        // the last capital of a run that a lower-case letter follows.
        let after_lower = at > 0 && (chars[at - 1].is_lowercase() || chars[at - 1].is_numeric());
        let end_of_run = at > 0
            && chars[at - 1].is_uppercase()
            && chars.get(at + 1).is_some_and(|next| next.is_lowercase());
        if ch.is_uppercase() && (after_lower || end_of_run) {
            out.push('_');
        }
        out.extend(ch.to_lowercase());
    }
    out
}

/// Runs exactly the tagged test (`ruby -Itest <file> -n <method>`).
///
/// The honest limit of this tongue, and §7.12 foresaw it: **ruby does
/// not tell "failed" from "did not load" by its exit code** -- both
/// are 1. What can be told apart is the text: a `SyntaxError` or a
/// `LoadError` before any test runs is a broken build, and everything
/// else that exits non-zero is a failure. Where the text does not
/// say, the failure is taken as a failure, which is the direction
/// that cannot turn red into green.
pub fn run_test(root: &Path, tag: &TestTag) -> Result<crate::adapter::Outcome, Refusal> {
    let relative = tag.file.strip_prefix(root).unwrap_or(&tag.file);
    let out = Command::new("ruby")
        .arg("-Itest")
        .arg(relative)
        .arg("-n")
        .arg(&tag.test)
        .current_dir(root)
        .output()
        .map_err(|e| Refusal {
            file: root.to_path_buf(),
            reason: ta("adapter-ruby-failed", targs!("error" => e.to_string())),
            instead: t("adapter-ruby-failed-instead"),
        })?;
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    Ok(classify(&said, out.status.success()))
}

/// The whole battery in one ruby run per file, verdicts per test.
/// The key is (file stem, test name), as the courts above expect.
///
/// The verdicts are minitest's own, asked for by `-v`: one line per
/// test it actually ran, `Class#method = <time> s = <mark>`. Reading
/// the file's text instead and calling every `def test...` green
/// until named among the failures was three lies at once (review
/// 0038 R-1, R-2, R-6, R-20): a file that did not parse came out all
/// green, a `def testify` in a plain class swelled the count, and a
/// test whose name merely begins another was dragged red with it.
/// A file that ran nothing at all is not a green file -- it is the
/// adapter's refusal aloud, the same answer cargo's hand gives over
/// "could not compile".
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    let mut out: BTreeMap<(String, String), bool> = BTreeMap::new();
    for file in test_files(root)? {
        let relative = file.strip_prefix(root).unwrap_or(&file);
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let run = Command::new("ruby")
            .arg("-Itest")
            .arg(relative)
            .arg("-v")
            .current_dir(root)
            .output()
            .map_err(|e| Refusal {
                file: root.to_path_buf(),
                reason: ta("adapter-ruby-failed", targs!("error" => e.to_string())),
                instead: t("adapter-ruby-failed-instead"),
            })?;
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&run.stdout),
            String::from_utf8_lossy(&run.stderr)
        );
        let mut ran = 0usize;
        for line in said.lines() {
            let Some((name, green)) = verdict_in(line) else {
                continue;
            };
            ran += 1;
            // Two classes in one file may name a method alike. The
            // safe direction joins them: green only if both were.
            let key = (stem.clone(), name);
            let held = out.get(&key).copied().unwrap_or(true);
            out.insert(key, held && green);
        }
        if ran == 0 {
            // Nothing ran. Either the file declares no test at all --
            // which is not a fault -- or it did not load, and then
            // there is no verdict for anyone.
            if let crate::adapter::Outcome::BuildBroken(words) =
                classify(&said, run.status.success())
            {
                return Err(Refusal {
                    file: file.clone(),
                    reason: ta("adapter-ruby-broken", targs!("error" => words)),
                    instead: t("adapter-ruby-broken-instead"),
                });
            }
        }
    }
    Ok(out)
}

/// One line of minitest's verbose voice: `ToyTest#test_it_works =
/// 0.00 s = .`, and green is the bare dot -- `F` failed, `E` errored,
/// `S` was skipped, and none of the three proves a promise. Anything
/// that is not that shape is not a verdict: the failure reports below
/// carry `Class#method` too, and were read as verdicts once.
fn verdict_in(line: &str) -> Option<(String, bool)> {
    let trimmed = line.trim();
    let (head, mark) = trimmed.rsplit_once(" = ")?;
    let (name, timing) = head.rsplit_once(" = ")?;
    if !timing.ends_with(" s") || name.split_whitespace().count() != 1 {
        return None;
    }
    let method = name.rsplit_once('#')?.1;
    if method.is_empty() {
        return None;
    }
    Some((method.to_string(), mark.trim() == "."))
}

/// What a run came to, read from what ruby said.
///
/// The order of the questions is the whole of it (review 0038 R-2):
/// minitest leaves with **0** when `-n` names a method it does not
/// know, so asking the exit code first turned "nothing ran" into
/// "green" and let work through a gate over a test that never ran.
/// What ruby said is asked before how it left.
pub fn classify(said: &str, success: bool) -> crate::adapter::Outcome {
    // Nothing ran: the file did not parse, or a require failed.
    if said.contains("SyntaxError")
        || said.contains("LoadError")
        || said.contains("cannot load such file")
    {
        return crate::adapter::Outcome::BuildBroken(
            said.lines()
                .find(|line| {
                    line.contains("SyntaxError")
                        || line.contains("LoadError")
                        || line.contains("cannot load such file")
                })
                .unwrap_or("")
                .trim()
                .to_string(),
        );
    }
    // Minitest ran and named nothing: the method does not exist.
    if said.contains("0 runs") {
        return crate::adapter::Outcome::NotRun;
    }
    if success {
        return crate::adapter::Outcome::Green;
    }
    crate::adapter::Outcome::Failed
}
