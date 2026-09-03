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
    // git keeps no empty directories, so a clone of this sandbox
    // would arrive with no keel/ at all and be refused before any
    // verdict is reached.
    for kept in ["keel/waves/.gitkeep", "keel/contracts/.gitkeep"] {
        std::fs::write(dir.join(kept), "").unwrap();
    }
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
    // Several commits, so a depth-1 clone is genuinely cut off: with
    // a single commit git fetches the lot and marks nothing shallow.
    for n in 1..4 {
        std::fs::write(dir.join(format!("note{n}.txt")), format!("{n}")).unwrap();
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
                "more",
            ],
        );
    }
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

    // The heaviest clause, and the one nothing guarded: a SHALLOW
    // clone. Review 0031 R-2 removed the shallow limit outright and
    // the whole battery stayed green, because no probe ever made
    // one -- so the wave's own promise, that a shallow verdict and a
    // full one no longer end alike, was held by nobody.
    let shallow = dir.join("shallow");
    Command::new("git")
        .args(["clone", "-q", "--depth", "1"])
        // file:// rather than a path: a local clone hard-links the
        // whole object store and comes out with the full history,
        // shallow depth or not.
        .arg(format!("file://{}", dir.display()))
        .arg(&shallow)
        .status()
        .unwrap();
    assert_eq!(
        git_out(&shallow, &["rev-parse", "--is-shallow-repository"]),
        "true",
        "the clone really is shallow"
    );
    let (out, err, _) = keel(&["check", shallow.to_str().unwrap()]);
    let cut = format!("{out}{err}");
    let cut_summary = cut
        .lines()
        .find(|line| line.contains("підсумок"))
        .unwrap_or_else(|| {
            panic!(
                "a summary line in:
{cut}"
            )
        })
        .to_string();
    assert!(
        cut_summary.contains("меж"),
        "a shallow verdict says so in the line everyone reads:\n{cut_summary}"
    );
    assert!(
        cut.contains("shallow") && cut.contains("git fetch --unshallow"),
        "names what was cut off and how to get it back:\n{cut}"
    );
    assert_ne!(
        level_summary, cut_summary,
        "and a shallow verdict no longer ends with the same words as a full one"
    );

    // A directory with no git at all is not a clone with problems --
    // it is not a clone. Review 0031 R-8 found it told "this clone
    // knows no remote trunk", which is true of a shoebox too.
    let bare_dir = keel_sandbox("nogit");
    std::fs::write(bare_dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let (out, err, _) = keel(&["check", bare_dir.to_str().unwrap()]);
    let quiet = format!("{out}{err}");
    assert!(
        !quiet.contains("межа вироку"),
        "a directory with no repository is asked nothing:\n{quiet}"
    );
}

/// One line of git output from a directory, for the probe's own
/// questions.
fn git_out(dir: &std::path::Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
