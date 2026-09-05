//! The generated integrations (contract tool-generated; NEW-CONCEPT,
//! "Distribution"): what agents and CI read must lie in the
//! repository, though it is no source of the tool. The artefacts
//! are a table -- a block inside a person's document (AGENTS.md),
//! and files wholly ours (the loop skill, the CI workflow).
//!
//! The heart is the boundary: what is the person's does not change
//! by a byte, and an artefact a hand has touched is refused aloud,
//! never overwritten.

use crate::config::Config;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use sha2::{Digest, Sha256};
use std::path::Path;

const BEGIN: &str = "<!-- keel:begin -->";
const END: &str = "<!-- keel:end -->";

/// How much of a file is ours (wave 0023). A document belongs to
/// the person and holds our block between markers; a file that is
/// wholly ours is judged whole.
enum Kind {
    Block,
    Whole,
    /// A guest in someone else's file (wave 0025): keel gives birth
    /// to it when it is absent, and then judges it whole like any
    /// file of ours -- but it never writes over a file it did not
    /// write. `.claude/settings.json` holds a person's SETTINGS, and
    /// "delete the file" would be harm there, not advice.
    ///
    /// A guest carries what it would add and WHERE, because a whole
    /// document is not something one can paste into a document that
    /// already exists: review 0025 R-1 measured that advice taken
    /// literally -- two of the three readings give invalid JSON, and
    /// the third silently eats the person's own hooks. So the word
    /// names the key and hands the entries that go under it.
    Guest {
        key: &'static str,
        entries: String,
    },
}

