//! The cargo adapter (contract tool-adapter-cargo): the one place
//! that knows how a Rust project names its test files and how to run
//! a single test. Other languages will get their own waves next to
//! this file.

use crate::config::Language;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::tags::TestTag;
use crate::targs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The directory this adapter builds into: the home of that
/// knowledge is here, because it is the adapter's own (wave 0020).
/// The frame asks it to advise an ignore rule; the next language
/// arrives with its own adapter and its own name.
pub const BUILD_DIR: &str = "target";

/// Cargo.toml at the root -- the root itself; otherwise exactly one
/// first-level directory carrying one -- that; zero or several -- a
/// refusal aloud with what was found. No guessing.
pub fn crate_root(root: &Path) -> Result<PathBuf, Refusal> {
    if root.join("Cargo.toml").is_file() {
        return Ok(root.to_path_buf());
    }
    let mut found: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(root).map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("Cargo.toml").is_file() {
            found.push(path);
        }
    }
    found.sort();
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(Refusal {
            file: root.to_path_buf(),
            reason: t("adapter-no-crate"),
            instead: t("adapter-no-crate-instead"),
        }),
        _ => Err(Refusal {
            file: root.to_path_buf(),
            reason: ta(
                "adapter-many-crates",
                targs!("found" => found
                    .iter()
                    .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")),
            ),
            instead: t("adapter-many-crates-instead"),
        }),
    }
}

/// Where this language builds, if it builds at all.
pub enum BuildDir {
    /// It builds, and here.
    At(PathBuf),
    /// It builds nothing at all -- ruby compiles ahead of nothing,
    /// so there is no directory to measure or to warn about, and the
    /// closing court's price line says so rather than naming a path
    /// that will never exist (wave 0038).
    Nothing,
    /// It builds, but where could not be told: the adapter has its
    /// own refusal to give, and that refusal is not "builds nothing"
    /// (review 0038 R-18 -- a Rust project with no Cargo.toml was
    /// told its tongue compiles nothing, one line above the refusal
    /// that it has no crate).
    Unknown,
}

/// The key a test has in the battery map -- and it is the SAME key
/// on both sides: the adapters build it from what the runner said,
/// the closing court builds it from a tag's file. The path from the
/// tests directory down, without the extension: `a/test_x`, never
/// the bare stem. A stem is not unique -- `tests/a/test_x.py` and
/// `tests/b/test_x.py` collided, the second overwrote the first, and
/// a red test vanished with the wave closing over it, the verdict
/// depending on collection order (review 0045 R-1). The same shape
/// stood in ruby. For rust the tests directory is flat, so this is
/// the stem it always was.
pub fn battery_key(root: &Path, file: &Path) -> String {
    let base = tests_dir(root).unwrap_or_else(|_| root.to_path_buf());
    let relative = file
        .strip_prefix(&base)
        .or_else(|_| file.strip_prefix(root))
        .unwrap_or(file);
    relative
        .with_extension("")
        .to_string_lossy()
        .replace('\\', "/")
}

/// cargo's own count from a block's closing line -- `ok. 2 passed; 0
/// failed; 0 ignored; …` or `FAILED. 1 passed; 1 failed; …` -- as
/// (passed, failed). None where the line is not that shape.
fn result_counts(rest: &str) -> Option<(u64, u64)> {
    let mut passed = None;
    let mut failed = None;
    for part in rest.split(';') {
        let mut words = part.split_whitespace().rev();
        let (Some(word), Some(number)) = (words.next(), words.next()) else {
            continue;
        };
        match word {
            "passed" => passed = number.parse().ok(),
            "failed" => failed = number.parse().ok(),
            _ => {}
        }
    }
    Some((passed?, failed?))
}

/// Whether this tongue's build is the kind that wants gigabytes.
/// cargo's is; mix's `_build` measured 148 KiB on the same battery
/// (review 0042 R-4). A warning four orders of magnitude out is not
/// a warning, and a refusal over free space it does not need is
/// worse.
pub fn builds_heavily(root: &Path) -> bool {
    matches!(language_of(root), None | Some(Language::Rust))
}

