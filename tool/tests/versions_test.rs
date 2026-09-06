//! Scenario test of wave 0041: versions live side by side.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;
use common::versions::{install, world};

use std::fs;

/// proves: versions-live-side-by-side@6b234f -- the concept asks for
/// `~/.keel/versions/<version>/`, one binary per version, so two
/// projects on two pins work at once. Measured before the wave: the
/// installer put ONE tree in ~/.keel and ONE binary on PATH, and a
/// second version overwrote the first.
#[test]
fn versions_live_side_by_side() {
    let dir = sandbox("sidebyside");
    let w = world(&dir);

    let (said, code) = install(&w, Some(&w.old_ref));
    assert_eq!(code, 0, "the older version installs:\n{said}");
    let old_home = w.home.join("versions").join(&w.old_ref);
    assert!(
        old_home.join("keel").is_file(),
        "into a home of its own:\n{said}"
    );
    assert_eq!(
        fs::read_to_string(old_home.join(".keel-version"))
            .unwrap()
            .trim(),
        w.old_version,
        "which records the version it builds"
    );
    let sha = fs::read_to_string(old_home.join(".keel-sha")).unwrap();
    assert_eq!(sha.trim().len(), 40, "and the commit it was built from");

    // A second version, and the first is not touched by a byte.
    let (said, code) = install(&w, Some("v2.0.0"));
    assert_eq!(code, 0, "the newer version installs beside it:\n{said}");
    let new_home = w.home.join("versions").join("v2.0.0");
    assert!(new_home.join("keel").is_file(), "in its own home:\n{said}");
    assert_eq!(
        fs::read_to_string(new_home.join(".keel-version"))
            .unwrap()
            .trim(),
        w.new_version,
        "recording its own version"
    );
    assert_eq!(
        fs::read_to_string(old_home.join(".keel-version"))
            .unwrap()
            .trim(),
        w.old_version,
        "and the older one still answers for itself -- a second \
         version used to overwrite the first, so two projects on two \
         pins could not work at all"
    );
    assert_eq!(
        fs::read_to_string(old_home.join(".keel-sha")).unwrap(),
        sha,
        "down to the commit it was built from"
    );

    // Both stand, and nothing else does.
    let mut standing: Vec<String> = fs::read_dir(w.home.join("versions"))
        .unwrap()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    standing.sort();
    assert_eq!(
        standing,
        vec!["v1.0.0".to_string(), "v2.0.0".to_string()],
        "the versions on this machine are the two that were asked for"
    );
}
