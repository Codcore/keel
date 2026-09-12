//! Scenario tests of wave 0050: every red reaches the court.
//!
//! The global review of 2026-09-06 (bugs cut R-2, methodology cut
//! R-8) measured the closing court claiming a red test by the bare
//! NAME of its scenario: a red test tagged with a stale or foreign
//! revision, or with the tag of a withdrawn scenario, was neither a
//! lack of the wave nor a red nobody claims -- the court printed "red
//! test" and "closed" in one breath and left with 0. And `keel check`
//! walked past a live tag over a withdrawn scenario in silence, where
//! §2.12 says the test goes with the promise, in the same PR.
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

/// A rust crate under the methodology: one library, the review of
/// the wave already beside it, git around it, the wave's branch out.
fn crate_under(dir: &Path, wave_text: &str) {
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
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    // The branch does its work: a wave that declares a file and
    // never touches it is unfinished, and since wave 0068 the
    // closing court says so before it spends a battery.
    let touched = dir.join("src/lib.rs");
    let mut body = std::fs::read_to_string(&touched).unwrap_or_default();
    body.push_str("\n// touched by the branch\n");
    std::fs::write(&touched, body).unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "work: the declared file"]);
}

/// Every cut decided but the ones named -- a wave file that passes
/// the §10.3 court on its own.
fn decisions_except(covered: &[&str]) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// proves: a-red-nobody-claims-holds-the-wave@1da940
#[test]
fn a_red_nobody_claims_holds_the_wave() {
    let dir = keel_sandbox("unclaimed");
    // Two scenarios: `s1` alive and proven by a green test with the
    // current revision; `s2` withdrawn. The cut of the dead cover is
    // decided too -- a dead cover does not count (§2.12).
    let wave = format!(
        "---\nscenarios:\n  s1:\n    covers: [functional.correctness]\n  s2:\n    covers: [performance.capacity]\n    withdrawn: \"folded into s1\"\ntransforms:\n  work:\n    implements:\n      - s1\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: s1\n{BODY}## scenario: s2\n\nстаре тіло\n\n## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    );
    crate_under(&dir, &wave);
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/toy_test.rs"),
        format!(
            "/// proves: s1@{rev}\n#[test]\nfn holds_s1() {{\n    assert!(toy::works());\n}}\n"
        ),
    )
    .unwrap();
    // A red test carrying s1's name under a revision no wave ever
    // gave it -- and a red test carrying the tag of the withdrawn s2.
    std::fs::write(
        dir.join("tests/stale_test.rs"),
        "/// proves: s1@dead00\n#[test]\nfn stale_and_red() {\n    panic!(\"red under a crooked revision\");\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/dead_test.rs"),
        "/// proves: s2@beef00\n#[test]\nfn withdrawn_and_red() {\n    panic!(\"red under a dead promise\");\n}\n",
    )
    .unwrap();
    settle(&dir);

    // The closing court: both reds are named, both are counted, and
    // the wave does not close over them.
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("червоний тест: stale_and_red")
            && said.contains("червоний тест: withdrawn_and_red"),
        "the court names what it watched fail:\n{said}"
    );
    assert!(
        said.contains("батарея бачила червоне: 2"),
        "and counts BOTH among the reds nobody claims -- a claim is a tag \
         with the current revision of a live scenario, never a bare \
         name:\n{said}"
    );
    assert_ne!(code, 0, "a court that saw red does not close:\n{said}");
    assert!(
        !said.contains("0001-a-wave: закрита"),
        "no verdict reads as closure:\n{said}"
    );

    // The documents court: a live tag over a withdrawn scenario is a
    // finding by name (§2.12) -- the test goes with the promise, in
    // the same PR -- beside the stale tag it already saw.
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "the check is red:\n{said}");
    assert!(
        said.contains("withdrawn_and_red") && said.contains("\"s2\"") && said.contains("§2.12"),
        "the live tag over the dead promise is named, with its test and \
         the paragraph:\n{said}"
    );
}