/// Where the project's battery actually runs -- the directory this
/// release's own adapter uses as its working directory, and so the
/// only directory a generated CI step may name.
///
/// `None` means the adapter cannot say: two crates on the first
/// level, or one deeper than it looks. A step written under that
/// silence would fail on a runner without ever saying why, so the
/// generator says it instead (review 0044 R-6).
///
/// Ruby and Elixir answer with the ROOT, always. Their adapters run
/// from there (`ruby -Itest <file>`, `mix test`, both
/// `.current_dir(root)`), and `tests_dir` looks in `root/test`. A
/// generated step that went to a `Gemfile`'s own directory would run
/// a DIFFERENT battery from the one the courts judge -- measured by
/// review 0044 R-2 turning a red tree green, which is the exact
/// shape this whole wave exists to stop.
pub fn battery_dir(root: &Path) -> Option<PathBuf> {
    match language_of(root) {
        Some(Language::Ruby)
        | Some(Language::Elixir)
        | Some(Language::Python)
        | Some(Language::JavaScript) => Some(root.to_path_buf()),
        _ => crate_root(root).ok(),
    }
}

/// The files this tongue's own runner leaves in a project, as paths
/// relative to the root: lock files, and nothing else (wave 0055).
///
/// A runner writes them without being asked -- the first `red:`
/// commit through the hook builds the crate, and `Cargo.lock`
/// appeared under an author who had touched nothing of the sort, so
/// every court called it drift the wave never declared (final review
/// 2026-09-06, bugs R-20). They are the tongue's furniture, as the
/// frame's own files are the methodology's (§4.8): outside scope in
/// both directions, and named here rather than known by heart in
/// three courts. What a lock file RECORDS still comes from a
/// manifest, and a manifest no transform names is drift like any
/// other file -- so nothing about a dependency slips past unseen.
pub fn lockfiles(root: &Path) -> Vec<String> {
    match language_of(root) {
        Some(Language::Ruby) => vec!["Gemfile.lock".to_string()],
        Some(Language::Elixir) => vec!["mix.lock".to_string()],
        // npm's own; yarn and pnpm are named as a border in
        // tool-scope, not read here.
        Some(Language::JavaScript) => vec!["package-lock.json".to_string()],
        // pytest locks nothing: what it leaves is a directory, and
        // build directories are ignore rules, not scope (wave 0045).
        Some(Language::Python) => Vec::new(),
        _ => match crate_root(root) {
            Ok(dir) => {
                let relative = dir.strip_prefix(root).unwrap_or(Path::new(""));
                let lock = relative.join("Cargo.lock");
                vec![lock.to_string_lossy().replace('\\', "/")]
            }
            Err(_) => Vec::new(),
        },
    }
}

/// One thing the tongue's runner leaves in the tree, and WHERE it
/// may be met (wave 0057; review R-7 measured the second half
/// missing: a project's own `docs/target/notes.md` became furniture
/// because `target` is a name rust leaves).
pub struct Leaving {
    /// The path from the project's root, or the bare directory name
    /// when the runner writes it at any depth. A directory carries
    /// its trailing slash.
    pub path: String,
    /// Met at ANY depth -- pytest writes a `__pycache__` beside every
    /// module it imports, npm a `node_modules` in every workspace
    /// member -- or only at the one place the adapter names, as a
    /// build directory is.
    pub anywhere: bool,
}

