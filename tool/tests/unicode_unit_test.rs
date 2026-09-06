//! Scenario test of wave 0055: the form court reads every letter.
//!
//! `found_bounded` walked the source in BYTES: after a match that
//! failed its token boundary it stepped one byte on and sliced the
//! haystack inside a multi-byte letter -- `keel check` and `keel
//! close` died with "byte index 23 is not a char boundary" over a
//! contract whose export begins with a non-ASCII letter and a source
//! carrying a longer twin of the name (final review 2026-09-06, bugs
//! R-3). And the boundary itself was asked of a byte: a letter like
//! `é` before the name is not ASCII-alphanumeric, so `éünïcode` held
//! the promise `ünïcode`.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn keel(dir: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
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

/// A crate whose module carries names that begin with letters
/// outside ASCII: a longer twin of the promised name, the same name
/// behind another letter, one held exactly, and one diverged.
fn crate_with_letters(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod alpha;\n");
    write(
        &dir,
        "src/alpha.rs",
        "pub fn ünïcodex(a: u32) -> u32 {\n    a\n}\n\npub fn éünïcode(a: u32) -> u32 {\n    a\n}\n\npub fn éxact(a: u32) -> u32 {\n    a\n}\n\npub fn çhanged(a: u64) -> u64 {\n    a\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn ünïcode(a: u32) -> u32\"\n  - \"pub fn éxact(a: u32) -> u32\"\n  - \"pub fn çhanged(a: u32) -> u32\"\n---\n\nThe alpha promise, spelled with every letter.\n",
    );
    dir
}

/// What either court must say over that crate: findings, never a
/// panic; the twins are not the name; the exact name is compared.
fn reads_every_letter(said: &str, code: i32, court: &str) {
    assert_ne!(
        code, 101,
        "{court}: the form court does not die over a letter outside \
         ASCII -- the boundary is read in chars:\n{said}"
    );
    assert!(
        !said.contains("panicked"),
        "{court}: and says nothing of a panic:\n{said}"
    );
    assert!(
        said.contains("\"ünïcode\"") && said.contains("no such unit"),
        "{court}: the longer twin `ünïcodex` and the twin behind a \
         letter `éünïcode` are not the unit -- a boundary is a letter, \
         ASCII or not, on both sides:\n{said}"
    );
    assert!(
        said.contains("pub fn çhanged(a: u32) -> u32") && said.contains("does not match"),
        "{court}: a name that stands exactly is found, and its form \
         compared -- the diverged one carries the promised text:\n{said}"
    );
    assert!(
        !said.lines().any(|line| line.contains("éxact")),
        "{court}: and the held one is silence:\n{said}"
    );
}

/// proves: the-form-court-reads-every-letter@02d26a -- `keel check`
/// left with 101, "byte index 23 is not a char boundary", over a
/// contract export beginning with `ü` and a source with the longer
/// twin `ünïcodex`; `keel close` the same (final review 2026-09-06,
/// bugs R-3; reproduced on f744252).
#[test]
fn the_form_court_reads_every_letter() {
    let dir = crate_with_letters("everyletter");
    let (said, code) = keel(&dir, &["check"]);
    reads_every_letter(&said, code, "check");
    assert_eq!(code, 1, "check: two findings are a red check:\n{said}");

    // The closing court runs the same form court over the same
    // contracts (§7.6), and died the same way.
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"not about this sandbox\"\n"));
        }
    }
    write(
        &dir,
        "keel/waves/0001-a-wave.md",
        &format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/alpha.rs\n{d}---\n\n## scenario: it-works\nthe promise\n\n## transform: work\nthe work\n"
        ),
    );
    write(&dir, "keel/reviews/0001-a-wave.md", "# Review\n\nok\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["close"]);
    reads_every_letter(&said, code, "close");
}
