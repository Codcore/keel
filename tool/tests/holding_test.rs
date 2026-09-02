//! Scenario tests of wave 0010-contracts-hold, transform
//! holding-floor: §7.6's form court -- promised signatures compared
//! as collapsed text against the module's source; the incomparable
//! is a word aloud, never green.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0010h-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
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

/// proves: exports-held@ec9457 -- holds §7.6/§2.9: a promised
/// signature diverged in code is a finding carrying the promised
/// text; a vanished unit is a finding by name; a matching contract
/// is silence; and where there is nothing to compare with -- no
/// cargo adapter, a deeper module path -- the report says "no one
/// compared the form" aloud instead of green.
#[test]
fn exports_held() {
    let dir = sandbox("form");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "src/lib.rs",
        "pub mod alpha;\npub mod beta;\npub mod deep;\n",
    );
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n\npub fn drifted(x: u64) -> u64 {\n    x\n}\n",
    );
    write(
        &dir,
        "src/beta.rs",
        "pub fn beta_keeps(word: &str) -> String {\n    word.to_string()\n}\n",
    );
    write(&dir, "src/deep.rs", "pub mod inner {}\n");
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn drifted(x: u32) -> u32\"\n  - \"pub fn vanished() -> bool\"\n---\n\nThe alpha promise.\n",
    );
    write(
        &dir,
        "keel/contracts/toy-beta.md",
        "---\nmodule: toy::beta\nexports:\n  - \"pub fn beta_keeps(word: &str) -> String\"\n---\n\nThe beta promise, held.\n",
    );
    write(
        &dir,
        "keel/contracts/toy-deep.md",
        "---\nmodule: toy::deep::inner\nexports:\n  - \"pub fn hidden() -> bool\"\n---\n\nDeeper than this generation compares.\n",
    );

    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "diverged and vanished are findings:\n{out}");
    assert!(
        out.contains("pub fn drifted(x: u32) -> u32") && out.contains("does not match"),
        "the diverged signature carries the promised text (§2.9):\n{out}"
    );
    assert!(
        out.contains("\"vanished\"") && out.contains("no such unit"),
        "the vanished unit is a finding by name:\n{out}"
    );
    assert!(
        !out.lines()
            .any(|l| l.contains("toy-beta") && l.contains("no such unit")),
        "the held contract is silence:\n{out}"
    );
    assert!(
        !out.lines()
            .any(|l| l.contains("beta_keeps") && l.contains("does not match")),
        "the held signature is not a finding:\n{out}"
    );
    assert!(
        out.contains("no one compared the form") && out.contains("deeper"),
        "the deep module gets the word, not green (§7.6):\n{out}"
    );
    assert!(
        out.contains("signatures checked:"),
        "the form court counts aloud:\n{out}"
    );

    // No adapter named: nothing to compare with -- the word again,
    // and no finding invented.
    let dir = sandbox("no-adapter");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n---\n\nThe promise with no one to compare it.\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "nothing to compare is not a finding:\n{out}");
    assert!(
        out.contains("no one compared the form") && out.contains("no adapter"),
        "the skip is said aloud with its reason:\n{out}"
    );
}

/// proves: exports-held@ec9457 -- the second birth out of review
/// 0010 (R-1a/R-3/R-6), riding exports-held:
/// green must mean the form stands -- a short-form promise is not
/// satisfied by a longer neighbour's name, a signature surviving
/// only in a comment is vanished, the verdict words tell divergence
/// from disappearance apart; and on a plan branch the form court
/// does not run at all (§8.3) -- said aloud, not judged.
#[test]
fn exports_held_second_birth() {
    // Short form against a longer neighbour: `pub fn run` with only
    // run_all in the code is a vanished unit, never green (F1); and
    // the words say "no such unit", not "does not match" (F2/R-6).
    let dir = sandbox("boundary");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod gamma;\n");
    write(
        &dir,
        "src/gamma.rs",
        "pub fn run_all(x: u32) -> u32 {\n    x\n}\n// pub fn ghost(x: u32) -> u32 -- only remembered here\n",
    );
    write(
        &dir,
        "keel/contracts/toy-gamma.md",
        "---\nmodule: toy::gamma\nexports:\n  - \"pub fn run\"\n  - \"pub fn ghost(x: u32) -> u32\"\n---\n\nShort and remembered promises.\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "both green lies are findings now:\n{out}");
    assert!(
        out.contains("\"run\"") && out.contains("no such unit"),
        "the short form is vanished, not green and not 'does not match' (R-3/R-6):\n{out}"
    );
    assert!(
        out.contains("\"ghost\"") && out.contains("no such unit"),
        "a promise surviving only in a comment is vanished (R-3):\n{out}"
    );

    // A plan branch runs no form court (§8.3): exports may grow
    // ahead of the code there, and the skip is a word aloud.
    let dir = sandbox("planbranch");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod alpha;\n");
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn planned_ahead() -> bool\"\n---\n\nThe plan grows the promise ahead of the code (§4.9).\n",
    );
    git2(&dir, &["init", "-q", "-b", "plan/0070-x"]);
    git2(&dir, &["add", "."]);
    git2(&dir, &["commit", "-q", "-m", "the plan rides its branch"]);
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "a plan branch is not red for growing exports (§8.3):\n{out}"
    );
    assert!(
        out.contains("plan branch") && out.contains("form court does not run"),
        "the skip is said aloud (§8.3):\n{out}"
    );
}

fn git2(dir: &Path, args: &[&str]) {
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
