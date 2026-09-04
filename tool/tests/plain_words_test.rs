//! Scenario test of wave 0033: the tool speaks plainly.

mod common;

use common::keel_sandbox;
use std::process::Command;

/// Words this project invented on the way and never explained. They
/// mean nothing to anyone who has not read the commit history, and
/// §1.7 asks for the opposite: a word of the methodology's own is
/// explained in plain words where it is first used.
/// Written as the PHRASES they appear in: "поверх" on its own is an
/// ordinary preposition ("keel не пише поверх того, чого не писав"),
/// and a check that forbids a common word would be a check nobody
/// can keep.
const INVENTED: [&str; 5] = [
    "межа вироку",
    "межі вироку",
    "меж вироку",
    "цим поверхом",
    "щабель ",
];

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

    // What REPLACED them -- the plain "не перевірено" -- is judged
    // where such a line actually appears: verdict_limits_test, which
    // builds a clone that has something to not check. This probe
    // holds the other half: no road a person walks says a word this
    // project invented and never explained.
}
