//! Scenario test of wave 0034: the red birth is judged by the
//! branch, not only by a hook that may not be installed.

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

fn commit(dir: &std::path::Path, message: &str) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            message,
        ],
    );
}

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

const WAVE: &str = "---\nscenarios:\n  a-promise:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - a-promise\n    files:\n      - src/lib.rs\n---\n\n## scenario: a-promise\nтекст обіцянки\n\n## transform: work\nтекст роботи\n";

/// proves: the-red-birth-is-judged-by-the-branch@d13d4f -- the
/// load-bearing idea of the methodology is that a green test never
/// seen red proves nothing (§6.3), and §7.12 names two holders: the
/// hook and a branch check. The branch check did not exist, and
/// .git/hooks does not travel with git -- so the audit committed
/// work with no test and no red commit in a fresh clone and got
/// zero findings, then `keel close` called the wave closed.
#[test]
fn the_red_birth_is_judged_by_the_branch() {
    let dir = keel_sandbox("redbirth");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), WAVE).unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    commit(&dir, "start");

    // A wave branch that does the work and never earned its red.
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("src/lib.rs"),
        "/// proves: a-promise@0000\n#[test]\nfn a_promise() {}\n",
    )
    .unwrap();
    commit(&dir, "work: the promise, with no red behind it");

    let (said, _) = keel(&["check", dir.to_str().unwrap()]);
    assert!(
        said.contains("a-promise") && said.contains("red:"),
        "a scenario worked on without its red commit is a finding, and \
         the finding names the scenario and what is missing:\n{said}"
    );
    assert!(
        said.contains("червоне"),
        "and it is red, not advice:\n{said}"
    );

    // The same branch, with the red birth where it belongs.
    git(&dir, &["checkout", "-q", "main"]);
    git(&dir, &["branch", "-q", "-D", "0001-a-wave"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(dir.join("src/lib.rs"), "// nothing yet\n").unwrap();
    commit(&dir, "red: a-promise");
    std::fs::write(
        dir.join("src/lib.rs"),
        "/// proves: a-promise@0000\n#[test]\nfn a_promise() {}\n",
    )
    .unwrap();
    commit(&dir, "work: the promise, born red");

    let (said, _) = keel(&["check", dir.to_str().unwrap()]);
    assert!(
        !said.contains("red:"),
        "and a scenario that was born red is not accused of anything:\n{said}"
    );

    // The court does not depend on a hook: nothing was installed in
    // this sandbox, and both verdicts above were reached anyway.
    assert!(
        !dir.join(".git/hooks/commit-msg").exists(),
        "no hook was installed here -- the branch is what was judged"
    );
}
