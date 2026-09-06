//! Scenario test of wave 0054: check says only what it judged.
//!
//! Measured before the plan: in a project without git and without an
//! adapter `keel check` said "scope was not compared" and "tags were
//! not verified" -- and then printed a static line claiming scope,
//! tags and trust among "what exactly was checked", and a summary of
//! "0 findings" that counted nothing as unchecked (queue row
//! `check.rs:745/:732`).
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

/// The number the summary line names as not checked -- zero when it
/// names none.
fn summary_unchecked(said: &str) -> usize {
    let line = said
        .lines()
        .find(|l| l.starts_with("підсумок"))
        .expect("a summary line");
    line.split(',')
        .find(|piece| piece.contains("не перевірено"))
        .and_then(|piece| {
            piece
                .split_whitespace()
                .find_map(|word| word.parse::<usize>().ok())
        })
        .unwrap_or(0)
}

fn checked_line(said: &str) -> &str {
    said.lines()
        .find(|l| l.starts_with("що саме перевірено"))
        .expect("the line naming what was checked")
}

/// proves: check-says-only-what-it-judged@d473ec
#[test]
fn check_says_only_what_it_judged() {
    // --- no git, no adapter: scope and tags were not judged, and the
    // verdict counts them so, and does not list them as checked ---
    // A wave that declares files and a promise: the scope court has
    // something to compare and no git to compare with, the tag court
    // something to verify and no adapter -- two stand-downs, counted.
    // (A directory with no wave at all is asked nothing: wave 0031
    // holds that such a verdict names no limit.)
    let dir = keel_sandbox("checkedbare");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(dir.join("README.md"), "# readme\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - README.md\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло\n",
            decisions_except(&["functional.correctness"])
        ),
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "nothing is red in a bare project:\n{said}");
    let unchecked = said
        .lines()
        .filter(|l| l.starts_with("не перевірено"))
        .count();
    assert!(
        unchecked >= 2,
        "scope and tags are said not to have been judged, each as a limit line:\n{said}"
    );
    assert_eq!(
        summary_unchecked(&said),
        unchecked,
        "and the summary counts exactly those lines:\n{said}"
    );
    let line = checked_line(&said);
    assert!(
        !line.contains("scope гілки") && !line.contains("тегах тестів") && !line.contains("довіра"),
        "the line naming what was checked does not name the scope court, the tag court or the \
         trust court -- two stood down, the third had nothing declared (review 0054 R-2):\n{line}"
    );

    // --- git and an adapter: the same courts ran, and the line says
    // so ---
    let dir = keel_sandbox("checkedfull");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{}}\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
            decisions_except(&["functional.correctness"])
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["check"]);
    let line = checked_line(&said);
    assert!(
        line.contains("scope гілки") && line.contains("тегах тестів"),
        "with git and an adapter the line names the scope court and the tag court among what \
         was checked:\n{line}"
    );
    assert_eq!(
        summary_unchecked(&said),
        said.lines()
            .filter(|l| l.starts_with("не перевірено"))
            .count(),
        "and the summary still counts exactly the limit lines:\n{said}"
    );

    // --- the courts that do not apply say so and are not counted;
    // the ones that stood down over something are (review 0054 R-5,
    // R-8) ---
    let counted = |said: &str| {
        (
            summary_unchecked(said),
            said.lines()
                .filter(|l| l.starts_with("не перевірено"))
                .count(),
        )
    };
    // A plan branch: the form court does not run (§8.3) -- printed,
    // not counted; and on a trunk named otherwise the comparison
    // that could not happen is one line, not two.
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    let (said, _) = keel(&dir, &["check"]);
    assert!(
        said.contains("суд форми не біжить"),
        "the plan branch says the form court does not run:\n{said}"
    );
    let (number, lines) = counted(&said);
    assert_eq!(number, lines, "and the count is the lines:\n{said}");
    assert!(
        !checked_line(&said).contains("тримання форми"),
        "and the form court is not among the checked:\n{said}"
    );
    git(&dir, &["branch", "-m", "main", "trunk"]);
    let (said, _) = keel(&dir, &["check"]);
    // The remote's absence is its own limit; the trunk's absence --
    // the comparison that could not happen -- is one line, though two
    // courts asked for it.
    let about_trunk = said
        .lines()
        .filter(|l| l.starts_with("не перевірено") && l.contains("стовбура"))
        .count();
    assert_eq!(
        about_trunk, 1,
        "a comparison that could not happen is counted once (review 0054 R-8):\n{said}"
    );
    let (number, lines) = counted(&said);
    assert_eq!(number, lines, "and the count is the lines:\n{said}");
    git(&dir, &["branch", "-m", "trunk", "main"]);
    // Research (§4.13): scope is outside judgement by design.
    git(&dir, &["checkout", "-q", "-b", "spike/idea"]);
    let (said, _) = keel(&dir, &["check"]);
    assert!(said.contains("§4.13"), "research is named:\n{said}");
    let (number, lines) = counted(&said);
    assert_eq!(number, lines, "and not counted as unchecked:\n{said}");
    // A broken document: trust over rubble stands down, counted.
    git(&dir, &["checkout", "-q", "main"]);
    std::fs::write(dir.join("keel/contracts/broken.md"), "---\nmodule: X\n").unwrap();
    let (said, _) = keel(&dir, &["check"]);
    assert!(
        said.lines()
            .any(|l| l.starts_with("не перевірено") && l.contains("verify/ci")),
        "trust over a broken document is a counted stand-down:\n{said}"
    );
    let (number, lines) = counted(&said);
    assert_eq!(number, lines, "and the count is the lines:\n{said}");
    std::fs::remove_file(dir.join("keel/contracts/broken.md")).unwrap();
    // The branch of a cancelled wave: outside judgement, not counted.
    let cancelled = std::fs::read_to_string(dir.join("keel/waves/0001-a-wave.md"))
        .unwrap()
        .replacen(
            "---\nscenarios:",
            "---\ncancelled: \"передумали\"\nscenarios:",
            1,
        );
    git(&dir, &["checkout", "-q", "0001-a-wave"]);
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), cancelled).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "called off"]);
    let (said, _) = keel(&dir, &["check"]);
    assert!(
        said.contains("скасован"),
        "the cancelled wave's branch is named:\n{said}"
    );
    let (number, lines) = counted(&said);
    assert_eq!(number, lines, "and not counted as unchecked:\n{said}");
}
