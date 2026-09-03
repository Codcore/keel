//! Scenario tests of wave 0017-adapter-named-by-language, transform
//! language-name: the adapter is named by the project's language --
//! rust is canonical, cargo an accepted synonym said aloud.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

#[allow(unused_imports)]
use common::{Sandbox, keel_sandbox};

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

fn keel(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn crate_files(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(dir, "src/lib.rs", "");
}

/// proves: adapter-named-by-language@139959 -- holds the concept's
/// letter and the operator's decision of 2026-09-03: rust is the
/// canonical adapter name and every adapter-needing court accepts
/// it; the old spelling cargo still works and keel check says the
/// synonym aloud with the canonical name beside it; an unknown
/// adapter refuses with rust in the instead; and the init
/// scaffolding recommends rust.
#[test]
fn adapter_named_by_language() {
    // rust is canonical: the courts run as they did with cargo.
    let dir = keel_sandbox("rustname");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "rust is the canonical adapter name:\n{out}");
    assert!(
        out.contains("counted:"),
        "the stage eye runs under the language name:\n{out}"
    );
    let (out, err, code) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the step hand runs under the language name:\n{out}"
    );

    // cargo still works -- and check says the synonym aloud.
    let dir = keel_sandbox("synonym");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the old spelling still works -- no config breaks silently:\n{out}"
    );
    let (out, err, _) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("synonym") && out.contains("\"rust\""),
        "check says the synonym aloud with the canonical name (§9.7):\n{out}"
    );

    // An unknown adapter refuses with the canonical name to reach for.
    let dir = keel_sandbox("unknown");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"elixir\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "an unknown adapter refuses aloud:\n{out}");
    assert!(
        out.contains("\"rust\""),
        "the instead names the canonical language name:\n{out}"
    );

    // The init scaffolding recommends the language name.
    let dir = keel_sandbox("initword");
    git(&dir, &["init", "-q", "-b", "main"]);
    let (_, _, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands");
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        config.contains("# adapter = \"rust\""),
        "the scaffolding recommends rust, the concept's letter:\n{config}"
    );
}

/// proves: adapter-named-by-language@139959 -- the second birth out
/// of review 0017 (R-1..R-5): the synonym question lives in the
/// config home too; the unknown-adapter word of check names rust;
/// a named-yet-unknown adapter is never called "not named" by the
/// form court or the map; gate -- the one court that physically
/// runs the toolchain -- asks the home and passes with a word
/// instead of running cargo blindly; and rust is pinned across
/// close, rev --write and check, not a corner.
#[test]
fn adapter_named_by_language_second_birth() {
    // R-2/R-3: the unknown yet NAMED adapter -- every word tells
    // the truth: check points at rust, the form court and the map
    // say "not of this release", never "not named".
    let dir = keel_sandbox("namedunknown");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"elixir\"\n");
    crate_files(&dir);
    write(
        &dir,
        "keel/contracts/c.md",
        "---\nmodule: toy\nexports:\n  - \"pub fn one()\"\n---\n\na promise\n",
    );
    let (out, err, _) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("\"rust\""),
        "the unknown-adapter word of check names the canonical rust (R-2):\n{out}"
    );
    assert!(
        !out.contains("not named") && out.contains("not of this release"),
        "named-yet-unknown is not painted as absent (R-3):\n{out}"
    );

    // R-4: gate asks the home -- an unknown adapter passes with a
    // word, cargo is never run blindly for a foreign language.
    let dir = keel_sandbox("gatehome");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"elixir\"\n");
    crate_files(&dir);
    write(
        &dir,
        "keel/waves/0600-w.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t\n",
    );
    git(&dir, &["init", "-q", "-b", "0600-w"]);
    write(&dir, ".git/COMMIT_MSG_PROBE", "red: s\n");
    let msg = dir.join(".git/COMMIT_MSG_PROBE");
    let (out, err, code) = keel(&["gate", msg.to_str().unwrap(), dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "gate passes with a word instead of judging blind (R-4):\n{out}"
    );
    assert!(
        out.contains("not judged") && out.contains("\"elixir\""),
        "the unjudged verdict is a word aloud with the adapter's name (R-4, §9.7):\n{out}"
    );

    // R-5: rust is pinned across the three courts the first birth
    // left to sandboxes -- close, rev --write and check.
    let dir = keel_sandbox("rustpinned");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the closure court runs under rust (R-5):\n{out}");
    assert!(
        out.contains("battery:"),
        "the battery line proves close truly ran:\n{out}"
    );
    let (out, err, code) = keel(&["rev", "--write", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the rewriting hand runs under rust (R-5):\n{out}");
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "check runs under rust (R-5):\n{out}");
    assert!(
        !out.contains("synonym"),
        "the canonical name earns no synonym word (R-1):\n{out}"
    );
}
