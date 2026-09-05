//! Rung 16, the version lamp (contract tool-version; NEW-CONCEPT,
//! commands table: "keel version -- which version runs, where the
//! pin points"). The lamp looks with the unpinned eye and never
//! refuses over a mismatch: the mismatch is its answer.

use crate::config;
use crate::i18n::{t, ta};
use crate::targs;
use std::path::Path;

/// The two rows of the lamp: the running binary and the pin verdict.
/// Infallible by design -- where keel.toml cannot be read the row
/// says so with the reason aloud, and the config court speaks the
/// full refusal in its own turn (§7.9).
pub fn report(root: &Path) -> String {
    let running = env!("CARGO_PKG_VERSION");
    let mut out = ta("version-running", targs!("version" => running.to_string()));
    out.push('\n');
    let pin_row = match config::read_unpinned(root) {
        Ok(config) if !config.present => t("version-no-file"),
        Ok(config) => {
            if let Some(pin) = config.pin_mismatch(running) {
                // And the hand that makes them meet (wave 0039): the
                // verdict used to say the courts refuse until the pin
                // and the binary agree, and name nothing a person
                // could run to bring that about.
                format!(
                    "{}\n{}\n{}",
                    ta("version-pin-mismatch", targs!("pin" => pin.to_string())),
                    ta(
                        "version-pin-hand",
                        targs!(
                            "pin" => pin.to_string(),
                            "installer" => crate::generated::INSTALLER.to_string()
                        )
                    ),
                    // And what the hand is NOT (review 0039 R-3): the
                    // pin field holds a version number, the repository
                    // holds refs, and the two are not one word. Advice
                    // that hides that is advice that fails on the
                    // first real pin.
                    t("version-pin-hand-border")
                )
            } else {
                match config.version.as_deref() {
                    Some(pin) => ta("version-pin-held", targs!("pin" => pin.to_string())),
                    None => ta("version-pin-none", targs!("version" => running.to_string())),
                }
            }
        }
        Err(refusal) => ta("version-unread", targs!("reason" => refusal.reason)),
    };
    out.push_str(&pin_row);
    out.push('\n');
    out
}
