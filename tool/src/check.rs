//! Перший поверх перевірок: «документи читаються» (щабель 1).

use crate::docs::Refusal;
use std::path::Path;

pub struct Outcome {
    pub report: String,
    pub findings: usize,
}

/// Обходить документи під коренем і складає звіт по кожному файлу.
pub fn run(root: &Path) -> Result<Outcome, Refusal> {
    let _ = root;
    todo!("трансформа check-walks-project")
}
