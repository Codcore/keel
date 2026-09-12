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
/// Whether a path of the tree, relative to the root, is one
/// `test_files` would read: under `test/`, named `*_test.exs`. The
/// §7.15 court asks this of a tree that is not on disk (wave 0050).
pub fn is_test_path(rel: &str) -> bool {
    rel.starts_with("test/") && rel.ends_with("_test.exs")
}

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
    // By the file and the LINE of the declaration, never by the name:
    // `mix test --only 'test:test ünïcode holds'` excludes everything
    // (measured, global review 2026-09-06 R-11), while `mix test
    // test/x_test.exs:LINE` runs the test declared at that line -- the
    // very line the tag reader saw. The name never enters the command.
    let relative = tag.file.strip_prefix(root).unwrap_or(&tag.file);
    let out = mix(root, &[format!("{}:{}", relative.display(), tag.line)])?;
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
        let stem = crate::adapter::battery_key(root, &file);
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
///
/// A test mix did NOT run is named too, and taken out: `@tag :skip`
/// prints the start line `* test it works [L#6]` and then `* test it
/// works (skipped) [L#6]`, and the first reading counted both -- the
/// skip as a run and the `(skipped)` as a third, phantom test, both
/// green, and `keel close` closed the wave over them (final review
/// 2026-09-06, bugs R-1, R-23; wave 0055). A skipped test is in the
/// battery neither as green nor as red.
pub fn ran(said: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut skipped: Vec<String> = Vec::new();
    for line in said.split(['\n', '\r']) {
        let Some(rest) = line.trim().strip_prefix("* ") else {
            continue;
        };
        // `[L#N]` closes a trace line and nothing else.
        let Some(head) = rest.rsplit_once(" [L#") else {
            continue;
        };
        // `(skipped)` and `(excluded)`: ExUnit prints a state where
        // a duration would stand. A test excluded BY A TAG --
        // `ExUnit.start(exclude: [:integration])` and `@tag
        // :integration`, an everyday shape -- printed the same pair
        // of lines as a skip, and the first cut of this wave knew
        // only the word `skipped`: the start line counted as a run,
        // the state line as a third, phantom test, and `keel close`
        // closed a wave over a test that FALLS while the gate called
        // it red (review 0055 R-2).
        if let Some(named) = head
            .0
            .trim_end()
            .strip_suffix("(skipped)")
            .or_else(|| head.0.trim_end().strip_suffix("(excluded)"))
        {
            if let Some(name) = without_kind(named) {
                skipped.push(name);
            }
            continue;
        }
        let named = strip_timing(head.0);
        let Some(name) = without_kind(named) else {
            continue;
        };
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out.retain(|name| !skipped.contains(name));
    out
}

/// Whether ExUnit's summary line says no test ran: `N tests, F
/// failures[, X excluded][, S skipped]` -- doctests and properties
/// counted with the tests -- with nothing left once the excluded and
/// the skipped are taken off. `mix test file:line` over a `@tag
/// :skip` test leaves with 0 and says `2 tests, 0 failures, 1
/// excluded, 1 skipped` (measured; wave 0055).
fn nothing_ran(said: &str) -> bool {
    said.split(['\n', '\r']).any(|line| {
        let mut counted = 0usize;
        let mut off = 0usize;
        let mut tests = false;
        let mut failures = false;
        for part in line.trim().split(", ") {
            let Some((n, word)) = part.split_once(' ') else {
                return false;
            };
            let Ok(n) = n.parse::<usize>() else {
                return false;
            };
            match word.trim_end() {
                "test" | "tests" => {
                    tests = true;
                    counted += n;
                }
                "doctest" | "doctests" | "property" | "properties" => counted += n,
                "failure" | "failures" => failures = true,
                "excluded" | "skipped" => off += n,
                _ => {}
            }
        }
        tests && failures && counted.saturating_sub(off) == 0
    })
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
        // A line that selects nothing leaves with 0 and says so
        // (measured: "All tests have been excluded."): not green.
        0 if said.contains("All tests have been excluded") => crate::adapter::Outcome::NotRun,
        // The one test the line named was skipped: mix leaves with 0
        // and its summary says nothing ran (wave 0055).
        0 if nothing_ran(said) => crate::adapter::Outcome::NotRun,
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
    // The hand says what encoding it wants read, instead of taking
    // whatever the environment happens to say. Measured in the field
    // (release 1.2.0, an older OTP) and again on OTP 29: with the
    // locale stripped the old runner printed latin1, and with
    // `+pc latin1` the new one prints `\x{456}\x{43C}…` -- either
    // way the name no longer matches the one in the test file, the
    // battery's key misses, and the closing court says the battery
    // did not run a test it ran and passed.
    //
    // Appended, not substituted: whatever a person or a CI put there
    // stays. Ours comes last and therefore wins any `+pc` already
    // present -- a runner that cannot print its own test names is not
    // a choice worth honouring, and the wave says so aloud rather
    // than leaving the verdict to the environment.
    let mut flags = std::env::var("ERL_FLAGS").unwrap_or_default();
    if !flags.trim().is_empty() {
        flags.push(' ');
    }
    flags.push_str("+pc unicode");
    command.env("ERL_FLAGS", flags);
    let out = command.output().map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: ta("adapter-elixir-failed", targs!("error" => e.to_string())),
        instead: t("adapter-elixir-failed-instead"),
    })?;
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // And where the runner would not be told -- an explicit
    // `+pc latin1` beats anything this hand appends, measured -- the
    // tool says SO, instead of reading an escape as a name and
    // accusing the battery of not running a test it ran and passed.
    //
    // Two shapes, one wound: a replacement character means the bytes
    // were not UTF-8 at all (an older OTP with no locale, the field
    // report of release 1.2.0), and `\x{...}` means this OTP escaped
    // what it would not print (`+pc latin1`). Neither is a name.
    if said.contains('\u{FFFD}') || said.contains("\\x{") {
        return Err(Refusal {
            file: root.to_path_buf(),
            reason: t("adapter-elixir-not-utf8"),
            instead: t("adapter-elixir-not-utf8-instead"),
        });
    }
    Ok((said, out.status.code().unwrap_or(-1)))
}
