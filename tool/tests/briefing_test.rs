//! Scenario test of wave 0032: the reviewer is briefed by the tool.

mod common;

use common::keel_sandbox;
use std::process::Command;

fn keel(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// proves: the-reviewer-is-briefed-by-the-tool@c8dbb1 -- the second
/// half of the promise, and the half nothing held: review 0032 R-3
/// cut the prohibitions out of the ENGLISH briefing and the battery
/// stayed 96/96 green, so an English-speaking reviewer would have
/// been handed a briefing with no prohibitions at all -- exactly the
/// 0026 situation this wave exists to prevent.
#[test]
fn both_tongues_carry_the_same_briefing() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let keys = |file: &str| -> Vec<String> {
        std::fs::read_to_string(root.join("i18n").join(file))
            .unwrap()
            .lines()
            // A key starts at column zero; an indented line carrying
            // an "=" is part of a multiline value, not a key of its
            // own.
            .filter(|line| !line.starts_with([' ', '\t', '#', '-']))
            .filter_map(|line| line.split_once(" ="))
            .map(|(key, _)| key.trim().to_string())
            .filter(|key| !key.is_empty())
            .collect()
    };
    let uk = keys("uk.ftl");
    let en = keys("en.ftl");

    // Not only the briefing: a key that exists in one tongue and not
    // the other prints its own NAME to whoever reads that tongue.
    let only_uk: Vec<&String> = uk.iter().filter(|key| !en.contains(key)).collect();
    let only_en: Vec<&String> = en.iter().filter(|key| !uk.contains(key)).collect();
    assert!(
        only_uk.is_empty() && only_en.is_empty(),
        "every line exists in both tongues; only in Ukrainian: {only_uk:?}; \
         only in English: {only_en:?}"
    );

    // And the English briefing really carries its prohibitions, in
    // the text and not merely as a key.
    let dir = keel_sandbox("briefed-en");
    std::fs::write(dir.join("keel.toml"), "lang = \"en\"\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  a-promise:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - a-promise\n    files:\n      - src/lib.rs\n---\n\n## scenario: a-promise\ntext\n\n## transform: work\ntext\n",
    )
    .unwrap();
    for args in [
        &["init", "-q", "-b", "0001-a-wave"][..],
        &["add", "-A"][..],
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            "wave",
        ][..],
    ] {
        Command::new("git")
            .args(args)
            .current_dir(&dir)
            .status()
            .unwrap();
    }
    let package = keel(&["review", dir.to_str().unwrap()]);
    assert!(
        package
            .to_lowercase()
            .contains("touch nothing that is not yours"),
        "the English briefing forbids touching what is not yours:\n{package}"
    );
    assert!(
        !package.contains("briefing-forbidden"),
        "and carries the text, not the name of a missing line:\n{package}"
    );
}

/// proves: the-reviewer-is-briefed-by-the-tool@c8dbb1 -- until this
/// wave the package carried the wave's own material and not one word
/// about what a reviewer should do with it. Every briefing was
/// written by hand in a chat, so every reviewer got a different one,
/// and reviewer 0026 destroyed 10,128 directories belonging to other
/// sessions because the prohibition was missing from his.
#[test]
fn the_reviewer_is_briefed_by_the_tool() {
    let dir = keel_sandbox("briefed");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  a-promise:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - a-promise\n    files:\n      - src/lib.rs\n---\n\n## scenario: a-promise\nтекст\n\n## transform: work\nтекст\n",
    )
    .unwrap();
    for args in [
        &["init", "-q", "-b", "0001-a-wave"][..],
        &["add", "-A"][..],
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            "wave",
        ][..],
    ] {
        Command::new("git")
            .args(args)
            .current_dir(&dir)
            .status()
            .unwrap();
    }

    let package = keel(&["review", dir.to_str().unwrap()]);
    // The briefing shouts its prohibitions, so the probe reads in one case.
    let quiet = package.to_lowercase();

    // The prohibitions come first, and the one that cost 10,128
    // directories is among them.
    assert!(
        quiet.contains("чужого не чіпай"),
        "the briefing forbids touching what is not yours:\n{package}"
    );
    assert!(
        package.contains("--no-local") && package.contains("CARGO_TARGET_DIR"),
        "and names the hygiene: an own clone, an own target:\n{package}"
    );
    assert!(
        package.contains("§9.9"),
        "and the four questions it is written against:\n{package}"
    );
    assert!(
        package.contains("keel/reviews/"),
        "and where the report goes:\n{package}"
    );
    assert!(
        quiet.contains("тільки з бігів"),
        "and that numbers come from runs, not from reading:\n{package}"
    );

    // And the briefing is JUDGED -- really judged, this time.
    // Review 0032 R-2: the first cut of this probe walked a
    // hardcoded list of four words and asked whether the WHOLE
    // package contained them, so a briefing naming `keel doctor`,
    // `keel audit` and §42.7 left the battery 96/96 green. It reads
    // the briefing itself now, takes what the briefing actually
    // names, and requires each of it to exist.
    let briefing = package
        .split_once("ДОРУЧЕННЯ")
        .map(|(_, tail)| tail.to_string())
        .expect("the package carries a briefing");
    let help = keel(&["help"]);

    let mut named: Vec<String> = Vec::new();
    for (at, _) in briefing.match_indices("keel ") {
        let word: String = briefing[at + 5..]
            .chars()
            .take_while(|c| c.is_ascii_lowercase())
            .collect();
        if !word.is_empty() && !named.contains(&word) {
            named.push(word);
        }
    }
    assert!(
        named.len() >= 2,
        "the briefing names commands at all: {named:?}"
    );
    let unknown: Vec<&String> = named
        .iter()
        .filter(|word| !help.contains(&format!("keel {word}")))
        .collect();
    assert!(
        unknown.is_empty(),
        "every command the briefing names exists in this binary; \
         these do not: {unknown:?}"
    );

    // The same for the paragraphs it cites: a briefing that sends a
    // reviewer to §42.7 sends them nowhere. Each is asked of the
    // mouth directly -- `keel method` with no argument serves the
    // CONTENTS, not the paragraphs, so searching it proves nothing.
    let mut cited: Vec<String> = Vec::new();
    for (at, _) in briefing.match_indices('§') {
        let number: String = briefing[at + '§'.len_utf8()..]
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        let number = number.trim_end_matches('.').to_string();
        if !number.is_empty() && !cited.contains(&number) {
            cited.push(number);
        }
    }
    assert!(!cited.is_empty(), "the briefing cites paragraphs at all");
    let missing: Vec<&String> = cited
        .iter()
        .filter(|number| {
            let said = keel(&["method", &format!("§{number}"), dir.to_str().unwrap()]);
            said.contains("відмова") || said.trim().is_empty()
        })
        .collect();
    assert!(
        missing.is_empty(),
        "every paragraph the briefing cites is in the methodology; \
         these are not: {missing:?}"
    );

    // And the prohibitions come FIRST -- the order the wave promises
    // and nothing held: swapping the two blocks left the battery
    // green (R-2).
    let forbidden = briefing
        .find("ЧУЖОГО НЕ ЧІПАЙ")
        .expect("the prohibition is there");
    let hygiene = briefing.find("ГІГІЄНА").expect("the hygiene is there");
    assert!(
        forbidden < hygiene,
        "the prohibitions stand before the hygiene, not after it"
    );
}
