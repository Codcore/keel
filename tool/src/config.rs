//! keel.toml — єдиний конфіг проєкту (контракт tool-config).

use crate::docs::Refusal;
use std::path::Path;

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

/// Читає `<root>/keel.toml` весь і суворо; відсутній файл — не
/// помилка, а чесні типові значення (`present = false`).
pub fn read(root: &Path) -> Result<Config, Refusal> {
    let _ = root;
    todo!("трансформа read-config")
}
