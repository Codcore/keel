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
            Some((_, body)) if body.split_whitespace().next().is_none() => {
                return Err(Refusal {
                    file: path.to_path_buf(),
                    reason: ta("rev-empty-section", targs!("name" => name.clone())),
                    instead: ta("rev-empty-section-instead", targs!("name" => name.clone())),
                });
            }
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

/// The `keel rev` report: current revisions of every document, in
/// the project language; broken documents stand next to them as
/// refusals, never silence (the command inherits scan's refusals).
pub fn report(root: &Path, config: &crate::config::Config) -> Result<(String, usize), Refusal> {
    let scan = docs::scan(root)?;
    let mut refusals: Vec<Refusal> = scan.refusals;

    let mut lines: Vec<String> = Vec::new();
    for contract in &scan.contracts {
        let path = root
            .join("keel/contracts")
            .join(format!("{}.md", contract.slug));
        match contract_rev(&path) {
            Ok(revision) => lines.push(format!("  {}@{revision}", contract.slug)),
            Err(refusal) => refusals.push(refusal),
        }
    }
    for wave in &scan.waves {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        match scenario_revs(&path) {
            Ok(revs) => {
                for (name, revision) in revs {
                    lines.push(format!("  {}/{name}@{revision}", wave.slug));
                }
            }
            Err(refusal) => refusals.push(refusal),
        }
    }

    let mut report = crate::i18n::t("rev-title");
    report.push('\n');
    let config_line = if !config.present {
        crate::i18n::t("check-config-absent")
    } else if config.lang_set {
        ta(
            "check-config-present",
            targs!("lang" => config.lang.clone()),
        )
    } else {
        crate::i18n::t("check-config-lang-default")
    };
    report.push_str(&config_line);
    report.push('\n');
    report.push('\n');
    for line in &lines {
        report.push_str(line);
        report.push('\n');
    }
    for refusal in &refusals {
        let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
        report.push_str(&format!(
            "  {:<8} {} — {}\n           {}: {}\n",
            crate::i18n::t("word-red"),
            shown.display(),
            refusal.reason,
            crate::i18n::t("word-instead"),
            refusal.instead
        ));
    }
    report.push('\n');
    report.push_str(&crate::i18n::t("rev-next"));
    report.push('\n');
    Ok((report, refusals.len()))
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
