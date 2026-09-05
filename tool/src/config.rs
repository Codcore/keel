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

/// The agents this release generates integrations for (the
/// operator's §8.6 decision of 2026-09-03: keel serves more than one
/// agent, each with its own option). `codex` is postponed by his
/// word, so this release does not know the name at all -- an
/// accepted key that writes nothing would promise what is not there.
pub const AGENTS: [&str; 2] = ["claude", "cursor"];

/// The config as read. `lang` (wave 0002) and `mode` (wave 0005)
/// carry semantics; the other fields are read as data -- their rungs
/// are ahead.
/// The languages this release can lead a project in. Adding one is a
/// module and a row here -- which is the whole point of the wave that
/// made this an enum instead of a yes-or-no question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    Rust,
    Ruby,
}

impl Language {
    /// Every spelling this release accepts, canonical name first.
    /// `cargo` is the old spelling of `rust`, kept and said aloud by
    /// check (wave 0017, review R-1).
    pub const NAMES: [(&'static str, Language); 3] = [
        ("rust", Language::Rust),
        ("cargo", Language::Rust),
        ("ruby", Language::Ruby),
    ];

    pub fn named(word: &str) -> Option<Language> {
        Self::NAMES
            .iter()
            .find(|(name, _)| *name == word)
            .map(|(_, language)| *language)
    }

    /// The canonical names, for a refusal that says what it knows.
    pub fn known() -> String {
        let mut out: Vec<&str> = Vec::new();
        for (name, _) in Self::NAMES {
            if !out.contains(&name) {
                out.push(name);
            }
        }
        out.join(", ")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub version: Option<String>,
    pub adapter: Option<String>,
    pub ci: Option<String>,
    pub lang: String,
    pub mode: String,
    pub trust: Vec<(String, String)>,
    pub generated: Vec<(String, String)>,
    /// The agents the project named, as written. Judged at read
    /// time: an empty list and an unknown name are refusals, so what
    /// stands here is either empty (the key was absent) or known.
    pub agents: Vec<String>,
    /// Whether the hook configs are generated at all (wave 0026, the
    /// operator's question in the wizard). Absent acts as true: a
    /// question whose answer changes nothing is not a question, so
    /// "no" really means no hook artefacts.
    pub hooks: bool,
    /// The file really existed -- defaults do not pass themselves
    /// off as something read.
    pub present: bool,
    /// The lang field was written by hand; false means the default is
    /// in effect and must be named aloud, not printed as if read.
    pub lang_set: bool,
    /// Same honesty for mode: absent acts as strict and says so.
    pub mode_set: bool,
    /// And for agents: absent acts as ["claude"] -- the behaviour of
    /// every release before 0024 -- and the flag keeps that default
    /// from passing itself off as something read.
    pub agents_set: bool,
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
            agents: Vec::new(),
            hooks: true,
            present: false,
            lang_set: false,
            mode_set: false,
            agents_set: false,
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
        self.language() == Some(Language::Rust)
    }

    /// Which language this release will judge here, by the name the
    /// project wrote (wave 0038). `None` means the project named no
    /// adapter at all -- the language-shaped courts do not run, and
    /// say so. A name this release does not know never reaches here:
    /// the config court refuses it by name, listing what it knows,
    /// because a typo swallowed as silence is a court skipped in
    /// silence.
    pub fn language(&self) -> Option<Language> {
        Language::named(self.adapter.as_deref()?)
    }

    /// The second question of the same home (review 0017 R-1): is
    /// the written spelling the accepted synonym? check asks here
    /// for its aloud word -- no court compares the string itself.
    pub(crate) fn adapter_synonym(&self) -> bool {
        self.adapter.as_deref() == Some("cargo")
    }

    /// The one home of the agent question (wave 0024): the canonical
    /// order and no duplicates, so the table of artefacts is judged
    /// the same however the list was written. An empty list never
    /// reaches here -- `read_unpinned` refuses it, because "at least
    /// one" is the operator's law and it belongs in the config, not
    /// only in the wizard that will ask the question.
    pub fn agents(&self) -> Vec<&'static str> {
        if !self.agents_set {
            return vec!["claude"];
        }
        AGENTS
            .iter()
            .copied()
            .filter(|known| self.agents.iter().any(|named| named == known))
            .collect()
    }

