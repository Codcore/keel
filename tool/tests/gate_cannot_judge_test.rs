//! Scenario test of wave 0052: a court that cannot judge does not
//! pass.
//!
//! The global review of 2026-09-06 (methodology R-10, bugs R-13, bugs
//! R-5) measured the commit court passing what it could not judge: an
//! adapter this release does not lead -- "the commit is not judged",
//! exit 0 -- over a red test; a wave header with a field the reader
//! does not know -- "the branch is not named as any wave that reads",
//! exit 0; and a message whose first lines git strips before it
//! records the subject -- "outside the judgement", exit 0, while the
//! recorded subject is `work: …` over a red test.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
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

fn gate(dir: &Path, message: &str) -> (String, i32) {
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, message).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
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

const BODY: &str = "тіло обіцянки\n\n";

fn decisions_except(covered: &[&str]) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// A rust crate: Cargo.toml, src/lib.rs, tests/ -- and the frame of
/// the methodology with the wave text given.
fn crate_with(name: &str, adapter: &str, wave_text: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), wave_text).unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    dir
}

/// The plain full wave: scenario `it-works`, transform `work` over
/// src/lib.rs.
fn plain_wave() -> String {
    format!(
        "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    )
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// proves: a-court-that-cannot-judge-does-not-pass@3b9819
#[test]
fn a_court_that_cannot_judge_does_not_pass() {
    let rev = keel::rev::text_rev(BODY);
    let red = format!(
        "/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    panic!(\"red\");\n}}\n"
    );

    // --- an adapter this release does not lead: a birth and work
    // are refused; a commit outside the judgement passes with the
    // word, as before ---
    let dir = crate_with("cannotgo", "go", &plain_wave());
    std::fs::write(dir.join("tests/w_test.rs"), &red).unwrap();
    settle(&dir);
    for message in ["red: it-works\n", "work: тіло\n"] {
        let (said, code) = gate(&dir, message);
        assert_ne!(
            code,
            0,
            "an adapter this release does not lead cannot judge `{}`, and a court \
             that cannot judge does not pass:\n{said}",
            message.trim()
        );
        assert!(
            said.contains("\"go\""),
            "and the refusal names the adapter:\n{said}"
        );
    }
    // A slug-shaped head unknown to the wave (`docs: …`) is a typo
    // refusal on any adapter (§8.4); outside the judgement is a
    // message with no such head at all.
    let (said, code) = gate(&dir, "Merge branch 'main' into 0001-a-wave\n");
    assert_eq!(
        code, 0,
        "a commit outside the judgement still passes:\n{said}"
    );
    assert!(
        said.contains("не суджений"),
        "with the word that the adapter is not this release's:\n{said}"
    );

    // --- a wave header the reader cannot read: the branch IS the
    // wave, and the refusal carries the header's own words ---
    let mut broken = plain_wave();
    broken = broken.replacen("---\nscenarios:", "---\npriority: high\nscenarios:", 1);
    let dir = crate_with("cannotheader", "rust", &broken);
    std::fs::write(dir.join("tests/w_test.rs"), &red).unwrap();
    settle(&dir);
    let (said, code) = gate(&dir, "work: тіло\n");
    assert_ne!(
        code, 0,
        "a header the court cannot read is a refusal, not a pass:\n{said}"
    );
    assert!(
        said.contains("priority") && !said.contains("не зветься як жодна"),
        "with the header's own words, never \"not named as any wave\":\n{said}"
    );

    // --- the message as git records it: blank lines and `#` comment
    // lines before the subject are not the subject ---
    let dir = crate_with("cannotstrip", "rust", &plain_wave());
    std::fs::write(dir.join("tests/w_test.rs"), &red).unwrap();
    settle(&dir);
    let (said, code) = gate(
        &dir,
        "\n# Please enter the commit message for your changes.\n# Lines starting with '#' will be ignored.\nwork: typed after an Enter\n",
    );
    assert_ne!(
        code, 0,
        "the subject git records is `work: …`, and the test is red:\n{said}"
    );
    assert!(
        said.contains("падає") && !said.contains("поза судом"),
        "the court read the subject, not the comment:\n{said}"
    );
}
