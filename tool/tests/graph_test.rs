//! Unit-level tests of the graph module (wave 0004-scope-and-links,
//! transform graph-checks). The scenario promises live in
//! check_test.rs -- these pin the vocabulary and the cycle mechanics.

use keel::graph;

/// The embedded vocabulary is exactly the forty cuts, unique.
#[test]
fn vocabulary_is_forty_unique() {
    let cuts = graph::cuts();
    assert_eq!(cuts.len(), 40);
    let unique: std::collections::BTreeSet<_> = cuts.iter().collect();
    assert_eq!(unique.len(), 40);
    assert!(cuts.contains(&"functional.correctness"));
    assert!(cuts.contains(&"safety.safe-integration"));
}

/// A three-wave cycle is reported once, with the chain named.
#[test]
fn cycle_reported_once() {
    use keel::docs::{ScopeLine, Transform, TransformKind, Wave};
    let wave = |slug: &str, dep: &str| Wave {
        slug: slug.to_string(),
        depends_on: vec![dep.to_string()],
        transforms: vec![(
            "t".to_string(),
            Transform {
                kind: TransformKind::Chore("tidy".to_string()),
                contracts: vec![],
                files: vec![ScopeLine::Path("a".to_string())],
            },
        )],
        ..Wave::default()
    };
    let waves = [
        wave("0001-a", "0002-b"),
        wave("0002-b", "0003-c"),
        wave("0003-c", "0001-a"),
    ];
    let findings = graph::cross_findings(&waves, &[]);
    let cycles: Vec<_> = findings
        .iter()
        .filter(|(_, r, _)| r.contains("->"))
        .collect();
    assert_eq!(cycles.len(), 1, "one cycle, reported once: {findings:?}");
    let (_, reason, _) = cycles[0];
    for name in ["0001-a", "0002-b", "0003-c"] {
        assert!(reason.contains(name), "chain names {name}: {reason}");
    }
}