    /// The one home of the pin question (wave 0018; NEW-CONCEPT,
    /// Distribution): the project pins the tool's exact version --
    /// string equality, ranges are not of this generation. Some is
    /// the pinned name to say aloud beside the running one; None
    /// means the pin holds or none is set.
    pub fn pin_mismatch(&self, running: &str) -> Option<&str> {
        match self.version.as_deref() {
            Some(pin) if pin != running => Some(pin),
            _ => None,
        }
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
    agents: Option<Vec<String>>,
    hooks: Option<bool>,
}

/// Reads and judges: the raw read plus the pin court (wave 0018) --
/// a foreign pin is a refusal before any court runs, so the wrong
/// binary never judges a project. The refusal is deliberately
/// English: the binary refusing is not the one the project asked
/// for, and it does not guess past what it read.
pub fn read(root: &Path) -> Result<Config, Refusal> {
    let config = read_unpinned(root)?;
    let running = env!("CARGO_PKG_VERSION");
    if let Some(pin) = config.pin_mismatch(running) {
        return Err(Refusal {
            file: root.join("keel.toml"),
            reason: format!("keel.toml pins version \"{pin}\", but this binary is keel {running}"),
            instead: "run the pinned version, or move the pin -- one line in a \
                      diff, approved by merge; to move it forward, run the new binary \
                      first: its own gate passes the new pin (NEW-CONCEPT, \
                      Distribution)"
                .to_string(),
        });
    }
    Ok(config)
}

/// Reads `<root>/keel.toml` whole and strictly; an absent file is
/// not an error but honest defaults (`present = false`). No pin
/// court here -- this is the `keel version` lamp's eye
/// (tool-version): it must answer exactly where the courts refuse.
pub fn read_unpinned(root: &Path) -> Result<Config, Refusal> {
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
                  lang, mode, agents, hooks, [trust], [generated] (NEW-CONCEPT, Config)"
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

    // The agents (wave 0024, the operator's §8.6 decision). An empty
    // list is not an answer -- "at least one" is his law, and it
    // stands here rather than only in the wizard that will ask.
    let agents_set = raw.agents.is_some();
    let agents = raw.agents.unwrap_or_default();
    if agents_set {
        if agents.is_empty() {
            return Err(Refusal {
                file: path,
                reason: "agents = [] names nobody, and at least one is required".to_string(),
                instead: format!(
                    "name at least one of {} -- or remove the key, and the default \"claude\" \
                     stands, exactly as before (NEW-CONCEPT, Config)",
                    AGENTS.join(", ")
                ),
            });
        }
        if let Some(unknown) = agents
            .iter()
            .find(|named| !AGENTS.contains(&named.as_str()))
        {
            return Err(Refusal {
                file: path,
                reason: format!(
                    "agent \"{unknown}\" is not one this release knows: {}",
                    AGENTS.join(", ")
                ),
                instead: format!(
                    "name one of {} -- an agent whose integrations this release does not \
                     generate is not accepted quietly, because a key that writes nothing \
                     promises what is not there",
                    AGENTS.join(", ")
                ),
            });
        }
    }

    Ok(Config {
        version: raw.version,
        adapter: raw.adapter,
        ci: raw.ci,
        lang,
        mode,
        trust: raw.trust.unwrap_or_default().into_iter().collect(),
        generated: raw.generated.unwrap_or_default().into_iter().collect(),
        agents,
        hooks: raw.hooks.unwrap_or(true),
        present: true,
        lang_set,
        mode_set,
        agents_set,
    })
}
