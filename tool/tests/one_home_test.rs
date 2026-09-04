//! Scenario test of wave 0035: a scenario name belongs to one wave.

mod common;

use common::keel_sandbox;
use std::process::Command;

const WAVE: &str = "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-works\nтіло обіцянки\n\n## transform: work\nтіло роботи\n";

/// proves: a-scenario-name-belongs-to-one-wave@e10ffa -- the bug
/// audit copied a wave under a new number and `keel close` called
/// BOTH closed, though the second had no test of its own. A test tag
/// is a bare name, so the machine has no way to know whose promise
/// it proves. The norm never says the slugs are unique either
/// (methodology audit С-9).
#[test]
fn a_scenario_name_belongs_to_one_wave() {
    let dir = keel_sandbox("onehome");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-first.md"), WAVE).unwrap();
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();

    let said = check(&dir);
    assert!(
        !said.contains("it-works") || !said.contains("червоне"),
        "one wave with the name is nobody's business:\n{said}"
    );

    // The same name in a second wave: the tag cannot say whose it is.
    std::fs::write(dir.join("keel/waves/0002-second.md"), WAVE).unwrap();
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("it-works"),
        "two waves with one scenario name is a finding:\n{said}"
    );
    assert!(
        said.contains("0001-first") && said.contains("0002-second"),
        "and it names both homes, so the fix is obvious:\n{said}"
    );
}

fn check(dir: &std::path::Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}
