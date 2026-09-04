//! Scenario test of wave 0035: the tool answers when asked for help.

mod common;

use common::keel_sandbox;
use std::process::Command;

fn keel(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

/// proves: the-tool-answers-when-asked-for-help@e04b56 -- the
/// bug audit measured the first thing a person types giving a
/// refusal about a directory called "--help", extra arguments
/// swallowed by seventeen commands of twenty, and an unreplaced
/// { $snippet } sitting in the output of both tongues.
#[test]
fn the_tool_answers_when_asked_for_help() {
    // The first thing a person types.
    for asked in ["--help", "-h", "help"] {
        let (said, code) = keel(&[asked]);
        assert_eq!(
            code, 0,
            "`keel {asked}` answers rather than refusing:\n{said}"
        );
        assert!(
            said.contains("keel check") && said.contains("keel plan"),
            "and what it answers is the list of commands:\n{said}"
        );
        assert!(
            !said.contains(asked),
            "not a complaint about a directory named `{asked}`:\n{said}"
        );
    }

    // An unknown flag is refused with the list, not read as a path.
    let (said, code) = keel(&["check", "--wat"]);
    assert_eq!(code, 2, "an unknown flag is refused:\n{said}");
    assert!(
        said.contains("keel check"),
        "and the refusal shows what the tool does know:\n{said}"
    );

    // A second path is a typo, not a choice -- in every command, not
    // in three of twenty.
    let dir = keel_sandbox("help");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let there = dir.to_str().unwrap();
    for command in ["check", "rev", "map", "status", "trust", "version"] {
        let (said, code) = keel(&[command, there, "junk"]);
        assert_eq!(
            code, 2,
            "`keel {command}` refuses a second path instead of swallowing it:\n{said}"
        );
    }
    // Including the commands that take a word of their own first.
    // Review 0035 R-6: these three were left out of the table and
    // went on swallowing the typo.
    for line in [
        vec!["gate", "MSG", there, "junk"],
        vec!["plan", "a-slug", there, "junk"],
        vec!["new", "contract", "a-slug", there, "junk"],
    ] {
        let (said, code) = keel(&line);
        assert_eq!(
            code,
            2,
            "`keel {}` refuses the extra word:\n{said}",
            line.join(" ")
        );
        assert!(
            said.contains("keel gate"),
            "and shows what the tool does know:\n{said}"
        );
    }

    // The help meets a person in the project's language. Review 0035
    // R-8: it was printed before any language was chosen, so the
    // Ukrainian one -- the first thing a person types -- was text
    // nobody could reach.
    for (tongue, word) in [
        ("uk", "інструмент методики"),
        ("en", "the methodology's tool"),
    ] {
        std::fs::write(dir.join("keel.toml"), format!("lang = \"{tongue}\"\n")).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(["--help"])
            .current_dir(&dir)
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(
            said.contains(word),
            "`keel --help` inside a {tongue} project answers in {tongue}:\n{said}"
        );
        // And so does the usage line beside it.
        let (usage, _) = keel(&["check", there, "junk"]);
        assert!(
            usage.contains("keel check"),
            "the usage line stands in {tongue} too:\n{usage}"
        );
    }
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();

    // A flag is read wherever it stands, or it is not let through at
    // all. Review 0035 R-7: the guard admitted `--write` at any
    // position and `rev` still looked for it only at the first, so
    // `keel rev <dir> --write` printed a reading report and wrote
    // nothing -- silent swallowing, added by the wave that came to
    // end it.
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    let (first, _) = keel(&["rev", "--write", there]);
    let (last, _) = keel(&["rev", there, "--write"]);
    assert_eq!(
        first, last,
        "`--write` before the directory and after it are the same \
         command:\nbefore:\n{first}\nafter:\n{last}"
    );
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();

    // The two words every command-line tool answers to (review 0035
    // R-15).
    let (said, code) = keel(&["--version", there]);
    assert_eq!(code, 0, "`keel --version` answers:\n{said}");
    assert!(
        said.contains("keel 0."),
        "and what it answers is the version:\n{said}"
    );
    for line in [vec!["check", "--help"], vec!["check", there, "-h"]] {
        let (said, code) = keel(&line);
        assert_eq!(
            code,
            0,
            "`keel {}` is a question, not a typo:\n{said}",
            line.join(" ")
        );
        assert!(
            said.contains("keel check") && said.contains("keel plan"),
            "and it is answered with the list of commands:\n{said}"
        );
    }

    // And no line of any output leaves a placeholder unreplaced --
    // on EVERY road and in both tongues. Review 0035 R-1: the first
    // cut of this walked seven commands chosen by hand, and none of
    // them was `keel update`, which is exactly where the operator
    // saw a bare "{$snippet}" printed at them. A promise about "any
    // output" is kept by taking the list from the code.
    let commands = commands_of_the_binary();
    assert!(
        commands.len() >= 15,
        "the list of roads comes from the code, and there are many: {commands:?}"
    );
    for tongue in ["uk", "en"] {
        std::fs::write(dir.join("keel.toml"), format!("lang = \"{tongue}\"\n")).unwrap();
        for command in &commands {
            let (mut said, _) = keel(&[command.as_str(), dir.to_str().unwrap()]);
            // `keel review` ends with the branch's whole diff, and a
            // diff that touches the words file quotes placeholders by
            // the dozen. A citation is not the tool speaking.
            if let Some(at) = said
                .find("## Повний diff")
                .or_else(|| said.find("## Full diff"))
            {
                said.truncate(at);
            }
            assert!(
                !said.contains("{ $") && !said.contains("{$"),
                "`keel {command}` in {tongue} says its words, not the \
                 shape of them:\n{said}"
            );
        }
    }
}

/// The same promise judged where it can be judged whole: a walk can
/// only reach the lines a fixture happens to provoke, and the one
/// the operator saw -- a bare "{$snippet}" from `keel update` --
/// needed a project whose keel.toml records a file since deleted by
/// hand. So the words file is read at source and every call site with
/// it: a message that names an argument must be given that argument.
///
/// Its one margin, said aloud: four keys are named through a variable
/// rather than written out, and for those the probe can only ask that
/// some call in the same file supplies what the message names.
#[test]
fn every_word_is_given_what_it_names() {
    let crate_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    // What each message names. Both tongues together: a placeholder
    // used in one and not the other must still be supplied.
    let mut names: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
        std::collections::BTreeMap::new();
    for tongue in ["uk", "en"] {
        let text = std::fs::read_to_string(crate_dir.join(format!("i18n/{tongue}.ftl"))).unwrap();
        let mut key: Option<String> = None;
        for line in text.lines() {
            if line.trim().is_empty() || line.starts_with('#') || line.starts_with('-') {
                key = None;
                continue;
            }
            if let Some(head) = starts_message(line) {
                key = Some(head.clone());
                names.entry(head).or_default();
            }
            if let Some(k) = &key {
                let found = placeholders(line);
                names.entry(k.clone()).or_default().extend(found);
            }
        }
    }
    assert!(
        names.len() > 400,
        "the words file was read, and it is large: {} keys",
        names.len()
    );

    let mut sources: Vec<(String, String)> = Vec::new();
    let mut stack = vec![crate_dir.join("src")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                sources.push((
                    path.display().to_string(),
                    std::fs::read_to_string(&path).unwrap(),
                ));
            }
        }
    }
    assert!(
        sources.len() > 10,
        "the code was read: {} files",
        sources.len()
    );

    let mut spoken: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for (file, code) in &sources {
        for (at, _) in code.match_indices('"') {
            let Some(key) = quoted_key(code, at) else {
                continue;
            };
            let before = code[..at].trim_end();
            // `t("key")` -- nothing is supplied, so the message must
            // name nothing.
            if before.ends_with("t(") && !before.ends_with("ta(") {
                spoken.insert(key.clone());
                let wants = names.get(&key).cloned().unwrap_or_default();
                assert!(
                    wants.is_empty(),
                    "{file}: `t(\"{key}\")` supplies nothing, but the \
                     message names {wants:?} -- it would print the shape \
                     of the word instead of the word (review 0035 R-1)"
                );
            }
            // `ta("key", targs!(...))` -- everything named must be
            // supplied.
            if before.ends_with("ta(") {
                spoken.insert(key.clone());
                let given = given_args(&code[at..]);
                let wants = names.get(&key).cloned().unwrap_or_default();
                let missing: Vec<&String> = wants.iter().filter(|w| !given.contains(*w)).collect();
                assert!(
                    missing.is_empty(),
                    "{file}: `ta(\"{key}\", ...)` does not supply {missing:?}, \
                     which the message names; given {given:?} (review 0035 R-1)"
                );
            }
        }
    }
    assert!(
        spoken.len() > 300,
        "and most of the words are spoken from named call sites: {}",
        spoken.len()
    );

    // The margin: keys reached through a variable. Each must still be
    // named in the code, and the file that names it must supply what
    // the message names.
    for (key, wants) in &names {
        if wants.is_empty() || spoken.contains(key) {
            continue;
        }
        let quoted = format!("\"{key}\"");
        let home = sources.iter().find(|(_, code)| code.contains(&quoted));
        let Some((file, code)) = home else {
            panic!(
                "the words file carries \"{key}\", which names {wants:?} \
                 and which no line of the code ever says -- dead words, or \
                 a call site nobody can find"
            );
        };
        let covered = code.match_indices("targs!(").any(|(at, _)| {
            let given = given_args(&code[at..]);
            wants.iter().all(|w| given.contains(w))
        });
        assert!(
            covered,
            "{file} names \"{key}\" through a variable, and nothing in \
             that file supplies {wants:?}"
        );
    }
}

