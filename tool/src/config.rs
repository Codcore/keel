//! keel.toml -- the single project config (contract tool-config).
//!
//! This module's refusals are deliberately English: when the config
//! is broken or the language unknown, the project language is not
//! known yet -- and will not be guessed.

use crate::docs::Refusal;
use std::collections::BTreeMap;
use std::path::Path;

/// Languages embedded in this release (i18n/<lang>.ftl).
pub const LANGUAGES: [&str; 2] = ["en", "uk"];

/// The modes of the commit judgement (journal A3).
pub const MODES: [&str; 3] = ["strict", "soft", "manual"];

/// The config as read. `lang` (wave 0002) and `mode` (wave 0005)
/// carry semantics; the other fields are read as data -- their rungs
/// are ahead.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub version: Option<String>,
    pub adapter: Option<String>,
    pub ci: Option<String>,
    pub lang: String,
    pub mode: String,
    pub trust: Vec<(String, String)>,
    pub generated: Vec<(String, String)>,
    /// The file really existed -- defaults do not pass themselves
    /// off as something read.
    pub present: bool,
    /// The lang field was written by hand; false means the default is
    /// in effect and must be named aloud, not printed as if read.
    pub lang_set: bool,
    /// Same honesty for mode: absent acts as strict and says so.
    pub mode_set: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: None,
            adapter: None,
            ci: None,
            lang: "en".to_string(),
            mode: "strict".to_string(),
            trust: Vec::new(),
            generated: Vec::new(),
            present: false,
            lang_set: false,
            mode_set: false,
        }
    }
}

impl Config {
    /// The one home of the adapter question (wave 0017, the
    /// operator's decision of 2026-09-03): the adapter is named by
    /// the project's language -- `rust` is canonical in this
    /// release, executed by the cargo adapter; the old spelling
    /// `cargo` stays an accepted synonym, said aloud by check. The
    /// courts ask here instead of comparing strings themselves.
    pub fn rust_adapter(&self) -> bool {
        matches!(self.adapter.as_deref(), Some("rust") | Some("cargo"))
    }

    /// The second question of the same home (review 0017 R-1): is
    /// the written spelling the accepted synonym? check asks here
    /// for its aloud word -- no court compares the string itself.
    pub(crate) fn adapter_synonym(&self) -> bool {
        self.adapter.as_deref() == Some("cargo")
    }
}

/// The full keel.toml vocabulary from the concept. An unknown field
/// fails the parse (deny_unknown_fields) -- a typo never reads as
/// "nothing".
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    version: Option<String>,
    adapter: Option<String>,
    ci: Option<String>,
    lang: Option<String>,
    mode: Option<String>,
    trust: Option<BTreeMap<String, String>>,
    generated: Option<BTreeMap<String, String>>,
}

/// Reads `<root>/keel.toml` whole and strictly; an absent file is
/// not an error but honest defaults (`present = false`).
pub fn read(root: &Path) -> Result<Config, Refusal> {
    let path = root.join("keel.toml");
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Config::default());
        }
        Err(e) => {
            return Err(Refusal {
                file: path,
                reason: format!("keel.toml cannot be read: {e}"),
                instead: "check the path and file permissions".to_string(),
            });
        }
    };

    let raw: Raw = toml::from_str(&text).map_err(|e| Refusal {
        file: path.clone(),
        reason: format!("keel.toml does not parse: {e}"),
        instead: "fix the named field; the vocabulary is: version, adapter, ci, \
                  lang, mode, [trust], [generated] (NEW-CONCEPT, Config)"
            .to_string(),
    })?;

    let lang_set = raw.lang.is_some();
    let lang = raw.lang.unwrap_or_else(|| "en".to_string());
    if !LANGUAGES.contains(&lang.as_str()) {
        return Err(Refusal {
            file: path,
            reason: format!(
                "language \"{lang}\" is not in this release: available are {}",
                LANGUAGES.join(", ")
            ),
            instead: "pick an available language, or add i18n/<lang>.ftl and \
                      ship a release (NEW-CONCEPT, Config)"
                .to_string(),
        });
    }

    let mode_set = raw.mode.is_some();
    let mode = raw.mode.unwrap_or_else(|| "strict".to_string());
    if !MODES.contains(&mode.as_str()) {
        return Err(Refusal {
            file: path,
            reason: format!(
                "mode \"{mode}\" is not one the gate knows: {}",
                MODES.join(", ")
            ),
            instead: "strict blocks, soft warns, manual turns the judgement off \
                      (journal A3)"
                .to_string(),
        });
    }

    Ok(Config {
        version: raw.version,
        adapter: raw.adapter,
        ci: raw.ci,
        lang,
        mode,
        trust: raw.trust.unwrap_or_default().into_iter().collect(),
        generated: raw.generated.unwrap_or_default().into_iter().collect(),
        present: true,
        lang_set,
        mode_set,
    })
}
