//! Scenario test of wave 0037: a write that lies is red.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

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

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"rust\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn decided(except: &str) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != except {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// proves: a-write-that-lies-is-red@b5a305 -- the bug audit (B5)
/// measured `keel rev --write` printing a red line, then the words
/// "nothing drifts", and exiting ZERO. CI sees the zero and drives
/// on: a hand that failed reported success.
#[test]
fn a_write_that_lies_is_red() {
    // A wave naming a contract that is not there: the write cannot
    // land, and says so in red.
    let dir = project("liar");
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    proves: nowhere@abcdef\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided("functional.correctness")
        ),
    )
    .unwrap();

    let (said, code) = keel(&dir, &["rev", "--write"]);
    assert!(
        said.contains("червоне"),
        "the write says what stopped it:\n{said}"
    );
    assert_ne!(
        code, 0,
        "and the exit code says the same as the words -- a hand that \
         failed does not report success (bug audit B5):\n{said}"
    );
    assert!(
        !said.contains("нічого не дрейфує"),
        "and it does not claim all is well two lines under its own \
         red:\n{said}"
    );

    // Where there is genuinely nothing to write, zero is honest.
    let dir = project("quiet");
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided("functional.correctness")
        ),
    )
    .unwrap();
    let (said, code) = keel(&dir, &["rev", "--write"]);
    assert_eq!(code, 0, "nothing to write is a green answer:\n{said}");
    assert!(
        said.contains("нічого не дрейфує"),
        "and it says so:\n{said}"
    );

    // And a write that lands is green too, with its count.
    let dir = project("lands");
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    let rev = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    proves: anchor@beef00\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    contracts: [anchor@beef00]\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided("functional.correctness")
        ),
    )
    .unwrap();
    let (said, code) = keel(&dir, &["rev", "--write"]);
    assert_eq!(code, 0, "a write that landed is green:\n{said}");
    assert!(
        said.contains(&rev),
        "and it names the revision it wrote:\n{said}"
    );
}
