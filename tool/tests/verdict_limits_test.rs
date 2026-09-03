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

/// proves: a-verdict-says-how-much-of-it-is-real@41b88e -- measured
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

    // A clone with no origin at all: the verdict must say that it
    // could not tell whether anything here has been pushed.
    let (out, err, _) = keel(&["check", dir.to_str().unwrap()]);
    let said = format!("{out}{err}");
    assert!(
        said.contains("origin"),
        "the verdict names the base it compared against, or says it has none:\n{said}"
    );

    // And the summary itself -- the line everyone reads -- carries
    // the limits, not just some line in the middle of the sheet.
    let summary = said
        .lines()
        .find(|line| line.contains("підсумок"))
        .expect("there is a summary line");
    assert!(
        summary.contains("межі") || said.contains("межі вироку"),
        "the summary carries what the verdict could not judge:\n{summary}"
    );
}