/// What this tongue's runner leaves in the tree.
///
/// Measured in sandboxes before the plan of wave 0057, not guessed:
/// `pytest` writes `__pycache__/` beside every module it imports and
/// `.pytest_cache/` at the root; `npm install` writes `node_modules/`
/// as soon as the project has one dependency; ruby's minitest and
/// rspec left NOTHING in a bare project, and so ruby's list is empty
/// and the frame says so; elixir and rust leave the build directory
/// the adapter already names -- and that one is met at its place
/// alone.
///
/// Two courts read this one list (queue after 0055, bugs R-21): the
/// ignore advice of `keel init` -- which told python, javascript and
/// ruby alike that "this tongue builds nothing", while pytest was
/// filling the tree -- and the scope court, for which a directory the
/// runner wrote without being asked is furniture, not this wave's
/// work.
pub fn leavings(root: &Path) -> Vec<Leaving> {
    let anywhere = |name: &str| Leaving {
        path: name.to_string(),
        anywhere: true,
    };
    match language_of(root) {
        Some(Language::Python) => vec![anywhere("__pycache__/"), anywhere(".pytest_cache/")],
        Some(Language::JavaScript) => vec![anywhere("node_modules/")],
        // `bin/rails test` writes: `log/test.log`, `tmp/`, and the
        // test database. Rails's own `.gitignore` covers them, so
        // nothing hurt -- but "this tongue leaves nothing of its own"
        // was said to a person for whom it is false (review 0059
        // R-12), and the scope court reads this same list.
        Some(Language::Ruby) if crate::ruby::rails_root(root) => {
            vec![anywhere("log/"), anywhere("tmp/")]
        }
        Some(Language::Ruby) => Vec::new(),
        _ => match build_dir(root) {
            BuildDir::At(path) => {
                let relative = path.strip_prefix(root).unwrap_or(Path::new(""));
                vec![Leaving {
                    path: format!("{}/", relative.to_string_lossy().replace('\\', "/")),
                    anywhere: false,
                }]
            }
            _ => Vec::new(),
        },
    }
}

pub fn build_dir(root: &Path) -> BuildDir {
    match language_of(root) {
        // pytest builds nothing -- and, told so, writes nothing
        // either (wave 0045).
        Some(Language::Ruby) | Some(Language::Python) | Some(Language::JavaScript) => {
            BuildDir::Nothing
        }
        Some(Language::Elixir) => BuildDir::At(root.join(crate::elixir::BUILD_DIR)),
        _ => match crate_root(root) {
            Ok(dir) => BuildDir::At(dir.join(BUILD_DIR)),
            Err(_) => BuildDir::Unknown,
        },
    }
}

/// Where this tongue keeps its test files, said as a person would
/// say it. The hand of §9.2 used to name `tests/` of a crate to a
/// ruby project and refuse over a missing Cargo.toml (review 0038
/// R-5): the courts above must not know one language's layout.
pub fn tests_dir(root: &Path) -> Result<PathBuf, Refusal> {
    match language_of(root) {
        Some(Language::Ruby) | Some(Language::Elixir) => Ok(root.join("test")),
        Some(Language::Python) => Ok(root.join("tests")),
        // `test/` by node's own convention; `tests/` is read too, and
        // a key from there keeps its `tests/` in front.
        Some(Language::JavaScript) => Ok(root.join("test")),
        _ => Ok(crate_root(root)?.join("tests")),
    }
}

