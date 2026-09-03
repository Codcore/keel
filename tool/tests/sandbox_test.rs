//! Scenario test of wave 0030-probes-clean-up: a sandbox does not
//! outlive its test -- and a failing test keeps its own.

mod common;

use common::Sandbox;
use std::path::PathBuf;

/// proves: a-sandbox-does-not-outlive-its-test@d845a8 -- by wave 0029
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
    // The path travels in memory, not through a file in the shared
    // temp: review 0030 R-8 caught this probe leaving its own marker
    // behind in exactly the place it exists to keep empty.
    let told: std::sync::Arc<std::sync::Mutex<Option<PathBuf>>> =
        std::sync::Arc::new(std::sync::Mutex::new(None));
    let telling = std::sync::Arc::clone(&told);
    let evidence = std::thread::spawn(move || {
        let sandbox = Sandbox::new("red-case");
        *telling.lock().unwrap() = Some(sandbox.path().to_path_buf());
        // Deliberate: this failure is the thing being measured, and
        // the backtrace it prints belongs to a GREEN test (R-11).
        panic!("this test failed on purpose -- the probe measures what a panic leaves behind");
    })
    .join();
    assert!(evidence.is_err(), "the thread really panicked");
    let path = told
        .lock()
        .unwrap()
        .clone()
        .expect("the thread named its sandbox before it fell");
    assert!(
        path.is_dir(),
        "the sandbox of a failed test is kept for inspection: {}",
        path.display()
    );
    std::fs::remove_dir_all(&path).unwrap();

    // The hand can only ever remove what it made: the path is built
    // inside it, from its own prefix and this process's id, and no
    // path from outside is accepted. Review 0030 R-7 found this
    // clause of the scenario judged by nothing -- teaching the hand
    // to take an absolute path from its caller left all 88 tests
    // green.
    let mine = Sandbox::new("own-path-only");
    let neighbour = Sandbox::new("own-path-too");
    assert_eq!(
        mine.path().parent(),
        neighbour.path().parent(),
        "every sandbox hangs under one root the hand chooses itself"
    );
    let name = mine
        .path()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert!(
        name.starts_with(&format!("keel-{}-", std::process::id())),
        "and its name carries this process, so no other process's directory can be meant: {name}"
    );

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
        // The needle is the ACT, not the name. Review 0030 R-2: the
        // check used to look for "fn sandbox(" and was blind to the
        // same leak under another name -- and one already stood in
        // the tree, `speak_test.rs::bare()`, which made its own
        // directories and swept them on the last line of a test
        // body, the very pattern this wave calls broken. A directory
        // in the shared temp is made by that one call of the
        // standard library, whatever the function around it is
        // called.
        // Both needles are built from parts, so this file does not
        // match itself: a check that flags its own source is a check
        // nobody can keep green.
        let long = format!("{}::temp_{}", "std::env", "dir");
        let short = format!("temp_{}()", "dir");
        if text.contains(&long) || text.contains(&short) {
            own.push(path.file_name().unwrap().to_string_lossy().to_string());
        }
    }
    assert!(
        own.is_empty(),
        "every probe takes the one hand; these still carry their own: {own:?}"
    );
}
