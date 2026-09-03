//! Scenario test of wave 0031: the closing court names its price.

mod common;

use common::keel_sandbox;
use std::process::Command;

/// proves: the-closing-court-names-its-price@8f6a26 -- tool/target
/// stood at 3.3 GB when this wave was planned, and the reviewers of
/// waves 0028 and 0029 both skipped running `keel close` because the
/// disk was too tight. The court builds its own target ON PURPOSE
/// (adapter.rs: an inherited cache shifts verdicts), so the fix is
/// to say the price, not to stop paying it.
#[test]
fn the_closing_court_names_its_price() {
    let dir = keel_sandbox("price");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();

    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        said.contains("ГБ") || said.contains("target"),
        "the court says what it is about to build and where:\n{said}"
    );
}
