//! Scenario test of wave 0032: the reviewer is briefed by the tool.

mod common;

use common::keel_sandbox;
use std::process::Command;

fn keel(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// proves: the-reviewer-is-briefed-by-the-tool@c8dbb1 -- until this
/// wave the package carried the wave's own material and not one word
/// about what a reviewer should do with it. Every briefing was
/// written by hand in a chat, so every reviewer got a different one,
/// and reviewer 0026 destroyed 10,128 directories belonging to other
/// sessions because the prohibition was missing from his.
#[test]
fn the_reviewer_is_briefed_by_the_tool() {
    let dir = keel_sandbox("briefed");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  a-promise:\n    covers: [functional.correctness]\ntransforms: {}\n---\n\n## scenario: a-promise\nтекст\n",
    )
    .unwrap();
    for args in [
        &["init", "-q", "-b", "0001-a-wave"][..],
        &["add", "-A"][..],
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            "wave",
        ][..],
    ] {
        Command::new("git")
            .args(args)
            .current_dir(&dir)
            .status()
            .unwrap();
    }

    let package = keel(&["review", dir.to_str().unwrap()]);

    // The prohibitions come first, and the one that cost 10,128
    // directories is among them.
    assert!(
        package.contains("чужого не чіпай"),
        "the briefing forbids touching what is not yours:\n{package}"
    );
    assert!(
        package.contains("--no-local") && package.contains("CARGO_TARGET_DIR"),
        "and names the hygiene: an own clone, an own target:\n{package}"
    );
    assert!(
        package.contains("§9.9"),
        "and the four questions it is written against:\n{package}"
    );
    assert!(
        package.contains("keel/reviews/"),
        "and where the report goes:\n{package}"
    );
    assert!(
        package.contains("тільки з бігів"),
        "and that numbers come from runs, not from reading:\n{package}"
    );

    // And the briefing is JUDGED, not just written: it may name only
    // commands this binary really has. School of wave 0024, where a
    // block advised what the tool could not do.
    let help = keel(&["help"]);
    for word in ["keel check", "keel close", "keel review", "keel rev"] {
        if package.contains(word) {
            let command = word.split_whitespace().nth(1).unwrap();
            assert!(
                help.contains(command),
                "the briefing names `{word}`, which this binary must have:\n{help}"
            );
        }
    }
}
