//! Scenario tests of wave 0044: the generated CI must be able to run,
//! and must say what it judged with.
//!
//! Measured before this was written, on the tool's own PR: the
//! battery step was `cargo test --no-fail-fast` at the repository
//! root, and keel's crate lives in `tool/` --
//! `could not find Cargo.toml`. Reproduced on a fresh fixture, so it
//! is the generator's defect and not this repository's layout.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

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

/// A project whose crate sits where `where_crate` says -- "" for the
/// repository root, "tool" for a subdirectory, which is the ordinary
/// shape and the one keel itself has.
fn project(name: &str, where_crate: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    let crate_dir = if where_crate.is_empty() {
        dir.join(".")
    } else {
        dir.join(where_crate)
    };
    std::fs::create_dir_all(crate_dir.join("src")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"rust\"\nci = \"github\"\n",
    )
    .unwrap();
    std::fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(crate_dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    dir
}

/// The step a person's runner will execute, taken out of the file the
/// tool wrote.
fn battery_step(dir: &Path) -> String {
    let flow = std::fs::read_to_string(dir.join(".github/workflows/keel.yml"))
        .expect("the workflow was written");
    let at = flow
        .find("the battery")
        .expect("the workflow carries a battery step");
    flow[at..].to_string()
}

/// proves: a-court-runs-where-the-crate-is@245ded
#[test]
fn a_court_runs_where_the_crate_is() {
    // The crate in a subdirectory -- the shape keel itself has.
    let dir = project("ciunder", "tool");
    let (said, code) = keel(&dir, &["update"]);
    assert_eq!(code, 0, "the workflow is written:\n{said}");
    let step = battery_step(&dir);
    assert!(
        step.contains("working-directory: tool"),
        "the battery step runs where the crate is:\n{step}"
    );

    // And the proof that it is not only a word in a file: the command
    // as written, run from the repository root, really runs.
    let out = Command::new("sh")
        .args(["-c", "cd tool && cargo metadata --no-deps --format-version 1 >/dev/null"])
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "and from the repository root that directory really holds a \
         crate: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // The other side: a crate at the root gets the step it always
    // got, with no directory bolted on.
    let dir = project("ciroot", "");
    let (said, code) = keel(&dir, &["update"]);
    assert_eq!(code, 0, "the workflow is written:\n{said}");
    let step = battery_step(&dir);
    assert!(
        !step.contains("working-directory"),
        "a crate at the root needs no directory said:\n{step}"
    );
}