/// The command a person would type to run exactly this one test --
/// the very one `run_test` runs, in the tongue's own words.
pub fn run_line(root: &Path, file: &Path, test: &str, line: usize) -> String {
    let relative = file.strip_prefix(root).unwrap_or(file);
    match language_of(root) {
        // By file and line, which is what `run_test` runs (wave
        // 0051): `--only 'test:test <name>'` over a name with letters
        // past ASCII excludes everything -- "no test was executed" --
        // and the hand of §9.2 handed a person exactly that line
        // while the court beside it ran another (final review
        // 2026-09-06, bugs R-12; wave 0055).
        Some(Language::Elixir) => {
            format!("mix test {}:{line}", relative.display())
        }
        Some(Language::Python) => format!("pytest {}::{test}", relative.display()),
        Some(Language::JavaScript) => format!(
            "node --test --test-name-pattern={} {}",
            shell_quoted(&format!("^{}$", crate::javascript::escape_regex(test))),
            relative.display()
        ),
        // The second reading's line is rspec's own -- and `-e` is a
        // substring match, which the person is told beside it.
        Some(Language::Ruby) if crate::ruby::is_spec(file) => {
            format!(
                "rspec {} -e {}  {}",
                relative.display(),
                shell_quoted(test),
                t("run-line-rspec-note")
            )
        }
        // The third reading's line is the one a person in a Rails
        // application actually types: `bin/rails test <file> -n
        // <method>`. Measured 2026-09-07 -- `ruby -Itest` boots no
        // application, so the advice was a line that could not work
        // where it was given (wave 0059).
        Some(Language::Ruby) if crate::ruby::rails_root(root) => {
            // Quoted, as the rspec line beside it is: a Rails name is
            // arbitrary text, so apostrophes, quotes and `#` survive
            // into the method name, and an unquoted line left the
            // person's shell waiting for a closing quote (review 0046
            // R-5, measured again here as review 0059 R-5).
            format!(
                "bin/rails test {} -n {}",
                relative.display(),
                shell_quoted(test)
            )
        }
        Some(Language::Ruby) => format!("ruby -Itest {} -n {test}", relative.display()),
        _ => {
            let stem = file
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            format!("cargo test --test {stem} {test} -- --exact")
        }
    }
}

/// A word a shell will hand on whole: single-quoted, with any
/// apostrophe inside it closed, escaped and reopened -- `it's` becomes
/// `'it'\\''s'`. The line is for a person to paste, and an
/// apostrophe in a test name left the pasted line waiting for a
/// closing quote (review 0046 R-5; the elixir line had the same
/// latent flaw). The RUN itself hands the name as one argument and
/// never crosses a shell.
fn shell_quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}

/// Which language leads this project, read from its config. The
/// courts above call these hands without carrying the config along,
/// so the question is asked here -- one file read, and the answer
/// decides whose hand runs (wave 0038).
fn language_of(root: &Path) -> Option<Language> {
    crate::config::read_unpinned(root).ok()?.language()
}

/// Whether a path of the tree, relative to the root, is a file
/// `test_files` would read -- asked by the §7.15 court of the tree at
/// the fork point, which is not on disk. The court used to ask for a
/// crate and list `<crate>/tests`, so every other tongue's vanished
/// tag went unjudged while the summary line claimed the court
/// (global review 2026-09-06, methodology cut R-1). Each tongue
/// answers by its own naming rule; rust by the crate's flat `tests/`.
pub fn is_test_path(root: &Path, rel: &str) -> bool {
    match language_of(root) {
        Some(Language::Ruby) => crate::ruby::is_test_path(rel),
        Some(Language::Elixir) => crate::elixir::is_test_path(rel),
        Some(Language::Python) => crate::python::is_test_path(rel),
        Some(Language::JavaScript) => crate::javascript::is_test_path(rel),
        _ => {
            let Ok(crate_dir) = crate_root(root) else {
                return false;
            };
            let tests = crate_dir
                .strip_prefix(root)
                .map(|p| p.join("tests"))
                .unwrap_or_else(|_| PathBuf::from("tests"));
            let prefix = format!("{}/", tests.display().to_string().replace('\\', "/"));
            rel.strip_prefix(&prefix)
                .is_some_and(|name| !name.contains('/') && name.ends_with(".rs"))
        }
    }
}

