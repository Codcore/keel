//! Tags of tests (§5.5, §7.5; contract tool-tags): the line
//! `proves: <scenario>@<revision>` in a comment before a test is the
//! record whose revision a wave holds. This module only parses --
//! which files to read is the adapter's knowledge, and nothing here
//! runs or writes.

use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::{Path, PathBuf};

pub struct TestTag {
    pub file: PathBuf,
    pub test: String,
    pub scenario: String,
    pub rev: String,
}

/// Reads the named test files and collects the tags. A tag with no
/// test function following it is a refusal by name: a record that
/// holds nothing is worse than none.
pub fn scan(files: &[PathBuf]) -> Result<Vec<TestTag>, Refusal> {
    let mut out = Vec::new();
    for file in files {
        let text = read(file)?;
        out.extend(scan_text(file, &text)?);
    }
    Ok(out)
}

/// The same parse from a ready string -- for texts that do not lie
/// on disk, such as a test file at the fork point read out of git
/// (§7.15).
pub fn scan_text(file: &Path, text: &str) -> Result<Vec<TestTag>, Refusal> {
    let mut out = Vec::new();
    {
        let marks = marks(file);
        let declares = declares(file);
        let mut pending: Option<(String, String)> = None;
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some((scenario, rev)) = tag_in(trimmed, marks) {
                if let Some((held, held_rev)) = pending.take() {
                    return Err(dangling(file, &held, &held_rev));
                }
                // The record's shape is §5.2's: 4-6 hex characters --
                // a crooked record refuses as itself, not as a
                // dressed-up staleness (review R-8).
                if !(4..=6).contains(&rev.len()) || !rev.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(Refusal {
                        file: file.to_path_buf(),
                        reason: ta("tags-bad-rev", targs!("scenario" => scenario, "rev" => rev)),
                        instead: t("tags-bad-rev-instead"),
                    });
                }
                pending = Some((scenario, rev));
                continue;
            }
            if let Some(name) = fn_name(trimmed, declares) {
                if let Some((scenario, rev)) = pending.take() {
                    out.push(TestTag {
                        file: file.to_path_buf(),
                        test: name,
                        scenario,
                        rev,
                    });
                }
                continue;
            }
            // Doc lines and attributes may stand between the tag and
            // its fn; any other code line orphans the tag.
            if pending.is_some()
                && !trimmed.is_empty()
                && !trimmed.starts_with("///")
                && !trimmed.starts_with("//")
                && !trimmed.starts_with("#")
            {
                let (scenario, rev) = pending.take().unwrap();
                return Err(dangling(file, &scenario, &rev));
            }
        }
        if let Some((scenario, rev)) = pending {
            return Err(dangling(file, &scenario, &rev));
        }
    }
    Ok(out)
}

/// Which marks open a comment in this file -- the file's own name
/// answers, not the project's config (review 0038 R-3). `#` is a
/// comment in ruby and the fence of a raw string in Rust, so a Rust
/// probe that builds a ruby fixture is still a Rust file: reading
/// `#` there turned a line inside `r#"..."#` into a tag and reddened
/// projects that had done nothing.
fn marks(file: &Path) -> &'static [&'static str] {
    match file.extension().and_then(|e| e.to_str()) {
        Some("rb") => &["#"],
        _ => &["///", "//!", "//"],
    }
}

/// The keyword a test declaration opens with in this file, for the
/// same reason and by the same answer.
fn declares(file: &Path) -> &'static [&'static str] {
    match file.extension().and_then(|e| e.to_str()) {
        Some("rb") => &["def "],
        _ => &["fn "],
    }
}

/// `proves: <scenario>@<rev>` inside a comment line; words after the
/// record are the author's -- only the record is read.
fn tag_in(trimmed: &str, marks: &[&str]) -> Option<(String, String)> {
    // `#[test]` is not a comment in Rust, and it never becomes one
    // here: what follows must read `proves: `, and an attribute does
    // not.
    let comment = marks.iter().find_map(|mark| trimmed.strip_prefix(mark))?;
    let rest = comment.trim_start().strip_prefix("proves: ")?;
    let token = rest.split_whitespace().next()?;
    let (scenario, rev) = token.split_once('@')?;
    if scenario.is_empty() || rev.is_empty() {
        return None;
    }
    Some((scenario.to_string(), rev.to_string()))
}

fn fn_name(trimmed: &str, declares: &[&str]) -> Option<String> {
    // `fn` in Rust, `def` in Ruby -- whichever this file writes.
    let after = declares
        .iter()
        .find_map(|word| trimmed.strip_prefix(word))
        .or_else(|| {
            declares
                .contains(&"fn ")
                .then(|| trimmed.find(" fn ").map(|i| &trimmed[i + " fn ".len()..]))?
        })?;
    let name: String = after
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() { None } else { Some(name) }
}

fn dangling(file: &Path, scenario: &str, rev: &str) -> Refusal {
    Refusal {
        file: file.to_path_buf(),
        reason: ta(
            "tags-dangling",
            targs!("scenario" => scenario.to_string(), "rev" => rev.to_string()),
        ),
        instead: t("tags-dangling-instead"),
    }
}

fn read(path: &Path) -> Result<String, Refusal> {
    std::fs::read_to_string(path).map_err(|e| Refusal {
        file: path.to_path_buf(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })
}
