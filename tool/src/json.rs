//! The machine's road out (contract tool-json; NEW-CONCEPT, the CLI
//! contract: "every command has `--json`; the output is a
//! self-sufficient package").
//!
//! One envelope for every command, so a harness learns the shape once
//! and reads any of them. What it carries is what the courts already
//! computed -- the exit code, the findings with their files, the
//! limits, the summary's numbers -- plus the whole prose report, so
//! nothing a person would read is lost to a machine.
//!
//! The border, said here rather than discovered: this does NOT turn
//! every sentence of a verdict into a typed field. Where a command
//! has no structure beyond its prose, the package carries `report`
//! and says so with `structured: false` -- faking a shape would be
//! worse than naming its absence.
//!
//! The package is built by a real serializer. Wave 0025 learnt that
//! lesson on the hook configs: a path with a quote or a backslash in
//! it, or a refusal quoting a file's own words, breaks string
//! concatenation and does not break this.

use crate::refusal::Refusal;
use serde_json::{Map, Value, json};
use std::path::Path;

/// The version of the envelope itself. A harness reads this first and
/// knows whether it understands the rest; it changes only when a
/// field changes meaning, never when one is added.
pub const ENVELOPE: u64 = 1;

pub struct Package {
    command: &'static str,
    root: String,
    lang: String,
    report: String,
    exit: i32,
    structured: bool,
    refusal: Option<Refusal>,
    fields: Map<String, Value>,
}

impl Package {
    pub fn new(command: &'static str, root: &Path, lang: &str) -> Self {
        Package {
            command,
            root: root.display().to_string(),
            lang: lang.to_string(),
            report: String::new(),
            exit: 0,
            structured: false,
            refusal: None,
            fields: Map::new(),
        }
    }

    /// The prose a person would have read, whole.
    pub fn report(mut self, text: impl Into<String>) -> Self {
        self.report = text.into();
        self
    }

    pub fn exit(mut self, code: i32) -> Self {
        self.exit = code;
        self
    }

    /// The command's own answer, refused. The reason and the
    /// "instead" are fields, because the CLI contract asks for both
    /// by name and a harness should not have to split a sentence.
    pub fn refused(mut self, refusal: &Refusal) -> Self {
        self.report = refusal.to_string();
        self.refusal = Some(Refusal {
            file: refusal.file.clone(),
            reason: refusal.reason.clone(),
            instead: refusal.instead.clone(),
        });
        self
    }

    /// One field of this command's own. Naming any at all is what
    /// makes the package structured rather than prose in a box.
    pub fn field(mut self, key: &str, value: Value) -> Self {
        self.structured = true;
        self.fields.insert(key.to_string(), value);
        self
    }

    fn value(&self) -> Value {
        let mut out = Map::new();
        out.insert("keel".to_string(), json!(ENVELOPE));
        out.insert("command".to_string(), json!(self.command));
        out.insert("ok".to_string(), json!(self.exit == 0));
        out.insert("exit".to_string(), json!(self.exit));
        out.insert("root".to_string(), json!(self.root));
        out.insert("lang".to_string(), json!(self.lang));
        out.insert("report".to_string(), json!(self.report));
        out.insert("structured".to_string(), json!(self.structured));
        if let Some(refusal) = &self.refusal {
            out.insert(
                "refusal".to_string(),
                json!({
                    "file": refusal.file.display().to_string(),
                    "reason": refusal.reason,
                    "instead": refusal.instead,
                }),
            );
        }
        for (key, value) in &self.fields {
            out.insert(key.clone(), value.clone());
        }
        Value::Object(out)
    }

    /// One object on stdout and nothing else -- a harness reads the
    /// whole of stdout and parses it, so a stray line would be a
    /// parse error rather than a warning.
    pub fn print(&self) {
        println!("{}", self.value());
    }
}

/// The findings a court gathered, as a harness wants them: the file
/// that is wrong and why, rather than a line of prose to split.
pub fn findings(rows: &[(String, Option<String>)]) -> Value {
    Value::Array(
        rows.iter()
            .filter_map(|(file, said)| {
                said.as_ref()
                    .map(|reason| json!({"file": file, "reason": reason}))
            })
            .collect(),
    )
}

/// The limits -- what was not judged, and why -- as they were said.
pub fn limits(rows: &[String]) -> Value {
    Value::Array(rows.iter().map(|row| json!(row)).collect())
}
