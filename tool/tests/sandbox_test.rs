//! Scenario test of wave 0030-probes-clean-up: a sandbox does not
//! outlive its test -- and a failing test keeps its own.

mod common;

use common::Sandbox;
use std::path::PathBuf;

/// proves: a-sandbox-does-not-outlive-its-test@1e8f3d -- by wave 0029
/// the probes had left 11,511 directories holding 20 GB in /tmp, and
/// the disk hit 100% mid-wave. Twenty-six copies of `sandbox()` each
/// cleaned up after the PREVIOUS run of the same name and never after
/// themselves.
#[test]
fn a_sandbox_does_not_outlive_its_test() {
    // A sandbox that ends well leaves nothing behind.
    let kept: PathBuf = {
        let sandbox = Sandbox::new("green-case");
        std::fs::write(sandbox.join("file.txt"), "something").unwrap();
        assert!(
            sandbox.path().is_dir(),
            "the sandbox exists while the test holds it"
        );
        sandbox.path().to_path_buf()
    };
    assert!(
        !kept.exists(),
        "and it is gone the moment the test lets go of it: {}",
        kept.display()
    );

    // A sandbox whose test FAILED stays, because that is the one a
    // person opens. Played on a thread that panics while holding it:
    // Drop runs during the unwind, and must decide not to remove.
    let evidence = std::thread::spawn(|| {
        let sandbox = Sandbox::new("red-case");
        let path = sandbox.path().to_path_buf();
        // Hand the path out before the panic, so the assertion below
        // can look for it afterwards.
        let _ = std::fs::write(
            std::env::temp_dir().join(format!("keel-{}-red-case.path", std::process::id())),
            path.display().to_string(),
        );
        panic!("this test failed on purpose");
    })
    .join();
    assert!(evidence.is_err(), "the thread really panicked");
    let marker = std::env::temp_dir().join(format!("keel-{}-red-case.path", std::process::id()));
    let path = PathBuf::from(std::fs::read_to_string(&marker).unwrap());
    assert!(
        path.is_dir(),
        "the sandbox of a failed test is kept for inspection: {}",
        path.display()
    );
    std::fs::remove_dir_all(&path).unwrap();
    std::fs::remove_file(&marker).unwrap();

    // And no probe carries a copy of this hand any more: one rule,
    // one place (§9.6 -- a rule is held by machinery, not by memory).
    let tests = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut own: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&tests).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        // The needle is built rather than written, so this file does
        // not match itself: a check that flags its own source is a
        // check nobody can keep green.
        if text.contains(&format!("fn {}(", "sandbox")) {
            own.push(path.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(
        own.is_empty(),
        "every probe takes the one hand; these still carry their own: {own:?}"
    );
}
