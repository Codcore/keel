//! Scenario test of wave 0075: the barrier stands at the plan.
//!
//! Measured on the released 1.4.0 binary before any of this was
//! written: a full wave, its plan on `plan/0019-a-full-wave`, and
//! `keel/reviews/` empty.
//!
//!     keel check .  -> exit 0
//!     keel close .  -> exit 0
//!
//! The plan PR is green and mergeable with no reviewer, at the only
//! moment a plan can still be refused as a plan (§6.6). §9.9 says the
//! machine holds that barrier; it does not.

mod common;

use common::keel_sandbox;
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
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

/// A full wave -- a promise, a transform that implements it -- with
/// its plan on `plan/<wave>` and nothing else.
fn plan_sandbox(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!(
                "  {cut}: \"не про цю пісочницю, вона грає інше\"\n"
            ));
        }
    }
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0019-a-full-wave"]);
    fs::write(
        dir.join("keel/waves/0019-a-full-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n      - tests/toy_test.rs\n{d}---\n\n## scenario: it-holds\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "plan: 0019"]);
    dir
}

/// proves: the-plan-has-its-own-reader-and-its-own-report@1cad4d
#[test]
fn the_plan_has_its_own_reader_and_its_own_report() {
    // --- a plan with no reader does not merge as a plan -----------
    let dir = plan_sandbox("planbare");
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a plan PR is approved by merging it (§6.6), and §9.9 says a \
         fresh eye reads it first -- so the court that lets that merge \
         through holds the barrier, or nothing does:\n{said}"
    );
    assert!(
        said.contains("0019-a-full-wave-plan.md"),
        "and it names the file it wants, which is NOT the work's \
         report:\n{said}"
    );
    // And the footer agrees with the exit code. This is the fourth
    // wave to be told the same thing (reviews 0052 R-13, 0055 R-6,
    // 0068 R-1, 0075 R-1): a report that counts a blocker and signs
    // off "no blockers" two lines later is the riddle these courts
    // exist to end. The footer is derived from ONE sum now, so a
    // fifth source cannot be forgotten -- and this assert is what
    // says so.
    assert!(
        !said
            .lines()
            .any(|line| line.starts_with("блокерів нема") || line.starts_with("no blockers")),
        "and the footer does not deny what the exit code carries:\n{said}"
    );

    // --- with the plan's own report it merges ---------------------
    let dir = plan_sandbox("planread");
    fs::write(
        dir.join("keel/reviews/0019-a-full-wave-plan.md"),
        "# Рецензія плану\n\nчитав, питання нижче\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the plan met a reader"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "a plan that met a reader merges:\n{said}");

    // --- and the WORK's report is not the plan's ------------------
    //
    // Without this side the barrier is imaginary. The issue that
    // asked for this wave named the trap itself: a report filed under
    // the wave's own name rides onto the work branch from birth and
    // satisfies the gate that exists to demand a review of the WORK.
    // So the court asks by an exact name, and a substring match would
    // undo the whole wave in silence.
    let dir = plan_sandbox("planwrongname");
    fs::write(
        dir.join("keel/reviews/0019-a-full-wave.md"),
        "# Рецензія\n\nце звіт РОБОТИ, і плану він не заміняє\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "the work's report, too early"],
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "the work's report is not the plan's: it would ride onto the \
         work branch and satisfy §9.9's gate over the work without \
         anybody reading the work:\n{said}"
    );

    // --- an empty report is not a report --------------------------
    let dir = plan_sandbox("planempty");
    fs::write(dir.join("keel/reviews/0019-a-full-wave-plan.md"), "").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "an empty file"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "an empty file is not a review, here as over the work \
         (§9.9):\n{said}"
    );

    // --- and the plan's report does NOT satisfy the work's gate ---
    //
    // The card calls this side the knot the whole wave hangs on, and
    // the first cut left it unmeasured (review R-2). If `<wave>-plan.md`
    // were accepted over the work, the barrier would have moved
    // rather than been added: a plan reviewed once would count as a
    // work reviewed never.
    let dir = plan_sandbox("planonwork");
    fs::write(
        dir.join("keel/reviews/0019-a-full-wave-plan.md"),
        "# Рецензія плану\n\nчитав\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the plan met a reader"]);
    git(&dir, &["checkout", "-q", "-b", "0019-a-full-wave"]);
    // The work itself, so the wave is finished but for its review.
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    let rev = keel::rev::text_rev(BODY);
    fs::write(
        dir.join("tests/toy_test.rs"),
        format!("/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{}}\n"),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--allow-empty", "-m", "red: it-holds"],
    );
    fs::write(
        dir.join("src/lib.rs"),
        "pub fn a() {}\npub fn b() {}\npub fn c() {}\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: the declared file"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "the plan's report is not the work's: a wave whose work \
         nobody read does not close because its PLAN was read \
         (§9.9):\n{said}"
    );
    // The court's own line about a missing WORK report, and not a
    // scope finding that happens to carry the wave's file name
    // (review 0075 R-2 measured the first cut matching the wrong
    // thing, and the mutant that unties the whole knot survived it).
    // The fixture is red for this reason and no other: the wave
    // declares every file the branch touched.
    assert!(
        said.contains("звіту рецензії keel/reviews/<хвиля>.md нема в історії гілки"),
        "and the court still asks for the WORK's own report:\n{said}"
    );

    // --- a plan branch named after no wave at all -----------------
    //
    // The barrier vanished in silence there: check 0, close 0, not a
    // word (review R-5). `keel check` says it aloud and `keel review`
    // refuses with the reason; only this court was mute, and a mute
    // green over a branch nobody can name is what §4.10 calls worse
    // than a red.
    let dir = plan_sandbox("plannowave");
    git(&dir, &["checkout", "-q", "-b", "plan/0099-nothing-here"]);
    git(
        &dir,
        &["commit", "-q", "--allow-empty", "-m", "a plan of nothing"],
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(code, 0, "a plan branch with no plan is not a plan:\n{said}");
    assert!(
        said.contains("0099-nothing-here"),
        "and the court names what it looked for:\n{said}"
    );

    // --- and `keel next` leads to the reader BEFORE the merge -----
    //
    // The Ukrainian line already said it in that order; the English
    // one said "merge the plan PR" first and "give the plan a fresh
    // eye ... before the merge" second, in ONE sentence. An agent
    // reading top to bottom merges. Both tongues are measured here,
    // because the defect was that they disagreed.
    let dir = plan_sandbox("plannext");
    for (lang, reader, merge) in [
        ("uk", "свіжому оку", "зливай план-PR"),
        ("en", "a fresh eye", "merge the plan PR"),
    ] {
        fs::write(
            dir.join("keel.toml"),
            format!("lang = \"{lang}\"\nadapter = \"rust\"\n"),
        )
        .unwrap();
        let (said, _) = keel(&dir, &["next"]);
        let step = said
            .lines()
            .find(|line| line.contains(reader))
            .unwrap_or_else(|| panic!("the step names the reader in {lang}:\n{said}"));
        let at_reader = step.find(reader).unwrap();
        let at_merge = step
            .find(merge)
            .unwrap_or_else(|| panic!("the step names the merge in {lang}:\n{said}"));
        assert!(
            at_reader < at_merge,
            "in {lang} the fresh eye comes BEFORE the merge -- one \
             sentence that says merge first and review second is read \
             top to bottom:\n{step}"
        );
    }
}
