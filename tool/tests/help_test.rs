//! Scenario test of wave 0035: the tool answers when asked for help.

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

/// proves: the-tool-answers-when-asked-for-help@e04b56 -- the
/// bug audit measured the first thing a person types giving a
/// refusal about a directory called "--help", extra arguments
/// swallowed by seventeen commands of twenty, and an unreplaced
/// { $snippet } sitting in the output of both tongues.
#[test]
fn the_tool_answers_when_asked_for_help() {
    // The first thing a person types.
    for asked in ["--help", "-h", "help"] {
        let (said, code) = keel(&[asked]);
        assert_eq!(
            code, 0,
            "`keel {asked}` answers rather than refusing:\n{said}"
        );
        assert!(
            said.contains("keel check") && said.contains("keel plan"),
            "and what it answers is the list of commands:\n{said}"
        );
        assert!(
            !said.contains(asked),
            "not a complaint about a directory named `{asked}`:\n{said}"
        );
    }

    // An unknown flag is refused with the list, not read as a path.
    let (said, code) = keel(&["check", "--wat"]);
    assert_eq!(code, 2, "an unknown flag is refused:\n{said}");
    assert!(
        said.contains("keel check"),
        "and the refusal shows what the tool does know:\n{said}"
    );

    // A second path is a typo, not a choice -- in every command, not
    // in three of twenty.
    let dir = keel_sandbox("help");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    for command in ["check", "rev", "map", "status", "trust", "version"] {
        let (said, code) = keel(&[command, dir.to_str().unwrap(), "junk"]);
        assert_eq!(
            code, 2,
            "`keel {command}` refuses a second path instead of swallowing it:\n{said}"
        );
    }

    // And no line of any output leaves a placeholder unreplaced.
    for command in ["check", "rev", "map", "status", "next", "cuts", "version"] {
        let (said, _) = keel(&[command, dir.to_str().unwrap()]);
        assert!(
            !said.contains("{ $") && !said.contains("{$"),
            "`keel {command}` says its words, not the shape of them:\n{said}"
        );
    }
}