/// Whose artefact a row is (wave 0024). A document every agent
/// reads belongs to none of them in particular; a skill belongs to
/// the agent whose directory it lies in.
enum Owner {
    Any,
    One(&'static str),
}

/// The artefacts this release generates: path, kind of boundary, and
/// the text it writes. Only the rows whose owner the project named
/// are written -- nothing of an agent it never named. Adding one
/// more is a row here.
///
/// The same skill lives in two homes on purpose: Claude Code reads
/// only `.claude/skills/`, while `.agents/skills/` is the
/// vendor-neutral home of the Agent Skills standard, which is what
/// Cursor reads (and Codex, whose option waits for its own wave --
/// a fact about the directory, not a promise of this release).
fn artefacts(config: &Config) -> Vec<(&'static str, Kind, String)> {
    let named = config.agents();
    let skill = skill(config);
    let rows: Vec<(&'static str, Kind, Owner, String)> = vec![
        ("AGENTS.md", Kind::Block, Owner::Any, block(config)),
        (
            ".claude/skills/keel/SKILL.md",
            Kind::Whole,
            Owner::One("claude"),
            skill.clone(),
        ),
        (
            ".agents/skills/keel/SKILL.md",
            Kind::Whole,
            Owner::One("cursor"),
            skill,
        ),
        (
            ".claude/settings.json",
            Kind::Guest {
                key: "hooks",
                entries: claude_entries(),
            },
            Owner::One("claude"),
            claude_hooks(),
        ),
        (
            ".cursor/hooks.json",
            Kind::Guest {
                key: "hooks",
                entries: cursor_entries(),
            },
            Owner::One("cursor"),
            cursor_hooks(),
        ),
        (
            ".github/workflows/keel.yml",
            Kind::Whole,
            Owner::Any,
            workflow(config),
        ),
    ];
    rows.into_iter()
        .filter(|(_, _, owner, _)| match owner {
            Owner::Any => true,
            Owner::One(agent) => named.contains(agent),
        })
        .map(|(path, kind, _, text)| (path, kind, text))
        .collect()
}

/// The block as this release writes it, in the project's language:
/// what the project is, what the loop is, and which commands say
/// the next word. It says of itself that it is generated -- editing
/// it by hand is work the next `keel update` would undo.
/// The body of the block, per language of this release. It is a
/// generated DOCUMENT, not a word of the tool's own -- so it lives
/// here as a template, while i18n keeps the rows that report what
/// happened to it.
const BODY_EN: &str = r#"# keel (generated -- do not edit; keel update rewrites this {what})

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
- `keel cuts` -- the forty quality cuts, with the question each asks
- `keel method` -- the methodology itself; `keel method §8.6` for one
  piece of it

What a wave promises is a person's decision, never the tool's and
never an agent's alone: bring a card -- the problem, two to four
options with their consequences, a recommendation and why -- and
write the plan after their word (§8.6). `keel plan` lays the
skeleton and never the content."#;

const BODY_UK: &str = r#"# keel (згенеровано — не правити руками; keel update перепише цей {what})

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
- `keel cuts` — сорок розрізів якости, з питанням кожного
- `keel method` — сама методика; `keel method §8.6` — один її шматок

Що обіцяє хвиля — рішення людини, ніколи не інструмента і ніколи не
агента самого: неси картку — проблема, два-чотири варіанти з
наслідками, рекомендація і чому — і пиши план після її слова
(§8.6). `keel plan` кладе риштування і ніколи не зміст."#;

/// What the machine really holds, per mode (review 0022 R-10).
const RULE_STRICT_EN: &str = r#"Two rules a machine holds here, so no memory has to: a scenario is born red -- the commit `red: <scenario>` passes the commit-msg hook only when its test really fails -- and the work commit `<transform>: <words>` passes only when that scenario's tests are green. Ask `keel next` instead of guessing the order."#;

const RULE_SOFT_EN: &str = r#"Two rules stand here as warnings (`mode = "soft"`): a scenario is born red -- the commit `red: <scenario>` is judged, and a commit that has not earned it is told so aloud without being blocked -- and the same for the work commit `<transform>: <words>`. The words are the machine's; holding to them is yours. Ask `keel next` instead of guessing the order."#;

const RULE_MANUAL_EN: &str = r#"The commit judgement is off in this project (`mode = "manual"`): the two rules -- a scenario born red, and work committed only over green tests -- are held by people alone here. `keel close` still judges before a merge. Ask `keel next` instead of guessing the order."#;

const RULE_STRICT_UK: &str = r#"Два правила тримає тут машина, і памʼять їх тримати не мусить: сценарій народжується червоним — commit `red: <сценарій>` проходить крізь commit-msg hook лише тоді, коли його тест справді падає, — а робочий commit `<трансформа>: <слова>` проходить лише зеленими тестами того сценарію. Питай `keel next`, а не вгадуй порядок."#;

const RULE_SOFT_UK: &str = r#"Два правила стоять тут попередженням (`mode = "soft"`): сценарій народжується червоним — commit `red: <сценарій>` судиться, і незароблене кажеться вголос, але не заслоняє commit, — те саме для робочого commit-а `<трансформа>: <слова>`. Слова — машинні, тримати їх — твоє. Питай `keel next`, а не вгадуй порядок."#;

const RULE_MANUAL_UK: &str = r#"Суд commit-ів у цьому проєкті вимкнено (`mode = "manual"`): обидва правила — народження червоним і робота лише поверх зелених тестів — тримають тут самі люди. `keel close` перед злиттям судить далі. Питай `keel next`, а не вгадуй порядок."#;

/// The loop skill an agent reads (wave 0023): the same words as the
/// block, shaped as a skill file, because that is what Claude Code
/// loads. Wholly ours -- a hand's edit is refused, never overwritten.
fn skill(config: &Config) -> String {
    let uk = config.lang == "uk";
    let (name, description) = if uk {
        (
            "keel",
            "Луп методики Keel v2: що робити далі і що судить машина",
        )
    } else {
        (
            "keel",
            "The Keel v2 loop: what to do next, and what the machine judges",
        )
    };
    let what = if uk { "файл" } else { "file" };
    let body = if uk { BODY_UK } else { BODY_EN }.replace("{what}", what);
    let rule = rule_for(config);
    // The description carries a colon, and a plain YAML scalar with
    // ": " in it opens a mapping: quoted, it is text (review 0023
    // R-1 -- the file exists to be read by a parser).
    format!("---\nname: \"{name}\"\ndescription: \"{description}\"\n---\n\n{body}\n\n{rule}\n")
}

/// The session hook of Claude Code (wave 0025), in the shape its own
/// documentation gives: `.claude/settings.json`, the `hooks` key, the
/// event name, a matcher group, and a handler of type "command". A
/// hook's plain stdout on exit 0 goes into the agent's context, so
/// printing the step is the whole of it. The matcher names all five
/// documented sources of a session start, so the step is there after
/// a resume and after a compact too -- exactly when an agent has
/// forgotten it.
///
/// `${CLAUDE_PROJECT_DIR}` is their documented variable; it is quoted
/// because a space in a real path would otherwise split the command
/// (the school of v1, which learned it the hard way).
///
/// No line inside says "generated": JSON has no comment, and
/// inventing a key in someone else's schema is the very thing this
/// wave forbids. The tool says it instead -- in the report row and in
/// `[generated]` of keel.toml.
fn claude_entries() -> String {
    "{\n\
     \u{20}\u{20}\u{20}\u{20}\"SessionStart\": [\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}{\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\"matcher\": \"startup|resume|clear|compact|fork\",\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\"hooks\": [\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}{\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\"type\": \"command\",\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\"command\": \"keel next \\\"${CLAUDE_PROJECT_DIR}\\\"\",\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\"timeout\": 30\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}}\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}]\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}}\n\
     \u{20}\u{20}\u{20}\u{20}]\n\
     \u{20}\u{20}}"
        .to_string()
}

/// The whole document, when the file is ours to give birth to: the
/// same entries under the key they belong to, and nothing else.
fn claude_hooks() -> String {
    format!("{{\n\u{20}\u{20}\"hooks\": {}\n}}\n", claude_entries())
}

/// The session hook of Cursor (wave 0025), in the shape its own
/// documentation gives: `.cursor/hooks.json`, `version` 1, the event
/// `sessionStart`, and an entry whose only field is `command` --
/// nothing invented beside it. Cursor takes context at session start
/// ONLY as JSON, in `additional_context`, so the command asks for the
/// step in that shape: `keel next --for cursor`. The working
/// directory of a project hook is the project root (their docs), so
/// no path argument is needed.
///
/// Their `sessionStart` is fire-and-forget by their own docs, and
/// their forum carries reports that the context does not always reach
/// the agent. We write the documented shape; the delivery is their
/// side of the boundary, and the wave says so aloud.
fn cursor_entries() -> String {
    "{\n\
     \u{20}\u{20}\u{20}\u{20}\"sessionStart\": [\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}{\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\"command\": \"keel next --for cursor\"\n\
     \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}}\n\
     \u{20}\u{20}\u{20}\u{20}]\n\
     \u{20}\u{20}}"
        .to_string()
}

/// The whole document for Cursor: their `version` is required beside
/// the hooks, so the birth carries it; the entries handed to a person
/// who already has a file of their own do not, because their file has
/// its own version line already.
fn cursor_hooks() -> String {
    format!(
        "{{\n\u{20}\u{20}\"version\": 1,\n\u{20}\u{20}\"hooks\": {}\n}}\n",
        cursor_entries()
    )
}

/// The CI workflow (wave 0023): the three courts a merge needs.
/// It calls keel as a command and does NOT install it -- the
/// installing step arrives with the distribution rung, and the file
/// says so itself.
fn workflow(config: &Config) -> String {
    // The battery step is the tongue's own, and its absence is never
    // silence (review 0038 R-9): a project whose adapter this release
    // does not lead gets a line saying so, in the file where a person
    // would otherwise look for the step and find nothing.
    let courts = match config.language() {
        Some(language) => format!(
            "      - name: the battery\n        run: {}\n",
            language.battery_command()
        ),
        None => "      # No battery step: keel.toml names no adapter this\n\
                 \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# release leads, so it does not know how this project\n\
                 \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# runs its tests. `keel close` still runs the battery\n\
                 \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# where it can; add your own step here otherwise.\n"
            .to_string(),
    };
    format!(
        "# keel (generated -- do not edit; keel update rewrites this file)\n\
         #\n\
         # This workflow calls `keel` as a command and does NOT\n\
         # install it: the installing step arrives with the\n\
         # distribution rung of the concept (~/.keel/versions/).\n\
         # Until then, add a step of your own above these that puts\n\
         # `keel` on PATH.\n\
         name: keel\n\
         \n\
         on:\n\
         \u{20}\u{20}push:\n\
         \u{20}\u{20}pull_request:\n\
         \n\
         jobs:\n\
         \u{20}\u{20}keel:\n\
         \u{20}\u{20}\u{20}\u{20}runs-on: ubuntu-latest\n\
         \u{20}\u{20}\u{20}\u{20}steps:\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}- uses: actions/checkout@v4\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}with:\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}fetch-depth: 0\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}- name: the documents judged\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# On a pull_request event actions/checkout leaves a\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# detached HEAD and git serves no branch, so the scope\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# court would be skipped in silence. Sec. 4.10 asks for\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# the branch to be named where git hides it; this names\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}# it. Where git does know the branch, git is believed.\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}env:\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}KEEL_BRANCH: ${{{{ github.head_ref || github.ref_name }}}}\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}run: keel check .\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}- name: the closure court\n\
         \u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}\u{20}run: keel close .\n\
         {courts}"
    )
}

/// The rule paragraph for this project's mode -- shared by the
/// block and the skill, so the two never disagree.
fn rule_for(config: &Config) -> &'static str {
    let uk = config.lang == "uk";
    match (config.mode.as_str(), uk) {
        ("manual", true) => RULE_MANUAL_UK,
        ("manual", false) => RULE_MANUAL_EN,
        ("soft", true) => RULE_SOFT_UK,
        ("soft", false) => RULE_SOFT_EN,
        (_, true) => RULE_STRICT_UK,
        (_, false) => RULE_STRICT_EN,
    }
}

