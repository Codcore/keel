//! Scenario test of wave 0031: the closing court names its price.

mod common;

use common::keel_sandbox;
use std::process::Command;

fn write(dir: &std::path::Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// proves: the-closing-court-names-its-price@8f6a26 -- tool/target
/// stood at 3.3 GB when this wave was planned, and the reviewers of
/// waves 0028 and 0029 BOTH skipped running `keel close` because the
/// disk was too tight, leaving the closing court unverified twice in
/// a row. The court builds its own target ON PURPOSE (an inherited
/// cache shifts verdicts, §6.7), so the fix is to say the price, not
/// to stop paying it.
#[test]
fn the_closing_court_names_its_price() {
    let dir = keel_sandbox("price");
    // The probe names its own tongue rather than leaning on a
    // default in another module (school of review 0029 R-12).
    write(&dir, "keel.toml", "lang = \"uk\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    // Said BEFORE the work, and said in full: what it builds, where,
    // why it cannot share a cache, and roughly what it wants.
    assert!(
        said.contains("tool/target"),
        "the court names what it builds and where:\n{said}"
    );
    assert!(
        said.contains("ГБ"),
        "and what it wants in gigabytes, so a person on a tight disk can stop now:\n{said}"
    );
    assert!(
        said.contains("§6.7"),
        "and why it cannot simply reuse the caller's cache:\n{said}"
    );
}
