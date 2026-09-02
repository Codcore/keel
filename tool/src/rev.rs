//! Chapter 5: revisions (contract tool-rev).

use crate::docs::Refusal;
use std::path::Path;

/// The recipe (§5.3-§5.4): the hand recipe of waves 0001-0002,
/// reproduced byte for byte.
pub fn text_rev(text: &str) -> String {
    let _ = text;
    todo!("transform compute-revisions")
}

/// A contract is hashed as the whole file, header included (§5.3).
pub fn contract_rev(path: &Path) -> Result<String, Refusal> {
    let _ = path;
    todo!("transform compute-revisions")
}

/// A scenario is hashed as its section body (§5.3); a scenario
/// declared in the header without a body section refuses by name.
pub fn scenario_revs(path: &Path) -> Result<Vec<(String, String)>, Refusal> {
    let _ = path;
    todo!("transform compute-revisions")
}

/// Prefix comparison (§5.2): 4-6 characters match the current head.
pub fn matches(recorded: &str, actual: &str) -> bool {
    let _ = (recorded, actual);
    todo!("transform compute-revisions")
}
