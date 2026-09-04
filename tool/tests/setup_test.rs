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

/// proves: setup-never-breaks-what-it-edits@9aee6c -- the bug
/// audit found one `keel setup` turning a healthy project into one
/// where nothing works, including setup itself.
#[test]
fn setup_never_breaks_what_it_edits() {
    let dir = keel_sandbox("no-harm");
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();
    // A config the wizard was never asked about: no agents key at
    // all, which is the default state of `keel init --no-ask`.
    keel(&["init", "--no-ask", dir.to_str().unwrap()]);
    let (_, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the project starts healthy");

    // Somebody's own words, and somebody's own trust.
    let config = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        config.replace(
            "# keel.toml",
            "# a line a person wrote
# keel.toml",
        ),
    )
    .unwrap();

    let (said, code) = keel(&["setup", "--no-ask", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "setup runs:\n{said}");

    let after = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        !after.contains("agents = []"),
        "and never writes a value the tool itself refuses:\n{after}"
    );
    let (verdict, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "the project the wizard edited is still one the tool can read:\n{verdict}"
    );
    assert!(
        after.contains("a line a person wrote"),
        "and the words a person wrote are still there:\n{after}"
    );

    // Trust: what the project still runs stays, what nobody runs any
    // more goes. Review 0034 R-4 measured the wizard keeping a dead
    // record and turning the gate red -- the very defect R-10 of
    // review 0032 had fixed once already.
    let config = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        format!("{config}\n[trust]\n\"a command nothing runs\" = \"deadbeef\"\n"),
    )
    .unwrap();
    keel(&["setup", "--no-ask", dir.to_str().unwrap()]);
    let after = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        !after.contains("a command nothing runs"),
        "trust for a command nobody runs goes with it, instead of \
         reddening the gate:\n{after}"
    );
    let (verdict, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "and the gate stays green:\n{verdict}");

    // A config that cannot be read stops the command rather than
    // being overwritten unseen.
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nthis is not toml\n").unwrap();
    let (said, code) = keel(&["setup", "--no-ask", dir.to_str().unwrap()]);
    assert_eq!(code, 2, "a config it cannot read stops it:\n{said}");
    let kept = std::fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        kept.contains("this is not toml"),
        "and the file is left exactly as it was:\n{kept}"
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
        // This project says no to hooks, and must still say no after
        // a setup (R-4).
        "--no-hooks",
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
    // Line by line, not by substring: review 0032 R-5 cut the whole
    // seeding out of setup and this probe stayed green, because the
    // COMMENTED default `# mode = "strict"` contains the very string
    // it was looking for. An answer lost and a default shown are not
    // the same thing.
    let answered = |field: &str, value: &str| {
        after
            .lines()
            .any(|line| line.trim() == format!("{field} = {value}"))
    };
    for (field, value) in [
        ("mode", "\"strict\""),
        ("adapter", "\"rust\""),
        ("agents", "[\"claude\"]"),
    ] {
        assert!(
            answered(field, value),
            "the answers not asked about this time survive as ANSWERS, \
             not as commented defaults -- {field} did not:\n{after}"
        );
    }
    assert!(
        after.lines().any(|line| line.trim() == "hooks = false"),
        "including the one that says no: a project that asked for no \
         hooks does not get them back from a setup (R-4):\n{after}"
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
