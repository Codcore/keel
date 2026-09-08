//! Scenario test of wave 0064: the plan meets a reader before the
//! merge.
//!
//! Measured before the plan: `keel review` on a plan branch refuses --
//! "the branch is not named as a wave (§8.2)" -- so the package a
//! fresh reader gets is assembled for the WORK branch alone. The forty
//! answers to the cuts are written at planning; the reader of §9.9
//! arrives at closing, when all the work is already done under that
//! plan. Between the two stands the approval of §6.6, held by nothing
//! but a person's reading.
//!
//! The tool catches ABSENCE well -- `graph-silence` shouts over a cut
//! with no answer. It does not catch UNTRUTH: the answer is there, it
//! is wrong, and the machine says nothing. This probe holds the
//! narrowed package that shows a person WHERE to look.

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

/// A project whose plan is FULL by every mechanical measure -- every
/// cut answered, `keel check` green -- and whose answers a person
/// would call wrong on sight: one cut closed by a promise that does
/// not prove it, one decided with a shrug.
fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut == "functional.correctness" || *cut == "compatibility.interoperability" {
            continue;
        }
        if *cut == "performance.time-behaviour" {
            // A shrug where a reason belongs: mechanically an answer,
            // and empty of one.
            decided.push_str(&format!("  {cut}: \"не застосовується\"\n"));
        } else {
            // A REASON, as §10.3 asks -- so the one shrug below is the
            // only line the package should name, and the probe plays
            // exactly what it promises.
            decided.push_str(&format!(
                "  {cut}: \"не застосовується, бо ця пісочниця грає лише один розріз\"\n"
            ));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  sqlite-is-the-database:\n    covers: [functional.correctness, compatibility.interoperability]\ntransforms:\n  work:\n    implements:\n      - sqlite-is-the-database\n    files:\n      - src/lib.rs\n{decided}---\n\n## scenario: sqlite-is-the-database\nтіло обіцянки: база даних — sqlite\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: the-plan-meets-a-reader-before-the-merge@6db853
#[test]
fn the_plan_meets_a_reader_before_the_merge() {
    let dir = project("planreview");
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);

    // The plan is full by every mechanical measure.
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "the plan is green by the machine:\n{said}");

    // And the package for a reader assembles HERE, on the plan
    // branch, where it was a refusal before this wave.
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(
        code, 0,
        "`keel review` on a plan branch gives a package, not a refusal \
         -- the cheapest review is the one nobody can get:\n{said}"
    );
    assert!(
        !said.contains("не зветься як хвиля"),
        "and does not send the reader to a branch that does not exist \
         yet:\n{said}"
    );

    // What the package narrows to: the cut closed by a promise, with
    // the question a person answers yes or no.
    assert!(
        said.contains("compatibility.interoperability") && said.contains("sqlite-is-the-database"),
        "a cut closed by a promise is named beside the promise that \
         closes it -- the reader's one question is whether that promise \
         proves THIS cut:\n{said}"
    );
    // And the decided one whose reason is a shrug.
    assert!(
        said.contains("performance.time-behaviour"),
        "a cut decided with a bare «не застосовується» is named too: \
         mechanically an answer, and empty of a reason:\n{said}"
    );
    // The line it points at, verbatim -- not a retelling.
    assert!(
        said.contains("не застосовується"),
        "each question carries the line it points at, word for word, so \
         the reader judges the text and not a summary:\n{said}"
    );

    // On the WORK branch the package is what it always was.
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(code, 0, "the work branch still gets its package:\n{said}");
    assert!(
        said.contains("дрейф") || said.contains("мапа"),
        "with the three lists §9.9 asks for:\n{said}"
    );
}

