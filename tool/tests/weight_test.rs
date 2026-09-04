//! Scenario test of wave 0036: the weight comes from the file.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
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

fn keel(dir: &Path, command: &str) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args([command, dir.to_str().unwrap()])
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

fn decided() -> String {
    let mut out = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        out.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    out
}

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    // `status` is the eye of the stages and needs the adapter.
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
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

/// proves: the-weight-comes-from-the-file@5194b3 -- §6.8 states the
/// rule exactly (one transform, no contracts, nothing withdrawn) and
/// nothing computed it. The norm audit (В-2) and the conformance
/// audit (ВАЖКА-6) both landed on the same consequence: a chore with
/// a NEW CONTRACT calls itself light and rides in on one PR --
/// without the second human look §6.8 demands for exactly that case.
#[test]
fn the_weight_comes_from_the_file() {
    // A light wave: one transform, no contract, nothing withdrawn.
    // It may ride one branch, and the weight is said aloud.
    let dir = project("light");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            {
                let mut d = String::from("decisions:\n");
                for cut in keel::graph::cuts() {
                    if *cut != "functional.correctness" {
                        d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
                    }
                }
                d
            }
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: the light wave"]);

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("легка"),
        "the weight is derived and said aloud (§6.8):\n{said}"
    );
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("повна хвиля"),
        "a light wave riding one branch is lawful:\n{said}"
    );

    // A wave that grows a CONTRACT is full whatever it calls itself,
    // and a full wave born on its own work branch never had the
    // plan PR §6.8 asks for -- that is the finding.
    let dir = project("full");
    git(&dir, &["checkout", "-q", "-b", "0002-b-wave"]);
    std::fs::write(
        dir.join("keel/contracts/fresh.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0002-b-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n      - keel/contracts/fresh.md\n{}---\n\n## transform: work\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn c() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "work: a chore with a new contract"],
    );

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("повна"),
        "a wave that grows a contract is full, whatever it calls \
         itself (§6.8):\n{said}"
    );
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 1, "and riding one branch is a finding:\n{said}");
    assert!(
        said.contains("0002-b-wave") && said.contains("двох"),
        "the finding names the wave and says what is missing -- the \
         two human looks §6.8 asks for:\n{said}"
    );

    // Withdrawing a promise makes a wave full too: a promise dying is
    // exactly the risk §6.8 wants two people to see.
    let dir = project("withdrawn");
    git(&dir, &["checkout", "-q", "-b", "0003-c-wave"]);
    std::fs::write(
        dir.join("keel/waves/0003-c-wave.md"),
        format!(
            "---\nscenarios:\n  gone:\n    covers: [functional.correctness]\n    withdrawn: \"згорнуто\"\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: gone\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn d() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: a withdrawal"]);

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("повна"),
        "a wave that withdraws a promise is full (§6.8):\n{said}"
    );
}
