//! A refusal is interface, not service noise: the file, the reason
//! in human language, and what to do instead. The frame is printed
//! in the project language.

use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Refusal {
    pub file: PathBuf,
    pub reason: String,
    pub instead: String,
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}\n  {}: {}\n  {}: {}",
            crate::i18n::t("word-refusal"),
            self.file.display(),
            crate::i18n::t("word-reason"),
            self.reason,
            crate::i18n::t("word-instead"),
            self.instead
        )
    }
}
