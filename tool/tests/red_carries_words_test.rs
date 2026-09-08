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
