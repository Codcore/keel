//! The settings wizard of `keel init` (contract tool-ask; the
//! operator's §8.6 decision of 2026-09-03: "the way some tools ask
//! you in the terminal at install time").
//!
//! The questions are DATA, so a machine can judge them without a
//! terminal: the drawing of them cannot be played without a pty, and
//! that limit is named in the contract rather than painted over.
//!
//! The heart of this hand is not the asking but the SILENCE. A tool
//! that asks where nobody is listening hangs -- in CI, in a test
//! sandbox, in a pipe. So the caller asks only where both ends are
//! terminals, and every answer keeps a road through a flag.

use crate::config::{AGENTS, LANGUAGES, MODES};
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::{Path, PathBuf};

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

/// The answers, as they will be written into keel.toml. None is "not
/// answered", and an unanswered field stays a comment in the file --
/// a default never passes itself off as a choice.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Answers {
    pub lang: Option<String>,
    pub adapter: Option<String>,
    pub mode: Option<String>,
    pub agents: Option<Vec<String>>,
    pub hooks: Option<bool>,
}

impl Answers {
    fn any(&self) -> bool {
        self.lang.is_some()
            || self.adapter.is_some()
            || self.mode.is_some()
            || self.agents.is_some()
            || self.hooks.is_some()
    }
}

/// The five questions, in the order they are asked -- exactly the
/// ones the operator named: the human language, the code's language,
/// the mode of the commit court, the agents, the hooks.
pub fn questions() -> Vec<Question> {
    vec![
        Question {
            field: "lang",
            choices: LANGUAGES.to_vec(),
            default: Some("en"),
            many: false,
            at_least_one: false,
            skippable: false,
        },
        Question {
            field: "adapter",
            choices: vec!["rust"],
            default: None,
            many: false,
            at_least_one: false,
            // A project of another tongue is not refused: it leaves
            // the field unwritten and waits for its own wave.
            skippable: true,
        },
        Question {
            field: "mode",
            choices: MODES.to_vec(),
            default: Some("strict"),
            many: false,
            at_least_one: false,
            skippable: false,
        },
        Question {
            field: "agents",
            choices: AGENTS.to_vec(),
            default: Some("claude"),
            many: true,
            at_least_one: true,
            skippable: false,
        },
        Question {
            field: "hooks",
            choices: vec!["yes", "no"],
            default: Some("yes"),
            many: false,
            at_least_one: false,
            skippable: false,
        },
    ]
}

/// The answers given on the command line, each judged by the
/// vocabulary of its own question -- before anything is written.
pub fn from_flags(given: &[(String, String)]) -> Result<Answers, Refusal> {
    let asked = questions();
    let mut answers = Answers::default();
    let mut answered: Vec<&str> = Vec::new();
    for (field, value) in given {
        let question = asked
            .iter()
            .find(|q| q.field == field.as_str())
            .ok_or_else(|| unknown_field(field))?;
        // Twice is a typo, not a choice (review 0026 R-17: the last
        // one used to win in silence).
        if answered.contains(&question.field) {
            return Err(twice(question));
        }
        answered.push(question.field);
        // The skip a terminal offers has a spelling on the command
        // line too, and it is the same one (R-11).
        if question.skippable && value == "-" {
            continue;
        }
        let named: Vec<String> = if question.many {
            value
                .split(',')
                .map(str::trim)
                .filter(|word| !word.is_empty())
                .map(str::to_string)
                .collect()
        } else {
            vec![value.clone()]
        };
        if question.many && question.at_least_one && named.is_empty() {
            return Err(nobody(question));
        }
        for word in &named {
            if !question.choices.contains(&word.as_str()) {
                return Err(unknown_value(question, word));
            }
        }
        match question.field {
            "lang" => answers.lang = Some(named[0].clone()),
            "adapter" => answers.adapter = Some(named[0].clone()),
            "mode" => answers.mode = Some(named[0].clone()),
            "agents" => answers.agents = Some(named),
            "hooks" => answers.hooks = Some(named[0] == "yes"),
            _ => return Err(unknown_field(field)),
        }
    }
    Ok(answers)
}

/// The answers asked for in a terminal, by the library the operator
/// chose. Called ONLY where both ends are terminals -- the caller
/// holds that law, and this hand is never reached in CI or in a test.
pub fn ask(questions: &[Question]) -> Result<Answers, Refusal> {
    let mut answers = Answers::default();
    for question in questions {
        let prompt = t(&format!("ask-{}", question.field));
        if question.many {
            let chosen = inquire::MultiSelect::new(&prompt, question.choices.clone())
                .with_validator(inquire::validator::MinLengthValidator::new(1))
                .prompt()
                .map_err(|e| interrupted(question, &e.to_string()))?;
            let chosen: Vec<String> = chosen.into_iter().map(str::to_string).collect();
            match question.field {
                "agents" => answers.agents = Some(chosen),
                other => return Err(unknown_field(other)),
            }
            continue;
        }
        let mut choices = question.choices.clone();
        if question.skippable {
            choices.push("-");
        }
        let chosen = inquire::Select::new(&prompt, choices)
            .prompt()
            .map_err(|e| interrupted(question, &e.to_string()))?;
        if chosen == "-" {
            continue;
        }
        match question.field {
            "lang" => {
                answers.lang = Some(chosen.to_string());
                // From here on the wizard speaks the language just
                // chosen (review 0026 R-3: eight Ukrainian words of
                // this hand could never be reached, because the
                // wizard runs only where there is no config to name
                // a language, so the process was nailed to English).
                crate::i18n::init(chosen);
            }
            "adapter" => answers.adapter = Some(chosen.to_string()),
            "mode" => answers.mode = Some(chosen.to_string()),
            "hooks" => answers.hooks = Some(chosen == "yes"),
            other => return Err(unknown_field(other)),
        }
    }
    Ok(answers)
}

