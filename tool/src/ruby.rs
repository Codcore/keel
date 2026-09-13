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

/// Whether a path of the tree, relative to the root, is one either
/// reading would take: `test/**/*_test.rb` or `spec/**/*_spec.rb`.
/// The §7.15 court asks this of a tree that is not on disk (wave
/// 0050).
pub fn is_test_path(rel: &str) -> bool {
    (rel.starts_with("test/") && rel.ends_with("_test.rb"))
        || (rel.starts_with("spec/") && rel.ends_with("_spec.rb"))
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

/// Is this a Rails application? TWO marks together, and neither
/// alone: `bin/rails` (the command Rails gives a person) and
/// `config/application.rb` (the application it boots). Measured on a
/// real application 2026-09-07; asked of the PROJECT and never of the
/// config, because a person writing `adapter = "ruby"` should not
/// have to know that their project is called something else.
///
/// Rails is not another tongue -- it is another LAYOUT of this one,
/// and this is the third reading of ruby beside minitest and rspec
/// (the second arrived in wave 0047). What changes is the command:
/// the battery of a Rails application is `bin/rails test`, because
/// `ruby -Itest` never boots the application at all.
pub fn rails_root(root: &Path) -> bool {
    root.join("bin/rails").is_file() && root.join("config/application.rb").is_file()
}

/// The runner for this project, with the arguments that reach the
/// same minitest either way: Rails through its own `bin/rails test`,
/// a plain project through `ruby -Itest`. The adapter's word to its
/// child about the encoding of the arguments (wave 0051) rides on
/// both roads -- as the flag where ruby is called directly, and as
/// RUBYOPT where the call goes through the project's own script,
/// which would read a `-E` of its own as an argument of `rails test`
/// (review 0059 R-1 measured the loss before it was said this way).
fn minitest_command(root: &Path, args: &[String]) -> Command {
    if rails_root(root) {
        let mut command = Command::new(root.join("bin/rails"));
        command.arg("test").args(args).current_dir(root);
        // Rails boots the application to run its tests, and the
        // environment is the one Rails itself names for that.
        command.env("RAILS_ENV", "test");
        // The SAME word about encoding, said the only way this road
        // can say it: `bin/rails` is a script of the project, and a
        // `-E UTF-8` handed to it would be an argument of `rails
        // test`, not of ruby. Review 0059 R-1 measured the loss on a
        // real sandbox -- under `LC_ALL=C` a test named `вітає
        // ünïcode` ran NOTHING on this road while the plain one ran
        // it green, which is bug R-19 of the global review
        // 2026-09-06 (wave 0051) risen again on a new road. RUBYOPT
        // is ruby's own door for exactly this, and what already
        // stands in it is kept: the project may have put its own
        // options there.
        let mut opts = std::env::var("RUBYOPT").unwrap_or_default();
        if !opts.contains("-EUTF-8") && !opts.contains("-E UTF-8") {
            if !opts.is_empty() {
                opts.push(' ');
            }
            opts.push_str("-EUTF-8");
        }
        command.env("RUBYOPT", opts);
        return command;
    }
    let mut command = Command::new("ruby");
    command
        .args(["-E", "UTF-8"])
        .arg("-Itest")
        .args(args)
        .current_dir(root);
    command
}

/// Whose failure it was, said by name (review 0059 R-4): on the
/// Rails road the thing that did not start is `bin/rails` -- a script
/// of the project, which may simply be non-executable -- and telling
/// a person to "put ruby on PATH" sends them looking where nothing is
/// wrong.
fn runner_failed(root: &Path, error: &std::io::Error) -> String {
    if rails_root(root) {
        return ta("adapter-rails-failed", targs!("error" => error.to_string()));
    }
    ta("adapter-ruby-failed", targs!("error" => error.to_string()))
}

fn runner_failed_instead(root: &Path) -> String {
    if rails_root(root) {
        return t("adapter-rails-failed-instead");
    }
    t("adapter-ruby-failed-instead")
}

/// Runs exactly the tagged test (`ruby -Itest <file> -n <method>`, or
/// `bin/rails test <file> -n <method>` where the project is Rails).
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
    // The encoding of the arguments is the adapter's word to its
    // child, not the machine's locale: under `LANG=` ruby read `-n
    // test_ünïcode` in ASCII-8BIT and selected nothing (global review
    // 2026-09-06 R-19; wave 0051), while `-E UTF-8` runs it. Where
    // the project is Rails, the same selection rides on `bin/rails
    // test <file> -n <method>` -- measured on a real application,
    // exit 0 green and 1 red.
    let mut command = minitest_command(
        root,
        &[
            relative.display().to_string(),
            "-n".to_string(),
            tag.test.clone(),
        ],
    );
    // The first reading forgot to forget the hook (global review
    // 2026-09-06, bugs cut R-18): a test that asks git for its
    // repository saw the hook's under GIT_DIR.
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: runner_failed(root, &e),
        instead: runner_failed_instead(root),
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
        // One process per file on both roads, so the key of the
        // battery stays the file it came from (§7.13's verdicts are
        // per test, and the courts above ask by file stem).
        let mut command =
            minitest_command(root, &[relative.display().to_string(), "-v".to_string()]);
        crate::scope::forget_the_hook(&mut command);
        let run = command.output().map_err(|e| Refusal {
            file: root.to_path_buf(),
            reason: runner_failed(root, &e),
            instead: runner_failed_instead(root),
        })?;
        let voice = String::from_utf8_lossy(&run.stdout).into_owned();
        // Both streams for the courts that ask what ruby SAID -- a
        // LoadError arrives on stderr -- and stdout alone for the
        // roll. minitest writes its verdicts to stdout, and splicing
        // the streams let a verdict-shaped line on stderr take a
        // place in the sequence that was never its own (review 0074
        // R2-5).
        let said = format!("{voice}{}", String::from_utf8_lossy(&run.stderr));
        let roll = roll_of(&voice);
        let mut ran = 0usize;
        for (name, mark) in roll.verdicts.iter().cloned() {
            ran += 1;
            // A skipped test did not run: it is in the battery neither
            // as green nor as red, exactly as python's and node's
            // skips are not (waves 0045, 0046) -- and the closing
            // court then says "did not run" of the promise it was to
            // prove (wave 0055).
            let green = match mark {
                Mark::Green => true,
                Mark::Fallen => false,
                Mark::Skipped => continue,
            };
            // Two classes in one file may name a method alike. The
            // safe direction joins them: green only if both were.
            let key = (stem.clone(), name);
            let held = out.get(&key).copied().unwrap_or(true);
            out.insert(key, held && green);
        }
        if ran == 0 {
            // Nothing ran. Either the file declares no test at all --
            // which is not a fault, and minitest still says `0 runs`
            // -- or it did not load, or minitest never ran because
            // nothing required `minitest/autorun` (bugs cut R-7): in
            // the last two there is no verdict for anyone, and the
            // refusal says which.
            match classify(&said, run.status.success()) {
                crate::adapter::Outcome::BuildBroken(words) => {
                    return Err(Refusal {
                        file: file.clone(),
                        reason: ta("adapter-ruby-broken", targs!("error" => words)),
                        instead: t("adapter-ruby-broken-instead"),
                    });
                }
                crate::adapter::Outcome::NotRun if !summarised(&said) => {
                    return Err(Refusal {
                        file: file.clone(),
                        reason: ta(
                            "adapter-ruby-silent",
                            targs!("file" => relative.display().to_string()),
                        ),
                        instead: t("adapter-ruby-silent-instead"),
                    });
                }
                _ => {}
            }
        }
        // ...and only where the file ran at all. A file that did
        // not load, or that never required `minitest/autorun`, has
        // its own refusal below with ruby's own words, and it is the
        // truer one: the roll courts would otherwise shout "a test
        // was named and never judged" over a LoadError (review 0074
        // R-4). The order is the whole of that fix.
        // A name opened and never closed: the reader cannot say what
        // that test came to, and neither can anybody else. Refusing
        // is the only honest answer -- and it is the ONLY guard
        // against a forged verdict line, which swaps a real test for
        // a ghost one for one and leaves every count agreeing.
        if roll.abandoned > 0 {
            return Err(Refusal {
                file: file.clone(),
                reason: ta(
                    "adapter-ruby-roll-lost",
                    targs!(
                        "file" => relative.display().to_string(),
                        "count" => roll.abandoned as u64
                    ),
                ),
                instead: t("adapter-ruby-roll-lost-instead"),
            });
        }
        // The roll against the runner's own count (wave 0074, issue
        // #55). Minitest says `N runs,` and keel never compared: a
        // roll that had lost two tests looked exactly like a roll of
        // the right length, and the battery reported a number that
        // was simply untrue. Both directions are a refusal -- short
        // means the reader could not name something that ran, long
        // means something named itself that did not.
        if let Some(runs) = runs_said(&said).filter(|runs| *runs != roll.verdicts.len() as u64) {
            return Err(Refusal {
                file: file.clone(),
                reason: ta(
                    "adapter-ruby-roll",
                    targs!(
                        "file" => relative.display().to_string(),
                        "said" => runs,
                        "read" => roll.verdicts.len() as u64
                    ),
                ),
                instead: t("adapter-ruby-roll-instead"),
            });
        }
    }
    Ok(out)
}

