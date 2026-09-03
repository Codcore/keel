//! Rung 13, the frame hand (contract tool-init; NEW-CONCEPT
//! "cross-cutting", §8.7): the methodology's frame in one move --
//! and never a trampled byte. Every piece is its own line: born,
//! already stands, or a refusal aloud; the frame lands piece by
//! piece, and a second run builds only what is missing.

use crate::gate;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::Path;

/// The `keel init` report and the count of pieces that did not
/// stand: zero is a green exit, anything else is honest red while
/// the rest of the frame still landed.
pub fn run(root: &Path) -> Result<(String, usize), Refusal> {
    let mut report = t("init-title");
    report.push('\n');
    let mut failed = 0usize;

    // The three keel/ directories, each born with .gitkeep so an
    // empty one outlives git. A standing directory is a fact and
    // stays untouched -- except a missing .gitkeep, which is fed
    // with its own word (review 0014 R-2): a new empty file
    // tramples nothing, and "builds what is missing" stays true.
    for rel in ["keel/waves", "keel/contracts", "keel/reviews"] {
        let dir = root.join(rel);
        if dir.is_dir() {
            let keep = dir.join(".gitkeep");
            if keep.is_file() {
                report.push_str(&ta("init-stands", targs!("piece" => rel.to_string())));
            } else {
                match std::fs::write(&keep, "") {
                    Ok(()) => {
                        report.push_str(&ta("init-fed", targs!("piece" => rel.to_string())));
                    }
                    Err(e) => {
                        failed += 1;
                        report.push_str(&ta(
                            "init-failed",
                            targs!("piece" => rel.to_string(), "error" => e.to_string()),
                        ));
                    }
                }
            }
            report.push('\n');
            continue;
        }
        let born =
            std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(dir.join(".gitkeep"), ""));
        match born {
            Ok(()) => {
                report.push_str(&ta("init-born", targs!("piece" => rel.to_string())));
                report.push('\n');
            }
            Err(e) => {
                failed += 1;
                report.push_str(&ta(
                    "init-failed",
                    targs!("piece" => rel.to_string(), "error" => e.to_string()),
                ));
                report.push('\n');
            }
        }
    }

    // keel.toml with the commented config vocabulary (NEW-CONCEPT,
    // Config), enabling
    // nothing: the defaults stay with config's own words. An
    // existing file -- whoever's -- is a fact: not read, not
    // touched (its content is config's court, §7.9). The write
    // rides a dot-temp and a rename (the 0013 school): whole or
    // refused, never a stub.
    let config = root.join("keel.toml");
    if config.is_file() {
        report.push_str(&ta(
            "init-stands",
            targs!("piece" => "keel.toml".to_string()),
        ));
        report.push('\n');
    } else {
        let text = format!(
            "# {}\n# lang = \"uk\"\n# adapter = \"cargo\"\n# mode = \"strict\"\n",
            t("init-config-header")
        );
        match write_new(&config, &text) {
            Ok(()) => {
                report.push_str(&ta("init-born", targs!("piece" => "keel.toml".to_string())));
                report.push('\n');
            }
            Err(e) => {
                failed += 1;
                report.push_str(&ta(
                    "init-failed",
                    targs!("piece" => "keel.toml".to_string(), "error" => e),
                ));
                report.push('\n');
            }
        }
    }

    // The commit-msg hook by gate's own hand (§9.3) -- no double: a
    // foreign hook or a silent git is gate's refusal, said here as a
    // row, and the frame keeps landing around it.
    match gate::install_hook(root) {
        Ok(words) => {
            report.push_str("  ");
            report.push_str(&words);
            report.push('\n');
        }
        Err(refusal) => {
            failed += 1;
            let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
            // Where the refusal points at the root itself the
            // stripped name is empty -- the piece keeps its name.
            let shown = if shown.as_os_str().is_empty() {
                std::path::Path::new("commit-msg")
            } else {
                shown
            };
            report.push_str(&format!(
                "  {:<8} {} — {}\n           {}: {}\n",
                t("word-red"),
                shown.display(),
                refusal.reason,
                t("word-instead"),
                refusal.instead
            ));
        }
    }

    report.push('\n');
    report.push_str(&t("init-eight-seven"));
    report.push('\n');
    report.push_str(&t("init-next"));
    report.push('\n');
    Ok((report, failed))
}

/// One whole new file by dot-temp and rename (the 0013 write
/// school of plan) -- or the error's words, never a stub.
fn write_new(file: &Path, text: &str) -> Result<(), String> {
    let name = file.file_name().map(|n| n.to_string_lossy().into_owned());
    let tmp = file.with_file_name(format!(".{}.tmp", name.unwrap_or_default()));
    std::fs::write(&tmp, text).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, file)
        .inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
        .map_err(|e| e.to_string())
}
