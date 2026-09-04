//! Scenario test of wave 0036: a plan branch carries no code.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn check(dir: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

const WAVE: &str = "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n";

/// The sandbox at the fork point: a project with git, a base commit,
/// and nothing of the wave yet.
fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(dir.join("README.md"), "проєкт\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: a-plan-branch-carries-no-code@fde06c -- the conformance
/// audit (ВАЖКА-4) measured §4.9 held by nothing: a branch called
/// `plan/<wave>` is not named after a wave, so the scope court is
/// skipped entirely and code laid down there is seen by no one --
/// in the work branch it is no longer in the diff.
#[test]
fn a_plan_branch_carries_no_code() {
    // Code on a plan branch is a finding, and it names the file.
    let dir = project("planbranch");
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), WAVE).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn sneaked() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "plan: wave 0001"]);

    let said = check(&dir);
    assert!(
        said.contains("червоне")
            && said.contains("src/lib.rs")
            && said.contains("несе план, а не код"),
        "code on a plan branch is a finding that names the file (§4.9):\n{said}"
    );
    assert!(
        said.contains("plan/0001-a-wave"),
        "and it names the branch it judged:\n{said}"
    );

    // The plan itself is silence: keel/ is the wave's own furniture
    // (§4.8), and a plan branch is exactly where it is written.
    let dir = project("planonly");
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), WAVE).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "plan: wave 0001"]);

    let said = check(&dir);
    assert!(
        !said.contains("несе план, а не код"),
        "a plan branch carrying only the plan is silence:\n{said}"
    );
    assert!(
        said.contains("plan/0001-a-wave"),
        "and the court still says which branch it judged, rather than \
         skipping in silence:\n{said}"
    );

    // A `plan/` of something that is no wave gets an honest word, not
    // a silent pass.
    let dir = project("plannowave");
    git(&dir, &["checkout", "-q", "-b", "plan/0099-nowhere"]);
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "plan: nothing"]);

    let said = check(&dir);
    assert!(
        said.contains("0099-nowhere"),
        "a plan branch of no wave is named aloud:\n{said}"
    );
}