/// What one `-v` line came to: green, fallen, or skipped -- and a
/// skip is its own state, not a red one. Minitest's `S` was read as
/// "not the dot" and so as red, and `keel close` called a skipped
/// test one that "fell in every run" (final review 2026-09-06, bugs
/// R-2, R-25; wave 0055).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mark {
    Green,
    Fallen,
    Skipped,
}

/// The mark minitest ends a verdict line with, and only those four:
/// the bare dot is green, `F` failed, `E` errored, `S` was skipped,
/// and none of the last three proves a promise.
///
/// Strict on purpose (wave 0074). The reading before this took
/// ANYTHING that was not `.` or `S` for a failure, which is fine
/// while the line is whole -- and the line is not always whole. Once
/// a reader starts piecing a verdict together out of two lines, a
/// loose mark turns every stray `x = y` into a fallen test. What the
/// strictness costs is named and paid elsewhere: a mark we do not
/// know makes the roll shorter than the runner's own count, and the
/// court says THAT aloud instead of guessing.
fn mark_of(mark: &str) -> Option<Mark> {
    match mark.trim() {
        "." => Some(Mark::Green),
        "S" => Some(Mark::Skipped),
        "F" | "E" => Some(Mark::Fallen),
        _ => None,
    }
}

/// The name a verdict line opens with: `ToyTest#test_it_works`, taken
/// from the START of the line and not from what is left after the
/// marks are cut off the end.
///
/// That direction is the whole fix of issue #55. A test that prints
/// while it runs lands INSIDE its own verdict line -- minitest writes
/// `Class#method = `, the test's own output goes next, then the
/// timing and the mark -- and a reader working backwards from the end
/// walked into whatever the test had printed. Measured: a print
/// containing ` = ` moved the second split and the test vanished from
/// the battery.
fn head_name(line: &str) -> Option<String> {
    let (name, _) = line.trim_start().split_once(" = ")?;
    if name.split_whitespace().count() != 1 {
        return None;
    }
    let method = name.rsplit_once('#')?.1;
    if method.is_empty() {
        return None;
    }
    Some(method.to_string())
}

