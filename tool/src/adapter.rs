//! The cargo adapter (contract tool-adapter-cargo): the one place
//! that knows how a Rust project names its test files and how to run
//! a single test. Other languages will get their own waves next to
//! this file.

use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::tags::TestTag;
use crate::targs;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The directory this adapter builds into: the home of that
/// knowledge is here, because it is the adapter's own (wave 0020).
/// The frame asks it to advise an ignore rule; the next language
/// arrives with its own adapter and its own name.
pub const BUILD_DIR: &str = "target";

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

/// What one run of one test came to -- in cargo's own words where
/// they matter. "Did not compile" and "did not run" are not
/// "failed" (journal A3).
pub enum Outcome {
    Failed,
    Green,
    BuildBroken(String),
    NotRun,
}

/// Runs exactly the tagged test (`cargo test --test <file> <fn> --
/// --exact`) and classifies the consequence. cargo is called as a
/// command of the system; its refusal to start is a refusal aloud.
pub fn run_test(root: &Path, tag: &TestTag) -> Result<Outcome, Refusal> {
    let crate_dir = crate_root(root)?;
    let stem = tag
        .file
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut command = Command::new("cargo");
    command
        .arg("test")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .args(["--test", &stem, &tag.test, "--", "--exact"])
        // The judged project builds into its own target directory:
        // an inherited shared cache shifts verdicts (§6.7 heal of
        // 0005 per review 0008 R-8; seen live in 0006 too), and the
        // cargo alias walks it back in through the side door
        // (review 0009 R-2).
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR");
    // And it runs in the project's own world: a hook's repository,
    // left in the environment, would otherwise reach the project's
    // own tests through cargo (review 0021 R-3).
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: crate_dir.clone(),
        reason: ta("adapter-cargo-failed", targs!("error" => e.to_string())),
        instead: t("adapter-cargo-failed-instead"),
    })?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("could not compile") || stderr.contains("error[E") {
        let words = stderr
            .lines()
            .find(|l| l.starts_with("error"))
            .unwrap_or("could not compile")
            .to_string();
        return Ok(Outcome::BuildBroken(words));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let Some(result) = stdout.lines().find(|l| l.starts_with("test result:")) else {
        return Err(Refusal {
            file: crate_dir,
            reason: ta(
                "adapter-cargo-failed",
                targs!("error" => stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no test result line").to_string()),
            ),
            instead: t("adapter-cargo-failed-instead"),
        });
    };
    let executed = count_before(result, " passed") + count_before(result, " failed");
    if executed == 0 {
        return Ok(Outcome::NotRun);
    }
    if count_before(result, " failed") > 0 {
        Ok(Outcome::Failed)
    } else {
        Ok(Outcome::Green)
    }
}

/// The whole battery in one cargo run, verdicts laid out per test:
/// the key is (test file stem, function name), the value is green.
/// One run instead of one per tag -- the closure court reads it
/// once. A build that does not build is a refusal aloud with the
/// compiler's words: without a build there is no verdict for anyone.
pub fn run_all(root: &Path) -> Result<BTreeMap<(String, String), bool>, Refusal> {
    let crate_dir = crate_root(root)?;
    let mut command = Command::new("cargo");
    command
        .arg("test")
        .arg("--manifest-path")
        .arg(crate_dir.join("Cargo.toml"))
        .arg("--no-fail-fast")
        // Same isolation as run_test: the shared cache lies, and the
        // hook's repository must not reach the project's tests.
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR");
    crate::scope::forget_the_hook(&mut command);
    let out = command.output().map_err(|e| Refusal {
        file: crate_dir.clone(),
        reason: ta("adapter-cargo-failed", targs!("error" => e.to_string())),
        instead: t("adapter-cargo-failed-instead"),
    })?;
    let stderr = String::from_utf8_lossy(&out.stderr);
    if stderr.contains("could not compile") || stderr.contains("error[E") {
        let words = stderr
            .lines()
            .find(|l| l.starts_with("error"))
            .unwrap_or("could not compile")
            .to_string();
        return Err(Refusal {
            file: crate_dir,
            reason: ta("adapter-cargo-failed", targs!("error" => words)),
            instead: t("adapter-cargo-failed-instead"),
        });
    }
    // cargo splits its word: the target list ("Running tests/x.rs")
    // goes to stderr, the verdicts go to stdout -- one block per
    // target, in the same order, since targets run one after another.
    // The stems and the blocks are stitched back by that order.
    let mut stems: Vec<String> = Vec::new();
    for line in stderr.lines() {
        let trimmed = line.trim();
        if let Some(what) = trimmed.strip_prefix("Running ") {
            let path = what.split_whitespace().next().unwrap_or("");
            stems.push(
                Path::new(path)
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            );
        } else if trimmed.starts_with("Doc-tests ") {
            stems.push("doc-tests".to_string());
        }
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let mut verdicts = std::collections::BTreeMap::new();
    let mut block: usize = 0;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("running ") && trimmed.ends_with("tests")
            || trimmed == "running 1 test"
        {
            block += 1;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("test ")
            && let Some((name, verdict)) = rest.rsplit_once(" ... ")
        {
            let green = match verdict.trim() {
                "ok" => true,
                v if v.starts_with("FAILED") => false,
                _ => continue, // ignored and friends are no verdict
            };
            let stem = block
                .checked_sub(1)
                .and_then(|i| stems.get(i))
                .cloned()
                .unwrap_or_default();
            verdicts.insert((stem, name.trim().to_string()), green);
        }
    }
    // The stitch holds only when every announced target printed its
    // verdict block: a harness = false target prints "Running" and
    // no block, shifting every later verdict onto the wrong stem --
    // up to blessing a wave with a red tagged test (review R-1). A
    // seam that does not meet is a refusal, not a guess.
    if block != stems.len() {
        return Err(Refusal {
            file: crate_dir,
            reason: ta(
                "adapter-battery-mismatch",
                targs!("stems" => stems.len() as u64, "blocks" => block as u64),
            ),
            instead: t("adapter-battery-mismatch-instead"),
        });
    }
    if verdicts.is_empty() && !out.status.success() {
        return Err(Refusal {
            file: crate_dir,
            reason: ta(
                "adapter-cargo-failed",
                targs!("error" => stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("no test output").to_string()),
            ),
            instead: t("adapter-cargo-failed-instead"),
        });
    }
    Ok(verdicts)
}

/// The number standing right before the given marker in cargo's
/// "test result:" line.
fn count_before(line: &str, marker: &str) -> u64 {
    let Some(end) = line.find(marker) else {
        return 0;
    };
    line[..end]
        .split(|c: char| !c.is_ascii_digit())
        .next_back()
        .and_then(|n| n.parse().ok())
        .unwrap_or(0)
}
