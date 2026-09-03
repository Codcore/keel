//! Scenario test of wave 0027-the-tool-speaks: the tool serves the
//! methodology and the forty cuts it judges by -- and the list it
//! serves is the list it judges by, or it refuses.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

// The sandboxes THIS test made, swept when it ends (school of review
// 0026 R-18): per THREAD, not per process, because tests of one
// binary run in parallel and share a pid.
thread_local! {
    static MADE: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn sweep() {
    MADE.with(|made| {
        for dir in made.borrow_mut().drain(..) {
            let _ = fs::remove_dir_all(dir);
        }
    });
}

/// A bare project directory: git, and nothing else.
fn bare(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0028-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    MADE.with(|made| made.borrow_mut().push(dir.clone()));
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();
    dir
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

/// The checklist as this repository keeps it -- the document a person
/// reads. The tool embeds the same file; reading it here from disk is
/// what makes the comparison a court and not a mirror.
fn checklist() -> String {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    std::fs::read_to_string(root.join("QUALITY.md")).expect("QUALITY.md stands in this repository")
}

/// proves: the-tool-says-what-it-judges-by@4af6c4 -- the
/// operator's §8.6 decision: everything the thin block does not carry
/// must live in the tool. A project where keel stands has neither the
/// methodology nor the checklist -- they live in the tool's own
/// repository -- so an agent filling a wave's decisions had nowhere
/// to read the forty questions it is judged by.
#[test]
fn the_tool_says_what_it_judges_by() {
    // The forty cuts, served -- and served as the judge holds them.
    let (said, code) = keel(&["cuts"]);
    assert_eq!(code, 0, "the tool says what it judges by:\n{said}");
    let judged = keel::graph::cuts();
    assert_eq!(judged.len(), 40, "forty, as the methodology says");
    let mut at = 0usize;
    for slug in judged {
        let found = said[at..]
            .find(slug)
            .unwrap_or_else(|| panic!("the cut {slug:?} is served:\n{said}"));
        at += found + slug.len();
    }

    // Nine families, each named.
    for family in [
        "functional",
        "performance",
        "compatibility",
        "interaction",
        "reliability",
        "security",
        "maintainability",
        "flexibility",
        "safety",
    ] {
        assert!(
            said.contains(family),
            "the family {family:?} is named:\n{said}"
        );
    }

    // And each cut carries ITS OWN question from the checklist -- not
    // a paraphrase, and not a second list that can drift.
    let document = checklist();
    let pairs = keel::speak::cuts();
    assert_eq!(pairs.len(), 40, "one question per cut");
    for (slug, _family, question) in &pairs {
        assert!(
            !question.is_empty(),
            "the cut {slug:?} carries a question of its own"
        );
        assert!(
            document.contains(question),
            "and that question stands, byte for byte, in the document a person reads: {question:?}"
        );
        assert!(
            said.contains(question),
            "and the tool serves it: {question:?}\n{said}"
        );
    }

    // The court itself: a checklist that drifted from the judge is a
    // REFUSAL, not a quiet difference between what is judged and what
    // is read. Fed a document whose question was renamed, the hand
    // must say which cut lost its question.
    let doctored = document.replace("- **correctness**", "- **rightness**");
    assert_ne!(doctored, document, "the doctored document really differs");
    let refusal = keel::speak::cuts_from(&doctored)
        .expect_err("a checklist that lost a cut is refused, not served");
    assert!(
        format!("{refusal}").contains("correctness"),
        "and the refusal names the cut that went missing:\n{refusal}"
    );
    // Drift is not only a question that went missing. A document
    // whose questions were REORDERED, or one carrying a question no
    // court judges, is the same defect -- measured by my own hand
    // before the review, when both passed.
    let swapped = document
        .replace(
            "- **completeness** — is everything that was asked for here\n- **correctness** — is the result right\n",
            "- **correctness** — is the result right\n- **completeness** — is everything that was asked for here\n",
        );
    assert_ne!(swapped, document, "the reordered document really differs");
    let refusal = keel::speak::cuts_from(&swapped)
        .expect_err("a checklist that reordered the cuts is refused");
    let said = format!("{refusal}");
    assert!(
        said.contains("functional.completeness") && said.contains("functional.correctness"),
        "and the refusal names the place and both names:\n{said}"
    );
    let padded = document.replace(
        "- **correctness** — is the result right\n",
        "- **correctness** — is the result right\n- **timeliness** — does it arrive when it is needed\n",
    );
    assert_ne!(padded, document, "the padded document really differs");
    let refusal = keel::speak::cuts_from(&padded)
        .expect_err("a question no court judges is refused, not served");
    assert!(
        format!("{refusal}").contains("functional.timeliness"),
        "and the refusal names it:\n{refusal}"
    );

    // The healthy document passes the same hand.
    assert!(
        keel::speak::cuts_from(&document).is_ok(),
        "while the document as it stands is served"
    );

    // A question emptied by an accidental edit is drift too (review
    // 0027 R-1: it used to be served as a slug, a dash and nothing).
    let hollowed = document.replace(
        "- **correctness** — is the result right",
        "- **correctness** — ",
    );
    assert_ne!(hollowed, document, "the hollowed document really differs");
    let refusal =
        keel::speak::cuts_from(&hollowed).expect_err("a cut with no question at all is refused");
    assert!(
        format!("{refusal}").contains("functional.correctness"),
        "and the refusal names it:\n{refusal}"
    );

    // The court stands at the gate every project already runs, not
    // only in a command nobody has to type (review 0027 R-2).
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    // (The verdict of the whole check is not this scenario's
    // business -- a working tree mid-wave is red by design. The row
    // is.)
    let (judged, _) = keel(&["check", root.to_str().unwrap()]);
    assert!(
        judged.contains("сорок розрізів") || judged.contains("forty cuts"),
        "keel check judges the sameness of the vocabulary at the gate:\n{judged}"
    );

    // Every family is NAMED, not merely implied by a slug prefix
    // (review 0027 R-14).
    let (report, _) = keel(&["cuts"]);
    for family in ["[functional]", "[safety]", "[security]"] {
        assert!(
            report.contains(family),
            "the family heading {family} stands in the report:\n{report}"
        );
    }

    // The methodology: without an argument, the table of contents.
    let (contents, code) = keel(&["method"]);
    assert_eq!(code, 0, "the tool says the methodology:\n{contents}");
    // The names are English here because the sandbox names no
    // language and English is the default: since wave 0029 the mouth
    // serves the methodology of the project's tongue, so a probe that
    // asserts chapter names must say which tongue it is standing in.
    for chapter in ["Constitution", "Chapter 7", "Chapter 10", "Appendix"] {
        assert!(
            contents.contains(chapter),
            "the contents name {chapter:?}:\n{contents}"
        );
    }

    // With a paragraph, that paragraph -- spelled with or without the
    // section sign, and carrying the chapter it lives in.
    let (paragraph, code) = keel(&["method", "§8.6"]);
    assert_eq!(code, 0, "a paragraph is served:\n{paragraph}");
    assert!(
        paragraph.contains("8.6"),
        "the paragraph asked for:\n{paragraph}"
    );
    assert!(
        paragraph.contains("Chapter 8"),
        "with the chapter it lives in:\n{paragraph}"
    );
    let (plain, code) = keel(&["method", "8.6"]);
    assert_eq!(code, 0, "the section sign is optional:\n{plain}");
    assert_eq!(plain, paragraph, "and changes nothing");

    // A number this methodology does not have is a refusal that helps.
    let (missing, code) = keel(&["method", "§8.99"]);
    assert_ne!(code, 0, "an unknown paragraph is refused:\n{missing}");
    assert!(
        missing.contains("8.1") || missing.contains("8."),
        "and the word names the bounds of that chapter:\n{missing}"
    );

    // A chapter is served whole when asked by name -- the only way
    // to reach the Constitution's eight rules and the three
    // appendices, a sixth of the methodology that no paragraph number
    // can reach (review 0027 R-6).
    let (constitution, code) = keel(&["method", "Constitution"]);
    assert_eq!(code, 0, "a chapter is served whole:\n{constitution}");
    assert!(
        constitution.contains("Eight rules") && constitution.contains("8."),
        "with its eight rules in it:\n{constitution}"
    );

    // A piece is the piece, not its neighbour's fence: the last
    // paragraph of a chapter used to carry the document's own rule
    // (review 0027 R-11).
    let (last, code) = keel(&["method", "§10.7"]);
    assert_eq!(
        code, 0,
        "the last paragraph of a chapter is served:\n{last}"
    );
    assert!(
        !last.trim_end().ends_with("---"),
        "and carries no separator of the document with it:\n{last}"
    );

    // A typo is refused, not swallowed into "here are the contents"
    // (review 0027 R-8).
    for typo in ["абракадабра", "1.2.3", "8.6."] {
        let (out, code) = keel(&["method", typo]);
        assert_ne!(
            code, 0,
            "{typo:?} is refused, not answered with contents:\n{out}"
        );
    }
    let (extra, code) = keel(&["cuts", ".", "and-another"]);
    assert_ne!(code, 0, "a second argument is refused:\n{extra}");

    // Neither mouth reads the disk: served from a directory that has
    // no documents at all, they still speak in full.
    let bare = std::env::temp_dir().join(format!("keel-0027-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&bare);
    std::fs::create_dir_all(&bare).unwrap();
    let (elsewhere, code) = keel(&["cuts", bare.to_str().unwrap()]);
    assert_eq!(code, 0, "the cuts are served anywhere:\n{elsewhere}");
    assert!(
        elsewhere.contains("functional.correctness"),
        "in full, from a project that has no checklist of its own:\n{elsewhere}"
    );
    let _ = std::fs::remove_dir_all(&bare);
}

/// proves: the-checklist-speaks-the-project-language@2d1d52 -- the
/// operator's decision of 2026-09-03: everything is translated, and
/// there is no Ukrainian where the settings say English or the other
/// way round. Before this wave a project with lang = "uk" got forty
/// English questions at exactly the place a person sits down to fill
/// in its decisions.
#[test]
fn the_checklist_speaks_the_project_language() {
    // Two projects, two tongues, neither holding a document of its
    // own.
    let uk = bare("speaks-uk");
    fs::write(uk.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let en = bare("speaks-en");
    fs::write(en.join("keel.toml"), "lang = \"en\"\n").unwrap();

    let (said_uk, code) = keel(&["cuts", uk.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "the cuts are served to a Ukrainian project:\n{said_uk}"
    );
    let (said_en, code) = keel(&["cuts", en.to_str().unwrap()]);
    assert_eq!(code, 0, "and to an English one:\n{said_en}");

    // The forty questions arrive in the language the project speaks.
    assert!(
        said_uk.contains("чи все, що просили, тут є"),
        "the Ukrainian project reads its questions in Ukrainian:\n{said_uk}"
    );
    assert!(
        !said_uk.contains("is everything that was asked for here"),
        "and not in English as well:\n{said_uk}"
    );
    assert!(
        said_en.contains("is everything that was asked for here"),
        "the English project reads them in English:\n{said_en}"
    );
    assert!(
        !said_en.contains("чи все, що просили, тут є"),
        "and the English road did not change:\n{said_en}"
    );

    // Every tongue this release carries is served and judged -- not
    // "at least one" (review 0028 R-2: the loop below was true for a
    // single language, so the Ukrainian document could have fallen
    // out from under the courts and nothing would have noticed).
    assert_eq!(
        keel::speak::checklists().len(),
        keel::config::LANGUAGES.len(),
        "one checklist per language of this release"
    );

    // Both lists are the same list: the same forty slugs, in the same
    // order the courts hold them, in every tongue this release
    // carries.
    let judged = keel::graph::cuts();
    for (lang, document) in keel::speak::checklists() {
        let pairs = keel::speak::cuts_from(document)
            .unwrap_or_else(|refusal| panic!("the {lang} checklist stands: {refusal}"));
        assert_eq!(pairs.len(), 40, "forty cuts in {lang}");
        for (at, (slug, _, question)) in pairs.iter().enumerate() {
            assert_eq!(
                *slug, judged[at],
                "cut {at} of {lang} is the cut the courts judge"
            );
            assert!(!question.is_empty(), "and it carries a question in {lang}");
        }
    }

    // A translation that lost a cut is refused by name, exactly as
    // the original document is -- the court does not know which
    // tongue it is judging.
    let doctored = keel::speak::checklist_for("uk").replace("completeness", "повнота");
    let refusal = keel::speak::cuts_from(&doctored)
        .expect_err("a translation that renamed a slug is refused");
    assert!(
        format!("{refusal}").contains("completeness"),
        "and the refusal names it:\n{refusal}"
    );

    // The forty ENGLISH questions do not move: a wave that adds a
    // language and quietly edits the existing text has broken more
    // than it built. Held as a snapshot, since review 0028 R-1 found
    // this clause asserted by nobody -- and found the wave's own
    // claim of "not a byte" false for the chrome around the
    // questions, which did change and had to.
    for question in [
        "is everything that was asked for here",
        "what does it consume while it works",
        "what does it do to somebody trying",
        "what does adding it to a running system risk",
    ] {
        assert!(
            said_en.contains(question),
            "the English question stands unchanged: {question:?}\n{said_en}"
        );
    }

    // The gate judges every tongue, not only the one it serves.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let (checked, _) = keel(&["check", root.to_str().unwrap()]);
    // One row per language, counted -- not a substring that a single
    // row would satisfy (review 0028 R-2).
    let rows = checked
        .lines()
        .filter(|line| line.contains("сорок розрізів") || line.contains("forty cuts"))
        .count();
    assert_eq!(
        rows,
        keel::config::LANGUAGES.len(),
        "the gate judges the vocabulary of every tongue:\n{checked}"
    );

    // A language this release does not carry is a refusal, as it is
    // everywhere else in the frame -- not a silent fall back to
    // English (review 0028 R-5).
    let strange = bare("strange-tongue");
    fs::write(strange.join("keel.toml"), "lang = \"ua\"\n").unwrap();
    let (refused, code) = keel(&["cuts", strange.to_str().unwrap()]);
    assert_ne!(code, 0, "an unknown language is refused:\n{refused}");
    assert!(
        refused.contains("ua"),
        "and the refusal names what was asked for:\n{refused}"
    );

    sweep();
}

/// proves: the-methodology-speaks-the-project-language@aa2a19 -- the
/// second and last half of the operator's decision of 2026-09-03.
/// The checklist learned both tongues in wave 0028; the methodology
/// itself -- 15 chapters, 92 paragraphs of normative prose -- learns
/// them here.
#[test]
fn the_methodology_speaks_the_project_language() {
    let uk = bare("method-uk");
    fs::write(uk.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let en = bare("method-en");
    fs::write(en.join("keel.toml"), "lang = \"en\"\n").unwrap();

    // The contents, each in its own tongue.
    let (said_uk, code) = keel(&["method", uk.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "the methodology is served in Ukrainian:\n{said_uk}"
    );
    let (said_en, code) = keel(&["method", en.to_str().unwrap()]);
    assert_eq!(code, 0, "and in English:\n{said_en}");
    assert!(
        said_uk.contains("Конституція") && said_uk.contains("Глава 7"),
        "the Ukrainian contents name Ukrainian chapters:\n{said_uk}"
    );
    assert!(
        said_en.contains("Constitution") && said_en.contains("Chapter 7"),
        "the English contents name English chapters:\n{said_en}"
    );
    assert!(
        !said_en.contains("Глава"),
        "and no Ukrainian chapter is left in the English contents:\n{said_en}"
    );

    // The same paragraph under the same number, in each tongue.
    let (para_uk, code) = keel(&["method", "§8.6", uk.to_str().unwrap()]);
    assert_eq!(code, 0, "a paragraph is served:\n{para_uk}");
    let (para_en, code) = keel(&["method", "§8.6", en.to_str().unwrap()]);
    assert_eq!(code, 0, "in both tongues:\n{para_en}");
    assert!(
        para_uk.contains("Глава 8") && para_uk.contains("§8.6"),
        "the Ukrainian one carries its chapter:\n{para_uk}"
    );
    assert!(
        para_en.contains("Chapter 8") && para_en.contains("§8.6"),
        "and so does the English one:\n{para_en}"
    );
    assert!(
        para_en.contains("questions") || para_en.contains("decided"),
        "and the English paragraph is English:\n{para_en}"
    );

    // A chapter asked for by name is served in the tongue that names
    // it that way.
    let (whole, code) = keel(&["method", "Constitution", en.to_str().unwrap()]);
    assert_eq!(code, 0, "an English chapter by name:\n{whole}");
    assert!(
        whole.contains("Eight rules"),
        "served whole and in English:\n{whole}"
    );

    // Both texts carry the SAME SKELETON: the same chapters in the
    // same order, the same paragraph numbers, none of them empty.
    // That is what a machine can hold of a translation -- and it is
    // held here rather than promised.
    keel::speak::methods_agree().expect("the two methodologies carry one skeleton");
    assert_eq!(
        keel::speak::methods().len(),
        keel::config::LANGUAGES.len(),
        "one methodology per language of this release"
    );

    // And the gate says so, per tongue, as it does for the checklist.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let (checked, _) = keel(&["check", root.to_str().unwrap()]);
    let rows = checked
        .lines()
        .filter(|line| line.contains("методик") || line.contains("methodolog"))
        .count();
    assert!(
        rows >= 1,
        "the gate judges the methodology's skeleton too:\n{checked}"
    );

    sweep();
}
