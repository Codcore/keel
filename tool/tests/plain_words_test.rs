//! Scenario test of wave 0033: the tool speaks plainly.

mod common;

use common::keel_sandbox;
use std::process::Command;

/// Words this project invented on the way and never explained. They
/// mean nothing to anyone who has not read the commit history, and
/// §1.7 asks for the opposite: a word of the methodology's own is
/// explained in plain words where it is first used.
const INVENTED: [&str; 3] = ["межа вироку", "поверх", "щабель"];

/// proves: the-tool-speaks-plainly@6caef1 -- the operator read
/// the output of `keel check` and could not tell what "межа вироку"
/// was. It was mine, coined in wave 0031, and explained nowhere.
#[test]
fn the_tool_speaks_plainly() {
    let dir = keel_sandbox("plain");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  a-promise:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - a-promise\n    files:\n      - src/lib.rs\n---\n\n## scenario: a-promise\nтекст\n\n## transform: work\nтекст\n",
    )
    .unwrap();
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();

    // Every road a person walks, not only the one this wave touched.
    for command in [
        &["check"][..],
        &["status"][..],
        &["next"][..],
        &["map"][..],
        &["rev"][..],
    ] {
        let mut args: Vec<&str> = command.to_vec();
        let root = dir.to_str().unwrap();
        args.push(root);
        let out = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(&args)
            .output()
            .unwrap();
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        for word in INVENTED {
            assert!(
                !said.contains(word),
                "`keel {}` says \"{word}\" -- a word invented here and \
                 explained nowhere (§1.7):\n{said}",
                command[0]
            );
        }
    }

    // And what replaced them says what happened, in words that exist
    // outside this project.
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        said.contains("не перевірено"),
        "the tool says plainly what it did not check:\n{said}"
    );
}
