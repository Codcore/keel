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
- `keel review` -- the package for a fresh reviewer (§9.9)"#;

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
- `keel review` — пакет свіжому рецензентові (§9.9)"#;

/// What the machine really holds, per mode (review 0022 R-10).
const RULE_STRICT_EN: &str = r#"Two rules a machine holds here, so no memory has to: a scenario is born red -- the commit `red: <scenario>` passes the commit-msg hook only when its test really fails -- and the work commit `<transform>: <words>` passes only when that scenario's tests are green. Ask `keel next` instead of guessing the order."#;

const RULE_SOFT_EN: &str = r#"Two rules stand here as warnings (`mode = "soft"`): a scenario is born red -- the commit `red: <scenario>` is judged, and a commit that has not earned it is told so aloud without being blocked -- and the same for the work commit `<transform>: <words>`. The words are the machine's; holding to them is yours. Ask `keel next` instead of guessing the order."#;

const RULE_MANUAL_EN: &str = r#"The commit judgement is off in this project (`mode = "manual"`): the two rules -- a scenario born red, and work committed only over green tests -- are held by people alone here. `keel close` still judges before a merge. Ask `keel next` instead of guessing the order."#;

const RULE_STRICT_UK: &str = r#"Два правила тримає тут машина, і памʼять їх тримати не мусить: сценарій народжується червоним — commit `red: <сценарій>` проходить крізь commit-msg hook лише тоді, коли його тест справді падає, — а робочий commit `<трансформа>: <слова>` проходить лише зеленими тестами того сценарію. Питай `keel next`, а не вгадуй порядок."#;

const RULE_SOFT_UK: &str = r#"Два правила стоять тут попередженням (`mode = "soft"`): сценарій народжується червоним — commit `red: <сценарій>` судиться, і незароблене кажеться вголос, але не заслоняє commit, — те саме для робочого commit-а `<трансформа>: <слова>`. Слова — машинні, тримати їх — твоє. Питай `keel next`, а не вгадуй порядок."#;

const RULE_MANUAL_UK: &str = r#"Суд commit-ів у цьому проєкті вимкнено (`mode = "manual"`): обидва правила — народження червоним і робота лише поверх зелених тестів — тримають тут самі люди. `keel close` перед злиттям судить далі. Питай `keel next`, а не вгадуй порядок."#;

pub fn block(config: &Config) -> String {
    let uk = config.lang == "uk";
    let body = if uk { BODY_UK } else { BODY_EN };
    // What the machine really holds depends on the project's mode
    // (review 0022 R-10): under soft it warns, under manual it does
    // not judge at all, and the block must not promise otherwise.
    let rule = match (config.mode.as_str(), uk) {
        ("manual", true) => RULE_MANUAL_UK,
        ("manual", false) => RULE_MANUAL_EN,
        ("soft", true) => RULE_SOFT_UK,
        ("soft", false) => RULE_SOFT_EN,
        (_, true) => RULE_STRICT_UK,
        (_, false) => RULE_STRICT_EN,
    };
    format!("{BEGIN}\n{body}\n\n{rule}\n{END}")
}

/// The digest of a block: sha256 over its whitespace-collapsed text,
/// the first 12 hex -- the length of a trust fingerprint, because
/// this too is a judgement and not a document's revision.
pub fn digest(text: &str) -> String {
    // Byte-exact but for line endings (review 0022 R-6, R-7): an
    // edit of whitespace alone is an edit and must be seen, while a
    // checkout that turned LF into CRLF is not the person's doing
    // and must not be called one.
    let flat = text.replace("\r\n", "\n");
    let sum = Sha256::digest(flat.as_bytes());
    sum.iter().map(|b| format!("{b:02x}")).collect::<String>()[..12].to_string()
}

/// Writes the generated block and records its digest: the hand of
/// `keel init` and `keel update`. The second number counts what did
/// not stand -- zero is green, anything else honest red while the
/// rest of the frame still lands.
pub fn write(root: &Path, config: &Config) -> (String, usize) {
    if !config.present {
        // No keel.toml -- no project of ours, and nothing of a
        // stranger's directory is invented (review 0022 R-3; the
        // same guard trust has kept since 0010).
        return (t("generated-no-config"), 1);
    }
    let fresh = block(config);
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
            return (ta("generated-unread", targs!("error" => e.to_string())), 1);
        }
    };

    let (whole, word) = match text {
        // No document at all: born with the block alone.
        None => (format!("{fresh}\n"), t("generated-born")),
        Some(text) => {
            // More than one pair of markers: which block is ours is
            // not guessed (review 0022 R-11).
            if text.matches(BEGIN).count() > 1 || text.matches(END).count() > 1 {
                return (t("generated-many-blocks"), 1);
            }
            // The document keeps its own line endings (R-7).
            let crlf = text.contains("\r\n");
            let fresh = if crlf {
                fresh.replace('\n', "\r\n")
            } else {
                fresh.clone()
            };
            match span(&text) {
                None => {
                    if text.contains(BEGIN) || text.contains(END) {
                        // One marker without the other is not a block
                        // -- guessing where it ends would trample.
                        return (t("generated-half-marked"), 1);
                    }
                    if recorded.is_some() {
                        // The block was there and the person removed
                        // it: a decision, not a gap to fill -- and
                        // the word says how to have it back (R-2).
                        return (t("generated-removed"), 0);
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
                    if standing == fresh {
                        // Byte for byte what this release writes: it
                        // is ours by self-evidence, whatever the
                        // record says -- the state a failed write
                        // leaves must heal, not accuse (R-1).
                        if recorded.as_deref() != Some(digest(&fresh).as_str())
                            && let Err(refusal) = record(root, &digest(&fresh))
                        {
                            return (
                                ta("generated-config-failed", targs!("error" => refusal.reason)),
                                1,
                            );
                        }
                        return (t("generated-stands"), 0);
                    }
                    if recorded.as_deref() != Some(digest(standing).as_str()) {
                        // Not what this release wrote and not what
                        // was recorded: never trampled (R-8 names the
                        // fact, not a guess about who did it).
                        return (
                            ta(
                                "generated-changed",
                                targs!("file" => DOCUMENT.to_string(), "recorded" => recorded.unwrap_or_else(|| t("generated-none")), "actual" => digest(standing)),
                            ),
                            1,
                        );
                    }
                    let mut whole = String::with_capacity(text.len());
                    whole.push_str(&text[..from]);
                    whole.push_str(&fresh);
                    whole.push_str(&text[to..]);
                    (whole, t("generated-refreshed"))
                }
            }
        }
    };

    // The document first, the record after it (review 0022 R-1): a
    // digest recorded for a document that was never written is the
    // one state nothing can heal.
    if let Err(e) = std::fs::write(&path, &whole) {
        return (
            ta("generated-write-failed", targs!("error" => e.to_string())),
            1,
        );
    }
    let written =
        span(&whole).map_or_else(|| fresh.clone(), |(from, to)| whole[from..to].to_string());
    if let Err(refusal) = record(root, &digest(&written)) {
        return (
            ta("generated-config-failed", targs!("error" => refusal.reason)),
            1,
        );
    }
    (word, 0)
}

/// Where the block stands inside the document: the markers
/// INCLUDED, because they are part of what this release wrote and
/// therefore part of what its digest answers for (review 0022 R-12:
/// the comment used to say the opposite of the code).
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
