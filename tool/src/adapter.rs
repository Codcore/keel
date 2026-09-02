//! The cargo adapter (contract tool-adapter-cargo): the one place
//! that knows how a Rust project names its test files and how to run
//! a single test. Other languages will get their own waves next to
//! this file.

use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::{Path, PathBuf};

/// Cargo.toml at the root -- the root itself; otherwise exactly one
/// first-level directory carrying one -- that; zero or several -- a
/// refusal aloud with what was found. No guessing.
pub fn crate_root(root: &Path) -> Result<PathBuf, Refusal> {
    if root.join("Cargo.toml").is_file() {
        return Ok(root.to_path_buf());
    }
    let mut found: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(root).map_err(|e| Refusal {
        file: root.to_path_buf(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() && path.join("Cargo.toml").is_file() {
            found.push(path);
        }
    }
    found.sort();
    match found.len() {
        1 => Ok(found.remove(0)),
        0 => Err(Refusal {
            file: root.to_path_buf(),
            reason: t("adapter-no-crate"),
            instead: t("adapter-no-crate-instead"),
        }),
        _ => Err(Refusal {
            file: root.to_path_buf(),
            reason: ta(
                "adapter-many-crates",
                targs!("found" => found
                    .iter()
                    .map(|p| p.file_name().unwrap_or_default().to_string_lossy().into_owned())
                    .collect::<Vec<_>>()
                    .join(", ")),
            ),
            instead: t("adapter-many-crates-instead"),
        }),
    }
}

/// The crate's `tests/*.rs` -- where the proves tags live. A crate
/// without a tests directory has none, and that is not a refusal.
pub fn test_files(root: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let dir = crate_root(root)?.join("tests");
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut out: Vec<PathBuf> = Vec::new();
    let entries = std::fs::read_dir(&dir).map_err(|e| Refusal {
        file: dir.clone(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") && path.is_file() {
            out.push(path);
        }
    }
    out.sort();
    Ok(out)
}
