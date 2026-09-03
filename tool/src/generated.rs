//! The generated integrations (contract tool-generated; NEW-CONCEPT,
//! "Distribution"): what agents and CI read must lie in the
//! repository, though it is no source of the tool. This wave gives
//! the mechanism and the first artefact -- a block in AGENTS.md.
//!
//! The heart is the boundary: outside the markers not one byte
//! changes, and a block a person edited by hand is refused aloud,
//! never overwritten.

use crate::config::Config;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use sha2::{Digest, Sha256};
use std::path::Path;

/// The document the block lives in, and its key in `[generated]`.
const DOCUMENT: &str = "AGENTS.md";
const BEGIN: &str = "<!-- keel:begin -->";
const END: &str = "<!-- keel:end -->";

/// The block as this release writes it, in the project's language:
/// what the project is, what the loop is, and which commands say
/// the next word. It says of itself that it is generated -- editing
/// it by hand is work the next `keel update` would undo.
/// The body of the block, per language of this release. It is a
/// generated DOCUMENT, not a word of the tool's own -- so it lives
/// here as a template, while i18n keeps the rows that report what
/// happened to it.
const BODY_EN: &str = r#"# keel (generated -- do not edit; keel update rewrites this block)

This project follows the Keel v2 methodology. What lives here:
`keel/waves/` (what was promised and proven), `keel/contracts/`
(promises that outlive a wave), `keel/reviews/` (a fresh reader's
verdict on each wave) and `keel.toml` (the project's config).

The loop, one step at a time:

- `keel next` -- the single next step, and nothing beyond it
- `keel status` -- where the wave stands
- `keel plan <name>` / `keel new contract <name>` -- skeletons a
  person fills; the tool never writes the content of a plan
- `keel check` -- the documents judged
- `keel close` -- whether a wave may merge
- `keel review` -- the package for a fresh reviewer (§9.9)

Two rules a machine holds, so no memory has to: a scenario is born
red -- the commit `red: <scenario>` passes the commit-msg hook only
when its test really fails -- and the work commit
`<transform>: <words>` passes only when that scenario's tests are
green. Ask `keel next` instead of guessing the order."#;

const BODY_UK: &str = r#"# keel (згенеровано — не правити руками; keel update перепише цей блок)

Цей проєкт живе за методикою Keel v2. Що тут лежить:
`keel/waves/` (що обіцяно і доведено), `keel/contracts/`
(обіцянки, що переживають хвилю), `keel/reviews/` (вирок свіжого
читача на кожну хвилю) і `keel.toml` (конфіг проєкту).

Луп, по одному кроку:

- `keel next` — єдиний наступний крок, і нічого понад нього
- `keel status` — де стоїть хвиля
- `keel plan <імʼя>` / `keel new contract <імʼя>` — риштування, які
  заповнює людина; змісту плану інструмент не пише ніколи
- `keel check` — суд над документами
- `keel close` — чи можна зливати хвилю
- `keel review` — пакет свіжому рецензентові (§9.9)

Два правила тримає машина, і памʼять їх тримати не мусить: сценарій
народжується червоним — commit `red: <сценарій>` проходить крізь
commit-msg hook лише тоді, коли його тест справді падає, — а робочий
commit `<трансформа>: <слова>` проходить лише зеленими тестами того
сценарію. Питай `keel next`, а не вгадуй порядок."#;

pub fn block(lang: &str) -> String {
    let body = if lang == "uk" { BODY_UK } else { BODY_EN };
    format!("{BEGIN}\n{body}\n{END}")
}

/// The digest of a block: sha256 over its whitespace-collapsed text,
/// the first 12 hex -- the length of a trust fingerprint, because
/// this too is a judgement and not a document's revision.
pub fn digest(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let sum = Sha256::digest(flat.as_bytes());
    sum.iter().map(|b| format!("{b:02x}")).collect::<String>()[..12].to_string()
}

/// Writes the generated block and records its digest: the hand of
/// `keel init` and `keel update`. The second number counts what did
/// not stand -- zero is green, anything else honest red while the
/// rest of the frame still lands.
pub fn write(root: &Path, config: &Config) -> Result<(String, usize), Refusal> {
    let fresh = block(&config.lang);
    let path = root.join(DOCUMENT);
    let recorded = config
        .generated
        .iter()
        .find(|(key, _)| key == DOCUMENT)
        .map(|(_, value)| value.clone());

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return Ok((ta("generated-unread", targs!("error" => e.to_string())), 1));
        }
    };

    let (whole, word) = match text {
        // No document at all: born with the block alone.
        None => (format!("{fresh}\n"), t("generated-born")),
        Some(text) => match span(&text) {
            // A document of the person's own, with no block of ours:
            // the block goes after it, their text untouched.
            None => {
                if text.contains(BEGIN) || text.contains(END) {
                    // One marker without the other is not a block --
                    // guessing where it ends would trample.
                    return Ok((t("generated-half-marked"), 1));
                }
                if recorded.is_some() {
                    // The block was there and the person removed it:
                    // that is a decision, not a gap to fill.
                    return Ok((t("generated-removed"), 0));
                }
                let mut whole = text.clone();
                if !whole.ends_with('\n') {
                    whole.push('\n');
                }
                whole.push('\n');
                whole.push_str(&fresh);
                whole.push('\n');
                (whole, t("generated-appended"))
            }
            Some((from, to)) => {
                let standing = &text[from..to];
                let mine = recorded.as_deref() == Some(digest(standing).as_str());
                if !mine {
                    // A hand edited it (or a release wrote it before
                    // digests were kept): never trampled.
                    return Ok((
                        ta("generated-changed", targs!("file" => DOCUMENT.to_string())),
                        1,
                    ));
                }
                if standing == fresh {
                    return Ok((t("generated-stands"), 0));
                }
                let mut whole = String::with_capacity(text.len());
                whole.push_str(&text[..from]);
                whole.push_str(&fresh);
                whole.push_str(&text[to..]);
                (whole, t("generated-refreshed"))
            }
        },
    };

    // The config first: a document written without its digest would
    // be a stranger to the very next run.
    if let Err(refusal) = record(root, &digest(&fresh)) {
        return Ok((
            ta("generated-config-failed", targs!("error" => refusal.reason)),
            1,
        ));
    }
    if let Err(e) = std::fs::write(&path, whole) {
        return Ok((
            ta("generated-write-failed", targs!("error" => e.to_string())),
            1,
        ));
    }
    Ok((word, 0))
}

/// Where the block's own text stands inside the document: between
/// the markers, exclusive of them.
fn span(text: &str) -> Option<(usize, usize)> {
    let begin = text.find(BEGIN)?;
    let end = text[begin..].find(END)? + begin + END.len();
    Some((begin, end))
}

/// The digest into `[generated]`, by the one hand that edits the
/// config -- and never a config that would not parse afterwards
/// (the 0010 school).
fn record(root: &Path, digest: &str) -> Result<(), Refusal> {
    let path = root.join("keel.toml");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let written = crate::confedit::upsert(
        &text,
        "generated",
        &[(DOCUMENT.to_string(), digest.to_string())],
    );
    if let Err(e) = toml::from_str::<toml::Value>(&written) {
        return Err(Refusal {
            file: path,
            reason: e.to_string(),
            instead: t("generated-config-failed-instead"),
        });
    }
    std::fs::write(&path, written).map_err(|e| Refusal {
        file: path,
        reason: e.to_string(),
        instead: t("generated-config-failed-instead"),
    })
}