/// The key a `key = value` line opens, if it opens one.
fn starts_message(line: &str) -> Option<String> {
    let head = line.split_once(" =")?.0;
    if head.is_empty() || head.starts_with(char::is_whitespace) {
        return None;
    }
    let first = head.chars().next()?;
    (first.is_ascii_alphabetic()
        && head
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
    .then(|| head.to_string())
}

/// Every `$name` a line of the words file mentions.
fn placeholders(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    for (at, _) in line.match_indices('$') {
        let name: String = line[at + 1..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        if !name.is_empty() {
            out.push(name);
        }
    }
    out
}

/// A quoted lowercase key at this position, or nothing.
fn quoted_key(code: &str, at: usize) -> Option<String> {
    let rest = &code[at + 1..];
    let end = rest.find('"')?;
    let key = &rest[..end];
    let ok = !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    // The closing quote must be followed by the shape of a call.
    let after = rest[end + 1..].trim_start();
    (ok && (after.starts_with(')') || after.starts_with(','))).then(|| key.to_string())
}

/// The argument names a `targs!(...)` block hands over, read from the
/// first one after this point.
fn given_args(code: &str) -> std::collections::BTreeSet<String> {
    let mut out = std::collections::BTreeSet::new();
    let Some(start) = code.find("targs!(") else {
        return out;
    };
    let body = &code[start + "targs!(".len()..];
    let mut depth = 1usize;
    let mut end = body.len();
    for (at, ch) in body.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    end = at;
                    break;
                }
            }
            _ => {}
        }
    }
    let body = &body[..end];
    for (at, _) in body.match_indices("=>") {
        let before = body[..at].trim_end();
        if let Some(inner) = before.strip_suffix('"')
            && let Some(open) = inner.rfind('"')
        {
            out.insert(inner[open + 1..].to_string());
        }
    }
    out
}

/// Every road the binary knows, read out of its own dispatch -- so a
/// road added later is walked without anyone remembering to add it
/// here.
fn commands_of_the_binary() -> Vec<String> {
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
        if word.is_empty() || commands.contains(&word) {
            continue;
        }
        commands.push(word);
    }
    commands
}
