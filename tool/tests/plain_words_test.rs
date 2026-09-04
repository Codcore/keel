//! Scenario test of wave 0033: the tool speaks plainly.

mod common;

use common::keel_sandbox;
use std::process::Command;

/// Words this project invented on the way and never explained. They
/// mean nothing to anyone who has not read the commit history, and
/// §1.7 asks for the opposite: a word of the methodology's own is
/// explained in plain words where it is first used.
/// Written as the PHRASES they appear in: "поверх" on its own is an
/// ordinary preposition ("keel не пише поверх того, чого не писав"),
/// and a check that forbids a common word would be a check nobody
/// can keep.
const INVENTED: [&str; 9] = [
    "межа вироку",
    "межі вироку",
    "меж вироку",
    "цим поверхом",
    // Every case of the rung, not only the nominative with a space:
    // review 0033 R-7 found "щаблем" alive in the Ukrainian text --
    // the source of truth -- while the English side had already
    // dropped it.
    "щабель",
    "щаблем",
    "щаблі",
    "limit of this verdict",
    "by this floor",
];

/// proves: the-tool-speaks-plainly@6caef1 -- the operator read
/// the output of `keel check` and could not tell what "межа вироку"
/// was. It was mine, coined in wave 0031, and explained nowhere.
#[test]
fn the_tool_speaks_plainly() {
    let dir = keel_sandbox("plain");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  a-promise:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - a-promise\n    files:\n      - src/lib.rs\n---\n\n## scenario: a-promise\nтекст\n\n## transform: work\nтекст\n",
    )
    .unwrap();
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();

    // EVERY road, and the list is taken from the code rather than
    // written here: review 0033 R-2 measured the first cut of this
    // probe walking three commands of nineteen, in one tongue, on a
    // fixture where the changed lines never printed -- so putting
    // the invented words back into `keel close` or the English side
    // left the battery green.
    let main_rs = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/main.rs"),
    )
    .unwrap();
    let mut commands: Vec<String> = Vec::new();
    for (at, _) in main_rs.match_indices("Some(\"") {
        let word: String = main_rs[at + 6..]
            .chars()
            .take_while(|c| c.is_ascii_lowercase())
            .collect();
        // Three commands serve a carried document WHOLE -- the
        // methodology, the checklist, the concept. What they print
        // is a citation, not the tool speaking, exactly like the
        // diff below. The tool's own voice lives in the i18n lines,
        // and those are judged at their source further down.
        if word.is_empty()
            || commands.contains(&word)
            || matches!(word.as_str(), "method" | "cuts" | "concept")
        {
            continue;
        }
        commands.push(word);
    }
    assert!(
        commands.len() >= 15,
        "the list of commands comes from the code, and there are many: {commands:?}"
    );

    for tongue in ["uk", "en"] {
        std::fs::write(
            dir.join("keel.toml"),
            format!("lang = \"{tongue}\"\nadapter = \"rust\"\n"),
        )
        .unwrap();
        for command in &commands {
            let out = Command::new(env!("CARGO_BIN_EXE_keel"))
                .args([command.as_str(), dir.to_str().unwrap()])
                .output()
                .unwrap();
            let mut said = format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            // `keel review` ends with the branch's whole diff, and a
            // diff of THIS wave quotes the invented words dozens of
            // times. A citation is not the tool speaking, so the
            // diff is cut off -- and everything the package says in
            // its own voice is still judged.
            if let Some(at) = said
                .find("## Повний diff")
                .or_else(|| said.find("## Full diff"))
            {
                said.truncate(at);
            }
            for word in INVENTED {
                assert!(
                    !said.contains(word),
                    "`keel {command}` in {tongue} says \"{word}\" -- a word \
                     invented here and explained nowhere (§1.7):\n{said}"
                );
            }
        }
    }

    // And the WORDS THEMSELVES, not only those a fixture manages to
    // reach: review 0033 R-2 put an invented phrase into
    // `close-title` and the battery stayed green, because this
    // fixture has no crate and `keel close` refuses before it ever
    // prints its own title. Every line a person can be shown is
    // judged at its source.
    let i18n = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("i18n");
    for file in ["uk.ftl", "en.ftl"] {
        let text = std::fs::read_to_string(i18n.join(file)).unwrap();
        for (number, line) in text.lines().enumerate() {
            // Comments explain the lines; they are not shown to
            // anyone, and forbidding the word there would forbid
            // saying why it is forbidden.
            if line.trim_start().starts_with('#') {
                continue;
            }
            for word in INVENTED {
                assert!(
                    !line.contains(word),
                    "{file}:{} says \"{word}\" -- a word invented here and \
                     explained nowhere (§1.7):\n{line}",
                    number + 1
                );
            }
        }
    }

    // What REPLACED them -- the plain "не перевірено" -- is judged
    // where such a line actually appears: verdict_limits_test, which
    // builds a clone that has something to not check. This probe
    // holds the other half: no road a person walks says a word this
    // project invented and never explained.
}
