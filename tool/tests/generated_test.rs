//! Scenario tests of wave 0022-generated-block, transform
//! block-and-digest: the generated block lives between its markers,
//! and a block a person edited is never trampled.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0022b-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
    let out = command.arg("-C").arg(dir).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

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

fn project(name: &str) -> PathBuf {
    let dir = sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// The digest recorded for the block, if any.
fn recorded(dir: &Path) -> Option<String> {
    let text = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == "[generated]";
            continue;
        }
        if in_section && trimmed.contains("AGENTS.md") {
            return trimmed
                .split('=')
                .nth(1)
                .map(|v| v.trim().trim_matches('"').to_string());
        }
    }
    None
}

/// proves: generated-block-never-trampled@0751ba -- holds the
/// concept's letter (Distribution: the generated integrations lie
/// in the repository because agents and CI read them) and the
/// boundary that makes it safe: the block lives between its
/// markers, nothing outside them ever changes, and a block a person
/// edited by hand is refused aloud instead of overwritten.
#[test]
fn generated_block_never_trampled() {
    // No AGENTS.md at all: born with the block, and the digest
    // lands in [generated].
    let dir = project("born");
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        text.contains("<!-- keel:begin -->") && text.contains("<!-- keel:end -->"),
        "the block is born between its markers:\n{text}"
    );
    assert!(
        recorded(&dir).is_some_and(|d| d.len() == 12),
        "the digest of the block stands in [generated]"
    );

    // A file of the person's own, with no block: the block is
    // appended and their text stays byte for byte.
    let dir = project("appended");
    let mine = "# My project\n\nRules I wrote myself.\n";
    write(&dir, "AGENTS.md", mine);
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        text.starts_with(mine),
        "the person's own text is untouched, to the byte:\n{text}"
    );
    assert!(
        text.contains("<!-- keel:begin -->"),
        "the block is appended after it:\n{text}"
    );

    // Run again with the block standing and its digest true: the
    // block is refreshed, nothing else moves.
    let before = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    let config_before = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "update is green over its own block:\n{out}");
    assert_eq!(
        fs::read_to_string(dir.join("AGENTS.md")).unwrap(),
        before,
        "the same release writes the same block, byte for byte"
    );
    assert_eq!(
        fs::read_to_string(dir.join("keel.toml")).unwrap(),
        config_before,
        "the config keeps its comments, order and other sections"
    );

    // A block the person edited: refused aloud, and NOTHING is
    // written -- neither the document nor the config.
    let edited = before.replace("keel:end", "keel:end").replace(
        "<!-- keel:end -->",
        "A line a person added inside the block.\n<!-- keel:end -->",
    );
    fs::write(dir.join("AGENTS.md"), &edited).unwrap();
    let config_before = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a hand-edited block makes update red:\n{out}");
    assert!(
        out.contains("AGENTS.md"),
        "the refusal names the file:\n{out}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("AGENTS.md")).unwrap(),
        edited,
        "the person's edit is not trampled by a single byte"
    );
    assert_eq!(
        fs::read_to_string(dir.join("keel.toml")).unwrap(),
        config_before,
        "and nothing is recorded for a block that was not written"
    );

    // Markers removed altogether -- a person saying "not this":
    // update writes nothing back.
    let dir = project("removed");
    keel(&["init", dir.to_str().unwrap()]);
    write(&dir, "AGENTS.md", "# Mine alone\n");
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "no block is not an error:\n{out}");
    assert_eq!(
        fs::read_to_string(dir.join("AGENTS.md")).unwrap(),
        "# Mine alone\n",
        "a removed block is a decision, not a gap to fill"
    );
}
