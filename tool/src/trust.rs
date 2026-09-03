//! Trust of commands, TOFU (contract tool-trust; §7.16, §2.8): a
//! command from repository files -- a contract's `verify` or the
//! project's `ci` -- is trusted only by its fingerprint recorded in
//! keel.toml's `[trust]`. The module runs no collected command: the
//! court stands before any runner exists.

use crate::config::Config;
use crate::docs::Contract;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use sha2::{Digest, Sha256};
use std::path::Path;

/// sha256 over the command's verbatim text, edges trimmed, twelve
/// hex characters. Verbatim deliberately (0010 review R-4): inside
/// quotes whitespace is the shell's own -- a command changed there
/// must read as changed, so the §7.16 words "new or changed does
/// not run" hold at the runner. Whitespace collapse remains the
/// school for KEY matching only (twin detection), never for the
/// fingerprint itself.
pub fn fingerprint(command: &str) -> String {
    let digest = Sha256::digest(command.trim().as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    hex[..12].to_string()
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Commands the repository's files carry right now: the `verify` of
/// every live contract (§2.12 -- withdrawn does not count) and the
/// project's `ci` unless refused aloud ("none") or left unwritten.
/// Pairs of (place, command text).
pub(crate) fn live_commands(config: &Config, contracts: &[Contract]) -> Vec<(String, String)> {
    let mut live = Vec::new();
    for contract in contracts {
        if contract.withdrawn.is_none()
            && let Some(command) = &contract.verify
        {
            live.push((
                format!("keel/contracts/{}.md", contract.slug),
                command.clone(),
            ));
        }
    }
    match config.ci.as_deref() {
        None | Some("none") | Some("") => {}
        Some(command) => live.push(("keel.toml".to_string(), command.to_string())),
    }
    live
}

/// The §7.16 court, without running anything: a live command with no
/// matching `[trust]` record is new -- a finding with its text and
/// the hint to record trust; a record whose fingerprint does not
/// match its live command is crooked; a record no live command
/// answers to is a door opened in advance; ci written empty is
/// undecided. Triples of (place, reason, instead).
pub fn court(config: &Config, contracts: &[Contract]) -> Vec<(String, String, String)> {
    let mut out = Vec::new();

    if config.ci.as_deref() == Some("") {
        out.push((
            "keel.toml".to_string(),
            t("trust-ci-empty"),
            t("trust-ci-empty-instead"),
        ));
    }

    let live = live_commands(config, contracts);
    for (place, command) in &live {
        let flat = collapse(command);
        match config.trust.iter().find(|(key, _)| collapse(key) == flat) {
            None => out.push((
                place.clone(),
                ta("trust-untrusted", targs!("command" => command.clone())),
                t("trust-untrusted-instead"),
            )),
            Some((_, recorded)) if *recorded != fingerprint(command) => out.push((
                place.clone(),
                ta("trust-crooked", targs!("command" => command.clone())),
                t("trust-crooked-instead"),
            )),
            Some(_) => {}
        }
    }

    for (key, _) in &config.trust {
        let flat = collapse(key);
        if !live.iter().any(|(_, command)| collapse(command) == flat) {
            out.push((
                "keel.toml".to_string(),
                ta("trust-door", targs!("command" => key.clone())),
                t("trust-door-instead"),
            ));
        }
    }

    out
}

/// Whether a command's fingerprint is recorded and true -- the
/// read-only gate the §7.6 runner asks before running anything;
/// this module itself still executes nothing.
pub(crate) fn trusted(config: &Config, command: &str) -> bool {
    let flat = collapse(command);
    config
        .trust
        .iter()
        .any(|(key, recorded)| collapse(key) == flat && *recorded == fingerprint(command))
}

/// The recording hand of §7.16 (`keel trust`): writes the
/// fingerprints of untrusted commands as `[trust]` lines -- surgery
/// on that block only, the rest of keel.toml stays as the human
/// wrote it -- and rewrites a crooked line of a live command (the
/// run itself is the human's word). Nothing new -- says so and does
/// not touch the file. Doors opened in advance are not removed
/// here: that is a human line in the diff, hinted by the court.
pub fn record(root: &Path) -> Result<String, Refusal> {
    let config = crate::config::read(root)?;
    if !config.present {
        return Err(Refusal {
            file: root.join("keel.toml"),
            reason: t("trust-no-config"),
            instead: t("trust-no-config-instead"),
        });
    }
    let scan = crate::docs::scan(root)?;
    if let Some(refusal) = scan.refusals.into_iter().next() {
        // Trust recorded over unread documents would guess; check
        // names every broken file -- fix them first.
        return Err(refusal);
    }

    let mut to_write: Vec<(String, String)> = Vec::new();
    for (_, command) in live_commands(&config, &scan.contracts) {
        let flat = collapse(&command);
        if to_write.iter().any(|(c, _)| collapse(c) == flat) {
            continue;
        }
        let print = fingerprint(&command);
        // Clean means exactly one record and a true fingerprint: a
        // crooked line masked by a correct collapse-twin (review
        // R-1) is dirt too, and the run consolidates it.
        let records: Vec<&(String, String)> = config
            .trust
            .iter()
            .filter(|(key, _)| collapse(key) == flat)
            .collect();
        let clean = records.len() == 1 && records[0].1 == print;
        if !clean {
            to_write.push((command, print));
        }
    }

    let mut report = t("trust-title");
    report.push('\n');
    if to_write.is_empty() {
        report.push_str(&t("trust-nothing-new"));
        report.push('\n');
        return Ok(report);
    }

    let path = root.join("keel.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| Refusal {
        file: path.clone(),
        reason: format!("keel.toml cannot be read: {e}"),
        instead: "check the path and file permissions".to_string(),
    })?;
    let written = crate::confedit::upsert(&text, "trust", &to_write);
    // The net under the surgery (review R-1): the result must still
    // parse for the strict reader of 0002 -- otherwise nothing is
    // written and the refusal says so. A dead config with a success
    // report behind it is the one forbidden outcome.
    if let Err(e) = toml::from_str::<toml::Value>(&written) {
        return Err(Refusal {
            file: path,
            reason: ta("trust-surgery-broken", targs!("error" => e.to_string())),
            instead: t("trust-surgery-broken-instead"),
        });
    }
    std::fs::write(&path, written).map_err(|e| Refusal {
        file: path.clone(),
        reason: format!("keel.toml cannot be written: {e}"),
        instead: "check the file permissions".to_string(),
    })?;

    for (command, print) in &to_write {
        report.push_str(&ta(
            "trust-recorded-line",
            targs!("command" => command.clone(), "fingerprint" => print.clone()),
        ));
        report.push('\n');
    }
    report.push_str(&t("trust-approves"));
    report.push('\n');
    Ok(report)
}
