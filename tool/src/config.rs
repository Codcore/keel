//! keel.toml — єдиний конфіг проєкту (контракт tool-config).
//!
//! Відмови цього модуля свідомо англійською: коли конфіг зламаний
//! або мова невідома, мови проєкту ми ще не знаємо — і не вгадуємо.

use crate::docs::Refusal;
use std::collections::BTreeMap;
use std::path::Path;

/// Мови, вшиті в цей реліз (i18n/<мова>.ftl).
pub const LANGUAGES: [&str; 2] = ["en", "uk"];

/// Прочитаний конфіг. Семантику в цій хвилі несе лише `lang`; решта
/// полів читаються як дані — їхні щаблі попереду.
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub version: Option<String>,
    pub adapter: Option<String>,
    pub ci: Option<String>,
    pub lang: String,
    pub trust: Vec<(String, String)>,
    pub generated: Vec<(String, String)>,
    /// Файл справді існував — типові значення не видають себе за
    /// прочитане.
    pub present: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            version: None,
            adapter: None,
            ci: None,
            lang: "en".to_string(),
            trust: Vec::new(),
            generated: Vec::new(),
            present: false,
        }
    }
}

/// Повний словник keel.toml з концепту. Невідоме поле валить розбір
/// (deny_unknown_fields) — одрук не читається як «нічого».
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    version: Option<String>,
    adapter: Option<String>,
    ci: Option<String>,
    lang: Option<String>,
    trust: Option<BTreeMap<String, String>>,
    generated: Option<BTreeMap<String, String>>,
}

/// Читає `<root>/keel.toml` весь і суворо; відсутній файл — не
/// помилка, а чесні типові значення (`present = false`).
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
                  lang, [trust], [generated] (NEW-CONCEPT, Config)"
            .to_string(),
    })?;

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

    Ok(Config {
        version: raw.version,
        adapter: raw.adapter,
        ci: raw.ci,
        lang,
        trust: raw.trust.unwrap_or_default().into_iter().collect(),
        generated: raw.generated.unwrap_or_default().into_iter().collect(),
        present: true,
    })
}
