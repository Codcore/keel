//! Scenario tests of wave 0003-revisions, transform compute-revisions.
//! proves tags -- revisions per §5.3-§5.4, computed by hand for the
//! last time: this rung takes the counting over.

use keel::rev;
use std::fs;
use std::path::{Path, PathBuf};

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0003-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    dir
}

/// proves: revision-recipe-reproduced@54c8f0 -- holds §5.2-§5.4 and
/// the wave 0001 caveat: the tool reproduces the hand recipe on the
/// very files whose revisions sixteen references already hold.
#[test]
fn revision_recipe_reproduced() {
    // Golden vectors: reformatting is not a change, rewording is.
    assert_eq!(rev::text_rev("a  b\n\nc"), rev::text_rev("a b c"));
    assert_ne!(rev::text_rev("a b c"), rev::text_rev("a b d"));
    assert_eq!(rev::text_rev("  hello   world \n"), rev::text_rev("hello world"));
    assert_eq!(rev::text_rev("hello world"), "b94d27");

    // The live contracts of this repository give exactly the
    // hand-computed revisions (the wave 0001 promise).
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    assert_eq!(
        rev::contract_rev(&root.join("keel/contracts/tool-docs.md")).unwrap(),
        "2ab9a9"
    );
    assert_eq!(
        rev::contract_rev(&root.join("keel/contracts/tool-config.md")).unwrap(),
        "63406a"
    );

    // Prefix comparison (§5.2): 4-6 characters, from the start.
    assert!(rev::matches("2ab9", "2ab9a9"));
    assert!(rev::matches("2ab9a9", "2ab9a9"));
    assert!(!rev::matches("2ab", "2ab9a9"));
    assert!(!rev::matches("9a9", "2ab9a9"));
    assert!(!rev::matches("2ab9a9f", "2ab9a9"));

    // Scenario sections: hashed as their bodies, in declared order.
    let dir = sandbox("sections");
    let wave = dir.join("keel/waves/0009-w.md");
    fs::write(
        &wave,
        "---\nscenarios:\n  alpha: {covers: [functional.correctness]}\n  beta: {covers: [performance.capacity]}\ntransforms:\n  t:\n    implements: [alpha, beta]\n    files: [lib/a.ex]\n---\n\n## Why\n\nwhy text\n\n## scenario: alpha\n\nbody of alpha\n\n## scenario: beta\n\nbody of beta\n",
    )
    .unwrap();
    let revs = rev::scenario_revs(&wave).unwrap();
    assert_eq!(
        revs,
        vec![
            ("alpha".to_string(), rev::text_rev("body of alpha")),
            ("beta".to_string(), rev::text_rev("body of beta")),
        ]
    );

    // A scenario declared in the header without a body section
    // refuses by name -- half of §7.7 arrives here naturally.
    let wave = dir.join("keel/waves/0010-w.md");
    fs::write(
        &wave,
        "---\nscenarios:\n  ghost: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [ghost]\n    files: [lib/a.ex]\n---\n\n## Why\n\ntext\n",
    )
    .unwrap();
    let r = rev::scenario_revs(&wave).unwrap_err();
    assert!(r.reason.contains("ghost"), "names the scenario: {}", r.reason);
    assert!(!r.instead.is_empty());

    // A duplicated section refuses too.
    let wave = dir.join("keel/waves/0011-w.md");
    fs::write(
        &wave,
        "---\nscenarios:\n  twin: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [twin]\n    files: [lib/a.ex]\n---\n\n## scenario: twin\n\none\n\n## scenario: twin\n\ntwo\n",
    )
    .unwrap();
    let r = rev::scenario_revs(&wave).unwrap_err();
    assert!(r.reason.contains("twin"), "names the duplicate: {}", r.reason);
}
