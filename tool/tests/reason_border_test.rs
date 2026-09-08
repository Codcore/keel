//! The border of wave 0066's court, held end to end.
//!
//! The court itself is measured by `reason_test` through the library,
//! cheaply. This file measures the thing that lives in `check.rs` and
//! nowhere else: WHERE the court runs. Review 0066 F-2 and F-3 paid
//! for the difference -- a mutant that removed the border entirely
//! (`still_open = true`) left the whole battery green, and with no
//! adapter named the court printed 49 findings over waves merged long
//! ago, under its own line saying history is not rewritten.
//!
//! So these cases go through `keel check` as a person runs it, and
//! they cost a sandbox each. That is the price of measuring a border.

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

fn keel(dir: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
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

/// A project with one wave whose answers take the forms this tree
/// really holds, and one cut left as the bare formula.
///
/// `adapter` is written only when asked: the court's border leans on
/// whether the tag question could be asked at all, and a project that
/// names no adapter is the ordinary way it cannot be.
fn project(name: &str, bare_cut: &str, adapter: bool) -> common::Sandbox {
    let dir = keel_sandbox(name);
    let conf = if adapter {
        "lang = \"uk\"\nadapter = \"rust\"\n"
    } else {
        "lang = \"uk\"\n"
    };
    std::fs::write(dir.join("keel.toml"), conf).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        let said = if *cut == bare_cut {
            "не застосовується".to_string()
        } else if cut.starts_with("security") {
            "названо: ця пісочниця не має чого захищати".to_string()
        } else {
            "не застосовується, бо ця пісочниця грає один розріз".to_string()
        };
        decided.push_str(&format!("  {cut}: \"{said}\"\n"));
    }
    // No scenario and no transform: this wave is a chore, so the scope
    // court has nothing to compare and the border is measured alone.
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"пісочниця, що грає саму межу\"\n    files:\n      - src/lib.rs\n{decided}---\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn says_reason(said: &str) -> bool {
    said.contains("відповідь без причини")
}

/// proves: an-answer-without-a-reason-is-a-finding@9b1dce
#[test]
fn the_court_of_reasons_knows_where_it_may_run() {
    // --- a wave still being written: the bare formula is a finding,
    // and the finding names the cut ---
    let dir = project("reasonopen", "performance.capacity", true);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "a bare formula reddens a wave being written:\n{said}");
    assert!(
        says_reason(&said) && said.contains("performance.capacity"),
        "and the finding names the cut whose answer says nothing:\n{said}"
    );

    // --- the same wave, closed: history is not rewritten ---
    // Mutant of review 0066 F-3: `still_open = true` removes the
    // border, and NOTHING in the battery noticed. This case is that
    // notice.
    let dir = project("reasonclosed", "performance.capacity", true);
    std::fs::write(dir.join("keel/reviews/0001-a-wave.md"), "# Рецензія\n\nok\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the wave is in the trunk"]);
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        !says_reason(&said),
        "a wave that stands closed is history, and a new rule does not \
         rewrite it:\n{said}"
    );
    assert_eq!(code, 0, "and nothing else reddens either:\n{said}");

    // --- the project names no adapter: the question «is this wave
    // closed» was never really asked, so the court must not answer it
    // «open» and judge everything ---
    // Review 0066 F-2 measured this: 49 findings over waves merged
    // long ago, printed under the tool's own line saying history is
    // not rewritten.
    let dir = project("reasonnoadapter", "performance.capacity", false);
    std::fs::write(dir.join("keel/reviews/0001-a-wave.md"), "# Рецензія\n\nok\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the wave is in the trunk"]);
    let (said, _) = keel(&dir, &["check"]);
    assert!(
        !says_reason(&said),
        "with no adapter named, «not closed» means «not asked», and the \
         court of reasons stays out of history:\n{said}"
    );
}
