//! Scenario test of wave 0036: research does not merge.

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

fn keel(dir: &Path, command: &str) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args([command, dir.to_str().unwrap()])
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

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  it-holds:\n    withdrawn: \"не про це\"\ntransforms:\n  work:\n    chore: \"нічого\"\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: research-does-not-merge@5812f0 -- §4.13 promises the ban
/// is held BY MACHINE ("the check on a PR from `spike/*` is red with
/// an explanation"), and the norm audit (Л-8) plus the conformance
/// audit (ВАЖКА-2) both measured the same thing: the word `spike`
/// does not appear in the code at all. A research branch was judged
/// like any other stranger's branch -- that is, not at all -- and
/// merged without a word.
#[test]
fn research_does_not_merge() {
    let dir = project("spike");
    git(&dir, &["checkout", "-q", "-b", "spike/try-something"]);
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn tried() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "spike: playing with it"]);

    // check says so aloud and does not judge the branch (§4.13).
    let (said, code) = keel(&dir, "check");
    assert!(
        said.contains("spike/try-something") && said.contains("§4.13"),
        "a research branch is named aloud, not passed over in silence:\n{said}"
    );
    assert_eq!(
        code, 0,
        "and research itself is not a finding -- it is outside the \
         methodology, which the paragraph says in as many words:\n{said}"
    );

    // close is the court that says whether this may be merged, and
    // for research the answer is never.
    let (said, code) = keel(&dir, "close");
    assert_eq!(
        code, 1,
        "closing research is refused (§4.13):\n{said}"
    );
    assert!(
        said.contains("spike/try-something") && said.contains("хвилею"),
        "and the refusal says what to do instead -- bring the finding \
         back as a wave:\n{said}"
    );

    // An ordinary branch is untouched by any of this.
    let dir = project("ordinary");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("§4.13"),
        "a wave branch hears nothing about research:\n{said}"
    );
}
