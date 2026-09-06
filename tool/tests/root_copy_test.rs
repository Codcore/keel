//! Scenario test of wave 0054: the root copy does not drift.
//!
//! `METHODOLOGY.md` in the root is a copy of `docs/en/METHODOLOGY-V2.md`
//! with a preamble of its own; measured before the plan the bodies
//! agreed and nothing held them so (before 2026-09-04 the root carried
//! the v1 text, five months stale). Born green under the §6.3
//! exception: the mutant played is one line added to the root copy's
//! body, and the probe names the line.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::path::Path;

fn repo_file(rel: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// The body: from the first chapter heading to the end. The preamble
/// above it is each file's own.
fn body(text: &str) -> &str {
    let at = text.find("\n## ").expect("a first chapter heading");
    &text[at + 1..]
}

/// The first line where two bodies differ, counted from the body's
/// first line -- or None when they agree to the byte.
fn first_difference(a: &str, b: &str) -> Option<usize> {
    let mut left = a.lines();
    let mut right = b.lines();
    let mut n = 1;
    loop {
        match (left.next(), right.next()) {
            (None, None) => return None,
            (l, r) if l == r => n += 1,
            _ => return Some(n),
        }
    }
}

/// proves: the-root-copy-does-not-drift@78cca7
#[test]
fn the_root_copy_does_not_drift() {
    let root = repo_file("METHODOLOGY.md");
    let en = repo_file("docs/en/METHODOLOGY-V2.md");
    assert!(
        root.starts_with("# Keel: the methodology"),
        "the root copy keeps its own preamble"
    );
    if let Some(line) = first_difference(body(&root), body(&en)) {
        let shown = body(&root).lines().nth(line - 1).unwrap_or("<end>");
        let theirs = body(&en).lines().nth(line - 1).unwrap_or("<end>");
        panic!(
            "the root copy drifted from docs/en at body line {line}:\n  root: {shown}\n  en:   {theirs}"
        );
    }
    // The court can fall: one line more in a copy is named by line.
    let altered = format!("{}\nan extra line\n", body(&root).trim_end());
    assert_eq!(
        first_difference(&altered, body(&en)),
        Some(body(&en).lines().count() + 1),
        "a body that grew a line is told from the source at that line"
    );
}
