//! Scenario test of wave 0031: a verdict says how much of it is real.

mod common;

use common::keel_sandbox;
use std::process::Command;

fn keel(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
        out.status.code().unwrap_or(-1),
    )
}

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap();
}

/// proves: a-verdict-says-how-much-of-it-is-real@806a29 -- measured
/// before the wave: a full clone gave 208 lines carrying 141 old
/// revisions verified against history, a shallow one gave 67 lines
/// and zero -- and both ended with the identical "0 findings" line
/// above an identical list of what the storey claims to check.
#[test]
fn a_verdict_says_how_much_of_it_is_real() {
    let dir = keel_sandbox("limits");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            "start",
        ],
    );

    // Two states, two different last lines -- that is the whole
    // wave. First: a clone that has an origin and is level with it.
    let origin = dir.join("origin.git");
    Command::new("git")
        .args(["init", "-q", "--bare"])
        .arg(&origin)
        .status()
        .unwrap();
    git(&dir, &["remote", "add", "origin", origin.to_str().unwrap()]);
    git(&dir, &["push", "-q", "origin", "main"]);
    git(&dir, &["fetch", "-q", "origin"]);
    let (out, err, _) = keel(&["check", dir.to_str().unwrap()]);
    let level = format!("{out}{err}");
    let level_summary = level
        .lines()
        .find(|line| line.contains("підсумок"))
        .expect("there is a summary line")
        .to_string();
    assert!(
        !level_summary.contains("меж"),
        "a verdict that judged everything it claims says nothing about limits:\n{level_summary}"
    );

    // Then: the same tree on a branch that never reached origin. The
    // wave measured this on the real repository -- a shallow clone
    // gave 141 fewer checks and the SAME "0 findings" line.
    git(&dir, &["checkout", "-q", "-b", "0001-not-pushed"]);
    let (out, err, _) = keel(&["check", dir.to_str().unwrap()]);
    let alone = format!("{out}{err}");
    let alone_summary = alone
        .lines()
        .find(|line| line.contains("підсумок"))
        .expect("there is a summary line")
        .to_string();
    assert!(
        alone_summary.contains("меж"),
        "and one that could not says so in the line everyone reads:\n{alone_summary}"
    );
    assert_ne!(
        level_summary, alone_summary,
        "the two verdicts no longer end with the same words"
    );
    assert!(
        alone.contains("git push"),
        "and every limit carries its instead -- how to get the full verdict:\n{alone}"
    );
}
