//! The first floor of checks: "documents read" (bootstrap rung 1).
//! Not the whole methodology check yet -- and the report says so
//! itself: green about the unchecked is forbidden (lesson 4 of the
//! notes triage).

use crate::config::Config;
use crate::docs;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::fmt::Write as _;
use std::path::Path;

pub struct Outcome {
    pub report: String,
    pub findings: usize,
}

/// Walks the documents under the root and reports on every file:
/// intact ones checked, broken ones named with a reason; at the end
/// -- what this floor has checked, what it has not yet, and the next
/// step. The report language is lang from the config; the report
/// also says where the language came from.
pub fn run(root: &Path, config: &Config) -> Result<Outcome, Refusal> {
    let scan = docs::scan(root)?;

    let mut rows: Vec<(String, Option<String>)> = Vec::new();
    for wave in &scan.waves {
        rows.push((format!("keel/waves/{}.md", wave.slug), None));
    }
    for contract in &scan.contracts {
        rows.push((format!("keel/contracts/{}.md", contract.slug), None));
    }
    for refusal in &scan.refusals {
        let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
        rows.push((
            shown.display().to_string(),
            Some(format!(
                "{}\n           {}: {}",
                refusal.reason,
                t("word-instead"),
                refusal.instead
            )),
        ));
    }
    rows.sort();

    let mut report = t("check-title");
    report.push('\n');
    let config_line = if !config.present {
        t("check-config-absent")
    } else if config.lang_set {
        ta(
            "check-config-present",
            targs!("lang" => config.lang.clone()),
        )
    } else {
        t("check-config-lang-default")
    };
    writeln!(report, "{config_line}\n").unwrap();

    for (path, verdict) in &rows {
        match verdict {
            None => {
                writeln!(
                    report,
                    "  {:<8} {path} — {}",
                    t("word-green"),
                    t("check-header-reads")
                )
                .unwrap();
            }
            Some(text) => {
                writeln!(report, "  {:<8} {path} — {text}", t("word-red")).unwrap();
            }
        }
    }
    if rows.is_empty() {
        writeln!(report, "  {}", t("check-no-documents")).unwrap();
    }

    let findings = scan.refusals.len();
    let documents = scan.waves.len() + scan.contracts.len();
    writeln!(
        report,
        "\n{}\n{}\n{}",
        t("check-checked"),
        t("check-unchecked"),
        ta(
            "check-summary",
            targs!("docs" => documents as u64, "refusals" => findings as u64)
        )
    )
    .unwrap();
    let next = if findings > 0 {
        t("check-next-fix")
    } else if documents == 0 {
        t("check-next-first-wave")
    } else {
        t("check-next-rung")
    };
    writeln!(report, "{next}").unwrap();

    Ok(Outcome { report, findings })
}