pub fn block(config: &Config) -> String {
    let uk = config.lang == "uk";
    let what = if uk { "блок" } else { "block" };
    let body = if uk { BODY_UK } else { BODY_EN }.replace("{what}", what);
    let rule = rule_for(config);
    format!("{BEGIN}\n{body}\n\n{rule}\n{END}")
}

/// The digest of an artefact: sha256 over its text, the first 12
/// hex -- the length of a trust fingerprint, because this too is a
/// judgement and not a document's revision. Byte-exact but for line
/// endings (review 0022 R-6, R-7).
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
    let mut report = String::new();
    let mut lacked = 0usize;
    for (path, kind, fresh) in artefacts(config) {
        // A project that answered "no hooks" gets none written -- a
        // question whose answer changes nothing is not a question
        // (wave 0026). But silence belongs only where we never were:
        // a hook config we DID write, whose digest still stands in
        // [generated], keeps its row, because dropping it would take
        // the file out from under the court that guarded it while
        // leaving [generated] calling it ours (review 0026 R-5).
        if !config.hooks && matches!(kind, Kind::Guest { .. }) {
            let recorded = config.generated.iter().any(|(key, _)| key == path);
            if !recorded {
                continue;
            }
            if !report.is_empty() {
                report.push('\n');
                report.push_str("  ");
            }
            report.push_str(&ta(
                "generated-hooks-off",
                targs!("file" => path.to_string()),
            ));
            continue;
        }
        // One artefact's failure stops none of the others: each has
        // its own row and answers for itself (wave 0023).
        let (word, lack) = one(root, config, path, &kind, &fresh);
        if !report.is_empty() {
            report.push('\n');
            report.push_str("  ");
        }
        report.push_str(&word);
        lacked += lack;
    }
    (report, lacked)
}

