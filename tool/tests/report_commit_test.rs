//! Scenario test of wave 0055: the report is committed, and the
//! courts say how.
//!
//! Three measurements of the final review (2026-09-06): `keel next`
//! said where the review report lands and never how it is committed,
//! while `review: <wave>` -- the obvious subject -- was refused by
//! the hook as a typo in a slug, with no word about what to write
//! instead (bugs R-6); `keel close` read the report from the WORKING
//! TREE and called a wave closed over a file no commit carries
//! (bugs R-7); and of the gate's refusals only four of thirty-four
//! carried the "instead" §9.7 asks for (methodology R-4).
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, keel_sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
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

/// `keel gate` over a message, the way the hook calls it.
fn gate(dir: &Path, message: &str) -> (String, i32) {
    let msg = dir.join("COMMIT_EDITMSG");
    fs::write(&msg, message).unwrap();
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

const BODY: &str = "body of s\n";

/// A crate on the branch of wave 0009-w: one promise, one transform
/// over it, and the journal chore that carries the review file --
/// the shape every wave of this project has.
fn project(name: &str, lang: &str, test_body: &str) -> Sandbox {
    let dir = keel_sandbox(name);
    write(
        &dir,
        "keel.toml",
        &format!("lang = \"{lang}\"\nadapter = \"cargo\"\n"),
    );
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    let rev = keel::rev::text_rev(BODY);
    write(
        &dir,
        "tests/t_test.rs",
        &format!("/// proves: s@{rev}\n#[test]\nfn holds_s() {{ {test_body} }}\n"),
    );
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\n{BODY}\n## transform: t\n\nthe work\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the trunk"]);
    git(&dir, &["checkout", "-q", "-b", "0009-w"]);
    dir
}

/// The wave assembled: the work done in the transform's own file and
/// committed under its slug, which is what leaves the review as the
/// next step (§6.2, §9.9).
fn assembled(dir: &Path) {
    write(dir, "src/lib.rs", "pub fn works() -> bool {\n    true\n}\n");
    git(dir, &["add", "."]);
    git(dir, &["commit", "-q", "-m", "t: the work"]);
}

/// Every key the gate refuses with, read from its own source: the
/// list cannot go stale behind a fixture that reaches only six of
/// them (§9.7 asks for an instead in EVERY limit).
fn refusal_keys() -> Vec<String> {
    let source =
        fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("src/gate.rs")).unwrap();
    let mut keys: Vec<String> = Vec::new();
    for piece in source.split("Verdict::Refuse(").skip(1) {
        let Some(start) = piece.find("\"gate-") else {
            continue;
        };
        let rest = &piece[start + 1..];
        let Some(end) = rest.find('"') else { continue };
        let key = rest[..end].to_string();
        if !keys.contains(&key) {
            keys.push(key);
        }
    }
    assert!(
        keys.len() >= 12,
        "the reader found the gate's refusals: {keys:?}"
    );
    keys
}

/// proves: the-report-is-committed-and-the-courts-say-how@c63c5a --
/// the report of §9.9 had no commit named for it, the obvious
/// subject was refused without an instead, and the closing court
/// blessed a wave over a report that lived only in the working tree.
#[test]
fn the_report_is_committed_and_the_courts_say_how() {
    // -- the step names the commit the report lands as ----------------
    let dir = project("reportstep", "uk", "");
    assembled(&dir);
    let (said, code) = keel(&dir, &["next"]);
    assert_eq!(code, 0, "the step is given:\n{said}");
    assert!(
        said.contains("keel/reviews/0009-w.md"),
        "it names the file the report lands in:\n{said}"
    );
    assert!(
        said.contains("chore"),
        "and how it is committed -- under a chore of this wave:\n{said}"
    );
    assert!(
        said.contains("без слага"),
        "or with a subject that is no slug at all, which the gate \
         passes as outside its judgement (§8.4):\n{said}"
    );

    // -- and `review: …` is refused with that same advice --------------
    for (lang, instead) in [("uk", "натомість"), ("en", "instead")] {
        let dir = project(&format!("reportgate{lang}"), lang, "");
        let (said, code) = gate(&dir, "review: 0009-w\n");
        assert_eq!(
            code, 1,
            "{lang}: a slug the wave does not know refuses:\n{said}"
        );
        assert!(
            said.contains(instead),
            "{lang}: and the refusal carries what to do instead (§9.7):\n{said}"
        );
        assert!(
            said.contains("keel next"),
            "{lang}: and points at the hand that names the wave's own \
             slugs:\n{said}"
        );
    }

    // -- every refusal of this court carries an instead ---------------
    // Six shapes through the gate itself, in the project's tongue.
    let dir = project("refusalshapes", "uk", "assert!(true);");
    for message in [
        "review: 0009-w\n", // a slug the wave does not know
        "Red: s\n",         // the capitalized twin
        "red: nosuch\n",    // a scenario the wave does not know
        "red: s\n",         // a birth over a green test
        "t: work\n",        // work over a stale tag (below)
    ] {
        let (said, code) = gate(&dir, message);
        assert_eq!(code, 1, "the gate refuses {message:?}:\n{said}");
        assert!(
            said.contains("натомість"),
            "and says what to do instead (§9.7) over {message:?}:\n{said}"
        );
    }
    // Work over a test that falls, and over a tag whose revision is
    // stale: two more shapes, each with its own advice.
    let dir = project("workrefusals", "uk", "assert!(false);");
    let (said, code) = gate(&dir, "t: work\n");
    assert_eq!(code, 1, "work over a falling test refuses:\n{said}");
    assert!(
        said.contains("натомість"),
        "with the instead §9.7 asks for:\n{said}"
    );
    write(
        &dir,
        "tests/t_test.rs",
        "/// proves: s@beef00\n#[test]\nfn holds_s() {}\n",
    );
    let (said, code) = gate(&dir, "t: work\n");
    assert_eq!(code, 1, "a stale tag refuses:\n{said}");
    assert!(
        said.contains("натомість") && said.contains("keel rev"),
        "and names the hand that moves the tags:\n{said}"
    );

    // And the WORDS THEMSELVES, not only those a fixture reaches:
    // every key this court refuses with carries an instead in both
    // tongues (methodology R-4 counted four of thirty-four).
    let i18n = Path::new(env!("CARGO_MANIFEST_DIR")).join("i18n");
    for (file, tongue) in [("uk.ftl", "uk"), ("en.ftl", "en")] {
        let text = fs::read_to_string(i18n.join(file)).unwrap();
        for key in refusal_keys() {
            assert!(
                text.lines()
                    .any(|line| line.starts_with(&format!("{key}-instead ="))),
                "{tongue}: the refusal {key} carries no instead (§9.7) in {file}"
            );
        }
    }

    // -- the closing court reads the report from history --------------
    let dir = project("reportclose", "uk", "");
    assembled(&dir);
    // The report lies in the working tree and no commit carries it.
    write(&dir, "keel/reviews/0009-w.md", "# Рецензія\n\nok\n");
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a report no commit carries does not close a wave -- §9.9 puts \
         the record into history, and a file lying beside it merges \
         with nothing:\n{said}"
    );
    assert!(
        said.contains("keel/reviews") || said.contains("рецензі"),
        "and the lack is named as the report's:\n{said}"
    );
    // Committed, the same file closes it.
    git(&dir, &["add", "."]);
    git(
        &dir,
        &["commit", "-q", "-m", "journal: the record of the wave"],
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "and once the commit carries it, the wave closes:\n{said}"
    );
    assert!(
        said.contains("закрита"),
        "with the word of a closed wave:\n{said}"
    );
}
