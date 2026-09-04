//! Scenario tests of wave 0032: the mouth serves every text it
//! carries, and every text says when it was taken.

mod common;

use common::keel_sandbox;
use std::process::Command;

fn keel(dir: &std::path::Path, args: &[&str]) -> String {
    let mut all: Vec<&str> = args.to_vec();
    let root = dir.to_str().unwrap();
    all.push(root);
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

/// Every text this binary carries, and the word that asks for it.
/// A text added to speak.rs without a row here fails the probe: the
/// rule is held by machinery, not by memory (§9.6).
const CARRIED: [(&str, &str); 5] = [
    ("QUALITY.md", "cuts"),
    ("docs/uk/QUALITY.md", "cuts"),
    ("docs/uk/METHODOLOGY-V2.md", "method"),
    ("docs/en/METHODOLOGY-V2.md", "method"),
    ("docs/uk/NEW-CONCEPT.md", "concept"),
];

/// proves: the-mouth-serves-every-text-it-carries@856de7 -- named as
/// a limit in wave 0027 and never lifted: NEW-CONCEPT.md was a text
/// the project leaned on and the mouth would not give.
#[test]
fn the_mouth_serves_every_text_it_carries() {
    let source = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/speak.rs"),
    )
    .unwrap();

    // Every text the source carries must have a word that asks for
    // it -- otherwise the binary holds something nobody can read.
    let mut carried: Vec<String> = Vec::new();
    for line in source.lines() {
        let Some((_, rest)) = line.split_once("include_str!(\"") else {
            continue;
        };
        let Some((path, _)) = rest.split_once('"') else {
            continue;
        };
        carried.push(
            path.trim_start_matches("../")
                .trim_start_matches("../")
                .to_string(),
        );
    }
    assert!(!carried.is_empty(), "the source carries texts");

    let unserved: Vec<&String> = carried
        .iter()
        .filter(|path| !CARRIED.iter().any(|(known, _)| known == path))
        .collect();
    assert!(
        unserved.is_empty(),
        "every text this binary carries has a word that asks for it; \
         these have none: {unserved:?}"
    );

    // And the concept, which is the one this wave adds.
    let dir = keel_sandbox("carried");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let said = keel(&dir, &["concept"]);
    assert!(
        said.contains("NEW-CONCEPT") || said.contains("нове поняття"),
        "keel concept serves the text the project leans on:\n{}",
        &said[..said.len().min(400)]
    );
}

/// proves: the-texts-say-when-they-were-taken@6dbe29 -- named in
/// wave 0027 and said in the output of one command only: the texts
/// are taken by include_str! at BUILD time, so the mouth serves what
/// was on disk when the binary was made, not what is there now.
#[test]
fn the_texts_say_when_they_were_taken() {
    let dir = keel_sandbox("taken");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();

    for (_, word) in CARRIED {
        let said = keel(&dir, &[word]);
        assert!(
            said.contains("ЗБІРКИ") || said.contains("збірки"),
            "`keel {word}` says its text was taken when the binary was \
             built, not read from disk now:\n{}",
            &said[..said.len().min(300)]
        );
    }
    let said = keel(&dir, &["concept"]);
    assert!(
        said.contains("ЗБІРКИ") || said.contains("збірки"),
        "and so does the concept:\n{}",
        &said[..said.len().min(300)]
    );

    // And the roads a person actually types, not only the bare
    // command: review 0032 R-13 found the line riding the
    // argument-free road alone, so `keel method §1.8` -- the way
    // anybody reaches one rule -- said nothing about where its text
    // came from.
    for asked in ["§1.8", "1"] {
        let said = keel(&dir, &["method", asked]);
        assert!(
            said.contains("ЗБІРКИ") || said.contains("збірки"),
            "`keel method {asked}` says when its text was taken:\n{}",
            &said[..said.len().min(300)]
        );
    }
}
