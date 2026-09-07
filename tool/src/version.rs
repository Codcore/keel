//! Rung 16, the version lamp (contract tool-version; NEW-CONCEPT,
//! commands table: "keel version -- which version runs, where the
//! pin points"). The lamp looks with the unpinned eye and never
//! refuses over a mismatch: the mismatch is its answer.

use crate::config;
use crate::i18n::{t, ta};
use crate::targs;
use std::path::Path;

/// A word for a person to paste, safe in a shell: the pin is a
/// stranger's string, and `x; echo PWNED` in it made two commands of
/// the advised one (review 0057 R-10). The apostrophe inside is
/// closed, escaped and reopened, as the run lines of the adapter do.
fn shell_quoted(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
}

/// The road a pin's own SHAPE takes, mirrored from `release_tag` in
/// install.sh (wave 0057): `v?MAJOR.MINOR.PATCH(-prerelease)?` reaches
/// a published release -- archive and `.sha256`, no git and no cargo --
/// and anything else is a git ref, cloned and built from source. The
/// two readings must agree: these words promise what that script does,
/// and a pin with a slash, a space or `..` never reaches a URL.
fn release_shaped(pin: &str) -> bool {
    let body = pin.strip_prefix('v').unwrap_or(pin);
    let (core, pre) = match body.split_once('-') {
        Some((core, pre)) => (core, Some(pre)),
        None => (body, None),
    };
    let numbers: Vec<&str> = core.split('.').collect();
    if numbers.len() != 3
        || !numbers
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
    {
        return false;
    }
    match pre {
        None => true,
        Some(pre) => {
            !pre.is_empty()
                && pre
                    .split('.')
                    .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric()))
        }
    }
}

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
                    "{}\n{}\n{}\n{}",
                    ta("version-pin-mismatch", targs!("pin" => pin.to_string())),
                    ta(
                        "version-pin-hand",
                        targs!(
                            // Quoted, as the KEEL_REF form used to be
                            // (review 0057 R-10): the pin comes from a
                            // keel.toml a stranger wrote, the line is
                            // for a person to paste, and `x; echo` in
                            // it made two commands of one. An empty
                            // pin now shows as '' instead of a line
                            // that looks finished (R-14).
                            "pin" => shell_quoted(pin),
                            "installer" => crate::generated::INSTALLER.to_string()
                        )
                    ),
                    // WHICH road this pin takes is decided by its own
                    // shape, and the words say the one it really takes
                    // (wave 0057): measured against a `file://`
                    // release, `KEEL_REF="1.0.0" sh install.sh` fetched
                    // the archive and verified its sha256 -- while the
                    // lamp told every pin alike that nothing checks a
                    // checksum there (queue after 0055, bugs R-26).
                    if release_shaped(pin) {
                        ta(
                            "version-pin-road-release",
                            targs!("pin" => shell_quoted(pin)),
                        )
                    } else {
                        t("version-pin-road-ref")
                    },
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
    // One read, not two (review 0041 R-17).
    let standing = installed();
    if standing.is_empty() {
        out.push_str(&t("version-installed-none"));
        out.push('\n');
    }
    for one in standing {
        out.push_str(&ta(
            "version-installed",
            targs!("version" => one.version, "ref" => one.named),
        ));
        out.push('\n');
    }
    out
}

/// One version standing on this machine: what it answers for itself,
/// and the ref it was installed under.
pub struct Standing {
    pub version: String,
    pub named: String,
}

/// The versions in `~/.keel/versions/` (wave 0041). The concept asks
/// the lamp for three things -- which version runs, where the pin
/// points, and which versions stand locally -- and the third had no
/// answer, because until this wave only one could stand.
///
/// The home is `KEEL_HOME` where it is set, exactly as the installer
/// and the launcher read it, so a probe and a person with a moved
/// home see the same list.
pub fn installed() -> Vec<Standing> {
    let home = std::env::var("KEEL_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::env::var("HOME")
                .map(|home| std::path::PathBuf::from(home).join(".keel"))
                .unwrap_or_default()
        });
    let Ok(entries) = std::fs::read_dir(home.join("versions")) else {
        return Vec::new();
    };
    let mut out: Vec<Standing> = entries
        .flatten()
        .filter_map(|entry| {
            let version = std::fs::read_to_string(entry.path().join(".keel-version")).ok()?;
            // The ref as it was asked for, not the encoded directory
            // name: a ref may carry a slash, and the home encodes it
            // (review 0041 R-4).
            let named = std::fs::read_to_string(entry.path().join(".keel-ref"))
                .map(|named| named.trim().to_string())
                .unwrap_or_else(|_| entry.file_name().to_string_lossy().into_owned());
            Some(Standing {
                version: version.trim().to_string(),
                named,
            })
        })
        .collect();
    out.sort_by(|a, b| a.named.cmp(&b.named));
    out
}