/// The questions a person has NOT already answered with a flag.
/// A flag answers its own question and silences no others (review
/// 0026 R-4: one flag used to silence all five).
pub fn ask_unanswered(questions: &[Question], given: &Answers) -> Result<Answers, Refusal> {
    let left: Vec<Question> = questions
        .iter()
        .filter(|question| match question.field {
            "lang" => given.lang.is_none(),
            "adapter" => given.adapter.is_none(),
            "mode" => given.mode.is_none(),
            "agents" => given.agents.is_none(),
            "hooks" => given.hooks.is_none(),
            _ => true,
        })
        .cloned()
        .collect();
    if left.is_empty() {
        return Ok(given.clone());
    }
    let asked = ask(&left)?;
    Ok(Answers {
        lang: given.lang.clone().or(asked.lang),
        adapter: given.adapter.clone().or(asked.adapter),
        mode: given.mode.clone().or(asked.mode),
        agents: given.agents.clone().or(asked.agents),
        hooks: given.hooks.or(asked.hooks),
    })
}

/// The text of keel.toml the answers make. An answered field stands
/// as a line; an unanswered one stays a comment, so the vocabulary is
/// still there to read and a default never passes itself off as a
/// choice (the same honesty the config's own reading keeps).
pub fn config_text(answers: &Answers) -> String {
    let mut text = format!(
        "# {}\n# version = \"{}\"\n",
        t("init-config-header"),
        env!("CARGO_PKG_VERSION")
    );
    let line = |field: &str, value: Option<String>, shown: &str| match value {
        Some(value) => format!("{field} = {value}\n"),
        None => format!("# {field} = {shown}\n"),
    };
    text.push_str(&line(
        "lang",
        answers.lang.as_ref().map(|v| format!("\"{v}\"")),
        "\"uk\"",
    ));
    text.push_str(&line(
        "adapter",
        answers.adapter.as_ref().map(|v| format!("\"{v}\"")),
        "\"rust\"",
    ));
    text.push_str(&line(
        "mode",
        answers.mode.as_ref().map(|v| format!("\"{v}\"")),
        "\"strict\"",
    ));
    if answers.any() {
        // The two fields wave 0024 and 0025 gave the config are
        // written only where something was answered: a project that
        // was never asked keeps exactly the file it always got.
        text.push_str(&line(
            "agents",
            answers.agents.as_ref().map(|named| {
                let quoted: Vec<String> = named.iter().map(|name| format!("\"{name}\"")).collect();
                format!("[{}]", quoted.join(", "))
            }),
            "[\"claude\", \"cursor\"]",
        ));
        text.push_str(&line(
            "hooks",
            answers.hooks.map(|yes| yes.to_string()),
            "true",
        ));
    }
    text
}

/// The file every word of this hand is about: the config that is
/// being born (review 0026 R-14 -- "refusal: ." told nobody
/// anything).
fn about() -> PathBuf {
    Path::new(".").join("keel.toml")
}

fn twice(question: &Question) -> Refusal {
    Refusal {
        file: about(),
        reason: ta("ask-twice", targs!("field" => question.field.to_string())),
        instead: t("ask-twice-instead"),
    }
}

fn unknown_field(field: &str) -> Refusal {
    Refusal {
        file: about(),
        reason: ta("ask-unknown-field", targs!("field" => field.to_string())),
        instead: ta(
            "ask-unknown-field-instead",
            targs!("known" => questions().iter().map(|q| q.field).collect::<Vec<_>>().join(", ")),
        ),
    }
}

fn unknown_value(question: &Question, value: &str) -> Refusal {
    Refusal {
        file: about(),
        reason: ta(
            "ask-unknown-value",
            targs!("field" => question.field.to_string(), "value" => value.to_string()),
        ),
        instead: ta(
            "ask-unknown-value-instead",
            targs!("field" => question.field.to_string(), "known" => question.choices.join(", ")),
        ),
    }
}

fn nobody(question: &Question) -> Refusal {
    Refusal {
        file: about(),
        reason: ta("ask-nobody", targs!("field" => question.field.to_string())),
        instead: ta(
            "ask-nobody-instead",
            targs!("known" => question.choices.join(", ")),
        ),
    }
}

fn interrupted(question: &Question, said: &str) -> Refusal {
    Refusal {
        file: about(),
        reason: ta(
            "ask-interrupted",
            targs!("field" => question.field.to_string(), "error" => said.to_string()),
        ),
        instead: t("ask-interrupted-instead"),
    }
}
