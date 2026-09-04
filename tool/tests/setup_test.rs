//! Scenario tests of wave 0032: the wizard asks what a project needs,
//! and the answers can be changed afterwards.

mod common;

use common::keel_sandbox;
use std::process::Command;

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

/// proves: the-wizard-asks-what-a-project-needs@257425 -- named as a
/// limit in the plan of wave 0026 and never lifted: version, ci and
/// [trust] are exactly what makes a project ready for its own gates,
/// and the wizard asked for none of them.
#[test]
fn the_wizard_asks_what_a_project_needs() {
    let dir = keel_sandbox("asks");
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();

    let (said, code) = keel(&[
        "init",
        dir.to_str().unwrap(),
        "--lang",
        "uk",
        "--adapter",
        "rust",
        "--mode",
        "strict",
        "--agents",
        "claude",
        // --hooks is a switch, not a question with a value.
        "--hooks",
        "--version",
        "pin",
        "--ci",
        "cargo test",
        "--trust",
        "yes",
    ]);
    assert_eq!(code, 0, "the wizard takes the three new answers:\n{said}");

    let config = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        config.contains("version = "),
        "the pinned version is written, not left a comment:\n{config}"
    );
    assert!(
        config.contains("ci = \"cargo test\""),
        "and the ci command:\n{config}"
    );
    assert!(
        config.contains("[trust]") && config.contains("cargo test"),
        "and the trust of that very command, so the gate does not \
         refuse it on first run:\n{config}"
    );

    // The file it wrote is the file the tool reads.
    let (verdict, _) = keel(&["check", dir.to_str().unwrap()]);
    assert!(
        !verdict.contains("keel.toml") || !verdict.contains("причина"),
        "and the config it wrote is one the tool accepts:\n{verdict}"
    );
}

/// proves: answers-can-be-changed-after-init@cecf6a -- named as a
/// limit in review 0026 and never lifted: keel.toml was edited by
/// hand or not at all.
#[test]
fn answers_can_be_changed_after_init() {
    let dir = keel_sandbox("second-chance");
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();
    keel(&[
        "init",
        dir.to_str().unwrap(),
        "--lang",
        "uk",
        "--adapter",
        "rust",
        "--mode",
        "strict",
        "--agents",
        "claude",
        // --hooks is a switch, not a question with a value.
        "--hooks",
    ]);
    // init leaves a [generated] section behind -- the digests of the
    // integrations it wrote. The wizard never asks about it, and
    // setup must not eat it.
    let before = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        before.contains("[generated]"),
        "init records the digests it generated:\n{before}"
    );

    let (said, code) = keel(&["setup", dir.to_str().unwrap(), "--lang", "en"]);
    assert_eq!(
        code, 0,
        "setup runs on a project that already has a config:\n{said}"
    );

    let after = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        after.contains("lang = \"en\""),
        "the changed answer lands:\n{after}"
    );
    assert!(
        after.contains("mode = \"strict\""),
        "the answers not asked about this time survive:\n{after}"
    );
    assert!(
        after.contains("[generated]") && after.contains("AGENTS.md"),
        "and so does what the wizard never asked about at all:\n{after}"
    );

    // On a project with no config, setup behaves as init rather than
    // falling over.
    let fresh = keel_sandbox("no-config");
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&fresh)
        .status()
        .unwrap();
    let (said, code) = keel(&["setup", fresh.to_str().unwrap(), "--lang", "uk"]);
    assert_eq!(code, 0, "setup on a bare project writes a config:\n{said}");
    assert!(fresh.join("keel.toml").is_file(), "and the file is there");
}
