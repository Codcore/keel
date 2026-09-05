//! The elixir adapter (contract tool-adapter-elixir): the one place
//! that knows how a mix project keeps its tests and its modules. It
//! runs `mix` as a command of the system, exactly as a person would
//! in a terminal, and writes nothing anywhere.
//!
//! ExUnit, and said so aloud: this reads `mix test`, which is what a
//! mix project has. Other runners are other waves.

use crate::docs::Refusal;
use crate::i18n::{t, ta};
use crate::tags::TestTag;
use crate::targs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The directory mix builds into.
pub const BUILD_DIR: &str = "_build";

/// `test/**/*_test.exs` -- where the proves tags live. A project
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
                .is_some_and(|n| n.ends_with("_test.exs"))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Where a module's source lives: `Toy.Bar` is `lib/toy/bar.ex`, and
/// the bare `Toy` is `lib/toy.ex`. The layout mix itself generates.
pub fn module_paths(root: &Path, module: &str) -> Vec<PathBuf> {
    let joined = module
        .split('.')
        .map(snake_case)
        .collect::<Vec<_>>()
        .join("/");
    vec![
        root.join("lib").join(format!("{joined}.ex")),
        root.join("lib").join(&joined).join("init.ex"),
    ]
}

/// `SomeName` -> `some_name`. An acronym stays one word, as it does
/// in ruby and for the same reason (wave 0038 review R-15).
fn snake_case(word: &str) -> String {
    let chars: Vec<char> = word.chars().collect();
    let mut out = String::with_capacity(word.len() + 4);
    for (at, ch) in chars.iter().enumerate() {
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

/// Runs exactly the tagged test.
///
/// An ExUnit test is named by a string, so it is selected by that
/// string: `mix test --only 'test:test <name>'`. And this tongue,
/// unlike ruby, **tells a failure from a broken build by its exit
/// code**: 0 green, 2 failed, 1 did not compile. Sec. 7.12 was
/// written for exactly this case, and here the "where it can" branch
/// is the one that runs.
pub fn run_test(root: &Path, tag: &TestTag) -> Result<crate::adapter::Outcome, Refusal> {
    let out = mix(
        root,
        &["--only".to_string(), format!("test:test {}", tag.test)],
    )?;
    Ok(classify(&out.0, out.1))
}

/// The whole battery in one mix run, verdicts per test. The key is
/// (test file stem, test name), as the courts above expect.
///
/// `--trace` is what makes the verdicts readable one by one: it
/// prints `* test <name> (1.1ms) [L#5]` for each test it ran, and
/// lists every failure as `N) test <name> (<Module>)`.
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    let (said, code) = mix(root, &["--trace".to_string()])?;
    // A build that does not build is a refusal aloud: without a build
    // there is no verdict for anyone (the cargo hand's law, and here
    // the exit code says it outright).
    if code == 1 && !said.contains("no test was executed") {
        return Err(Refusal {
            file: root.to_path_buf(),
            reason: ta(
                "adapter-elixir-broken",
                targs!("error" => first_error(&said)),
            ),
            instead: t("adapter-elixir-broken-instead"),
        });
    }
    let fallen = failures(&said);
    let mut out: BTreeMap<(String, String), bool> = BTreeMap::new();
    for file in test_files(root)? {
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        for name in names_in(&std::fs::read_to_string(&file).map_err(|e| Refusal {
            file: file.clone(),
            reason: ta("docs-unreadable", targs!("error" => e.to_string())),
            instead: t("docs-unreadable-instead"),
        })?) {
            let green = !fallen.contains(&name);
            out.insert((stem.clone(), name), green);
        }
    }
    Ok(out)
}

/// The names ExUnit reported as failures: `  1) test it falls (ToyTest)`.
fn failures(said: &str) -> Vec<String> {
    said.lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            let rest = trimmed.split_once(") test ")?.1;
            // A digit before the parenthesis is what makes it a
            // failure report rather than prose.
            if !trimmed
                .split(')')
                .next()
                .is_some_and(|head| !head.is_empty() && head.chars().all(|c| c.is_ascii_digit()))
            {
                return None;
            }
            let name = rest.rsplit_once(" (")?.0;
            Some(name.trim().to_string())
        })
        .collect()
}

/// Every test a file declares, by the FULL name ExUnit gives it --
/// a `describe` block's own name in front, because that is what
/// `--trace` prints and what `--only` selects (measured, wave 0042).
pub fn names_in(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut describing: Option<(String, usize)> = None;
    let mut depth: usize = 0;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(group) = crate::tags::describe_name(trimmed) {
            describing = Some((group, depth));
        }
        if trimmed.starts_with("end") {
            if let Some((_, opened)) = &describing
                && depth == *opened
            {
                describing = None;
            }
            depth = depth.saturating_sub(1);
        } else if trimmed.ends_with(" do") || trimmed == "do" {
            depth += 1;
        }
        if let Some(name) = crate::tags::test_name(trimmed) {
            out.push(match &describing {
                Some((group, _)) => format!("{group} {name}"),
                None => name,
            });
        }
    }
    out
}

/// What a run came to, read from how mix left and what it said.
///
/// The order matters here too, but for the opposite reason to ruby's:
/// the codes are distinct, so they answer first, and the text is only
/// asked where a code carries two meanings -- 1 is both "did not
/// compile" and "--only matched nothing".
pub fn classify(said: &str, code: i32) -> crate::adapter::Outcome {
    match code {
        0 => crate::adapter::Outcome::Green,
        2 => crate::adapter::Outcome::Failed,
        1 if said.contains("no test was executed") => crate::adapter::Outcome::NotRun,
        1 => crate::adapter::Outcome::BuildBroken(first_error(said)),
        _ => crate::adapter::Outcome::Failed,
    }
}

fn first_error(said: &str) -> String {
    said.lines()
        .find(|line| line.contains("Compilation error") || line.starts_with("** ("))
        .unwrap_or("mix could not build this project")
        .trim()
        .to_string()
}

/// mix as a command of the system, in the project's own world.
fn mix(root: &Path, args: &[String]) -> Result<(String, i32), Refusal> {
    let mut command = Command::new("mix");
    command.arg("test").args(args).current_dir(root);
    // The judged project must not inherit the hook's repository, the
    // same law the cargo hand keeps (review 0021 R-3).
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: ta("adapter-elixir-failed", targs!("error" => e.to_string())),
        instead: t("adapter-elixir-failed-instead"),
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
