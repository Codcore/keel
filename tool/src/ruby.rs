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

/// The ruby that prints the roll BEFORE running anything, then runs.
///
/// Three rounds of review taught this wave one thing: minitest's `-v`
/// line cannot bear the weight put on it. Whatever character was
/// chosen to tell a real verdict from one a test printed -- a `#`, a
/// second ` = ` -- a test could print exactly that and close itself
/// green (reviews 0074 R-2, R2-1, R3-1). The format is not the place
/// to look.
///
/// So the roll is taken where no test body has run yet: the file is
/// LOADED, `Minitest::Runnable.runnables` is asked what it holds, and
/// the names are printed. `minitest/autorun`'s `at_exit` then runs the
/// tests as usual, in the same process -- no second run, no second
/// cost. A test can print anything it likes afterwards; it cannot add
/// a name to a list written before it existed, nor take its own off.
///
/// No mark of ours closes the report any more. Three rounds held one
/// and three more found a way round it -- a hook of the test body, a
/// `prepend` the FILE makes at load time (last in the chain, inside
/// our own), a reporter appended to minitest's, the mark read back out
/// of the process (reviews 0074 rounds six, seven and eight). The
/// region between marks was never minitest's alone, so the verdicts
/// are no longer read from a region at all: the whole voice is read,
/// and the two readings of it are joined in the safe direction (see
/// `roll_of`).
fn minitest_listing(root: &Path, file: &str) -> Command {
    if rails_root(root) {
        // Rails boots the application and owns the run; the listing
        // is not available on that road, and the courts below say so
        // rather than pretending otherwise.
        return minitest_command(root, &[file.to_string(), "-v".to_string()]);
    }
    let mut command = Command::new("ruby");
    command
        .args(["-E", "UTF-8"])
        .arg("-Itest")
        .arg("-e")
        .arg(concat!(
            // `$0` first: a test file may ask whether it is the
            // file being run, and under `-e` it is not, unless we
            // say so.
            "$0 = ARGV.first\n",
            "load ARGV.shift\n",
            // minitest is NOT required here, and that is the
            // point: requiring it before the project's own file
            // activates the system gem, and a project holding its
            // own version through bundler then meets
            // `Gem::LoadError` in keel's preamble rather than in
            // its own code (review 0074 R5-11). Every minitest
            // file requires minitest itself -- that is what makes
            // it one -- so by this line it is either loaded or
            // there are no tests to name.
            "if defined?(Minitest::Runnable)\n",
            "  Minitest::Runnable.runnables.each do |r|\n",
            "    r.runnable_methods.each { |m| puts \"KEEL-ROLL #{r}##{m}\" }\n",
            "  end\n",
            "end\n",
        ))
        .arg(file)
        .arg("-v")
        .current_dir(root);
    command
}

/// The names the file declared, read before any of them ran -- and
/// kept WHOLE, `Class#method`.
///
/// Dropping the class was the root of a wrong accusation (review 0074
/// round six): with only the method left, a block's line could not be
/// matched exactly and had to be guessed at by substring, and a file
/// declaring both `test_boom` and `test_boom:` made the guess pick
/// the innocent one -- the test that threw went into the battery
/// green. Whole, the match is an equality.
fn listed(said: &str) -> Vec<String> {
    said.lines()
        .filter_map(|line| line.trim().strip_prefix("KEEL-ROLL "))
        .map(|full| full.trim_end_matches('\r').to_string())
        .filter(|full| full.split_once('#').is_some_and(|(_, m)| !m.is_empty()))
        .collect()
}