/// A whole verdict on one line: a name at the front, a timing and a
/// mark at the back, and anything at all in between.
fn verdict_in(line: &str) -> Option<(String, Mark)> {
    let trimmed = line.trim();
    let name = head_name(trimmed)?;
    let (middle, mark) = trimmed.rsplit_once(" = ")?;
    if !middle.trim_end().ends_with(" s") {
        return None;
    }
    Some((name, mark_of(mark)?))
}

/// The BACK of a verdict whose front is on an earlier line: a timing
/// and a mark, and no name of its own.
///
/// This is the other half of the same wound. A test that prints a
/// line -- `puts`, which is what a system test's driver does -- cuts
/// minitest's verdict in two: the name stays on the first line, the
/// timing and mark begin the next. Neither half is a verdict, and the
/// test disappeared.
fn tail_mark(line: &str) -> Option<Mark> {
    let trimmed = line.trim();
    let (timing, mark) = trimmed.rsplit_once(" = ")?;
    // What tells a tail from a WHOLE verdict is a second ` = `, not a
    // `#` (review 0074 R2-2). The first cut refused any tail carrying
    // a hash, so `print "issue #55"` before the timing hid the real
    // verdict from the guard -- and a test could close itself green
    // behind that one character. A hash is a thing a test may print;
    // a second ` = ` is minitest's own shape for "name = timing =
    // mark", and that is the line that is not a tail.
    if !timing.trim_end().ends_with(" s") || timing.contains(" = ") {
        return None;
    }
    mark_of(mark)
}

