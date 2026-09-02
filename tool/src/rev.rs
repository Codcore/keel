//! Chapter 5: revisions (contract tool-rev). The short signature of
//! a text, held by whoever leans on it; this rung takes the counting
//! over from the author's hands.

use crate::docs::{self, Refusal};
use crate::i18n::ta;
use crate::targs;
use sha2::{Digest, Sha256};
use std::path::Path;

/// The recipe (§5.3-§5.4), reproducing the hand recipe of waves
/// 0001-0002 byte for byte: runs of whitespace collapse into one
/// space, edges trimmed, sha256, first six hex characters.
pub fn text_rev(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let digest = Sha256::digest(normalized.as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    hex[..6].to_string()
}

/// A contract is hashed as the whole file, header included (§5.3).
pub fn contract_rev(path: &Path) -> Result<String, Refusal> {
    Ok(text_rev(&read(path)?))
}

/// Scenario revisions of a wave, in declared order: each scenario is
/// hashed as its section body -- from the line after
/// `## scenario: <name>` to the next `## ` heading (§5.3). A scenario
/// declared in the header without a body section refuses by name, and
/// so does a duplicated section: half of §7.7 arrives here naturally.
pub fn scenario_revs(path: &Path) -> Result<Vec<(String, String)>, Refusal> {
    let wave = docs::read_wave(path)?;
    let text = read(path)?;

    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            if let Some(name) = rest.strip_prefix("scenario: ") {
                current = Some((name.trim().to_string(), String::new()));
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(section) = current.take() {
        sections.push(section);
    }

    for (name, _) in &sections {
        if sections.iter().filter(|(n, _)| n == name).count() > 1 {
            return Err(Refusal {
                file: path.to_path_buf(),
                reason: ta("rev-dup-section", targs!("name" => name.clone())),
                instead: ta("rev-dup-section-instead", targs!("name" => name.clone())),
            });
        }
    }

    let mut out = Vec::new();
    for (name, _) in &wave.scenarios {
        match sections.iter().find(|(n, _)| n == name) {
            Some((_, body)) => out.push((name.clone(), text_rev(body))),
            None => {
                return Err(Refusal {
                    file: path.to_path_buf(),
                    reason: ta("rev-missing-section", targs!("name" => name.clone())),
                    instead: ta(
                        "rev-missing-section-instead",
                        targs!("name" => name.clone()),
                    ),
                });
            }
        }
    }
    Ok(out)
}

/// Prefix comparison (§5.2): a record of four to six characters
/// passes when it matches the start of the current revision.
pub fn matches(recorded: &str, actual: &str) -> bool {
    (4..=6).contains(&recorded.len()) && actual.starts_with(recorded)
}

/// File reading with the same refusal school as docs.
fn read(path: &Path) -> Result<String, Refusal> {
    std::fs::read_to_string(path).map_err(|e| Refusal {
        file: path.to_path_buf(),
        reason: if e.kind() == std::io::ErrorKind::InvalidData {
            crate::i18n::t("docs-not-utf8")
        } else {
            ta("docs-unreadable", targs!("error" => e.to_string()))
        },
        instead: if e.kind() == std::io::ErrorKind::InvalidData {
            crate::i18n::t("docs-not-utf8-instead")
        } else {
            crate::i18n::t("docs-unreadable-instead")
        },
    })
}
