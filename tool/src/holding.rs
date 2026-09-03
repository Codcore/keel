//! Contracts' form court (contract tool-holding; §7.6, §2.7, §2.9):
//! promised signatures compared as collapsed text against the
//! module's source. Green form is not yet meaning (§7.8) -- that
//! gap is the reviewer's; and where there is nothing to compare
//! with, the report says so aloud instead of green. The module runs
//! nothing, builds nothing, writes nothing (§7.10).

use crate::adapter;
use crate::config::Config;
use crate::docs::{Contract, Wave};
use crate::i18n::{t, ta};
use crate::tags::TestTag;
use crate::targs;
use std::path::Path;

enum Comparability {
    Source(String),
    NoAdapter,
    Deep,
    NoFile,
}

/// The §7.6 form court over live contracts with module + exports: a
/// promised signature found in the module's collapsed source is
/// silence; the unit's name present with a different text is
/// "diverged" carrying the promise; an absent name is "vanished".
/// The incomparable is skipped here -- `survey` says it aloud.
pub fn court(
    root: &Path,
    config: &Config,
    contracts: &[Contract],
) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for contract in contracts {
        let Some((module, place)) = judged(contract) else {
            continue;
        };
        let Comparability::Source(source) = comparability(root, config, module) else {
            continue;
        };
        // Comments are not code (0010 review R-3): a promise that
        // survives only in a comment has vanished.
        let bare = strip_comments(&source);
        let flat_source = collapse(&bare);
        for signature in &contract.exports {
            if found_bounded(&flat_source, &collapse(signature)) {
                continue;
            }
            let name = unit_name(signature).unwrap_or_else(|| signature.clone());
            if found_bounded(&bare, &name) {
                out.push((
                    place.clone(),
                    ta(
                        "holding-diverged",
                        targs!("contract" => contract.slug.clone(), "signature" => signature.clone(), "name" => name),
                    ),
                    t("holding-diverged-instead"),
                ));
            } else {
                out.push((
                    place.clone(),
                    ta(
                        "holding-vanished",
                        targs!("contract" => contract.slug.clone(), "name" => name),
                    ),
                    t("holding-vanished-instead"),
                ));
            }
        }
    }
    out
}

/// The court's honest margins: how many signatures were compared,
/// and one line per contract whose form no one compared -- with the
/// reason, never painted green (§7.6).
pub(crate) fn survey(root: &Path, config: &Config, contracts: &[Contract]) -> (u64, Vec<String>) {
    let mut checked: u64 = 0;
    let mut uncompared: Vec<String> = Vec::new();
    for contract in contracts {
        let Some((module, _)) = judged(contract) else {
            continue;
        };
        let why = match comparability(root, config, module) {
            Comparability::Source(_) => {
                checked += contract.exports.len() as u64;
                continue;
            }
            Comparability::NoAdapter => {
                // A named yet unknown adapter is not painted absent
                // (review 0017 R-3): the words tell which it is.
                if config.adapter.is_some() {
                    t("holding-why-unknown-adapter")
                } else {
                    t("holding-why-no-adapter")
                }
            }
            Comparability::Deep => t("holding-why-deep"),
            Comparability::NoFile => t("holding-why-no-file"),
        };
        uncompared.push(ta(
            "check-holding-uncompared",
            targs!("contract" => contract.slug.clone(), "why" => why),
        ));
    }
    (checked, uncompared)
}

/// The approved-not-started window (§6.5; 0010 review R-1b healed
/// per §6.7): a contract held only by waves with no tag on any live
/// scenario was grown ahead of the code by a lawful plan -- its form
/// is not judged, and the skip is said aloud by name; any tag of a
/// holding wave brings the court back. Pairs of (contract, wave).
pub(crate) fn plan_window(
    root: &Path,
    waves: &[Wave],
    tags: &[TestTag],
    contracts: &[Contract],
) -> Vec<(String, String)> {
    use std::collections::BTreeMap;
    let mut holders: BTreeMap<&str, Vec<&Wave>> = BTreeMap::new();
    for wave in waves {
        let mut slugs: Vec<&str> = Vec::new();
        for (_, scenario) in &wave.scenarios {
            if scenario.withdrawn.is_none()
                && let Some(reference) = &scenario.proves
            {
                slugs.push(&reference.slug);
            }
        }
        for (_, transform) in &wave.transforms {
            for reference in &transform.contracts {
                slugs.push(&reference.slug);
            }
        }
        for slug in slugs {
            holders.entry(slug).or_default().push(wave);
        }
    }
    // A plan is a wave with at least one live scenario and none of
    // them tagged BY ITS OWN revision (review 0011 R-1/R-9): a wave
    // with every scenario withdrawn is not a plan -- the promised
    // first tag can never arrive, so the court stays; and a
    // namesake tag from a foreign wave, holding a foreign revision,
    // does not start this one.
    let is_plan = |wave: &Wave| {
        let live: Vec<&String> = wave
            .scenarios
            .iter()
            .filter(|(_, sc)| sc.withdrawn.is_none())
            .map(|(name, _)| name)
            .collect();
        if live.is_empty() {
            return false;
        }
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        let Ok(revs) = crate::rev::scenario_revs(&path) else {
            return false;
        };
        !live.iter().any(|name| {
            let own = revs
                .iter()
                .find(|(n, _)| &n == name)
                .map(|(_, r)| r.as_str())
                .unwrap_or("");
            tags.iter()
                .any(|t| t.scenario == **name && crate::rev::matches(&t.rev, own))
        })
    };
    let mut out = Vec::new();
    for contract in contracts {
        if judged(contract).is_none() {
            continue;
        }
        let Some(held) = holders.get(contract.slug.as_str()) else {
            continue;
        };
        if !held.is_empty() && held.iter().all(|w| is_plan(w)) {
            out.push((contract.slug.clone(), held[0].slug.clone()));
        }
    }
    out
}

