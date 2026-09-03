//! Scenario test of wave 0027-the-tool-speaks: the tool serves the
//! methodology and the forty cuts it judges by -- and the list it
//! serves is the list it judges by, or it refuses.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::process::Command;

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

/// proves: the-tool-says-what-it-judges-by@7a384b -- the
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
    // The healthy document passes the same hand.
    assert!(
        keel::speak::cuts_from(&document).is_ok(),
        "while the document as it stands is served"
    );

    // The methodology: without an argument, the table of contents.
    let (contents, code) = keel(&["method"]);
    assert_eq!(code, 0, "the tool says the methodology:\n{contents}");
    for chapter in ["Конституція", "Глава 7", "Глава 10", "Додаток"] {
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
        paragraph.contains("Глава 8"),
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