/// The method's own name out of `Class#method` -- the name a tag
/// writes, `-n` selects by, and the battery is keyed on.
///
/// Split at the FIRST `#`, not the last: a class name cannot hold
/// one, and a METHOD name can. minitest's spec style makes them --
/// `it "#add works"` declares `test_0001_#add works` -- and splitting
/// from the right put that test in the battery under `add works`,
/// which is not a name anything selects by.
fn method_of(full: &str) -> String {
    full.split_once('#').map_or(full, |(_, m)| m).to_string()
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
pub fn run_all(root: &Path) -> Result<crate::adapter::Ran, Refusal> {
    let mut out = crate::adapter::Ran::default();
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
        // What the PROCESS said, kept apart and kept raw. The voice
        // built from the JSON is this road's named exception, and it
        // is not the whole truth about a red file: a test that prints
        // -- or whose child process does -- says it here and nowhere
        // in the document (review 0071 round five, measured: nought
        // occurrences of either). `--format json --out <file>` sends
        // the report to a file, so this stream carries nothing but
        // what the project itself wrote.
        let printed = said.voice.clone();
        for example in examples(&said.json) {
            let green = match example.status.as_str() {
                "passed" => true,
                "failed" => false,
                _ => continue,
            };
            // Two examples of one full description are one key, red
            // if either is (the rule review 0046 R-2 asked to be one
            // in both courts).
            // rspec is the one named exception of this wave: its
            // JSON goes to a FILE, and what that file holds is every
            // example, green ones too. There is no raw voice to keep
            // whole, and keeping the document would put the data of
            // examples that did not fail into the log. So the voice
            // is built from the field keel already parses --
            // `exception.message` of the RED examples, and nothing
            // else.
            if !green {
                let said = example.message.clone().unwrap_or_default();
                let voice = out.voices.entry(key.clone()).or_default();
                if !said.is_empty() {
                    if !voice.is_empty() {
                        voice.push_str("\n\n");
                    }
                    voice.push_str(&format!("{}: {said}", example.description));
                }
            }
            out.verdicts
                .entry((key.clone(), example.description))
                .and_modify(|was| *was = *was && green)
                .or_insert(green);
        }
        // ...and the process's own words under the built ones, where
        // the file had a red and the process said anything at all.
        if !printed.trim().is_empty()
            && let Some(voice) = out.voices.get_mut(&key)
        {
            if !voice.is_empty() {
                voice.push_str("\n\n");
            }
            voice.push_str(&t("adapter-rspec-printed"));
            voice.push('\n');
            voice.push_str(&printed);
        }
    }
    for file in minitest_files(root)? {
        let relative = file.strip_prefix(root).unwrap_or(&file);
        let stem = crate::adapter::battery_key(root, &file);
        // One process per file on both roads, so the key of the
        // battery stays the file it came from (§7.13's verdicts are
        // per test, and the courts above ask by file stem).
        let mut command = minitest_listing(root, &relative.display().to_string());
        crate::scope::forget_the_hook(&mut command);
        let run = command.output().map_err(|e| Refusal {
            file: root.to_path_buf(),
            reason: runner_failed(root, &e),
            instead: runner_failed_instead(root),
        })?;
        let voice = String::from_utf8_lossy(&run.stdout).into_owned();
        // Both streams for the courts that ask what ruby SAID -- a
        // LoadError arrives on stderr -- and stdout alone for the roll
        // and the verdicts. minitest writes its verdicts to stdout,
        // and splicing the streams let a verdict-shaped line on stderr
        // take a place in the sequence that was never its own (review
        // 0074 R2-5).
        let said = format!("{voice}{}", String::from_utf8_lossy(&run.stderr));
        // The road picks the reader, and it is asked in the open:
        // the plain road has a roll, the Rails road does not (see
        // `minitest_listing`), and no reading falls back to another
        // one without saying so.
        //
        // The plain road reads the WHOLE voice -- no mark closes the
        // report (see `minitest_listing` for why a mark had to go) --
        // and joins two readings of it in the safe direction. The
        // courts below ask their own questions of the same voice.
        let rails = rails_root(root);
        let names = if rails { Vec::new() } else { listed(&voice) };
        let no_roll = !rails && names.is_empty();
        let roll = if rails {
            shapes_of(&voice)
        } else {
            roll_of(&voice, &names)
        };
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
            let held = out.verdicts.get(&key).copied().unwrap_or(true);
            out.verdicts.insert(key, held && green);
            if !green {
                // One process per file on this road: what minitest
                // said while that file ran IS the file's voice.
                out.voices
                    .entry(stem.clone())
                    .or_insert_with(|| said.clone());
            }
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
        // A name the file declared and the run never judged. With
        // the roll written before any test body ran, this is the
        // whole of the guard: a test cannot take its own name off the
        // list, so a name with no verdict means its verdict was lost
        // -- swallowed by the test's own output, or never printed.
        // minitest ran tests and keel has no roll of them. The file
        // swallowed the roll -- a `$stdout` replaced while it loads,
        // put back before the run -- and the older reader is not
        // quietly put in its place (review 0074 R5-2).
        // ruby left with a failure and keel read no verdict from it.
        // Before this the battery simply said "0 tests" and the wave
        // closed -- measured on a file that muffles STDOUT and never
        // puts it back, and on one that raises at the end of its own
        // load (review 0074 R5-13 and round six). A red exit is a red
        // exit: what cannot be read is refused, not counted as
        // nothing.
        // Asked of minitest's OWN stream. A file that muffles stdout
        // and writes a summary-shaped line to stderr would otherwise
        // buy itself a quiet nought (measured).
        if ran == 0 && !run.status.success() && !summarised(&voice) {
            return Err(Refusal {
                file: file.clone(),
                reason: ta(
                    "adapter-ruby-mute",
                    targs!("file" => relative.display().to_string()),
                ),
                instead: t("adapter-ruby-mute-instead"),
            });
        }
        if no_roll && let Some(runs) = runs_said(&voice).filter(|runs| *runs > 0) {
            return Err(Refusal {
                file: file.clone(),
                reason: ta(
                    "adapter-ruby-roll-none",
                    targs!(
                        "file" => relative.display().to_string(),
                        "said" => runs
                    ),
                ),
                instead: t("adapter-ruby-roll-none-instead"),
            });
        }
        // A verdict cut in two, on the road that has nothing else to
        // go on. Only `shapes_of` -- the Rails road -- can say this:
        // where there is a roll, a name with no block of its own is
        // simply green, and the two courts below hold the counts.
        if !roll.silent.is_empty() {
            return Err(Refusal {
                file: file.clone(),
                reason: ta(
                    "adapter-rails-shape",
                    targs!("file" => relative.display().to_string()),
                ),
                instead: t("adapter-rails-shape-instead"),
            });
        }
        // The roll against the runner's own count (wave 0074, issue
        // #55). Minitest says `N runs,` and keel never compared: a
        // roll that had lost two tests looked exactly like a roll of
        // the right length, and the battery reported a number that
        // was simply untrue. Both directions are a refusal -- short
        // means the reader could not name something that ran, long
        // means something named itself that did not.
        if let Some(runs) = runs_said(&voice).filter(|runs| *runs != roll.verdicts.len() as u64) {
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
        // The report against minitest's own totals. The verdicts are
        // read from the numbered blocks written AFTER the run; the
        // counts are written after them again. A test that forges a
        // block has to make the totals agree as well, and it cannot:
        // by the time it prints, the line that counts it has not
        // been written yet.
        //
        // No mark closes the report, so "the totals" is not one line
        // any more: the file may print its own summary-shaped line
        // -- during the run, or after minitest's own report, from a
        // hook or a reporter of its own -- and the LAST summary in
        // the voice is then the forger's, not minitest's. What cannot
        // be forged away is minitest's own summary itself: minitest
        // prints it after every run, and whatever is printed later
        // stands BESIDE it, not in its place. So the court asks
        // whether ANY summary line in the voice agrees with the
        // blocks -- one honest line is enough, and a forger who
        // prints an agreeing one has not yet won anything: the
        // verdicts themselves are joined in the safe direction, and
        // a fallen test the stream itself marked `F` stays red no
        // matter what the blocks and the numbers around it say.
        if !rails
            && !no_roll
            && let Some((fallen, skipped)) = roll.tally
        {
            let counted = summaries(&voice);
            let agreed = counted.iter().any(|(_, failures, errors, skips)| {
                fallen == failures + errors && skipped == *skips
            });
            if !agreed {
                // Every summary in the voice disagrees with the
                // blocks -- or there is no summary at all, which is
                // its own refusal: without minitest's own line the
                // blocks have no count to answer to, and blocks are
                // exactly what a test can write.
                if counted.is_empty() {
                    return Err(Refusal {
                        file: file.clone(),
                        reason: ta(
                            "adapter-ruby-report-none",
                            targs!("file" => relative.display().to_string()),
                        ),
                        instead: t("adapter-ruby-report-none-instead"),
                    });
                }
                let (_, failures, errors, skips) = *counted.last().expect("not empty");
                return Err(Refusal {
                    file: file.clone(),
                    reason: ta(
                        "adapter-ruby-report",
                        targs!(
                            "file" => relative.display().to_string(),
                            "counted" => failures + errors,
                            "blocks" => fallen,
                            "skips" => skips,
                            "blockskips" => skipped
                        ),
                    ),
                    instead: t("adapter-ruby-report-instead"),
                });
            }
        }
        // The last question, and the plainest: the runner's own voice
        // must have mentioned every name the roll holds. Under `-v`
        // minitest writes the name of every test it runs; keel's own
        // roll lines are cut out of the voice before the question, so
        // what must answer is the RUNNER. A name the runner never
        // said has no verdict from anybody -- the run and the roll are
        // not about the same tests, and reading the blocks as green
        // over it would be a green nobody earned. Measured shape: a
        // file prints its own summary and leaves by `exit!`, which
        // runs no `at_exit` at all -- minitest never ran, and without
        // this question the forged summary closed the file green.
        if !rails && !no_roll {
            let runner = runner_voice(&voice);
            if let Some(name) = names.iter().find(|name| !runner.contains(name.as_str())) {
                return Err(Refusal {
                    file: file.clone(),
                    reason: ta(
                        "adapter-ruby-unsaid",
                        targs!(
                            "file" => relative.display().to_string(),
                            "name" => name
                        ),
                    ),
                    instead: t("adapter-ruby-unsaid-instead"),
                });
            }
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
    // No guard on what stands before the timing any more, and that is
    // the point of the roll: three rounds of review each defeated the
    // character chosen here -- a `#`, then a second ` = ` -- because a
    // test can print whatever the guard forbids. The span decides
    // whose timing this is, and the LAST one in a span is the
    // runner's.
    if !timing.trim_end().ends_with(" s") {
        return None;
    }
    mark_of(mark)
}

/// The verdicts of the tests the file DECLARED -- joined from the
/// report minitest writes AFTER the run and from the `-v` stream it
/// writes during it, in the safe direction (`roll_of`).
///
/// Four rounds of review taught this wave its lesson twice over.
/// First: the shape of a `-v` line cannot be trusted, because a test
/// can print one (R-2, three times). Then: under `parallelize_me!`
/// -- which Rails turns on by default, and which is the environment
/// issue #55 came from -- the `-v` stream is not merely forgeable,
/// it is PHYSICALLY interleaved:
///
/// ```text
/// ToyTest#test_one = ToyTest#test_three = 0.00 s = .
/// ToyTest#test_two = 0.00 s = .
/// 0.00 s = F
/// ```
///
/// No line-ordered reader survives that, and no cleverness will.
///
/// What minitest writes after the run does survive it: a numbered
/// block per test that did not simply pass, naming it, and a summary
/// line counting them. So the verdicts come from two readings that
/// are JOINED, not from sources that must agree:
///
///   the ROLL      -- the names, taken when the file was loaded,
///                    before any test body could add or remove one;
///   the BLOCKS    -- which of those names failed, errored, skipped;
///   the STREAM    -- a second verdict on the same names, and only
///                    its red word is heard.
///
/// Everything not named in a block and not marked by the stream
/// passed. A test forging a skip over its own failure is beaten by
/// the stream's `F`; the counts the caller checks against are
/// minitest's own, and the direction that cannot happen is green
/// where a reading said red.
struct Roll {
    verdicts: Vec<(String, Mark)>,
    /// Names the reading could not account for. Only the Rails road
    /// fills this: it has no roll to compare against, so a verdict
    /// cut in two is all it has to go on.
    silent: Vec<String>,
    /// What minitest's own report came to: how many blocks said
    /// fallen, how many said skipped. `None` where there is no roll
    /// to read blocks against.
    tally: Option<(u64, u64)>,
}

/// The numbered blocks minitest writes after a run: `  1) Failure:`
/// and the `Class#method` on the line below it.
///
/// The name is taken FROM THE ROLL rather than parsed out of that
/// line. minitest's own spec style declares names with spaces in
/// them -- `a user of the shop#test_0001_greets by name` -- so there
/// is no rule about where a name ends that is both true and not
/// forgeable (review 0074 R5-5). The roll already holds the exact
/// strings; the block only has to say which of them it is, and a
/// block naming something not in the roll says nothing at all.
fn report_blocks(said: &str, names: &[String]) -> Vec<(Mark, String)> {
    let lines: Vec<&str> = said.split(['\n', '\r']).collect();
    let mut out: Vec<(Mark, String)> = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let Some((number, kind)) = trimmed.split_once(") ") else {
            continue;
        };
        if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let mark = match kind.trim() {
            "Failure:" | "Error:" => Mark::Fallen,
            "Skipped:" => Mark::Skipped,
            _ => continue,
        };
        let Some(named) = lines.get(at + 1) else {
            continue;
        };
        // `ToyTest#test_two [test/toy_test.rb:11]:` for a failure, or
        // `ToyTest#test_four:` for an error. The roll holds the whole
        // `Class#method`, so the block line must CARRY a roll name --
        // and where several do, the LONGEST is the one the block is
        // about: a block names its test as a PREFIX (`Class#method
        // [file:line]:`), so a file declaring `test_x` and
        // `test_x [foo]` puts both names in that line lawfully, and
        // the shorter winning sent the innocent test red while the one
        // that fell went green (review 0074 round seven). Where the
        // two readings differ from a plain containment match, the
        // stream has already named the guilty test on its own line --
        // measured, no shape tells them apart -- so the longest
        // carrier is the whole of the rule.
        let named = named.trim();
        let hit = names
            .iter()
            .filter(|name| named.contains(name.as_str()))
            .max_by_key(|name| name.len());
        if let Some(name) = hit {
            out.push((mark, name.clone()));
        }
    }
    out
}

/// minitest's own totals, as many summary lines as the voice holds:
/// runs, failures, errors, skips.
///
/// There is no mark closing the report, so there is no "THE summary
/// line" any more: a file may print a summary-shaped line of its own
/// while it runs, and again after minitest's own report. minitest's
/// own line is always among them -- it is written after every run,
/// and nothing printed later takes its place, only stands beside it
/// -- so the court asks whether ANY of these agrees with the blocks,
/// and takes the LAST for the numbers it quotes when none does.
fn summaries(said: &str) -> Vec<(u64, u64, u64, u64)> {
    said.lines()
        .filter(|line| line.contains(" assertions,") && line.contains(" failures,"))
        .filter_map(|line| {
            let mut runs = None;
            let mut failures = None;
            let mut errors = None;
            let mut skips = None;
            for part in line.split(',') {
                let part = part.trim();
                let (count, what) = part.split_once(' ')?;
                let count: u64 = count.parse().ok()?;
                match what {
                    "runs" => runs = Some(count),
                    "failures" => failures = Some(count),
                    "errors" => errors = Some(count),
                    "skips" => skips = Some(count),
                    _ => {}
                }
            }
            Some((runs?, failures?, errors?, skips?))
        })
        .collect()
}

/// The reading of the plain road: the names come from the roll, the
/// verdicts from TWO readings of the same voice, joined in the safe
/// direction.
///
/// The two readings are the report minitest writes after the run --
/// numbered blocks, one per test that did not simply pass -- and the
/// `-v` stream itself, which marks every test it ran. Neither can be
/// fenced off from the other: the file may print its own report-shaped
/// prose while it runs AND after the real report (from a hook, a
/// reporter or a `prepend` it hung on `Minitest.run` at load time --
/// reviews 0074 rounds six, seven and eight), so no ordering of "who
/// wrote last" holds. What holds is the DIRECTION of the join:
///
///   a name is FALLEN if either reading says so;
///   green only where NO reading says anything else.
///
/// A forgery that launders a failure into a skip (a forged
/// `1) Skipped:` block, later than the real one) is beaten by the
/// stream's own `F` on the same name. The price runs the other way
/// -- a test that prints a fallen verdict naming an innocent
/// neighbour can turn that neighbour red -- and that is the one
/// direction §7.12 calls affordable: red, said aloud and named, where
/// the trunk of the same tree would also have said red. It is named
/// in the wave card.
///
/// `None` means the roll itself is missing -- and that is never
/// quietly answered with the other reader: a file can swallow keel's
/// own lines (`$stdout` replaced while it loads is an ordinary way to
/// quieten a noisy boot), and a silent fall-back to the reader three
/// rounds of review defeated put the forgery back in business on the
/// road it was fixed on (review 0074 R5-2). The caller refuses
/// instead. An empty roll with no names -- a file that declares no
/// test -- is not missing: minitest says `0 runs` and means it.
fn roll_of(said: &str, names: &[String]) -> Roll {
    if names.is_empty() {
        return Roll {
            verdicts: Vec::new(),
            silent: Vec::new(),
            tally: None,
        };
    }
    let blocks = report_blocks(said, names);
    let stream = stream_fallen(said);
    // Green by absence is a verdict only where a run happened.
    // minitest prints a summary after EVERY run, a numbered block
    // after every test that did not simply pass, and a `-v` verdict
    // line for every test it ran. A voice holding none of these never
    // ran -- `require "minitest"` without `minitest/autorun` defines
    // the classes, prints nothing and leaves with 0 -- so the names
    // are a promise, not a result, and the empty roll sends the
    // caller down the "nothing ran" road where the honest refusal
    // lives (bugs cut R-7).
    if blocks.is_empty() && stream.is_empty() && summaries(said).is_empty() {
        return Roll {
            verdicts: Vec::new(),
            silent: Vec::new(),
            tally: None,
        };
    }
    let mut verdicts: Vec<(String, Mark)> = Vec::new();
    let mut block_marks: Vec<Mark> = Vec::new();
    for name in names {
        // The LAST block that names it. A test can print a block of
        // its own while it runs, and minitest writes its report after
        // every body has finished -- so where two blocks carry one
        // name, the real one is the later (review 0074 round six: a
        // forged `1) Skipped:` printed from a test body took a
        // failing test out of the battery altogether). The stream
        // stands beside this: wherever the last block was written by
        // somebody else, the stream's own `F` on the same name says
        // fallen all the same.
        let block = blocks
            .iter()
            .rev()
            .find(|(_, named)| named == name)
            .map(|(mark, _)| *mark)
            .unwrap_or(Mark::Green);
        block_marks.push(block);
        let mark = if block == Mark::Fallen || stream.iter().any(|named| named == name) {
            Mark::Fallen
        } else if block == Mark::Skipped {
            Mark::Skipped
        } else {
            Mark::Green
        };
        verdicts.push((method_of(name), mark));
    }
    // What the BLOCKS alone came to, for the court that compares the
    // report with minitest's own totals. The comparison is the
    // caller's, and it is said in its own words: a roll shorter than
    // the run and a report that does not add up are two different
    // faults, and answering both with one sentence about a "lost
    // verdict" sent the reader looking for the wrong thing (review
    // 0074 R5-8). Counted over the blocks' verdicts, one per NAME --
    // not over the raw blocks, and not over the JOINED verdicts: the
    // court asks about the report minitest wrote, and a false red the
    // stream added (an interleaved `-v` line under `parallelize_me!`
    // pairs one test's name with another's tail) must stay a red said
    // aloud, not turn a whole run into a refusal.
    let fallen = block_marks.iter().filter(|m| **m == Mark::Fallen).count() as u64;
    let skipped = block_marks.iter().filter(|m| **m == Mark::Skipped).count() as u64;
    Roll {
        verdicts,
        silent: Vec::new(),
        tally: Some((fallen, skipped)),
    }
}

/// The names the `-v` stream itself marked fallen: every whole
/// verdict line whose mark is `F` or `E`. Green and skipped lines say
/// nothing here -- a reading that could call a test green would be a
/// second mouth for the same stream three rounds of review defeated;
/// only the RED word of this stream is trusted, because red is the
/// direction a forgery cannot profit from (§7.12: the affordable
/// error is the false red, and it is named).
///
/// The name is the WHOLE `Class#method`. Who it names is the JOIN's
/// business, not this reader's: the caller matches against the roll
/// by equality (one home for the rule), so a ghost a test printed --
/// `GhostToyTest#test_one = 0.00 s = F` -- names nobody, and a mark
/// on a name that never ran says nothing about any test that did.
fn stream_fallen(said: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for line in said.split(['\n', '\r']) {
        let Some((name, mark)) = stream_verdict(line) else {
            continue;
        };
        if mark != Mark::Fallen {
            continue;
        }
        if !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// A whole verdict on one line of the `-v` stream, with the name kept
/// WHOLE, `Class#method`.
///
/// The name is the segment NEAREST the timing, and the timing must be
/// minitest's own shape -- a number and ` s`. Both details are the
/// parallel road's. Under `parallelize_me!` two tests' verdicts land
/// on ONE line -- `ToyTest#test_three = ToyTest#test_two = 0.00 s =
/// F` -- and the tail belongs to the NEARER name; reading the first
/// would mark the innocent one. A print that lands between the name
/// and the timing (`selenium: waiting0.00 s`) breaks the pairing
/// outright, and then the line says nothing to this reader at all --
/// the blocks carry the verdict.
fn stream_verdict(line: &str) -> Option<(String, Mark)> {
    let trimmed = line.trim();
    let (middle, mark) = trimmed.rsplit_once(" = ")?;
    let (front, timing) = middle.trim_end().rsplit_once(" = ")?;
    let body = timing.strip_suffix(" s")?;
    if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let name = front.rsplit_once(" = ").map_or(front, |(_, last)| last);
    Some((name.trim().to_string(), mark_of(mark)?))
}

/// The runner's own voice: everything on stdout except keel's own
/// roll lines. The court that asks whether the runner ever mentioned
/// a name must ask the RUNNER -- keel's `KEEL-ROLL` lines carry the
/// names too, and a roll the runner never echoed is exactly what the
/// court exists to catch.
fn runner_voice(said: &str) -> String {
    said.lines()
        .filter(|line| !line.trim_start().starts_with("KEEL-ROLL "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The older reading, by the shape of the line -- kept for the one
/// road that cannot give a roll: Rails boots the application and runs
/// the tests itself.
///
/// It keeps its own guard, and must (review 0074 R4-2): a timing with
/// no name open means a real verdict has already been eaten, and
/// dropping that guard here let a wave close over a failing test.
fn shapes_of(said: &str) -> Roll {
    let mut out: Vec<(String, Mark)> = Vec::new();
    let mut waiting: Option<String> = None;
    let mut broken = false;
    for line in said.split(['\n', '\r']) {
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
        if tail_mark(line).is_some() {
            broken = true;
            continue;
        }
        if let Some(name) = head_name(line) {
            waiting = Some(name);
        }
    }
    if waiting.is_some() {
        broken = true;
    }
    Roll {
        verdicts: if broken { Vec::new() } else { out },
        silent: if broken {
            vec![String::from("?")]
        } else {
            Vec::new()
        },
        tally: None,
    }
}

/// How many tests minitest itself says it ran: the `N runs,` of its
/// summary line.
///
/// The number keel's own roll is measured against (wave 0074). Before
/// this the summary was read for one thing only -- whether it said
/// zero -- so a roll that had silently lost two tests looked exactly
/// like a roll of the right length.
///
/// Asked of minitest's OWN stream, stdout, and never of the two
/// spliced together: stderr is appended after all of stdout, so the
/// last summary line of the splice is always the one a test wrote
/// there. Measured (review 0074 R5-3): one `warn "99 runs, …"` from
/// inside a test and the count court compared the roll against 99 --
/// enough to turn the verdict over and name an innocent test red.
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
        return crate::adapter::Outcome::BuildBroken(without_our_frame(line).to_string());
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
/// ruby's own line, with keel's preamble cut off the front of it.
///
/// The roll is taken by `ruby -e "… load ARGV.shift …"`, so ruby
/// blames the frame it was in: `-e:2:in 'load': test/toy_test.rb:5:
/// syntax error…`. The person reading the refusal wants their file,
/// not ours (review 0074 R5-11).
fn without_our_frame(line: &str) -> &str {
    let trimmed = line.trim();
    let Some(rest) = trimmed.strip_prefix("-e:") else {
        return trimmed;
    };
    match rest.find("': ") {
        Some(at) => rest[at + "': ".len()..].trim_start(),
        None => trimmed,
    }
}

fn broken_line(line: &str) -> bool {
    let trimmed = line.trim_end();
    trimmed.ends_with("(LoadError)")
        || trimmed.ends_with("(SyntaxError)")
        || trimmed.contains(": syntax error")
        || trimmed.contains("syntax errors found")
        || trimmed.contains("cannot load such file --")
}

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

/// Whether minitest spoke its summary line at all -- `3 runs, 3
/// assertions, 0 failures, …` -- the one line every run prints, a run
/// of zero tests included.
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
    /// What rspec said about a failed example, from its own JSON
    /// (wave 0071). This road has no raw voice to window: the JSON
    /// goes to a file and holds every example, green ones too, so a
    /// window over it would put into the log exactly what did not
    /// fail.
    message: Option<String>,
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
                        // The class and the message are asked
                        // INDEPENDENTLY. Mapping over the message
                        // alone threw away a class that was standing
                        // right beside it: an exception whose
                        // `message` method returns nil gives
                        // `message: null, class: "Silent"`, and the
                        // one word that made the battery red went out
                        // with the null (review 0071 round five).
                        message: {
                            let class = example["exception"]["class"].as_str().unwrap_or("");
                            let words = example["exception"]["message"].as_str().unwrap_or("");
                            match (class.is_empty(), words.is_empty()) {
                                (true, true) => None,
                                (true, false) => Some(words.to_string()),
                                (false, true) => Some(class.to_string()),
                                (false, false) => Some(format!("{class}: {words}")),
                            }
                        },
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
