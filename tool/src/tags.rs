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
        // By the file's own name, not by its comment marks: `#` opens
        // a comment in ruby AND in elixir, so comparing the marks put
        // every ruby file down the elixir road (caught by ruby's own
        // courts the moment the third tongue landed).
        let elixir = matches!(
            file.extension().and_then(|e| e.to_str()),
            Some("ex") | Some("exs")
        );
        let mut pending: Option<(String, String)> = None;
        let mut describing: Option<(String, usize)> = None;
        let mut depth: usize = 0;
        let mut heredoc = false;
        for line in text.lines() {
            let trimmed = line.trim();
            // `@doc """ … """` with an example inside is the most
            // ordinary thing an elixir file contains, and an example
            // is written in the language it documents. A line reading
            // `test "an example" is the shape we use` inside one is
            // prose: it declares nothing, and it must not orphan the
            // tag standing above the heredoc (measured -- a legal
            // file `mix test` runs green was refused). Counted, not
            // flagged, because a heredoc may open and close on one
            // line.
            if elixir {
                let fences = trimmed.matches("\"\"\"").count();
                let was = heredoc;
                if fences % 2 == 1 {
                    heredoc = !heredoc;
                }
                if was || heredoc {
                    continue;
                }
            }
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
            let declared = if elixir {
                // A `describe` opens a group whose name ExUnit puts
                // in front of every test inside it; `end` closes the
                // innermost block, and only a describe's own end
                // clears the group -- so the depth is counted.
                if let Some(group) = describe_name(trimmed) {
                    describing = Some((group, depth));
                }
                if trimmed.starts_with("end") || trimmed == "end" {
                    // The depth recorded when the block opened is
                    // the depth OUTSIDE it; its own `do` raised the
                    // count by one, so the `end` that closes it sits
                    // one deeper. Off by that one, the group leaked to
                    // the end of the module and named every test after
                    // it wrongly -- and a scenario was called proven by
                    // a test that had just failed (review 0042 R-1).
                    if let Some((_, opened)) = &describing
                        && depth == *opened + 1
                    {
                        describing = None;
                    }
                    depth = depth.saturating_sub(1);
                } else if trimmed.ends_with(" do") || trimmed.ends_with(" do:") || trimmed == "do" {
                    depth += 1;
                }
                test_name(trimmed).map(|name| match &describing {
                    Some((group, _)) => format!("{group} {name}"),
                    None => name,
                })
            } else {
                fn_name(trimmed, declares)
            };
            if let Some(name) = declared {
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
            if pending.is_some() && !stands_between(trimmed, elixir) {
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

/// What may stand between a tag and the declaration it holds. Blank
/// lines and doc lines always may. Rust's `#[test]` rides in on the
/// `#`; elixir writes its attributes with `@`, and `@tag :slow` --
/// an everyday ExUnit line -- orphaned the tag on a file mix runs
/// green (review 0042 R-7). Anything else is code, and code between
/// the two means the tag holds nothing.
fn stands_between(trimmed: &str, elixir: bool) -> bool {
    trimmed.is_empty()
        || trimmed.starts_with("///")
        || trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || (elixir && trimmed.starts_with('@'))
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
        Some("exs") | Some("ex") => ELIXIR_MARKS,
        _ => &["///", "//!", "//"],
    }
}

/// Elixir writes `#` as ruby does. Which is why the two are told
/// apart by the file's own extension and never by these marks --
/// their DECLARATIONS differ (`def name` there, `test "name" do`
/// here) and the marks do not.
const ELIXIR_MARKS: &[&str] = &["#"];

/// The keyword a test declaration opens with in this file, for the
/// same reason and by the same answer. Elixir is not in this list:
/// its tests are not named by an identifier at all (see `test_name`).
fn declares(file: &Path) -> &'static [&'static str] {
    match file.extension().and_then(|e| e.to_str()) {
        Some("rb") => &["def "],
        _ => &["fn "],
    }
}

/// An ExUnit test is named by a STRING, not an identifier: `test "it
/// works" do`. Rust and Ruby both take the word after a keyword, and
/// this is neither -- the third form the reader had to learn (wave
/// 0042). The name is the string itself, exactly as ExUnit reports it
/// and exactly as `mix test --only` selects it.
pub fn test_name(trimmed: &str) -> Option<String> {
    quoted_after(trimmed, "test ")
}

/// The name a `describe "..." do` block opens. ExUnit prefixes every
/// test inside it, so `mix test --only 'test:test <bare name>'`
/// matches NOTHING there -- measured, and it is why the reader tracks
/// the block rather than naming a border (wave 0042).
pub fn describe_name(trimmed: &str) -> Option<String> {
    quoted_after(trimmed, "describe ")
}

fn quoted_after(trimmed: &str, word: &str) -> Option<String> {
    let rest = trimmed.strip_prefix(word)?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let (name, tail) = rest.split_once('"')?;
    // `do` may sit on the line or open a block on the next; what must
    // not follow is more of the string.
    if name.is_empty() {
        return None;
    }
    let tail = tail.trim();
    if tail.is_empty() || tail.starts_with("do") || tail.starts_with(',') {
        Some(name.to_string())
    } else {
        None
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