/// proves: close-asks-the-form-of-every-contract@6a887d
#[test]
fn close_asks_the_form_of_every_contract() {
    let dir = keel_sandbox("closeform");
    let wave = format!(
        "---\nscenarios:\n  s:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - s\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: s\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    );
    crate_under(&dir, &wave);
    let rev = keel::rev::text_rev(BODY);
    std::fs::write(
        dir.join("tests/toy_test.rs"),
        format!("/// proves: s@{rev}\n#[test]\nfn holds_s() {{\n    assert!(toy::works());\n}}\n"),
    )
    .unwrap();
    // A contract promising a unit the module does not hold: the form
    // court of `keel check` is red over it (§7.6).
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: toy\nexports:\n  - \"pub fn missing() -> bool\"\n---\n\nОбіцянка форми, якої код не тримає.\n",
    )
    .unwrap();
    settle(&dir);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "check is red over the form:\n{said}");
    assert!(
        said.contains("обіцяє \"missing\""),
        "and names the promise the code does not hold:\n{said}"
    );

    // The closing court asks the same question of the same contracts,
    // and a finding is a blocker by name -- never "closed", exit 0,
    // over a form check says is red.
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("обіцяє \"missing\"") && said.contains("§7.6"),
        "close names the form finding, with the paragraph:\n{said}"
    );
    assert!(
        said.contains("форм"),
        "and counts the form court aloud, as it counts verify:\n{said}"
    );
    assert_ne!(
        code, 0,
        "a form the code does not hold does not merge:\n{said}"
    );

    // The window of §6.5 (review 0050 R-3): a contract grown ahead of
    // the code by a plan not yet started is not judged for form in
    // close either -- said in check's own words -- and the first tag
    // of that wave brings the court back.
    let window = keel_sandbox("closewindow");
    let wave1 = format!(
        "---\nscenarios:\n  s:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - s\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: s\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    );
    crate_under(&window, &wave1);
    std::fs::write(
        window.join("tests/toy_test.rs"),
        format!("/// proves: s@{rev}\n#[test]\nfn holds_s() {{\n    assert!(toy::works());\n}}\n"),
    )
    .unwrap();
    std::fs::write(
        window.join("keel/contracts/grown.md"),
        "---\nmodule: toy\nexports:\n  - \"pub fn later() -> bool\"\n---\n\nОбіцянка, яку ростить план.\n",
    )
    .unwrap();
    let grown_rev = keel::rev::contract_rev(&window.join("keel/contracts/grown.md")).unwrap();
    std::fs::write(
        window.join("keel/waves/0002-plan.md"),
        format!(
            "---\nscenarios:\n  later:\n    proves: grown@{grown_rev}\n    covers: [functional.correctness]\ntransforms:\n  work2:\n    implements:\n      - later\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: later\n\nтіло пізнішої обіцянки\n\n## transform: work2\nтіло\n",
            decisions_except(&["functional.correctness"])
        ),
    )
    .unwrap();
    settle(&window);
    let (said, code) = keel(&window, &["close"]);
    assert!(
        said.contains("grown — форма не судиться") && said.contains("0002-plan"),
        "inside the window the form is not judged, and close says so in \
         check's words:\n{said}"
    );
    assert!(
        !said.contains("обіцяє \"later\""),
        "and raises no form finding there:\n{said}"
    );
    assert_eq!(
        code, 0,
        "a plan grows its contract ahead of the code lawfully:\n{said}"
    );
    let later_rev = keel::rev::text_rev("тіло пізнішої обіцянки\n");
    std::fs::write(
        window.join("tests/later_test.rs"),
        format!("/// proves: later@{later_rev}\n#[test]\nfn holds_later() {{}}\n"),
    )
    .unwrap();
    git(&window, &["add", "-A"]);
    git(&window, &["commit", "-q", "-m", "the plan starts"]);
    let (said, code) = keel(&window, &["close"]);
    assert!(
        said.contains("обіцяє \"later\"") && said.contains("§7.6"),
        "the first tag of the wave brings the form court back:\n{said}"
    );
    assert_ne!(code, 0, "and the unheld form blocks:\n{said}");

    // On a plan branch the form is not judged -- exports grow ahead of
    // the code by design (§4.9) -- and close says so in check's words.
    git(&dir, &["checkout", "-q", "-b", "plan/0002-next"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("суд форми не біжить"),
        "the plan branch is outside the form court, said aloud:\n{said}"
    );
    assert!(
        !said.contains("обіцяє \"missing\""),
        "and no form finding is raised there:\n{said}"
    );
}
