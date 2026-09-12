//! Scenario test of wave 0068: the closing court is not narrower than
//! the check.
//!
//! Measured before the work, on one tree and one commit: a branch
//! touching a file no transform of its wave names gives
//! `keel check` exit 1 and `keel close` exit 0. The court that lets a
//! branch into a merge was blind to what the cheaper court beside it
//! had already found -- and it spent thirteen minutes of battery
//! before saying so.

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

fn all_decided() -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        block.push_str(&format!("  {cut}: \"n/a, бо ця пісочниця грає інше\"\n"));
    }
    block
}

/// A crate with git, the cargo adapter and one chore wave, standing on
/// the wave's own branch with a trunk behind it.
fn project(name: &str) -> Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "pub fn a() {}\n");
    write(
        &dir,
        "keel/waves/0001-a-chore.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
            all_decided()
        ),
    );
    write(&dir, "keel/reviews/0001-a-chore.md", "# Рецензія\n\nok\n");
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-chore"]);
    dir
}

/// proves: the-cheap-court-runs-first-and-locally@2a86bf
#[test]
fn the_cheap_court_runs_first_and_locally() {
    // --- a finding of `keel check` is a blocker of `keel close` ---
    let dir = project("closesees");
    write(&dir, "src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    // A file no transform of the wave names: §4.6 drift.
    write(&dir, "build.rs", "fn main() {}\n");
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "tidy: the work and a stranger"],
    );

    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 1,
        "the fixture must really be red for check, or this probe \
         proves nothing:\n{checked}"
    );
    assert!(
        checked.contains("build.rs"),
        "and red for the drifted file by name:\n{checked}"
    );

    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_ne!(
        close_code, 0,
        "the court that lets a branch into a merge is not narrower \
         than the one beside it: what `keel check` calls a finding, \
         `keel close` calls a blocker:\n{closed}"
    );
    assert!(
        closed.contains("build.rs"),
        "and it names the same file, not a number:\n{closed}"
    );

    // --- and it is a BLOCKER, counted, not a remark ---
    //
    // A probe that only read the exit code would pass over a court
    // that printed the finding and let the merge through anyway.
    //
    // The first cut of this assert read `contains("blockers")`, and
    // the substring `blockers` stands inside `no blockers` -- the
    // very line that was the defect (review R-2). A mutant that
    // renamed the court's own words to "a remark, nothing more" left
    // it green. So the words it hunts are the ones that can only
    // stand when the court counted: its own summary line, and the
    // ABSENCE of the footer that says nothing blocks.
    assert!(
        closed.contains("findings of the documents court on this branch's own files: 1"),
        "and the documents court has a counted line of its own, beside \
         the other reasons a wave does not close:\n{closed}"
    );
    assert!(
        !closed
            .lines()
            .any(|line| line.starts_with("no blockers") || line.starts_with("блокерів нема")),
        "and the footer does not say the opposite two lines below it: \
         a report that counts a blocker and signs off \"no blockers\" \
         is the riddle this wave came to end (reviews 0052 R-13, 0055 \
         R-6, and R-1 of this wave's own):\n{closed}"
    );

    // --- the OTHER half of "the branch's own files" ---------------
    //
    // `mine` is `scope::touched` UNION the wave's own file, and until
    // this probe the whole red side hung on the second half: every
    // finding of the branch court lands on `keel/waves/<slug>.md`, so
    // a mutant that deleted `scope::touched` outright left the
    // battery green (review R-3). A finding whose place is an
    // ordinary file of the tree is what measures the first half, and
    // a tag naming a scenario no wave declares is such a finding.
    let dir = project("closetouched");
    write(&dir, "src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    write(
        &dir,
        "tests/t_test.rs",
        "/// proves: nobody-declares-me@abc123\n#[test]\nfn holds() {}\n",
    );
    // The wave names the test file, so this is no drift -- the only
    // thing wrong is the tag, and it is wrong in a file the branch
    // touched.
    write(
        &dir,
        "keel/waves/0001-a-chore.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n      - tests/t_test.rs\n{}---\n\n## transform: tidy\nтіло\n",
            all_decided()
        ),
    );
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "-m",
            "tidy: the work and a tag nobody declares",
        ],
    );
    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 1,
        "the fixture must really be red for check:\n{checked}"
    );
    assert!(
        checked.contains("tests/t_test.rs"),
        "and red on the FILE, not on the wave -- otherwise this probe \
         measures the same half as the one above:\n{checked}"
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_ne!(
        close_code, 0,
        "a finding on a file the branch touched is the branch's to \
         answer for:\n{closed}"
    );
    assert!(
        closed.contains("tests/t_test.rs"),
        "and the court names that file:\n{closed}"
    );

    // --- and the mirror: the same finding, in a file the branch did
    // NOT touch, stays outside the count -------------------------
    //
    // Without this side the fix for the one above could be "count
    // every row check produced", which is the reading that reddened
    // forty sandboxes at once.
    let dir = project("closenotmine");
    write(
        &dir,
        "tests/t_test.rs",
        "/// proves: nobody-declares-me@abc123\n#[test]\nfn holds() {}\n",
    );
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "a tag nobody declares, on MAIN"],
    );
    git(&dir, &["checkout", "-q", "main"]);
    git(&dir, &["merge", "-q", "--ff-only", "0001-a-chore"]);
    git(&dir, &["checkout", "-q", "-b", "0002-later"]);
    write(
        &dir,
        "keel/waves/0002-later.md",
        &format!(
            "---\ntransforms:\n  later:\n    chore: \"пізніша дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: later\nтіло\n",
            all_decided()
        ),
    );
    write(&dir, "keel/reviews/0002-later.md", "# Рецензія\n\nok\n");
    write(&dir, "src/lib.rs", "pub fn a() {}\npub fn c() {}\n");
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "later: only what was named"]);
    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 1,
        "check still carries the whole tree's untidiness:\n{checked}"
    );
    assert!(
        checked.contains("tests/t_test.rs"),
        "and names the old file:\n{checked}"
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(
        close_code, 0,
        "but a court admitting THIS branch to a merge does not carry \
         somebody else's old untidiness -- `keel check` carries it, \
         and says so in full:\n{closed}"
    );

    // --- the plan branch: what the exception covers, and what it
    // must not ------------------------------------------------------
    //
    // A wave approved and NOT started merges as a plan (§6.6), and
    // the branch court can only say there that the work has not
    // begun -- the state the footer announces. So those findings are
    // printed and not counted. But the first cut of this wave
    // exempted EVERYTHING check had found, and a plan answering one
    // cut with a bare formula (§10.3) went out with exit 0: the very
    // example the card used to argue this wave was needed (review
    // R-5).
    let dir = keel_sandbox("closeplan");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "pub fn a() {}\n");
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    write(&dir, "keel/reviews/0001-a-plan.md", "# Рецензія\n\nok\n");
    // A promise with no tag anywhere: State::Plan. One cut answered
    // with the bare formula, which is §10.3's finding and NOT the
    // branch court's.
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut == "functional.correctness" {
            continue;
        }
        if *cut == "safety.hazard-warning" {
            decided.push_str("  safety.hazard-warning: \"не застосовується\"\n");
        } else {
            decided.push_str(&format!("  {cut}: \"n/a, бо ця пісочниця грає інше\"\n"));
        }
    }
    let plan_wave = format!(
        "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
    );
    write(&dir, "keel/waves/0001-a-plan.md", &plan_wave);
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-plan"]);
    git(
        &dir,
        &["commit", "-q", "--allow-empty", "-m", "the plan stands"],
    );
    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 1,
        "the plan really is red for check:\n{checked}"
    );
    assert!(
        checked.contains("an answer with no reason"),
        "and red for a reason that is NOT the branch court's:\n{checked}"
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert!(
        closed.contains("the declared work has not begun"),
        "close says which findings the plan exception covers, and \
         why:\n{closed}"
    );
    assert!(
        closed.contains("an answer with no reason"),
        "it prints the §10.3 finding too -- printed is not the \
         question, counted is:\n{closed}"
    );
    assert_ne!(
        close_code, 0,
        "and §10.3 is no part of \"the work has not begun\", so it \
         counts on a plan branch as anywhere: the exception covers the \
         BRANCH court and nothing else:\n{closed}"
    );
    assert!(
        closed.contains("the declared file"),
        "while the branch court's own finding is printed beside it, \
         uncounted:\n{closed}"
    );
    // And the advice is the ORDINARY one here, because something
    // counts: §10.3 is a blocker two lines above it, and telling a
    // person there is nothing to do would be false (review R3-2).
    assert!(
        closed.contains("fix the files named above"),
        "where anything counts, the advice is the plain one:\n{closed}"
    );

    // --- and the same wave on its PLAN branch (§8.2) ---------------
    //
    // `plan/<wave>` is not named after a wave, so the branch's own
    // file never entered the set and the court went silent: check
    // exit 1, close "no blockers", exit 0 -- the silent green
    // `safety.fail-safe` forbids, standing exactly where §9.9's
    // barrier at the plan is supposed to be (review R-6).
    git(&dir, &["checkout", "-q", "main"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-plan"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            "the plan, on its own branch",
        ],
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert!(
        closed.contains("an answer with no reason"),
        "on plan/<wave> the court is not blind to the wave it \
         plans:\n{closed}"
    );
    assert_ne!(
        close_code, 0,
        "and not silent either -- a court that prints nothing and \
         leaves with 0 is the worst of the three answers:\n{closed}"
    );

    // --- the exception covers "not begun" and NOT §6.8 -----------
    //
    // The second reading gathered the exempt rows by WHERE they were
    // written -- everything the branch arm pushed -- and so swallowed
    // §6.8's two findings, the ones §9.9's second human look exists
    // for: a light wave growing a contract, and a full wave whose
    // file was born on the work branch. Both mean work was done, and
    // both went out with exit 0 (review R2-1).
    let dir = keel_sandbox("closelightcontract");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "pub fn a() {}\n");
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    write(&dir, "keel/reviews/0001-a-plan.md", "# Рецензія\n\nok\n");
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            decided.push_str(&format!("  {cut}: \"n/a, бо ця пісочниця грає інше\"\n"));
        }
    }
    write(
        &dir,
        "keel/waves/0001-a-plan.md",
        &format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-plan"]);
    // A contract grown by a wave whose files do not name one: §6.8
    // and §5.7, and no tag anywhere, so the wave is still a plan.
    write(
        &dir,
        "keel/contracts/toy.md",
        "---\nmodule: toy\nexports:\n  - \"pub fn a()\"\n---\n\nThe promise, and what holds it.\n",
    );
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "-m",
            "work: a contract this wave never named",
        ],
    );
    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(check_code, 1, "check sees the contract:\n{checked}");
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_ne!(
        close_code, 0,
        "a light wave growing a contract is the second human look \
         skipped (§6.8, §5.7) -- it is no part of \"the declared work \
         has not begun\", and the plan exception must not swallow \
         it:\n{closed}"
    );
    // And the exit is not enough: a REFUSAL also leaves with a
    // non-zero code, and the first cut of this side wrote a contract
    // header that did not parse, so the court refused before it ever
    // reached the documents. The counted line is what says the
    // exception let this one through to the blockers.
    assert!(
        closed.contains("findings of the documents court on this branch's own files: 2"),
        "and the count says which ones it let through: the contract \
         (§6.8) and the promise worked on with no tag (§7.5), while \
         the untouched file stays exempt:\n{closed}"
    );

    // --- and where NOTHING counts, the plan has its own words -----
    //
    // This is the only shape in which "there is nothing to do" is
    // true: every finding is the branch court saying the work has
    // not begun. The first cut printed "fix them and run keel close
    // again" here, under an exit of 0 -- sending a person to repair
    // what the court had just let through (review R-15).
    let dir = project("closeplanclean");
    git(&dir, &["checkout", "-q", "main"]);
    write(
        &dir,
        "keel/waves/0003-a-clean-plan.md",
        &format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            {
                let mut block = String::from("decisions:\n");
                for cut in keel::graph::cuts() {
                    if *cut != "functional.correctness" {
                        block.push_str(&format!("  {cut}: \"n/a, бо ця пісочниця грає інше\"\n"));
                    }
                }
                block
            }
        ),
    );
    write(
        &dir,
        "keel/reviews/0003-a-clean-plan.md",
        "# Рецензія\n\nok\n",
    );
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the plan"]);
    git(&dir, &["checkout", "-q", "-b", "0003-a-clean-plan"]);
    git(
        &dir,
        &["commit", "-q", "--allow-empty", "-m", "the branch stands"],
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(
        close_code, 0,
        "a plan whose every finding says the work has not begun \
         merges as a plan (§6.6):\n{closed}"
    );
    assert!(
        closed.contains("nothing to do about THESE"),
        "and the advice says so, instead of sending a person to fix \
         what the court just let through:\n{closed}"
    );

    // --- the OTHER side of NotBegun, on the same branch -----------
    //
    // The `No` side held nothing: a one-line mutant making drift
    // `NotBegun::Yes` put the exemption back over a finding that
    // means work was DONE, and the whole battery stayed green
    // (review R3-1). §6.8's findings do not come through
    // `scope::findings` at all, so the probe above measures a
    // different door. This one stands on the same plan branch and
    // carries both kinds at once: the untouched file stays exempt,
    // the drifted file counts.
    let dir = keel_sandbox("closeplandrift");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "pub fn a() {}\n");
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    write(&dir, "keel/reviews/0001-a-plan.md", "# Рецензія\n\nok\n");
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            decided.push_str(&format!("  {cut}: \"n/a, бо ця пісочниця грає інше\"\n"));
        }
    }
    write(
        &dir,
        "keel/waves/0001-a-plan.md",
        &format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-plan"]);
    // The declared file is still untouched -- "not begun" -- and a
    // file no transform names is touched: work, and wrong.
    write(&dir, "build.rs", "fn main() {}\n");
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "a stranger, on a plan"]);
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert!(
        closed.contains("build.rs") && closed.contains("src/lib.rs"),
        "both findings are printed, whatever is counted:\n{closed}"
    );
    assert!(
        closed.contains("findings of the documents court on this branch's own files: 1"),
        "exactly one of the two counts: drift means the work HAS \
         begun and is wrong, the untouched file means it has not -- \
         and only the second is what the plan footer announces. A \
         `NotBegun` that answered Yes to both would leave this at \
         zero and the battery green (review R3-1):\n{closed}"
    );
    assert_ne!(
        close_code, 0,
        "and the one that counts carries the exit:\n{closed}"
    );

    // --- and a plan branch naming no wave at all -------------------
    //
    // §4.9 judges code on ANY `plan/*`, and `keel check` says so in
    // as many words. The first fix for R-6 filtered plan branches by
    // a known wave, which put the silence straight back (review
    // R2-2).
    let dir = project("closeplannowave");
    git(&dir, &["checkout", "-q", "main"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0002-nothing"]);
    write(
        &dir,
        "src/lib.rs",
        "pub fn a() {}\npub fn code_on_a_plan() {}\n",
    );
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "code, on a plan branch"]);
    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 1,
        "check judges code on a plan branch whatever it is named \
         (§4.9):\n{checked}"
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_ne!(
        close_code, 0,
        "and so does the court that admits it to a merge -- a plan \
         branch with no wave of its name is still a plan branch, and \
         a court that prints nothing and leaves with 0 is the silent \
         green §4.10 calls worse than red:\n{closed}"
    );
    assert!(
        closed.contains("src/lib.rs"),
        "and it names the file:\n{closed}"
    );

    // --- where check is silent, close reddens nothing extra ---
    let dir = project("closequiet");
    write(&dir, "src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "tidy: only what was named"]);
    let (checked, check_code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        check_code, 0,
        "the quiet side is quiet for check:\n{checked}"
    );
    let (closed, close_code) = keel(&["close", dir.to_str().unwrap()]);
    assert_eq!(
        close_code, 0,
        "and close adds no redness of its own where check is \
         silent:\n{closed}"
    );
    assert!(
        closed.contains("battery:"),
        "and the battery runs in both cases -- this wave widens the \
         court, it does not make it cheaper:\n{closed}"
    );
}
