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

/// The `.exs` files in `test/` this adapter does NOT read, because
/// ExUnit's convention is `*_test.exs`. Named aloud, exactly as the
/// ruby hand names its own (review 0042 R-6: `test/support/` is a
/// standard ExUnit layout, and a tag left there was skipped in
/// silence while ruby said so in the same case).
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
            } else if path.extension().is_some_and(|e| e == "exs")
                && !path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with("_test.exs") || n == "test_helper.exs")
            {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// Where a module's source lives: `Toy.Bar` is `lib/toy/bar.ex`, and
/// the bare `Toy` is `lib/toy.ex`. The layout mix itself generates.
pub fn module_paths(root: &Path, module: &str) -> Vec<PathBuf> {
    let joined = module
        .split('.')
        .map(snake_case)
        .collect::<Vec<_>>()
        .join("/");
    // One path, because elixir has one convention. `lib/<x>/init.ex`
    // stood here as a calque from the ruby hand and appears in no
    // elixir project (review 0042 R-14).
    vec![root.join("lib").join(format!("{joined}.ex"))]
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

/// The whole battery, verdicts per test, **in mix's own words**.
///
/// One run per test file, so the file a verdict belongs to is known
/// -- the courts above key on it -- and `--trace` names every test
/// that really ran. Review 0042 R-2: this used to read only the
/// failures from mix and rebuild the LIST of tests by re-parsing the
/// source, so anything the reader could not name did not exist for
/// the court and its failure vanished with it. Four independent
/// proofs of a false green followed, one of them on the file `mix
/// new` generates itself (a `doctest`, which no source reader of
/// ours was ever going to see). It is the same lesson wave 0038 R-1
/// paid for in ruby, walked back in a new tongue.
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    let mut out: BTreeMap<(String, String), bool> = BTreeMap::new();
    for file in test_files(root)? {
        let relative = file.strip_prefix(root).unwrap_or(&file);
        let stem = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let (said, code) = mix(
            root,
            &["--trace".to_string(), relative.display().to_string()],
        )?;
        if code == 1 && !said.contains("no test was executed") {
            return Err(Refusal {
                file: file.clone(),
                reason: ta(
                    "adapter-elixir-broken",
                    targs!("error" => first_error(&said)),
                ),
                instead: t("adapter-elixir-broken-instead"),
            });
        }
        let fallen = failures(&said);
        for name in ran(&said) {
            let green = !fallen.contains(&name);
            out.insert((stem.clone(), name), green);
        }
    }
    Ok(out)
}

/// Every test mix says it RAN, from the trace's own lines:
/// `  * test <name> (0.00ms) [L#12]`, and `doctest` alike. The start
/// line and the finished line arrive separated by a carriage return,
/// so the text is split on both.
pub fn ran(said: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in said.split(['\n', '\r']) {
        let Some(rest) = line.trim().strip_prefix("* ") else {
            continue;
        };
        // `[L#N]` closes a trace line and nothing else.
        let Some(head) = rest.rsplit_once(" [L#") else {
            continue;
        };
        let named = strip_timing(head.0);
        let Some(name) = without_kind(named) else {
            continue;
        };
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// The names ExUnit reported as failures: `  1) test it falls (ToyTest)`.
fn failures(said: &str) -> Vec<String> {
    said.split(['\n', '\r'])
        .filter_map(|line| {
            let trimmed = line.trim();
            let (head, rest) = trimmed.split_once(") ")?;
            // A number before the parenthesis is what makes it a
            // failure report rather than prose.
            if head.is_empty() || !head.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            // The module in the last parentheses is not the name.
            let named = rest.rsplit_once(" (")?.0;
            without_kind(named)
        })
        .collect()
}

/// `(0.00ms)` or `(1.2s)` at the end is the trace's timing, not part
/// of a name -- and a doctest's own name ends in `(1)`, so only a
/// group that reads as a duration is taken off.
fn strip_timing(named: &str) -> &str {
    let trimmed = named.trim_end();
    let Some((head, tail)) = trimmed.rsplit_once(" (") else {
        return trimmed;
    };
    let inside = tail.trim_end_matches(')');
    let looks_like_time = inside.ends_with("ms") || inside.ends_with('s');
    if looks_like_time && inside.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        head.trim_end()
    } else {
        trimmed
    }
}

/// `test <name>` and `doctest <name>` are how ExUnit prints and
/// selects them; the courts above hold the bare name, which is what a
/// `proves:` tag carries.
fn without_kind(named: &str) -> Option<String> {
    let named = named.trim();
    for kind in ["test ", "doctest ", "property "] {
        if let Some(rest) = named.strip_prefix(kind) {
            return Some(rest.trim().to_string());
        }
    }
    None
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

/// The compiler's own words, which is what the scenario promises --
/// not the banner above them. `== Compilation error in file … ==`
/// names the file and says nothing about what is wrong; the line
/// after it does (review 0042 R-5), and ruby's hand already did this
/// better.
fn first_error(said: &str) -> String {
    let lines: Vec<&str> = said.split(['\n', '\r']).map(str::trim).collect();
    let diagnosis = lines
        .iter()
        .find(|line| line.starts_with("** ("))
        .or_else(|| lines.iter().find(|line| line.contains("error:")));
    match diagnosis {
        Some(words) => words.to_string(),
        None => lines
            .iter()
            .find(|line| line.contains("Compilation error"))
            .map(|line| line.to_string())
            .unwrap_or_else(|| "mix could not build this project".to_string()),
    }
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
