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
/// constant.
fn snake_case(word: &str) -> String {
    let mut out = String::with_capacity(word.len() + 4);
    for (at, ch) in word.char_indices() {
        if ch.is_uppercase() && at > 0 {
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
        // Every test the file declares, green until minitest names it
        // among the failures. Minitest writes `ToyTest#test_it_falls`
        // for each one it did not like.
        let text = std::fs::read_to_string(&file).map_err(|e| Refusal {
            file: file.clone(),
            reason: ta("docs-unreadable", targs!("error" => e.to_string())),
            instead: t("docs-unreadable-instead"),
        })?;
        for line in text.lines() {
            let trimmed = line.trim();
            let Some(rest) = trimmed.strip_prefix("def ") else {
                continue;
            };
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.starts_with("test") {
                continue;
            }
            let failed = said
                .lines()
                .any(|line| line.contains(&format!("#{name}")) && !line.trim().starts_with("def"));
            out.insert((stem.clone(), name), !failed);
        }
    }
    Ok(out)
}

/// What a run came to, read from what ruby said. Public so the probe
/// of this wave can play the shapes without a project on disk.
pub fn classify(said: &str, success: bool) -> crate::adapter::Outcome {
    if success {
        return crate::adapter::Outcome::Green;
    }
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
    crate::adapter::Outcome::Failed
}
