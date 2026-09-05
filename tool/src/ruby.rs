//! The ruby adapter (contract tool-adapter-ruby): the one place that
//! knows how a ruby project keeps its tests and its modules. It runs
//! `ruby` as a command of the system, exactly as a person would in a
//! terminal, and writes nothing anywhere.
//!
//! Two readings of one tongue (wave 0047): minitest in `test/`,
//! named by method and judged from `-v`'s own lines; rspec in
//! `spec/`, named by rspec's FULL DESCRIPTION, selected by the id a
//! dry run gives an example, and judged from rspec's JSON. The courts
//! above ask one adapter and never learn which reading answered.

use crate::docs::Refusal;
use crate::i18n::{t, ta};
use crate::tags::TestTag;
use crate::targs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// `test/**/*_test.rb` and `spec/**/*_spec.rb` -- where the proves
/// tags of both readings live. A project without either directory
/// has none, and that is not a refusal.
pub fn test_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let mut out = minitest_files(root)?;
    out.extend(spec_files(root)?);
    out.sort();
    Ok(out)
}

/// The first reading's files: `test/**/*_test.rb`.
pub fn minitest_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    files_named(root, "test", "_test.rb")
}

/// The second reading's files: `spec/**/*_spec.rb` (wave 0047).
pub fn spec_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    files_named(root, "spec", "_spec.rb")
}

/// Whether a file is the second reading's: named `*_spec.rb`. The
/// readings are told apart by the FILE, never by the project's
/// config -- one project may keep both.
pub fn is_spec(file: &Path) -> bool {
    file.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with("_spec.rb"))
}

