//! The settings wizard of `keel init` (contract tool-ask; the
//! operator's §8.6 decision of 2026-09-03).
//!
//! The questions are DATA, so that a machine can judge them without a
//! terminal: the drawing of them cannot be played without a pty, and
//! that limit is named in the contract rather than painted over.

use crate::refusal::Refusal;

/// One question of the wizard: the field it answers, the vocabulary
/// it accepts, its default, and how it may be answered.
#[derive(Debug, Clone, PartialEq)]
pub struct Question {
    pub field: &'static str,
    pub choices: Vec<&'static str>,
    pub default: Option<&'static str>,
    /// Several may be ticked (the agents).
    pub many: bool,
    /// And at least one must be -- the operator's own law, held here
    /// rather than by our discipline.
    pub at_least_one: bool,
    /// May be left unanswered, leaving the field unwritten.
    pub skippable: bool,
}

/// The answers, as they will be written into keel.toml.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Answers {
    pub lang: Option<String>,
    pub adapter: Option<String>,
    pub mode: Option<String>,
    pub agents: Option<Vec<String>>,
    pub hooks: Option<bool>,
}

/// The five questions, in the order they are asked.
pub fn questions() -> Vec<Question> {
    Vec::new()
}

/// The answers given on the command line, each judged by the
/// vocabulary of its own question.
pub fn from_flags(_given: &[(String, String)]) -> Result<Answers, Refusal> {
    Ok(Answers::default())
}

/// The answers asked for in a terminal.
pub fn ask(_questions: &[Question]) -> Result<Answers, Refusal> {
    Ok(Answers::default())
}

/// The text of keel.toml the answers make.
pub fn config_text(_answers: &Answers) -> String {
    String::new()
}