/// One artefact of the table: eight outcomes, each with its word.
fn one(root: &Path, config: &Config, name: &str, kind: &Kind, fresh: &str) -> (String, usize) {
    let path = root.join(name);
    let recorded = config
        .generated
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.clone());

    let text = match std::fs::read_to_string(&path) {
        Ok(text) => Some(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
        Err(e) => {
            return (
                ta(
                    "generated-unread",
                    targs!("file" => name.to_string(), "error" => e.to_string()),
                ),
                1,
            );
        }
    };

    let (whole, mine, word) = match (&text, kind) {
        // Nothing on disk: born, unless a person deleted it while
        // its digest stood -- that is a decision, not a gap.
        (None, _) => {
            if recorded.is_some() {
                return (
                    ta("generated-removed", targs!("file" => name.to_string())),
                    0,
                );
            }
            let whole = match kind {
                Kind::Block => format!("{fresh}\n"),
                Kind::Whole | Kind::Guest { .. } => fresh.to_string(),
            };
            let mine = fresh.to_string();
            (
                whole,
                mine,
                ta("generated-born", targs!("file" => name.to_string())),
            )
        }
        // A file wholly ours: the whole text is judged.
        // A guest walks the same road as a file wholly ours -- born,
        // already standing, refreshed -- and parts from it at one
        // fork only: what it says when the file is not ours (wave
        // 0025).
        (Some(text), Kind::Whole | Kind::Guest { .. }) => {
            // The file keeps its own line endings, and "ours by
            // self-evidence" is measured the way the record is --
            // by digest, not by bytes (review 0023 R-3).
            let fresh = if text.contains("\r\n") {
                fresh.replace('\n', "\r\n")
            } else {
                fresh.to_string()
            };
            let fresh = fresh.as_str();
            if digest(text) == digest(fresh) {
                if recorded.as_deref() != Some(digest(fresh).as_str())
                    && let Err(refusal) = record(root, name, &digest(fresh))
                {
                    return (
                        ta(
                            "generated-config-failed",
                            targs!("file" => name.to_string(), "error" => refusal.reason),
                        ),
                        1,
                    );
                }
                return (
                    ta("generated-stands", targs!("file" => name.to_string())),
                    0,
                );
            }
            if recorded.as_deref() != Some(digest(text).as_str()) {
                // Two states, two words (review 0024 R-9, the school
                // of 0022 R-2: advice must work). Nothing recorded
                // means the file is not ours at all -- and there is
                // no line in [generated] to remove, so the word does
                // not send anybody looking for one. With .agents/
                // skills/ being a shared namespace, a stranger's file
                // on our path is a normal state now.
                // A guest never advises deleting the file it is a
                // guest in: those are someone's settings (wave
                // 0025). The word carries the snippet instead, and
                // the snippet is the very text that would have been
                // written -- advice that differs from the deed is a
                // lie.
                if let Kind::Guest { key, entries } = kind {
                    // Three states, three words (review 0025 R-3: one
                    // word for two states told a lie in one of them,
                    // and named no way back). An empty file is not a
                    // person's settings -- it is an empty file, and
                    // pasting entries into it would not make JSON.
                    let word = if text.trim().is_empty() {
                        ta(
                            "generated-guest-empty",
                            targs!("file" => name.to_string(), "snippet" => fresh.to_string()),
                        )
                    } else if recorded.is_none() {
                        ta(
                            "generated-guest-taken",
                            targs!("file" => name.to_string(), "key" => (*key).to_string(), "snippet" => entries.clone()),
                        )
                    } else {
                        ta(
                            "generated-guest-edited",
                            targs!("file" => name.to_string(), "key" => (*key).to_string(), "snippet" => entries.clone()),
                        )
                    };
                    return (word, 1);
                }
                let key = if recorded.is_none() {
                    "generated-foreign-file"
                } else {
                    "generated-changed-file"
                };
                return (
                    ta(
                        key,
                        targs!("file" => name.to_string(), "recorded" => recorded.unwrap_or_else(|| t("generated-none")), "actual" => digest(text)),
                    ),
                    1,
                );
            }
            (
                fresh.to_string(),
                fresh.to_string(),
                ta("generated-refreshed", targs!("file" => name.to_string())),
            )
        }
        // A document of the person's, holding our block.
        (Some(text), Kind::Block) => {
            if text.matches(BEGIN).count() > 1 || text.matches(END).count() > 1 {
                return (
                    ta("generated-many-blocks", targs!("file" => name.to_string())),
                    1,
                );
            }
            let crlf = text.contains("\r\n");
            let fresh = if crlf {
                fresh.replace('\n', "\r\n")
            } else {
                fresh.to_string()
            };
            match span(text) {
                None => {
                    if text.contains(BEGIN) || text.contains(END) {
                        return (
                            ta("generated-half-marked", targs!("file" => name.to_string())),
                            1,
                        );
                    }
                    if recorded.is_some() {
                        return (
                            ta("generated-removed", targs!("file" => name.to_string())),
                            0,
                        );
                    }
                    let mut whole = text.clone();
                    // A document with nothing in it gains no blank
                    // lines before the block (review 0023 R-11).
                    if !whole.trim().is_empty() {
                        if !whole.ends_with('\n') {
                            whole.push('\n');
                        }
                        whole.push('\n');
                    } else {
                        whole.clear();
                    }
                    whole.push_str(&fresh);
                    whole.push('\n');
                    (
                        whole,
                        fresh,
                        ta("generated-appended", targs!("file" => name.to_string())),
                    )
                }
                Some((from, to)) => {
                    let standing = &text[from..to];
                    if standing == fresh {
                        if recorded.as_deref() != Some(digest(&fresh).as_str())
                            && let Err(refusal) = record(root, name, &digest(&fresh))
                        {
                            return (
                                ta(
                                    "generated-config-failed",
                                    targs!("file" => name.to_string(), "error" => refusal.reason),
                                ),
                                1,
                            );
                        }
                        return (
                            ta("generated-stands", targs!("file" => name.to_string())),
                            0,
                        );
                    }
                    if recorded.as_deref() != Some(digest(standing).as_str()) {
                        return (
                            ta(
                                "generated-changed",
                                targs!("file" => name.to_string(), "recorded" => recorded.unwrap_or_else(|| t("generated-none")), "actual" => digest(standing)),
                            ),
                            1,
                        );
                    }
                    let mut whole = String::with_capacity(text.len());
                    whole.push_str(&text[..from]);
                    whole.push_str(&fresh);
                    whole.push_str(&text[to..]);
                    (
                        whole,
                        fresh,
                        ta("generated-refreshed", targs!("file" => name.to_string())),
                    )
                }
            }
        }
    };

    // The file first, its record after it (review 0022 R-1): a
    // digest recorded for a file that was never written is the one
    // state nothing can heal.
    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        return (
            ta(
                "generated-write-failed",
                targs!("file" => name.to_string(), "error" => e.to_string()),
            ),
            1,
        );
    }
    if let Err(e) = std::fs::write(&path, &whole) {
        return (
            ta(
                "generated-write-failed",
                targs!("file" => name.to_string(), "error" => e.to_string()),
            ),
            1,
        );
    }
    if let Err(refusal) = record(root, name, &digest(&mine)) {
        return (
            ta(
                "generated-config-failed",
                targs!("file" => name.to_string(), "error" => refusal.reason),
            ),
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
fn record(root: &Path, name: &str, digest: &str) -> Result<(), Refusal> {
    let path = root.join("keel.toml");
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let written = crate::confedit::upsert(
        &text,
        "generated",
        &[(name.to_string(), digest.to_string())],
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
