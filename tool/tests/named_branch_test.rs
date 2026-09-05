//! Scenario test of wave 0035: the branch can be named where git
//! hides it.

mod common;

use common::keel_sandbox;
use std::process::Command;

fn git(dir: &std::path::Path, args: &[&str]) {
    Command::new("git")
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap();
}

fn check(dir: &std::path::Path, branch: Option<&str>) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_keel"));
    command.args(["check", dir.to_str().unwrap()]);
    match branch {
        Some(name) => {
            command.env("KEEL_BRANCH", name);
        }
        None => {
            command.env_remove("KEEL_BRANCH");
        }
    }
    let out = command.output().unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

const WAVE: &str = "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n";

/// proves: the-branch-can-be-named-where-git-hides-it@8d0388 --
/// the conformance audit measured the scope court skipped ENTIRELY
/// on a detached HEAD, which is how `actions/checkout` leaves a
/// repository for a `pull_request` event: the commonest shape of CI
/// there is. §4.10 foresaw this and says to name the branch
/// explicitly; there was no way to name it.
#[test]
fn the_branch_can_be_named_where_git_hides_it() {
    let dir = keel_sandbox("named");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), WAVE).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "").unwrap();
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
    // Work on the wave's branch, then let go of the branch: this is
    // the state CI hands the tool.
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(dir.join("src/lib.rs"), "// work\n").unwrap();
    std::fs::write(dir.join("stray.txt"), "not declared anywhere\n").unwrap();
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
            "work: the promise",
        ],
    );
    git(&dir, &["checkout", "-q", "--detach"]);

    // Nobody names it: the honest "not compared", as before.
    let said = check(&dir, None);
    assert!(
        said.contains("scope") && !said.contains("stray.txt"),
        "with nothing to go on the court says it did not compare:\n{said}"
    );

    // The environment names it: the court runs, and finds the file
    // no transform declared.
    let said = check(&dir, Some("0001-a-wave"));
    assert!(
        said.contains("stray.txt"),
        "named by the environment, the scope court runs and finds what \
         the branch touched outside the declared files:\n{said}"
    );
    assert!(
        said.contains("червоне"),
        "and it is a finding, not a note:\n{said}"
    );
}