/// Everything minitest's `-v` voice says it ran, in order -- and how
/// many names it opened and never closed.
///
/// A name with no verdict yet is held until its timing arrives:
/// whatever the test printed in between belongs to the test, not to
/// the roll. A name that never gets its timing is ABANDONED, and that
/// is counted rather than shrugged off.
///
/// Abandonment is the one thing the count against `N runs` cannot
/// see, and it took a probe to find out. A test that prints a whole
/// verdict line of its own -- `puts` of `Ghost#test_x = 0.00 s = .`
/// -- makes the reader close a test that never ran and lose the one
/// that did, ONE FOR ONE: the roll stays exactly as long as the
/// runner's count, and every number agrees while the names are wrong.
/// So the reader says when it abandoned a name, and the court refuses
/// on that alone.
struct Roll {
    verdicts: Vec<(String, Mark)>,
    abandoned: usize,
}

fn roll_of(said: &str) -> Roll {
    let mut out: Vec<(String, Mark)> = Vec::new();
    let mut waiting: Option<String> = None;
    let mut broken = 0usize;
    for line in said.split(['\n', '\r']) {
        // While a name is open, ONLY its timing closes it. Everything
        // else on the way is the running test's own output, whatever
        // it looks like -- and it can look like anything.
        //
        // That rule is the answer to two measured faults at once
        // (review 0074 R-2, R-3). A test printing an ordinary line
        // with a `#` and a ` = ` in it -- `User#full_name = Jane` --
        // is not a second test starting, and the reading that took it
        // for one failed the whole battery and blamed the project.
        // And a test printing a whole verdict of its own is not a
        // test either; minitest is serial, so between two real names
        // there is always a timing.
        if waiting.is_some() {
            if let Some(mark) = tail_mark(line) {
                let name = waiting.take().expect("open");
                out.push((name, mark));
            }
            continue;
        }
        if let Some((name, mark)) = verdict_in(line) {
            out.push((name, mark));
            continue;
        }
        // A timing with no name open. Minitest never writes one: it
        // is a test's own output, and it means a test's real timing
        // has already been eaten by that same output -- the verdict
        // above belongs to nobody, or to the wrong body.
        //
        // This is the guard that catches the forge, and it took the
        // reviewer to find it: the first reading dropped an orphan
        // tail in silence, so a test printing `0.00 s = .` closed
        // ITSELF green and its real `0.00 s = F` went out with the
        // rubbish. Every count agreed. A false green is the one
        // answer §4.10 calls worse than a red.
        if tail_mark(line).is_some() {
            broken += 1;
            continue;
        }
        if let Some(name) = head_name(line) {
            waiting = Some(name);
        }
    }
    if waiting.is_some() {
        broken += 1;
    }
    Roll {
        verdicts: out,
        abandoned: broken,
    }
}

