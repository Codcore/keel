//! Scenario test of wave 0070: a red gate carries the words that made
//! it red.
//!
//! Issue #45, from a live project on keel 1.3.0: when the project's
//! gate fails, `keel close` prints one line and nothing else -- the
//! last non-empty line of the command's output. That project's
//! `bin/ci` is ten steps, so WHICH step failed is not in the log, and
//! a red CI run cannot be diagnosed from the run at all.
//!
//! The output is already in hand: `run_command` takes
//! `child.output()`, holds the whole of stdout and stderr, and throws
//! all but one line away.

mod common;

use common::{Sandbox, keel_sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn keel(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

/// Only stdout: the price line of the closing court goes to stderr,
/// and a package glued to it is no package.
fn keel_out(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn all_decided() -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        block.push_str(&format!(
            "  {cut}: \"n/a, бо ця пісочниця грає інший розріз\"\n"
        ));
    }
    block
}

/// A crate with one always-green test and a light wave, so nothing
/// but the gate can colour the exit.
fn project(name: &str, steps: &str) -> Sandbox {
    let ci = "sh bin/ci";
    let dir = keel_sandbox(name);
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    write(&dir, "tests/steady_test.rs", "#[test]\nfn steady() {}\n");
    write(&dir, "bin/ci", steps);
    write(
        &dir,
        "keel/waves/0022-tidy.md",
        &format!(
            "---\ntransforms:\n  tidy: {{chore: \"lad\", files: [src/lib.rs]}}\n{}---\n",
            all_decided()
        ),
    );
    write(
        &dir,
        "keel.toml",
        &format!(
            "lang = \"en\"\nadapter = \"rust\"\nci = \"{ci}\"\n\n[trust]\n\"{ci}\" = \"{}\"\n",
            keel::trust::fingerprint(ci)
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// A gate of several steps where the FAILING one is not the last to
/// speak -- the shape issue #45 met in the field, where `bin/ci`'s
/// last line came from tailwind and the failure came from rubocop.
/// A gate of several steps, written to a FILE and not inlined in the
/// command: the court prints the command by name, so a marker inside
/// the command text would be found in the report whether the output
/// was carried or thrown away. The first draft of this probe made
/// exactly that mistake and passed while the defect stood.
///
/// The failing step speaks EARLY and something else speaks last, on
/// both streams -- otherwise the one line the court keeps today would
/// be the useful one by luck.
const MANY_STEPS: &str = "echo step-one ok\necho RUBOCOP: 3 offenses detected >&2\necho tailwind: rebuilding >&2\necho Done in 116ms\nexit 1\n";

/// A gate that says something and passes -- the marker lives in the
/// file, not in the command, for the same reason.
const QUIET_STEPS: &str = "echo quiet-marker\nexit 0\n";

/// proves: a-red-gate-carries-the-words-that-made-it-red@372893
#[test]
fn a_red_gate_carries_the_words_that_made_it_red() {
    // --- the failing step is named, not swallowed ---
    let dir = project("gatewords", MANY_STEPS);
    let (out, code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a red gate blocks the wave (§7.16):\n{out}");
    assert!(
        out.contains("RUBOCOP: 3 offenses detected"),
        "and the report carries the line that made it red, not just \
         the last line the command printed:\n{out}"
    );
    // The line the old verdict quoted is the one that says nothing:
    // `Done in 116ms` looked like a duration and was tailwind's.
    assert!(
        !out.contains("(tailwind: rebuilding)"),
        "and the last line the command happened to print is no longer \
         the whole verdict:\n{out}"
    );

    // --- the window says it is a window ---
    assert!(
        out.contains("step-one ok"),
        "the earlier steps are in the window too -- the reader needs to \
         see how far the gate got:\n{out}"
    );

    // --- the window says it IS a window ---
    // A gate that speaks more than the window holds: the report must
    // carry both ends and say how much of the middle it cut, or the
    // window becomes the new silence -- longer, and still lying.
    let loud = "i=1\nwhile [ $i -le 200 ]; do echo line-$i; i=$((i+1)); done\nexit 1\n";
    let dir = project("gateloud", loud);
    let (out, code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a red gate blocks the wave:\n{out}");
    assert!(
        out.contains("line-1") && out.contains("line-200"),
        "both ends of a long output are in the report:\n{out}"
    );
    assert!(
        out.contains("cut from the middle") && out.contains("80 shown"),
        "and the window says BOTH how much it shows and how much it cut \
         -- a window that does not name itself is the new silence:\n{out}"
    );
    assert!(
        !out.contains("line-100"),
        "the middle really is cut, not merely announced:\n{out}"
    );

    // --- a green gate stays silent: success is silence (§7.16) ---
    let dir = project("gatequiet", QUIET_STEPS);
    let (out, code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "a green gate blocks nothing:\n{out}");
    assert!(
        !out.contains("quiet-marker"),
        "and a green gate's output is NOT poured into the report -- \
         success stays silence:\n{out}"
    );
}

/// proves: a-red-gate-carries-the-words-that-made-it-red@372893 -- the
/// promises the frame makes about itself.
///
/// Review 0070 measured eleven mutants against the first cut of this
/// work and five went through GREEN: the typed JSON field, the words
/// "unmasked", the NUMBER of cut lines, the name of the stream, and
/// the output of `verify`. Each of those is a promise the card makes
/// by name, and a promise no probe holds is a promise the next
/// refactor drops in silence. This is that probe.
#[test]
fn the_frame_keeps_the_promises_it_makes() {
    let dir = project("framewords", MANY_STEPS);
    let (out, code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a red gate blocks the wave:\n{out}");

    // The frame names WHICH stream it is quoting: rubocop spoke on
    // stderr and tailwind's timing on stdout, and a reader who cannot
    // tell them apart is reading one soup.
    assert!(
        out.contains("(stderr)") && out.contains("(stdout)"),
        "the frame names the stream it quotes:\n{out}"
    );
    // And it warns that the quote is unfiltered. The whole argument
    // of `security.confidentiality` -- that a masker weaker than
    // gitleaks gives a false calm -- rests on the tool saying so.
    assert!(
        out.contains("unmasked"),
        "and it warns that nothing in the quote is masked:\n{out}"
    );

    // The cut NUMBER is the number, not a decoration: a window that
    // says «some lines cut» is a window that can lie by drifting.
    let loud = "i=1\nwhile [ $i -le 200 ]; do echo line-$i; i=$((i+1)); done\nexit 1\n";
    let dir = project("framecount", loud);
    let (out, _) = keel(&["close", dir.to_str().unwrap()]);
    assert!(
        out.contains("120 cut from the middle"),
        "200 lines, 80 shown, so 120 remain in the middle of ONE stream \
         -- the number is counted, not decorative:\n{out}"
    );

    // A line longer than the cap is cut, and the cut says so IN the
    // line: a window that counts lines and not their length lets a
    // single three-megabyte line through whole (review 0070 R-2).
    let long = "python3 -c \"print('x'*5000)\"\nexit 1\n";
    let dir = project("framelong", long);
    let (out, _) = keel(&["close", dir.to_str().unwrap()]);
    assert!(
        out.contains("characters of this line cut"),
        "a line longer than the cap is cut and says so:\n{out}"
    );
    assert!(
        out.lines().all(|l| l.chars().count() < 1000),
        "and no line of the report is left unbounded:\n{}",
        out.lines().map(|l| l.chars().count()).max().unwrap_or(0)
    );

    // Output that was not valid UTF-8 comes back with the replacement
    // character, and the frame says so rather than letting the reader
    // believe the command printed it (review 0070 R-4).
    let bytes = "printf 'before \\351\\357 after\\n' >&2\nexit 1\n";
    let dir = project("framebytes", bytes);
    let (out, _) = keel(&["close", dir.to_str().unwrap()]);
    assert!(
        out.contains("not all UTF-8"),
        "a stream that was not UTF-8 is named as such:\n{out}"
    );

    // The typed JSON field carries the same words as the prose: a
    // harness must not have to parse a report to find the reason.
    let dir = project("framejson", MANY_STEPS);
    let out = keel_out(&["close", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(out.trim())
        .unwrap_or_else(|e| panic!("close --json is a package: {e}\n{out}"));
    let red = package["red_commands"]
        .as_array()
        .unwrap_or_else(|| panic!("the package carries red_commands:\n{out}"));
    assert_eq!(red.len(), 1, "one command failed, one entry:\n{out}");
    assert!(
        red[0]["words"]
            .as_str()
            .unwrap_or_default()
            .contains("RUBOCOP: 3 offenses detected"),
        "and the field carries the same words as the prose:\n{out}"
    );
    assert!(
        package["blockers"].as_u64().unwrap_or(0) >= 1,
        "the field that stood before this wave is untouched:\n{out}"
    );
}

/// proves: a-red-gate-carries-the-words-that-made-it-red@372893 -- the
/// quote keeps the shape the command gave it, and the ceiling eats
/// only quotes.
///
/// The second round of review 0070 measured four more promises with
/// no court: the indent, the tab, the report ceiling, and the output
/// of a red `verify`. Three of those ARE the fixes the first round
/// demanded -- a repair nobody guards is a repair the next refactor
/// undoes in silence.
#[test]
fn the_quote_keeps_its_shape_and_the_ceiling_eats_only_quotes() {
    // --- the indent and the tab survive: in a diagnostic they ARE
    // the meaning, and a caret that moved points at the wrong thing.
    let shaped = "printf '    name = \"andrii\"\\n' >&2\n\
                  printf '           ^^^^^^^^ offence here\\n' >&2\n\
                  printf 'col1\\tcol2\\tcol3\\n' >&2\n\
                  exit 1\n";
    let dir = project("shape", shaped);
    let (out, code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a red gate blocks the wave:\n{out}");
    assert!(
        out.contains("           ^^^^^^^^ offence here"),
        "the caret keeps its column -- trimming the quote moves it off \
         the offence it points at:\n{out}"
    );
    assert!(
        out.contains("col1\tcol2\tcol3"),
        "and the tab survives -- dropping it welds the columns:\n{out}"
    );

    // --- a red verify carries its output too: `run_command` serves
    // both, and the card says the radius is wider than the gate.
    let dir = project("verifyred", "exit 0\n");
    write(&dir, "src/lib.rs", "pub fn a() {}\n");
    write(
        &dir,
        "keel/contracts/toy.md",
        "---\nmodule: toy\nexports:\n  - \"pub fn a()\"\nverify: \"sh bin/verify\"\n---\n\nтіло\n",
    );
    write(
        &dir,
        "bin/verify",
        "echo VERIFY-STEP-ONE ok\necho VERIFY-BROKE-HERE >&2\necho later noise >&2\nexit 1\n",
    );
    let text = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let verify = "sh bin/verify";
    write(
        &dir,
        "keel.toml",
        &format!(
            "{text}\"{verify}\" = \"{}\"\n",
            keel::trust::fingerprint(verify)
        ),
    );
    let (out, code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a red verify blocks the wave (§2.8):\n{out}");
    assert!(
        out.contains("VERIFY-BROKE-HERE"),
        "and it carries its output, not the last line it printed -- \
         `run_command` serves verify and ci alike:\n{out}"
    );
    assert!(
        out.contains("unmasked"),
        "under the same frame, with the same warning:\n{out}"
    );

    // --- the ceiling eats quotes and NOTHING ELSE, and it shares
    // what is left instead of feeding the first commands only.
    let loud = "i=1\nwhile [ $i -le 400 ]; do echo line-$i; i=$((i+1)); done\nexit 1\n";
    let dir = project("ceiling", loud);
    for n in 1..=8 {
        write(
            &dir,
            &format!("keel/contracts/c{n}.md"),
            &format!(
                "---\nmodule: c{n}\nexports:\n  - \"pub fn a()\"\nverify: \"sh bin/ci\"\n---\n\nтіло\n"
            ),
        );
    }
    let (out, code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "the wave does not merge:\n{out}");
    let quoted = out.lines().filter(|l| l.starts_with("    ")).count();
    assert!(
        quoted <= 600,
        "the report is bounded, not merely windowed per command: {quoted} quoted lines\n{out}"
    );
    // Every red command got words: a first-come budget left the last
    // ones silent, and the reader could not tell they had spoken.
    assert!(
        out.matches("what the command said").count() >= 8,
        "and EVERY red command got a frame -- a shared budget must not \
         feed the first and starve the last:\n{out}"
    );
    // The court's own verdict is not quoted output, and the ceiling
    // must not touch it.
    assert!(
        out.contains("the missing, by name"),
        "the court's own verdict survives the ceiling whole -- a \
         ceiling that eats the verdict is worse than no ceiling:\n{out}"
    );
    // And every cut block still says it was cut. The first ceiling
    // marked the court's own frame like the quote and ate it: the
    // window stopped saying it was a window in exactly the report
    // where the ceiling made it one.
    assert!(
        out.matches("cut from the middle").count() >= 8,
        "every block that was cut says so -- the frame is the court's \
         word, not the command's, and the ceiling must not eat it:\n{out}"
    );
    // No mark of the report's own bookkeeping reaches the reader.
    assert!(
        !out.contains('\u{1}') && !out.contains('\u{2}'),
        "and the marks the report keeps for itself never reach the \
         page:\n{:?}",
        out.chars()
            .filter(|c| *c == '\u{1}' || *c == '\u{2}')
            .count()
    );

    // --- the package carries the same text, and no bookkeeping ---
    // The marks are the report's own; a field that carried them would
    // break the promise that it holds the same words as the prose.
    let dir = project("ceilingjson", MANY_STEPS);
    let out = keel_out(&["close", "--json", dir.to_str().unwrap()]);
    assert!(
        !out.contains("\\u0001") && !out.contains("\\u0002"),
        "the JSON package carries no mark of the report's bookkeeping:\n{out}"
    );
    let package: serde_json::Value = serde_json::from_str(out.trim()).unwrap();
    let words = package["red_commands"][0]["words"]
        .as_str()
        .unwrap_or_default()
        .to_string();
    assert!(
        !words.contains('\u{1}') && !words.contains('\u{2}'),
        "not in the field itself either:\n{words:?}"
    );
    assert!(
        words.contains("RUBOCOP: 3 offenses detected"),
        "and it is still the same text as the prose:\n{words}"
    );
}