/// A live contract with module + exports -- the only kind this
/// court judges (§2.12; a verify-only contract's proof runs in
/// close).
fn judged(contract: &Contract) -> Option<(&str, String)> {
    if contract.withdrawn.is_some() || contract.exports.is_empty() {
        return None;
    }
    let module = contract.module.as_deref()?;
    Some((module, format!("keel/contracts/{}.md", contract.slug)))
}

/// Where the module's source lives -- or why it cannot be compared:
/// the cargo adapter names the crate, the last module segment names
/// the file (the bare crate itself is src/lib.rs); deeper paths are
/// beyond this generation.
fn comparability(root: &Path, config: &Config, module: &str) -> Comparability {
    if !config.rust_adapter() {
        return Comparability::NoAdapter;
    }
    let segments: Vec<&str> = module.split("::").collect();
    if segments.len() > 2 {
        return Comparability::Deep;
    }
    let Ok(crate_dir) = adapter::crate_root(root) else {
        return Comparability::NoFile;
    };
    let path = if segments.len() == 1 {
        crate_dir.join("src/lib.rs")
    } else {
        crate_dir.join("src").join(format!("{}.rs", segments[1]))
    };
    match std::fs::read_to_string(path) {
        Ok(source) => Comparability::Source(source),
        Err(_) => Comparability::NoFile,
    }
}

/// A match is a match only on token boundaries (0010 review
/// R-3/R-6): `pub fn run` is not satisfied by `run_all`, and the
/// verdict words tell divergence from disappearance apart.
fn found_bounded(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let bytes = haystack.as_bytes();
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(needle) {
        let at = from + pos;
        let end = at + needle.len();
        let before_ok = at == 0 || !ident(bytes[at - 1]);
        let after_ok = end >= bytes.len() || !ident(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// Line and block comments cut out; string literals stay text --
/// that limit is named by the contract.
fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    loop {
        let line = rest.find("//");
        let block = rest.find("/*");
        match (line, block) {
            (None, None) => {
                out.push_str(rest);
                return out;
            }
            (Some(l), None) => {
                out.push_str(&rest[..l]);
                rest = rest[l..].split_once('\n').map_or("", |(_, tail)| tail);
                out.push('\n');
            }
            (Some(l), Some(b)) if l < b => {
                out.push_str(&rest[..l]);
                rest = rest[l..].split_once('\n').map_or("", |(_, tail)| tail);
                out.push('\n');
            }
            (_, Some(b)) => {
                out.push_str(&rest[..b]);
                rest = rest[b..].split_once("*/").map_or("", |(_, tail)| tail);
                out.push(' ');
            }
        }
    }
}

fn collapse(text: &str) -> String {
    // rustfmt's wrapping is formatting, not form (§2.9 compares what
    // the language writes): the trailing comma before a closing
    // brace or parenthesis and the spaces a line break leaves around
    // parentheses are normalized away, on both sides the same way --
    // proven by this court biting its own contract when fmt wrapped
    // the very signature promised here.
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("( ", "(")
        .replace(", )", ")")
        .replace(" )", ")")
        .replace(", }", " }")
}

/// The unit a signature promises: the word after the language's
/// keyword, so "diverged" and "vanished" are told apart honestly.
fn unit_name(signature: &str) -> Option<String> {
    const KEYWORDS: [&str; 8] = [
        "fn", "enum", "struct", "trait", "const", "static", "type", "mod",
    ];
    let mut tokens = signature.split_whitespace().peekable();
    while let Some(token) = tokens.next() {
        if KEYWORDS.contains(&token) {
            let name: String = tokens
                .next()?
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            return (!name.is_empty()).then_some(name);
        }
    }
    None
}