/// How many tests minitest itself says it ran: the `N runs,` of its
/// summary line.
///
/// The number keel's own roll is measured against (wave 0074). Before
/// this the summary was read for one thing only -- whether it said
/// zero -- so a roll that had silently lost two tests looked exactly
/// like a roll of the right length.
fn runs_said(said: &str) -> Option<u64> {
    // The LAST such line, and only one carrying minitest's own
    // neighbours (review 0074 R2-4): a test printing `5 runs, …` of
    // its own hijacked the first match, and the court then compared
    // the roll against a number the test had chosen.
    said.lines()
        .filter(|line| line.contains(" assertions,") && line.contains(" failures,"))
        .filter_map(|line| {
            let (count, _) = line.trim_start().split_once(" runs,")?;
            count.parse().ok()
        })
        .next_back()
}

/// What a run came to, read from what ruby said.
///
/// The order of the questions is the whole of it (review 0038 R-2):
/// minitest leaves with **0** when `-n` names a method it does not
/// know, so asking the exit code first turned "nothing ran" into
/// "green" and let work through a gate over a test that never ran.
/// What ruby said is asked before how it left.
pub fn classify(said: &str, success: bool) -> crate::adapter::Outcome {
    // Nothing ran: the file did not parse, or a require failed --
    // known by the SHAPE of ruby's own line, not by a word in it:
    // `flunk "this is not a LoadError"` said the word in a failure
    // message and was read as a broken build (global review
    // 2026-09-06, bugs R-22; wave 0051).
    if let Some(line) = said.lines().find(|line| broken_line(line)) {
        return crate::adapter::Outcome::BuildBroken(line.trim().to_string());
    }
    // Minitest ran and named nothing: the method does not exist. The
    // SUMMARY line says it -- `0 runs, 0 assertions, …` -- and only
    // that line: minitest also prints its speed, `995.6450 runs/s`,
    // and one run in ten ends in `0 runs/s`, which a substring match
    // read as "nothing ran" and the gate refused a green test
    // (review 0047 R-4; the flaw was wave 0038's).
    if said
        .lines()
        .any(|line| line.trim_start().starts_with("0 runs,"))
    {
        return crate::adapter::Outcome::NotRun;
    }
    // Every run was a skip: `1 runs, 0 assertions, 0 failures, 0
    // errors, 1 skips`, and minitest leaves with 0. A skip is not a
    // run -- `skip "later"` in the one test `-n` named blessed work at
    // the gate (final review 2026-09-06, bugs R-2; wave 0055).
    if all_skipped(said) {
        return crate::adapter::Outcome::NotRun;
    }
    // No summary line at all and a clean exit: minitest never ran.
    // Without `minitest/autorun` ruby loads the file, defines the
    // class, says nothing and leaves with 0 -- and the first reading
    // called that green (global review 2026-09-06, bugs cut R-7).
    // Silence is "nothing ran", in both courts. A silent exit that
    // is NOT clean stays a failure: the direction that cannot turn
    // red into green (§7.12).
    if success && !summarised(said) {
        return crate::adapter::Outcome::NotRun;
    }
    if success {
        return crate::adapter::Outcome::Green;
    }
    crate::adapter::Outcome::Failed
}

/// Whether a line is ruby's own word about a build that broke: the
/// exception class in parentheses at the end (`… (LoadError)`, `…
/// (SyntaxError)`), the parser's `: syntax error` and the loader's
/// `cannot load such file --`. A failure message that merely contains
/// the word is a failure. The border is text: a test that prints
/// ruby's very shape reads as a broken build, and is named.
fn broken_line(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with("(LoadError)")
        || trimmed.ends_with("(SyntaxError)")
        || trimmed.contains(": syntax error")
        || trimmed.contains("syntax errors found")
        || trimmed.contains("cannot load such file --")
}

