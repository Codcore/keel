//! Contracts' form court (contract tool-holding; §7.6, §2.7, §2.9):
//! promised signatures compared as collapsed text against the
//! module's source. Green form is not yet meaning (§7.8) -- that
//! gap is the reviewer's; and where there is nothing to compare
//! with, the report says so aloud instead of green. The module runs
//! nothing, builds nothing, writes nothing (§7.10).

use crate::adapter;
use crate::config::Config;
use crate::docs::Contract;
use crate::i18n::{t, ta};
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
        let flat_source = collapse(&source);
        for signature in &contract.exports {
            if flat_source.contains(&collapse(signature)) {
                continue;
            }
            let name = unit_name(signature).unwrap_or_else(|| signature.clone());
            if source.contains(&name) {
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
            Comparability::NoAdapter => t("holding-why-no-adapter"),
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
    if config.adapter.as_deref() != Some("cargo") {
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
