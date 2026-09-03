//! Scenario test of wave 0031: the closing court names its price.

mod common;

use common::{Sandbox, keel_sandbox};
use std::process::Command;

fn write(dir: &std::path::Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

/// A project the closing court will accept, in its own tongue rather
/// than another module's default (school of review 0029 R-12).
fn project(name: &str) -> Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"uk\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();
    dir
}

fn close_in(dir: &std::path::Path, path_prefix: Option<&std::path::Path>) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_keel"));
    command.args(["close", dir.to_str().unwrap()]);
    if let Some(prefix) = path_prefix {
        let path = std::env::var("PATH").unwrap_or_default();
        command.env("PATH", format!("{}:{path}", prefix.display()));
    }
    let out = command.output().unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// proves: the-closing-court-names-its-price@06b555 -- the reviewers
/// of waves 0028 and 0029 both read this court's own target
/// directory as a bug and both refused to run it, leaving the
/// closing court unverified twice in a row. It was never a bug: an
/// inherited cache shifts verdicts (§6.7). It was an unnamed price.
#[test]
fn the_closing_court_names_its_price() {
    let dir = project("price");

    // The price names the directory it ACTUALLY builds into. Review
    // 0031 R-4: both the price and the refusal said "tool/target" for
    // every project, while the adapter builds into the crate root's
    // own target -- here, the project root itself. The refusal named
    // a directory that never exists and told the reader to sweep it.
    let said = close_in(&dir, None);
    let target = dir.join("target");
    assert!(
        said.contains(&target.display().to_string()),
        "the court names the directory it builds into, not a guess:\n{said}"
    );
    assert!(
        !said.contains("tool/target"),
        "and never a path this project does not have:\n{said}"
    );
    assert!(
        said.contains("§6.7"),
        "and why it cannot reuse the caller's cache:\n{said}"
    );
    assert!(
        said.contains("ціна сплачена"),
        "and afterwards, what the price came to -- the sentence the \
         scenario promised and the first cut of this wave did not carry \
         (R-6):\n{said}"
    );

    // And when the disk cannot hold it, the court refuses AT THE
    // DOOR. Played with a df of our own on PATH, the way the reviewer
    // played it: removing the disk court used to leave the whole
    // battery green (R-3).
    let fake = keel_sandbox("df");
    write(
        &fake,
        "df",
        "#!/bin/sh\necho avail\necho 1048576\n", // one mebibyte free
    );
    let script = fake.join("df");
    let mut mode = std::fs::metadata(&script).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut mode, 0o755);
    std::fs::set_permissions(&script, mode).unwrap();

    let refused = close_in(&dir, Some(fake.path()));
    assert!(
        refused.contains("відмова"),
        "a disk that cannot hold the work stops the court at the door:\n{refused}"
    );
    assert!(
        refused.contains("no space left on device"),
        "and says why refusing now beats dying halfway through:\n{refused}"
    );
}