/// proves: the-plan-meets-a-reader-before-the-merge@6db853 -- every
/// promise this package makes, held by a measurement.
///
/// Review 0064 measured four of its five moving parts held by nothing
/// at all: the covered list had no limit (75 lines on a plan of six
/// promises), the promise body could come out EMPTY and the battery
/// stayed green, the crowding line fired upside down, and "…and N
/// more" was never asked for.
#[test]
fn the_plan_package_keeps_every_promise_it_makes() {
    // --- SPEED: the package stays short whatever the plan's size ---
    let dir = project("planbig");
    let mut wave = String::from("---\nscenarios:\n");
    for n in 1..=6 {
        wave.push_str(&format!(
            "  promise-{n}:\n    covers: [{}]\n",
            [
                "functional.correctness",
                "functional.completeness",
                "performance.capacity",
                "security.integrity",
                "reliability.availability"
            ]
            .iter()
            .enumerate()
            .filter(|(i, _)| *i < 5)
            .map(|(_, c)| format!("{c}-{n}"))
            .collect::<Vec<_>>()
            .join(", ")
            .replace("-1", "")
            .replace("-2", "")
            .replace("-3", "")
            .replace("-4", "")
            .replace("-5", "")
            .replace("-6", "")
        ));
        break;
    }
    // A plan with SIX promises over the real cut names: the shape
    // review 0064 measured at 75 lines.
    let mut scenarios = String::from("scenarios:\n");
    let cuts = [
        "functional.correctness",
        "functional.completeness",
        "performance.capacity",
        "security.integrity",
        "reliability.availability",
        "maintainability.modularity",
    ];
    for (n, cut) in cuts.iter().enumerate() {
        scenarios.push_str(&format!("  promise-{n}:\n    covers: [{cut}]\n"));
    }
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if cuts.contains(&cut.as_ref()) {
            continue;
        }
        decided.push_str(&format!(
            "  {cut}: \"не застосовується, бо ця пісочниця про інше\"\n"
        ));
    }
    let mut body = String::new();
    for (n, _) in cuts.iter().enumerate() {
        body.push_str(&format!(
            "## scenario: promise-{n}\nтіло обіцянки номер {n}: перше речення. Друге речення, якого читач не потребує.\n\n"
        ));
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!("---\n{scenarios}transforms:\n  work:\n    implements:\n      - promise-0\n    files:\n      - src/lib.rs\n{decided}---\n\n{body}## transform: work\nтіло роботи\n"),
    )
    .unwrap();
    let _ = wave;
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "six promises"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    let (said, code) = keel(&dir, &["review"]);
    assert_eq!(code, 0, "the package assembles:\n{said}");
    let lines = said.lines().count();
    assert!(
        lines <= 30,
        "a package of {lines} lines is the forty-question reading it was \
         meant to replace -- review 0064 R-1 measured 75 on this very \
         shape:\n{said}"
    );
    assert!(
        said.contains("…і ще обіцянок"),
        "and it says how many promises it did not show:\n{said}"
    );
    // The body is printed ONCE per promise, not once per cut.
    assert_eq!(
        said.matches("тіло обіцянки номер 0").count(),
        1,
        "the promise body stands once, beside the cuts it claims:\n{said}"
    );
    // A whole first sentence, not a line cut where the file wrapped.
    assert!(
        said.contains("тіло обіцянки номер 0: перше речення."),
        "and it is a SENTENCE, not the first line chopped mid-clause \
         (review 0064 R-2):\n{said}"
    );

    // --- crowding: several promises speaking about ONE quality is
    // what the line exists for. Review 0064 R-8 measured the first cut
    // of it upside down -- silent on four promises over one cut, loud
    // on the lawful one-promise-two-cuts.
    let dir = project("plancrowded");
    let mut scenarios = String::from("scenarios:\n");
    for n in 0..4 {
        scenarios.push_str(&format!(
            "  voice-{n}:\n    covers: [functional.correctness]\n"
        ));
    }
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut == "functional.correctness" {
            continue;
        }
        decided.push_str(&format!(
            "  {cut}: \"не застосовується, бо ця пісочниця про інше\"\n"
        ));
    }
    let mut body = String::new();
    for n in 0..4 {
        body.push_str(&format!("## scenario: voice-{n}\nголос номер {n}.\n\n"));
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!("---\n{scenarios}transforms:\n  work:\n    implements:\n      - voice-0\n    files:\n      - src/lib.rs\n{decided}---\n\n{body}## transform: work\nтіло\n"),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "four voices, one decision",
        ],
    );
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    let (said, _) = keel(&dir, &["review"]);
    assert!(
        said.contains("обіцянок 4") && said.contains("закривають 1"),
        "four promises over one cut is one decision in four voices, and \
         the package says so -- this is the crowding the rule exists \
         for (review 0064 R-8):\n{said}"
    );

    // --- an EMPTY promise section says so, and does not print a bare
    // label (review 0064 R-2: a mutant emptying it walked the battery) ---
    let dir = project("planempty");
    let text = std::fs::read_to_string(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        text.replace("тіло обіцянки: база даних — sqlite", ""),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "an empty promise"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    let (said, _) = keel(&dir, &["review"]);
    assert!(
        said.contains("нема — читати нема чого"),
        "an empty promise section is named as the finding it is, not \
         printed as an empty label:\n{said}"
    );

    // --- CRLF: the body survives Windows line endings (review 0064
    // R-3: the plan road had its own read and lost them) ---
    let dir = project("plancrlf");
    let text = std::fs::read_to_string(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        text.replace('\n', "\r\n"),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "crlf"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    let (said, _) = keel(&dir, &["review"]);
    assert!(
        said.contains("база даних — sqlite"),
        "the promise reads the same with CRLF -- one road to the text, \
         and no second one to forget its lesson:\n{said}"
    );

    // --- a cancelled wave is outside judgement, and the package says
    // so aloud (§6.3-a; review 0064 R-4) ---
    let dir = project("plancancelled");
    let text = std::fs::read_to_string(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        text.replacen("---\n", "---\ncancelled: \"передумали\"\n", 1),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "called off"]);
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    let (said, _) = keel(&dir, &["review"]);
    assert!(
        said.contains("скасовано") && said.contains("передумали"),
        "a package over a cancelled wave says so, rather than judging \
         what nobody is going to build:\n{said}"
    );

    // --- a cut with NO answer is named: the package must not send a
    // reader to judge a plan the tool already refused (R-5) ---
    let dir = project("plansilent");
    let text = std::fs::read_to_string(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        text.replace("  safety.hazard-warning: \"не про цю пісочницю\"\n", "")
            .replace(
                "  safety.hazard-warning: \"не застосовується, бо ця пісочниця грає лише один розріз\"\n",
                "",
            ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "--no-verify", "-m", "one cut unanswered"],
    );
    git(&dir, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    let (said, _) = keel(&dir, &["review"]);
    assert!(
        said.contains("safety.hazard-warning"),
        "a cut with no answer is named here too -- keel check reddens \
         over it, and a package that stays quiet sends the reader to \
         judge a plan the tool has already refused:\n{said}"
    );

    // --- a plan branch whose slug names no wave is told THAT, not
    // sent to a branch it is already on (R-9) ---
    let dir = project("planunknown");
    git(&dir, &["checkout", "-q", "-b", "plan/0009-no-such-wave"]);
    let (said, code) = keel(&dir, &["review"]);
    assert_ne!(code, 0, "an unknown plan slug is a refusal:\n{said}");
    assert!(
        said.contains("не називає жодної прочитаної хвилі"),
        "and it says WHAT is wrong -- the slug names no wave -- instead \
         of advising a branch the person already stands on:\n{said}"
    );
}
