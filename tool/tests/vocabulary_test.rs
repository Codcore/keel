//! Scenario test of wave 0034: the vocabulary cannot drift from the
//! norm.

mod common;

/// proves: the-vocabulary-cannot-drift-from-the-norm@d28d17 --
/// §3.4 calls QUALITY.md the vocabulary of cuts, so it is part of
/// the norm, not a leaflet. It said "one pass per wave" against
/// §10.2's two, and "one of three answers" -- silence among them --
/// against §10.3's two and "silence is forbidden at the field
/// level". An author who honestly read the vocabulary his own norm
/// pointed him at would do the very thing the gate calls red.
///
/// The machine could not see it: forty slugs are compared, the prose
/// never was.
#[test]
fn the_vocabulary_cannot_drift_from_the_norm() {
    let repo = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf();
    let method = std::fs::read_to_string(repo.join("docs/uk/METHODOLOGY-V2.md")).unwrap();

    // What the norm says, taken from the norm itself rather than
    // copied here: the two claims the vocabulary contradicted.
    assert!(
        method.contains("Розрізи проходяться двічі на хвилю"),
        "the norm says two passes, and this probe is reading the right paragraph"
    );
    assert!(
        method.contains("Третьої відповіді нема"),
        "and that there is no third answer"
    );

    for (file, claims) in [
        (
            "QUALITY.md",
            [
                "One pass per wave",
                "not walked a second time",
                "One of three answers",
            ],
        ),
        (
            "docs/uk/QUALITY.md",
            [
                "Один прохід на хвилю",
                "удруге не проходять",
                "з трьох відповідей",
            ],
        ),
    ] {
        let text = std::fs::read_to_string(repo.join(file)).unwrap();
        for claim in claims {
            assert!(
                !text.contains(claim),
                "{file} says \"{claim}\", and the methodology says otherwise -- \
                 §3.4 makes this file the vocabulary, so a reader who follows \
                 it does what §10.3 calls red"
            );
        }
    }

    // And it says what the norm says.
    for (file, wanted) in [
        ("QUALITY.md", ["Two passes", "two different heads"]),
        (
            "docs/uk/QUALITY.md",
            ["Два проходи", "двома різними головами"],
        ),
    ] {
        let text = std::fs::read_to_string(repo.join(file)).unwrap();
        for word in wanted {
            assert!(
                text.contains(word),
                "{file} carries the norm's own shape: \"{word}\" is missing"
            );
        }
    }
}
