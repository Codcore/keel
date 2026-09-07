//! Scenario test of wave 0057: declared and touched are one file.
//!
//! Measured before the plan, in a sandbox, on keel 1.0.0: a wave that
//! declared `./src/a.rs` over a branch that touched `src/a.rs` got
//! TWO findings -- "гілка чіпає src/a.rs, якого жодна трансформа не
//! називає" and "оголошений файл ./src/a.rs гілка не чіпає" -- and
//! neither said the two were one file (queue after 0055: bugs R-22).
//! The rows of scope were compared as bare strings, so `./` made
//! another file of the same one.
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

/// The wave of one transform over the given scope rows, written as
/// the person wrote them -- `./` and all.
fn wave_over(rows: &[&str]) -> String {
    let mut list = String::new();
    for row in rows {
        list.push_str(&format!("      - {row}\n"));
    }
    format!(
        "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n{list}{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    )
}

fn crate_with(name: &str, wave_text: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
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

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// The branch as the loop leaves it: a red birth, then the work --
/// both over `src/lib.rs` and the probe file, whatever spelling the
/// wave used for them.
fn birth_and_work(dir: &Path) {
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    panic!(\"red\");\n}}\n"),
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "--no-verify", "-m", "red: it-works"]);
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// work\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(toy::works());\n}}\n"
        ),
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "--no-verify", "-m", "work: done"]);
}

/// proves: declared-and-touched-are-one-file@423060
#[test]
fn declared_and_touched_are_one_file() {
    // --- the same file, written two ways: one file, no findings ---
    let dir = crate_with("dotslash", &wave_over(&["./src/lib.rs", "./tests/w_test.rs"]));
    settle(&dir);
    birth_and_work(&dir);
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        !said.contains("гілка чіпає \"src/lib.rs\""),
        "a file declared as ./src/lib.rs and touched as src/lib.rs is \
         not drift -- it is the same file:\n{said}"
    );
    assert!(
        !said.contains("оголошений файл \"./src/lib.rs\""),
        "and it is not untouched either -- one file, not two:\n{said}"
    );
    assert_eq!(code, 0, "so the branch is green:\n{said}");

    // --- the words keep the row as the person wrote it ---
    let dir = crate_with(
        "spelling",
        &wave_over(&["./src/lib.rs", "./tests/w_test.rs", "./src/never.rs"]),
    );
    settle(&dir);
    birth_and_work(&dir);
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains("оголошений файл \"./src/never.rs\""),
        "a row nothing touched is still a finding, and it quotes the \
         row as it stands in the wave -- the person must find it with \
         their eyes:\n{said}"
    );
    assert_eq!(code, 2, "and the branch is red for it:\n{said}");

    // --- a row that leaves the tree says exactly that ---
    let dir = crate_with(
        "outside",
        &wave_over(&["src/lib.rs", "tests/w_test.rs", "../outside.rs"]),
    );
    settle(&dir);
    birth_and_work(&dir);
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains("поза деревом"),
        "normalising `./` does not legalise `..`: a row outside the \
         project's root is a finding that says so, not a row that can \
         never match:\n{said}"
    );
    assert!(
        said.contains("../outside.rs"),
        "and it names the row:\n{said}"
    );
    assert_eq!(code, 2, "the branch is red for it:\n{said}");
}