fn files_named(root: &Path, dir: &str, suffix: &str) -> Result<Vec<PathBuf>, Refusal> {
    let dir = root.join(dir);
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
                .is_some_and(|n| n.ends_with(suffix))
            {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// The `.rb` files in `test/` and `spec/` this adapter does NOT read,
/// because the conventions are `*_test.rb` and `*_spec.rb`: helpers,
/// `spec/support/`, a `spec_helper.rb`. A border said aloud by the
/// tool rather than only by the README (review 0038 R-19): a skip
/// nobody names reads as "there was nothing to read".
pub fn unread_files(root: &Path) -> Vec<PathBuf> {
    let mut out: Vec<PathBuf> = Vec::new();
    for (dir, suffix) in [("test", "_test.rb"), ("spec", "_spec.rb")] {
        let mut stack = vec![root.join(dir)];
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
                        .is_some_and(|n| n.ends_with(suffix) || n == "spec_helper.rb")
                {
                    out.push(path);
                }
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
    if is_spec(&tag.file) {
        return run_spec(root, tag);
    }
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
    // The second reading's roll: one rspec run per spec file, the
    // list and the verdicts alike from rspec's JSON. Pending is
    // neither green nor red and not in the map (§7.12); a file that
    // did not load is the adapter's refusal aloud, with ruby's words.
    for file in spec_files(root)? {
        let relative = file.strip_prefix(root).unwrap_or(&file);
        let said = rspec(root, &[relative.display().to_string()])?;
        if let Some(words) = load_error(&said.json, &said.voice) {
            return Err(Refusal {
                file: file.clone(),
                reason: ta("adapter-rspec-broken", targs!("error" => words)),
                instead: t("adapter-rspec-broken-instead"),
            });
        }
        let key = crate::adapter::battery_key(root, &file);
        for example in examples(&said.json) {
            let green = match example.status.as_str() {
                "passed" => true,
                "failed" => false,
                _ => continue,
            };
            // Two examples of one full description are one key, red
            // if either is (the rule review 0046 R-2 asked to be one
            // in both courts).
            out.entry((key.clone(), example.description))
                .and_modify(|was| *was = *was && green)
                .or_insert(green);
        }
    }
    for file in minitest_files(root)? {
        let relative = file.strip_prefix(root).unwrap_or(&file);
        let stem = crate::adapter::battery_key(root, &file);
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

/// One example as rspec's JSON reports it: the id a run selects it
/// by (`./spec/toy_spec.rb[1:2:1]`), the full description a tag
/// names it by, and the state.
struct Example {
    id: String,
    description: String,
    status: String,
}

/// Runs exactly the tagged example -- by the ID rspec itself gives
/// it, found in a dry run by the full description. Never by name:
/// `-e` matches a SUBSTRING, so `-e works` would run `works too` as
/// well. Two runs per test are the price of exactness, and named.
/// Where the dry run names two examples of one description, both
/// run, and a red among them is red.
pub fn run_spec(root: &Path, tag: &TestTag) -> Result<crate::adapter::Outcome, Refusal> {
    let relative = tag.file.strip_prefix(root).unwrap_or(&tag.file);
    let dry = rspec(
        root,
        &["--dry-run".to_string(), relative.display().to_string()],
    )?;
    if let Some(words) = load_error(&dry.json, &dry.voice) {
        return Ok(crate::adapter::Outcome::BuildBroken(words));
    }
    let ids: Vec<String> = examples(&dry.json)
        .into_iter()
        .filter(|example| example.description == tag.test)
        .map(|example| example.id)
        .collect();
    if ids.is_empty() {
        return Ok(crate::adapter::Outcome::NotRun);
    }
    let said = rspec(root, &ids)?;
    if let Some(words) = load_error(&said.json, &said.voice) {
        return Ok(crate::adapter::Outcome::BuildBroken(words));
    }
    Ok(classify_spec(&said.json, &tag.test))
}

/// What one run came to, read from rspec's JSON and from it alone:
/// a load error before any example -- `errors_outside_of_examples`
/// -- is a broken build with ruby's own words; among the examples of
/// this description any `failed` is red, else any `passed` is green,
/// else (`pending` alone) nothing ran. No example of the name: nothing
/// ran -- whatever the exit code, which is 0 for an id that matched
/// nothing.
pub fn classify_spec(said: &str, test: &str) -> crate::adapter::Outcome {
    if let Some(words) = load_error(said, "") {
        return crate::adapter::Outcome::BuildBroken(words);
    }
    let named: Vec<Example> = examples(said)
        .into_iter()
        .filter(|example| example.description == test)
        .collect();
    if named.iter().any(|example| example.status == "failed") {
        crate::adapter::Outcome::Failed
    } else if named.iter().any(|example| example.status == "passed") {
        crate::adapter::Outcome::Green
    } else {
        crate::adapter::Outcome::NotRun
    }
}

/// The examples in rspec's JSON, in its order.
fn examples(said: &str) -> Vec<Example> {
    let Ok(json) = serde_json::from_str::<serde_json::Value>(said) else {
        return Vec::new();
    };
    json["examples"]
        .as_array()
        .map(|list| {
            list.iter()
                .filter_map(|example| {
                    Some(Example {
                        id: example["id"].as_str()?.to_string(),
                        description: example["full_description"].as_str()?.to_string(),
                        status: example["status"].as_str()?.to_string(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// ruby's own words when the examples could not even be loaded --
/// rspec still writes its JSON then, with the error counted outside
/// the examples. The WORDS are in two places: a `--require`d helper
/// that failed is narrated on stdout ("While loading spec_helper a
/// `raise SyntaxError` occurred …", then ruby's `file.rb:LINE: syntax
/// error …`, in colour whatever `--no-color` says), and the JSON's
/// own `messages` carry what followed (a `NameError` for the constant
/// the helper never defined). Both are read, colour stripped; the
/// loading line, the error and its detail, and the place. `None`
/// when the run loaded its files: a failed example is not a broken
/// build.
fn load_error(json: &str, voice: &str) -> Option<String> {
    let parsed = serde_json::from_str::<serde_json::Value>(json).ok()?;
    let errors = parsed["summary"]["errors_outside_of_examples_count"]
        .as_u64()
        .unwrap_or(0);
    if errors == 0 {
        return None;
    }
    let mut text = strip_ansi(voice);
    if let Some(messages) = parsed["messages"].as_array() {
        for message in messages.iter().filter_map(|m| m.as_str()) {
            text.push('\n');
            text.push_str(message);
        }
    }
    let lines: Vec<&str> = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    let mut words: Vec<String> = Vec::new();
    if let Some(loading) = lines.iter().find(|line| {
        line.starts_with("While loading") || line.starts_with("An error occurred while loading")
    }) {
        words.push((*loading).to_string());
    }
    if let Some(at) = lines.iter().position(|line| {
        line.ends_with("Error:") && line.chars().next().is_some_and(|c| c.is_ascii_uppercase())
    }) {
        let mut error = lines[at].to_string();
        if let Some(detail) = lines.get(at + 1) {
            error.push(' ');
            error.push_str(detail);
        }
        words.push(error);
    }
    if let Some(place) = lines
        .iter()
        .find(|line| line.contains(".rb:") && line.to_lowercase().contains("error"))
    {
        words.push((*place).to_string());
    }
    Some(if words.is_empty() {
        "rspec could not load the examples".to_string()
    } else {
        words.join(" -- ")
    })
}

/// Colour codes out: ruby paints its own syntax errors whatever rspec
/// is told, and a refusal's words are for a person to read.
fn strip_ansi(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' && chars.peek() == Some(&'[') {
            chars.next();
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
            continue;
        }
        out.push(ch);
    }
    out
}

/// rspec as a command of the system, in the project's own world --
/// with its JSON sent to a file OUTSIDE the project, because stdout
/// is not ours: a `.rspec` may carry the project's own `--format`,
/// and it must be read (a standard project needs its `--require
/// spec_helper`). `SPEC_OPTS` is dropped: rspec reads it, and a
/// `--tag` there filtered the run away (measured). `--no-color`, or
/// the error's words carry colour codes.
/// What one rspec run said: its JSON (from the file), and its voice
/// on stdout and stderr, where a load error is narrated.
struct Said {
    json: String,
    voice: String,
}

fn rspec(root: &Path, args: &[String]) -> Result<Said, Refusal> {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let out_file = std::env::temp_dir().join(format!(
        "keel-rspec-{}-{}.json",
        std::process::id(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let mut command = Command::new("rspec");
    command
        .args(["--format", "json", "--out"])
        .arg(&out_file)
        .arg("--no-color")
        .args(args)
        .current_dir(root)
        .env_remove("SPEC_OPTS");
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: ta("adapter-rspec-failed", targs!("error" => e.to_string())),
        instead: t("adapter-rspec-failed-instead"),
    })?;
    let json = std::fs::read_to_string(&out_file);
    let _ = std::fs::remove_file(&out_file);
    let voice = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    match json {
        Ok(json) => Ok(Said { json, voice }),
        Err(_) => Err(Refusal {
            file: root.to_path_buf(),
            reason: ta(
                "adapter-rspec-failed",
                targs!("error" => strip_ansi(voice.trim())),
            ),
            instead: t("adapter-rspec-failed-instead"),
        }),
    }
}
