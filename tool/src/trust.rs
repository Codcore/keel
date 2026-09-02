//! Trust of commands, TOFU (contract tool-trust; §7.16, §2.8): a
//! command from repository files -- a contract's `verify` or the
//! project's `ci` -- is trusted only by its fingerprint recorded in
//! keel.toml's `[trust]`. The module runs no collected command: the
//! court stands before any runner exists.

use crate::config::Config;
use crate::docs::Contract;
use crate::i18n::{t, ta};
use crate::targs;
use sha2::{Digest, Sha256};

/// The §5.3 school over a command's text -- whitespace runs to one
/// space, edges trimmed, sha256 -- but twelve hex characters, not
/// six: this is the trust court, and the concept's own example is
/// this long.
pub fn fingerprint(command: &str) -> String {
    let flat = collapse(command);
    let digest = Sha256::digest(flat.as_bytes());
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
