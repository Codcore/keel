//! The mouth of the tool (contract tool-speak; the operator's §8.6
//! decision): everything the thin generated block does not carry must
//! live in the binary itself -- the methodology and the forty cuts
//! included, because a project where keel stands has neither.

use crate::refusal::Refusal;

/// The forty cuts with the question each asks.
pub fn cuts() -> Vec<(&'static str, &'static str, &'static str)> {
    Vec::new()
}

/// The same, from a checklist handed in -- so the court between the
/// judge's list and the document a person reads can be played.
pub fn cuts_from(_checklist: &str) -> Result<Vec<(&str, &str, &str)>, Refusal> {
    Ok(Vec::new())
}

/// The cuts as a report.
pub fn cuts_report() -> String {
    String::new()
}

/// The methodology: its contents, or one paragraph of it.
pub fn method(_asked: Option<&str>) -> Result<String, Refusal> {
    Ok(String::new())
}