/// Whether minitest spoke its summary line at all -- `3 runs, 3
/// assertions, 0 failures, …` -- the one line every run prints, a run
/// of zero tests included.
/// Whether minitest's summary line says every run was a skip:
/// `N runs, …, N skips` with N above zero. Only that line: the speed
/// line says `runs/s` and never `runs, `.
fn all_skipped(said: &str) -> bool {
    said.lines().any(|line| {
        let Some((runs, rest)) = line.trim_start().split_once(" runs, ") else {
            return false;
        };
        let Ok(runs) = runs.parse::<usize>() else {
            return false;
        };
        let skips = rest
            .rsplit(", ")
            .next()
            .and_then(|last| last.trim_end().strip_suffix(" skips"))
            .and_then(|n| n.parse::<usize>().ok());
        runs > 0 && skips == Some(runs)
    })
}

fn summarised(said: &str) -> bool {
    said.lines().any(|line| {
        let trimmed = line.trim_start();
        let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
        digits > 0 && trimmed[digits..].starts_with(" runs, ")
    })
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
    let mut text = voice.to_string();
    if let Some(messages) = parsed["messages"].as_array() {
        for message in messages.iter().filter_map(|m| m.as_str()) {
            text.push('\n');
            text.push_str(message);
        }
    }
    // Colour out of BOTH voices: ruby paints the syntax error inside
    // rspec's `messages` as well as on stdout (review 0047 R-3).
    let text = strip_ansi(&text);
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
/// This run's own directory in the system's temp dir, removed when
/// the run is over -- however it is over.
struct Home(std::path::PathBuf);

impl Drop for Home {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// What one rspec run said: its JSON (from the file), and its voice
/// on stdout and stderr, where a load error is narrated.
struct Said {
    json: String,
    voice: String,
}

fn rspec(root: &Path, args: &[String]) -> Result<Said, Refusal> {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    // A directory of this run's own, created exclusively -- a file
    // under a name anyone could have set up in advance in the shared
    // temp dir was the classic race (global review 2026-09-06 R-26;
    // wave 0051). `create_dir` refuses what already stands there, a
    // symlink included, and that refusal is said aloud.
    let home = std::env::temp_dir().join(format!(
        "keel-rspec-{}-{}",
        std::process::id(),
        COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&home).map_err(|e| Refusal {
        file: home.clone(),
        reason: ta("adapter-rspec-tmp", targs!("error" => e.to_string())),
        instead: t("adapter-rspec-tmp-instead"),
    })?;
    // And gone with the run whichever way the run ends: the first
    // reading removed it after a successful launch only, and a
    // refusal -- rspec not on PATH -- left it standing in the shared
    // temp dir (review 0051 R-5). The guard removes it on every road
    // out of this function, the `?` ones included.
    let home = Home(home);
    let out_file = home.0.join("out.json");
    let mut command = Command::new("rspec");
    command
        .args(["--format", "json", "--out"])
        .arg(&out_file)
        .arg("--no-color")
        // The project's own `.rspec` and nothing else: without `-O`
        // rspec also reads `~/.rspec` and `./.rspec-local` -- the
        // machine's and the person's files, not the project's -- and
        // a `--dry-run` in either made every example "passed" without
        // running one (global review 2026-09-06, bugs cut R-6).
        // Measured on RSpec 3.13: with `--options .rspec` only the
        // project's file is read, and its absence is not an error.
        .args(["--options", ".rspec"])
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
    drop(home);
    let voice = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    match json {
        Ok(json) => Ok(Said { json, voice }),
        // rspec started and left without its JSON: an `abort` in
        // spec_helper, an `exit` in a config -- its own words say
        // which, and "put rspec on PATH" is not the answer (review
        // 0047 R-9).
        Err(_) => Err(Refusal {
            file: root.to_path_buf(),
            reason: ta(
                "adapter-rspec-silent",
                targs!("error" => strip_ansi(voice.trim())),
            ),
            instead: t("adapter-rspec-silent-instead"),
        }),
    }
}