/// The crate's `tests/*.rs` -- where the proves tags live. A crate
/// without a tests directory has none, and that is not a refusal.
pub fn test_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    match language_of(root) {
        Some(Language::Ruby) => return crate::ruby::test_files(root),
        Some(Language::Elixir) => return crate::elixir::test_files(root),
        Some(Language::Python) => return crate::python::test_files(root),
        Some(Language::JavaScript) => return crate::javascript::test_files(root),
        _ => {}
    }
    let dir = crate_root(root)?.join("tests");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| Refusal {
        file: dir.clone(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") && path.is_file() {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}

/// What one run of one test came to -- in cargo's own words where
/// they matter. "Did not compile" and "did not run" are not
/// "failed" (journal A3).
pub enum Outcome {
    Failed,
    Green,
    BuildBroken(String),
    NotRun,
}

/// Runs exactly the tagged test (`cargo test --test <file> <fn> --
/// --exact`) and classifies the consequence. cargo is called as a
/// command of the system; its refusal to start is a refusal aloud.
pub fn run_test(root: &Path, tag: &TestTag) -> Result<Outcome, Refusal> {
    match language_of(root) {
        Some(Language::Ruby) => return crate::ruby::run_test(root, tag),
        Some(Language::Elixir) => return crate::elixir::run_test(root, tag),
        Some(Language::Python) => return crate::python::run_test(root, tag),
        Some(Language::JavaScript) => return crate::javascript::run_test(root, tag),
        _ => {}
    }
    let crate_dir = crate_root(root)?;
    let target = test_target(&crate_dir, &tag.file)?;
    let mut command = Command::new("cargo");
    command
        .arg("test")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .args(["--test", &target, &tag.test, "--", "--exact"])
        // The judged project builds into its own target directory:
        // an inherited shared cache shifts verdicts (§6.7 heal of
        // 0005 per review 0008 R-8; seen live in 0006 too), and the
        // cargo alias walks it back in through the side door
        // (review 0009 R-2).
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR");
    // And it runs in the project's own world: a hook's repository,
    // left in the environment, would otherwise reach the project's
    // own tests through cargo (review 0021 R-3).
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: crate_dir.clone(),
        reason: ta("adapter-cargo-failed", targs!("error" => e.to_string())),
        instead: t("adapter-cargo-failed-instead"),
    })?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("could not compile") || stderr.contains("error[E") {
        let words = stderr
            .lines()
            .find(|l| l.starts_with("error"))
            .unwrap_or("could not compile")
            .to_string();
        return Ok(Outcome::BuildBroken(words));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let Some(result) = stdout.lines().find(|l| l.starts_with("test result:")) else {
        return Err(Refusal {
            file: crate_dir,
            reason: ta(
                "adapter-cargo-failed",
                targs!("error" => stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no test result line").to_string()),
            ),
            instead: t("adapter-cargo-failed-instead"),
        });
    };
    let executed = count_before(result, " passed") + count_before(result, " failed");
    if executed == 0 {
        return Ok(Outcome::NotRun);
    }
    if count_before(result, " failed") > 0 {
        Ok(Outcome::Failed)
    } else {
        Ok(Outcome::Green)
    }
}

/// The name cargo knows a test file's target by: the stem, unless
/// the manifest renames it -- `[[test]] name = "renamed" path =
/// "tests/w_test.rs"` -- in which case `--test w_test` is a refusal
/// while the battery reads the file fine, two courts about one test
/// (global review 2026-09-06 R-14; wave 0051). A manifest the reader
/// cannot parse is a refusal with the reader's words, never a guess.
fn test_target(crate_dir: &Path, file: &Path) -> Result<String, Refusal> {
    let stem = file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let manifest = crate_dir.join("Cargo.toml");
    let text = std::fs::read_to_string(&manifest).map_err(|e| Refusal {
        file: manifest.clone(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    let value: toml::Value = toml::from_str(&text).map_err(|e| Refusal {
        file: manifest.clone(),
        reason: ta("adapter-cargo-manifest", targs!("error" => e.to_string())),
        instead: t("adapter-cargo-manifest-instead"),
    })?;
    let relative = file
        .strip_prefix(crate_dir)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/");
    // `path = "./tests/w_test.rs"` is the same file to cargo (measured:
    // `cargo test --test renamed` runs it), so the leading `./` is not
    // part of the comparison (review 0051 R-8).
    let same = |declared: &str| {
        declared.replace('\\', "/").trim_start_matches("./") == relative.trim_start_matches("./")
    };
    let renamed = value
        .get("test")
        .and_then(|t| t.as_array())
        .into_iter()
        .flatten()
        .find(|table| table.get("path").and_then(|p| p.as_str()).is_some_and(same))
        .and_then(|table| table.get("name").and_then(|n| n.as_str()))
        .map(str::to_string);
    Ok(renamed.unwrap_or(stem))
}

/// The whole battery in one cargo run, verdicts laid out per test:
/// the key is (test file stem, function name), the value is green.
/// One run instead of one per tag -- the closure court reads it
/// once. A build that does not build is a refusal aloud with the
/// compiler's words: without a build there is no verdict for anyone.
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    match language_of(root) {
        Some(Language::Ruby) => return crate::ruby::run_all(root),
        Some(Language::Elixir) => return crate::elixir::run_all(root),
        Some(Language::Python) => return crate::python::run_all(root),
        Some(Language::JavaScript) => return crate::javascript::run_all(root),
        _ => {}
    }
    let crate_dir = crate_root(root)?;
    let mut command = Command::new("cargo");
    command
        .arg("test")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .arg("--no-fail-fast")
        // Same isolation as run_test: the shared cache lies, and the
        // hook's repository must not reach the project's tests.
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR");
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: crate_dir.clone(),
        reason: ta("adapter-cargo-failed", targs!("error" => e.to_string())),
        instead: t("adapter-cargo-failed-instead"),
    })?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("could not compile") || stderr.contains("error[E") {
        let words = stderr
            .lines()
            .find(|l| l.starts_with("error"))
            .unwrap_or("could not compile")
            .to_string();
        return Err(Refusal {
            file: crate_dir,
            reason: ta("adapter-cargo-failed", targs!("error" => words)),
            instead: t("adapter-cargo-failed-instead"),
        });
    }
    // cargo splits its word: the target list ("Running tests/x.rs")
    // goes to stderr, the verdicts go to stdout -- one block per
    // target, in the same order, since targets run one after another.
    // The targets and the blocks are stitched back by that order.
    //
    // A target is keyed as cargo announces it: `tests/x.rs` by its
    // stem -- the key a tag's file gives (`battery_key`) -- and any
    // other target (`unittests src/lib.rs`, `unittests src/main.rs`)
    // by the words themselves. The bare STEM let the library's and
    // the binary's unit tests share one key, and the second target's
    // green overwrote the first's red while cargo left with 101
    // (global review 2026-09-06, bugs cut R-1). Two targets announced
    // by one path -- a workspace with `a/tests/basic.rs` and
    // `b/tests/basic.rs` -- are told apart by nothing a tag could
    // name: a refusal aloud, never two verdicts folded into one.
    let mut targets: Vec<String> = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(what) = trimmed.strip_prefix("Running ") {
            let announced = what.split(" (").next().unwrap_or(what).trim();
            let key = if announced.starts_with("tests/") {
                Path::new(announced)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default()
            } else {
                announced.to_string()
            };
            if targets.contains(&key) {
                return Err(Refusal {
                    file: crate_dir,
                    reason: ta(
                        "adapter-battery-alike",
                        targs!("target" => announced.to_string()),
                    ),
                    instead: t("adapter-battery-alike-instead"),
                });
            }
            targets.push(key);
        } else if trimmed.starts_with("Doc-tests ") {
            targets.push("doc-tests".to_string());
        }
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut verdicts: BTreeMap<(String, String), bool> = BTreeMap::new();
    let mut block: usize = 0;
    let mut closed: usize = 0;
    // What the reader counted in the current block, checked against
    // cargo's own closing line of that block -- `test result: FAILED.
    // 1 passed; 1 failed; …`. A test may print anything into this
    // stream past libtest (a child process, review 0050 R-1): a
    // verdict line it forges, a `failures:` it echoes, a `running 1
    // test` -- and the first cut of this wave hid every verdict after
    // a `failures:` line, so a child that printed the word made the
    // red below it vanish. The numbers cargo counted are the court
    // now: a block whose read verdicts do not add up to them, a
    // closing line cargo did not print, or an exit 101 with no red
    // read at all -- each a refusal aloud, never a guess.
    let mut counted = (0u64, 0u64);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("running ") && trimmed.ends_with("tests")
            || trimmed == "running 1 test"
        {
            block += 1;
            counted = (0, 0);
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("test result: ") {
            let Some((passed, failed)) = result_counts(rest) else {
                continue;
            };
            if (passed, failed) != counted {
                return Err(Refusal {
                    file: crate_dir,
                    reason: ta(
                        "adapter-battery-counts",
                        targs!("passed" => passed, "failed" => failed, "green" => counted.0, "red" => counted.1),
                    ),
                    instead: t("adapter-battery-counts-instead"),
                });
            }
            closed += 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("test ")
            && let Some((name, verdict)) = rest.rsplit_once(" ... ")
        {
            // `#[should_panic]` is written by cargo into the verdict
            // line -- `test it_panics - should panic ... ok` -- and
            // the first reading kept the whole of it as the name, so
            // the closing court said the battery ran no test of the
            // tag's name while the gate, which runs one test by name,
            // was green (final review 2026-09-06, bugs R-9; wave
            // 0055). The suffix is cargo's word about the test, not
            // part of what anyone may write in a `proves:` tag.
            let name = name
                .trim()
                .strip_suffix(" - should panic")
                .unwrap_or(name.trim());
            let green = match verdict.trim() {
                "ok" => true,
                v if v.starts_with("FAILED") => false,
                _ => continue, // ignored and friends are no verdict
            };
            if green {
                counted.0 += 1;
            } else {
                counted.1 += 1;
            }
            let target = block
                .checked_sub(1)
                .and_then(|i| targets.get(i))
                .cloned()
                .unwrap_or_default();
            // One key, one verdict: cargo names a test once per
            // target, and the numbers above are what hold the text.
            verdicts.insert((target, name.to_string()), green);
        }
    }
    // The stitch holds only when every announced target printed its
    // verdict block and every block its closing line: a harness =
    // false target prints "Running" and no block, shifting every
    // later verdict onto the wrong target -- up to blessing a wave
    // with a red tagged test (review R-1). A seam that does not meet
    // is a refusal, not a guess.
    if block != targets.len() || closed != block {
        return Err(Refusal {
            file: crate_dir,
            reason: ta(
                "adapter-battery-mismatch",
                targs!("stems" => targets.len() as u64, "blocks" => block as u64),
            ),
            instead: t("adapter-battery-mismatch-instead"),
        });
    }
    // cargo left red and the reader saw none: whatever it was -- a
    // binary that crashed before its closing line, a shape this
    // reader does not know -- it is not a green battery.
    if !out.status.success() && !verdicts.values().any(|green| !green) {
        return Err(Refusal {
            file: crate_dir,
            reason: ta(
                "adapter-cargo-red-unseen",
                targs!("code" => out.status.code().unwrap_or(-1) as i64),
            ),
            instead: t("adapter-cargo-red-unseen-instead"),
        });
    }
    if verdicts.is_empty() && !out.status.success() {
        return Err(Refusal {
            file: crate_dir,
            reason: ta(
                "adapter-cargo-failed",
                targs!("error" => stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no test output").to_string()),
            ),
            instead: t("adapter-cargo-failed-instead"),
        });
    }
    Ok(verdicts)
}

/// The number standing right before the given marker in cargo's
/// "test result:" line.
fn count_before(line: &str, marker: &str) -> u64 {
    let Some(end) = line.find(marker) else {
        return 0;
    };
    line[..end]
        .split(|c: char| !c.is_ascii_digit())
        .next_back()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}
