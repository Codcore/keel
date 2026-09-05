//! Scenario tests of waves 0022-generated-block (the block and its
//! boundary) and 0023-generated-many (the table of artefacts, and
//! files wholly ours): what is generated is refreshed, what a hand
//! has touched is refused, and what a person removed stays
//! removed.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
    let out = command.arg("-C").arg(dir).args(args).output().unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

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

fn project(name: &str) -> Sandbox {
    let dir = sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// The digest recorded for the block, if any.
fn recorded(dir: &Path) -> Option<String> {
    let text = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let mut in_section = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_section = trimmed == "[generated]";
            continue;
        }
        if in_section && trimmed.contains("AGENTS.md") {
            return trimmed
                .split('=')
                .nth(1)
                .map(|v| v.trim().trim_matches('"').to_string());
        }
    }
    None
}

/// proves: generated-block-never-trampled@0751ba -- holds the
/// concept's letter (Distribution: the generated integrations lie
/// in the repository because agents and CI read them) and the
/// boundary that makes it safe: the block lives between its
/// markers, nothing outside them ever changes, and a block a person
/// edited by hand is refused aloud instead of overwritten.
#[test]
fn generated_block_never_trampled() {
    // No AGENTS.md at all: born with the block, and the digest
    // lands in [generated].
    let dir = project("born");
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        text.contains("<!-- keel:begin -->") && text.contains("<!-- keel:end -->"),
        "the block is born between its markers:\n{text}"
    );
    assert!(
        recorded(&dir).is_some_and(|d| d.len() == 12),
        "the digest of the block stands in [generated]"
    );

    // A file of the person's own, with no block: the block is
    // appended and their text stays byte for byte.
    let dir = project("appended");
    let mine = "# My project\n\nRules I wrote myself.\n";
    write(&dir, "AGENTS.md", mine);
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        text.starts_with(mine),
        "the person's own text is untouched, to the byte:\n{text}"
    );
    assert!(
        text.contains("<!-- keel:begin -->"),
        "the block is appended after it:\n{text}"
    );

    // Run again with the block standing and its digest true: the
    // block is refreshed, nothing else moves.
    let before = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    let config_before = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "update is green over its own block:\n{out}");
    assert_eq!(
        fs::read_to_string(dir.join("AGENTS.md")).unwrap(),
        before,
        "the same release writes the same block, byte for byte"
    );
    assert_eq!(
        fs::read_to_string(dir.join("keel.toml")).unwrap(),
        config_before,
        "the config keeps its comments, order and other sections"
    );

    // A block the person edited: refused aloud, and NOTHING is
    // written -- neither the document nor the config.
    let edited = before.replace(
        "<!-- keel:end -->",
        "A line a person added inside the block.\n<!-- keel:end -->",
    );
    fs::write(dir.join("AGENTS.md"), &edited).unwrap();
    let config_before = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a hand-edited block makes update red:\n{out}");
    assert!(
        out.contains("AGENTS.md"),
        "the refusal names the file:\n{out}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("AGENTS.md")).unwrap(),
        edited,
        "the person's edit is not trampled by a single byte"
    );
    assert_eq!(
        fs::read_to_string(dir.join("keel.toml")).unwrap(),
        config_before,
        "and nothing is recorded for a block that was not written"
    );

    // Markers removed altogether -- a person saying "not this":
    // update writes nothing back, and the word says how to have the
    // block again, because the refusal above pointed here (R-2).
    let dir = project("removed");
    keel(&["init", dir.to_str().unwrap()]);
    write(&dir, "AGENTS.md", "# Mine alone\n");
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "no block is not an error:\n{out}");
    assert_eq!(
        fs::read_to_string(dir.join("AGENTS.md")).unwrap(),
        "# Mine alone\n",
        "a removed block is a decision, not a gap to fill"
    );
    assert!(
        out.contains("[generated]"),
        "the word says the second step, so the advice of the refusal is true (R-2):\n{out}"
    );

    // Following that advice really works: remove the block AND its
    // line, and the block comes back (review 0022 R-2 -- the old
    // advice led nowhere).
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let without: String = config
        .lines()
        .filter(|l| !l.contains("AGENTS.md"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(dir.join("keel.toml"), format!("{without}\n")).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the advice of the refusal works:\n{out}");
    assert!(
        fs::read_to_string(dir.join("AGENTS.md"))
            .unwrap()
            .contains("<!-- keel:begin -->"),
        "the block returns when both steps are taken (R-2):\n{out}"
    );

    // ---- the second birth, out of review 0022 ----

    // R-3: no keel.toml -- no project of ours. Nothing is invented.
    let dir = sandbox("notaproject");
    write(&dir, "main.py", "print('mine')\n");
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_ne!(
        code, 0,
        "update refuses where there is no keel project:\n{out}"
    );
    assert!(
        !dir.join("AGENTS.md").exists() && !dir.join("keel.toml").exists(),
        "not one file is invented for a stranger's directory (R-3):\n{out}"
    );

    // R-1: a block byte-identical to what this release writes is
    // ours by self-evidence -- even with no digest recorded at all,
    // which is exactly the state a failed write leaves behind.
    let dir = project("selfevident");
    let config = keel::config::read(&dir).unwrap();
    let fresh = keel::generated::block(&config);
    write(&dir, "AGENTS.md", &format!("# Mine\n\n{fresh}\n"));
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a block equal to this release's own is never called a hand's edit (R-1):\n{out}"
    );
    assert!(
        keel::generated::digest(&fresh) == recorded(&dir).unwrap_or_default(),
        "and its digest is recorded, so the state heals itself (R-1)"
    );

    // R-6: an edit of whitespace alone is an edit.
    let dir = project("whitespace");
    keel(&["init", dir.to_str().unwrap()]);
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    let spaced = text.replace("- `keel next`", "-   `keel next`");
    assert_ne!(spaced, text, "the probe really changed the spacing");
    fs::write(dir.join("AGENTS.md"), &spaced).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "spacing is text too -- the edit is refused (R-6):\n{out}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("AGENTS.md")).unwrap(),
        spaced,
        "and the spacing a person chose is not trampled (R-6)"
    );

    // R-7: a document written with CRLF keeps its line endings.
    let dir = project("crlf");
    keel(&["init", dir.to_str().unwrap()]);
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    let crlf = text.replace('\n', "\r\n");
    fs::write(dir.join("AGENTS.md"), &crlf).unwrap();
    // Record the digest of the block as it now stands, so this is a
    // refresh and not a refusal.
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    let after = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        after.matches("\r\n").count() > 5,
        "the document keeps its own line endings (R-7), code {code}:\n{out}"
    );

    // R-11: two blocks -- which one is ours is not guessed.
    let dir = project("twoblocks");
    keel(&["init", dir.to_str().unwrap()]);
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    fs::write(dir.join("AGENTS.md"), format!("{text}\n{text}")).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "a second block is a refusal, not a silent choice (R-11):\n{out}"
    );

    // R-5: a REAL refresh -- a block of another release, recorded --
    // and the rest of the config kept byte for byte.
    let dir = sandbox("refresh");
    let older = "<!-- keel:begin -->\nan older release wrote this\n<!-- keel:end -->";
    write(
        &dir,
        "AGENTS.md",
        &format!("# Mine\n\n{older}\n\nAfter the block.\n"),
    );
    write(
        &dir,
        "keel.toml",
        &format!(
            "# my comment\nlang = \"en\"\nadapter = \"rust\"\n\n[trust]\n\"echo hi\" = \"aaaaaaaaaaaa\"\n\n[generated]\n\"AGENTS.md\" = \"{}\"\n",
            keel::generated::digest(older)
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a recorded block of an older release refreshes:\n{out}"
    );
    let after = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        after.starts_with("# Mine\n") && after.ends_with("After the block.\n"),
        "everything outside the markers survives the refresh (R-5):\n{after}"
    );
    assert!(
        after.contains("keel next"),
        "and the block itself is this release's:\n{after}"
    );
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        config.contains("# my comment") && config.contains("[trust]") && config.contains("echo hi"),
        "the config keeps its comment, its order and its other sections (R-5):\n{config}"
    );
}

/// proves: every-artefact-kept@ae746d -- holds wave 0023: the
/// mechanism of 0022 made many. Three artefacts, two kinds of
/// boundary -- a block inside a person's document, and files that
/// are wholly ours -- each with its own row, its own line in
/// [generated] and its own fate: born, standing, refused by name
/// when a hand changed it, and never resurrected when a person
/// deleted it. A neighbour's files in the same directories are none
/// of our business.
#[test]
fn every_artefact_kept() {
    // All three are born, each with its own row and its own line in
    // [generated] (wave 0023).
    let dir = project("three");
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    for artefact in [
        "AGENTS.md",
        ".claude/skills/keel/SKILL.md",
        ".github/workflows/keel.yml",
    ] {
        assert!(
            dir.join(artefact).is_file(),
            "{artefact} is born by the frame:\n{out}"
        );
        let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
        assert!(
            config.contains(artefact),
            "{artefact} has its own line in [generated]:\n{config}"
        );
    }

    // A whole-file artefact edited by hand: refused by name, and not
    // one byte of it is rewritten -- while the others still stand.
    let skill = dir.join(".claude/skills/keel/SKILL.md");
    let mine = format!(
        "{}\n\nA line I added myself.\n",
        fs::read_to_string(&skill).unwrap()
    );
    fs::write(&skill, &mine).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a hand-edited artefact reddens the run:\n{out}");
    assert!(
        out.contains("SKILL.md"),
        "the refusal names the artefact it means:\n{out}"
    );
    assert_eq!(
        fs::read_to_string(&skill).unwrap(),
        mine,
        "the person's work is not trampled"
    );
    assert!(
        out.contains("AGENTS.md") && out.contains("keel.yml"),
        "the other artefacts still get their own words -- one failure stops nothing:\n{out}"
    );

    // A whole-file artefact deleted while its digest stands: a
    // decision, not a gap -- it is not resurrected in silence.
    let dir = project("deleted");
    keel(&["init", dir.to_str().unwrap()]);
    let flow = dir.join(".github/workflows/keel.yml");
    fs::remove_file(&flow).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "a deleted artefact is not an error:\n{out}");
    assert!(
        !flow.exists(),
        "what a person deleted stays deleted:\n{out}"
    );

    // Foreign files in the same directories are none of our
    // business.
    let dir = project("neighbours");
    write(&dir, ".github/workflows/mine.yml", "name: mine\n");
    write(&dir, ".claude/skills/mine/SKILL.md", "# mine\n");
    keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(
        fs::read_to_string(dir.join(".github/workflows/mine.yml")).unwrap(),
        "name: mine\n",
        "a neighbour's workflow is untouched"
    );
    assert_eq!(
        fs::read_to_string(dir.join(".claude/skills/mine/SKILL.md")).unwrap(),
        "# mine\n",
        "a neighbour's skill is untouched"
    );

    // ---- the second birth, out of review 0023 ----

    // R-1: the skill's front matter must be valid YAML. A plain
    // scalar carrying ": " opens a mapping and breaks every parser
    // that reads it -- and the file exists to be read by one.
    let dir = project("frontmatter");
    keel(&["init", dir.to_str().unwrap()]);
    let skill = fs::read_to_string(dir.join(".claude/skills/keel/SKILL.md")).unwrap();
    let description = skill
        .lines()
        .find(|l| l.starts_with("description:"))
        .expect("the skill carries a description")
        .to_string();
    let value = description.trim_start_matches("description:").trim();
    assert!(
        value.starts_with('"') && value.ends_with('"'),
        "the description is quoted, so a colon in it is text, not a mapping (R-1): {description}"
    );

    // R-2: the word about a WHOLE file speaks of a file, not a
    // block, and its advice names what can really be removed.
    let dir = project("wholeword");
    keel(&["init", dir.to_str().unwrap()]);
    fs::write(dir.join(".github/workflows/keel.yml"), "name: mine\n").unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a hand-edited whole file is refused:\n{out}");
    assert!(
        !out.contains("block") && !out.contains("блок"),
        "a whole file is never called a block (R-2):\n{out}"
    );

    // R-3: a whole file with CRLF keeps its line endings, and is
    // ours by self-evidence even with nothing recorded.
    let dir = project("crlfwhole");
    keel(&["init", dir.to_str().unwrap()]);
    let flow = dir.join(".github/workflows/keel.yml");
    let text = fs::read_to_string(&flow).unwrap();
    fs::write(&flow, text.replace('\n', "\r\n")).unwrap();
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let without: String = config
        .lines()
        .filter(|l| !l.contains("keel.yml"))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(dir.join("keel.toml"), format!("{without}\n")).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "a CRLF file of ours is ours (R-3):\n{out}");
    assert!(
        fs::read_to_string(&flow).unwrap().contains("\r\n"),
        "the file keeps its own line endings (R-3):\n{out}"
    );

    // R-4: the second run says every artefact already stands.
    let dir = project("stands");
    keel(&["init", dir.to_str().unwrap()]);
    let (again, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the second run is green:\n{again}");
    // Each artefact's own row says it stands -- counted per row,
    // because the frame speaks the same words about its own pieces.
    let stands = |report: &str| -> usize {
        report
            .lines()
            .filter(|line| {
                line.contains("already stands")
                    && ["AGENTS.md", "SKILL.md", "keel.yml"]
                        .iter()
                        .any(|name| line.contains(name))
            })
            .count()
    };
    assert_eq!(
        stands(&again),
        3,
        "all three already stand, each in its own row (R-4):\n{again}"
    );
    let (byinit, _) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(
        stands(&byinit),
        3,
        "init and update agree about what stands (R-4):\n{byinit}"
    );

    // R-5: the file is written before its digest is recorded. An
    // artefact whose path cannot be written records nothing --
    // while the others land.
    let dir = project("orderly");
    fs::create_dir_all(dir.join(".github/workflows/keel.yml")).unwrap();
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "an artefact that cannot be written reddens:\n{out}"
    );
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        !config.contains("keel.yml"),
        "nothing is recorded for a file that was never written (R-5):\n{config}"
    );
    assert!(
        config.contains("AGENTS.md") && config.contains("SKILL.md"),
        "and the artefacts that did land are recorded (R-5):\n{config}"
    );

    // R-11: an empty AGENTS.md gains the block without a pile of
    // blank lines before it.
    let dir = project("emptyfile");
    write(&dir, "AGENTS.md", "");
    keel(&["init", dir.to_str().unwrap()]);
    let text = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    assert!(
        text.starts_with("<!-- keel:begin -->"),
        "an empty document gains no blank lines before the block (R-11): {text:?}"
    );
}

/// A project whose config names its agents (or does not name them at
/// all, which is a case of its own).
fn agents_project(name: &str, agents: Option<&str>) -> Sandbox {
    agents_project_in(name, agents, "en")
}

/// The same, in a named language of the release. Both halves of a
/// bilingual release are judged: until review 0024 R-1 every sandbox
/// wrote lang = "en", so the Ukrainian templates -- the ones this
/// very repository generates -- were read by no parser at all.
fn agents_project_in(name: &str, agents: Option<&str>, lang: &str) -> Sandbox {
    let dir = sandbox(name);
    let mut text = format!("lang = \"{lang}\"\nadapter = \"rust\"\n");
    if let Some(list) = agents {
        text.push_str(&format!("agents = {list}\n"));
    }
    write(&dir, "keel.toml", &text);
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// The top-level keys of a YAML front matter block and their scalar
/// values, read by a REAL YAML parser -- the foreign judge of shape.
/// A block that is not YAML panics here with the parser's own words,
/// which is exactly what review 0023 R-1 punished us for not doing.
fn front_matter(text: &str) -> Vec<(String, String)> {
    let body = text
        .strip_prefix("---\n")
        .unwrap_or_else(|| panic!("front matter opens with ---:\n{text}"));
    let end = body
        .find("\n---")
        .unwrap_or_else(|| panic!("front matter closes with ---:\n{text}"));
    let yaml = &body[..=end];

    let mut keys: Vec<(String, String)> = Vec::new();
    let mut depth = 0usize;
    let mut pending: Option<String> = None;
    for item in saphyr_parser::Parser::new_from_str(yaml) {
        let (event, _) = item.unwrap_or_else(|e| panic!("front matter is not YAML: {e}\n{yaml}"));
        match event {
            saphyr_parser::Event::MappingStart(..) | saphyr_parser::Event::SequenceStart(..) => {
                if depth == 1
                    && let Some(key) = pending.take()
                {
                    keys.push((key, "<collection>".to_string()));
                }
                depth += 1;
            }
            saphyr_parser::Event::MappingEnd | saphyr_parser::Event::SequenceEnd => {
                depth = depth.saturating_sub(1);
            }
            // Collapsed into the arm's own guard: clippy on a newer
            // stable than the author's says so, and this file was
            // green here and red on the runner for exactly that
            // (wave 0044).
            saphyr_parser::Event::Scalar(value, ..) if depth == 1 => match pending.take() {
                None => pending = Some(value.to_string()),
                Some(key) => keys.push((key, value.to_string())),
            },
            _ => {}
        }
    }
    keys
}

/// Every `keel <word>` an artefact names, in the order found.
fn advice(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    for tail in text.split("keel ").skip(1) {
        let word: String = tail
            .chars()
            .take_while(|c| c.is_ascii_lowercase())
            .collect();
        if !word.is_empty() && !words.contains(&word) {
            words.push(word);
        }
    }
    words
}

/// The skill of Claude Code and the same skill in the vendor-neutral
/// home Cursor reads.
const CLAUDE_SKILL: &str = ".claude/skills/keel/SKILL.md";
const SHARED_SKILL: &str = ".agents/skills/keel/SKILL.md";
const WORKFLOW: &str = ".github/workflows/keel.yml";

/// Judge one generated skill by a foreign parser: real YAML, exactly
/// the two keys every one of the three tools reads, the name equal to
/// the directory that gives the command, and a description inside the
/// documented cap.
fn judge_skill(text: &str, where_from: &str) {
    // The directory that gives the command, taken from the path
    // itself -- not a second literal that happens to agree with it
    // (review 0024 R-11).
    let home = where_from
        .rsplit('/')
        .nth(1)
        .unwrap_or_else(|| panic!("{where_from}: a skill lives in a directory"));
    let head = front_matter(text);
    let names: Vec<&str> = head.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        names,
        vec!["name", "description"],
        "{where_from}: exactly the keys all three tools read, in order"
    );
    let value = |key: &str| {
        head.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
            .unwrap()
    };
    assert_eq!(
        value("name"),
        home,
        "{where_from}: the name is the directory that gives the command"
    );
    let description = value("description");
    assert!(
        !description.is_empty() && description.len() <= 1536,
        "{where_from}: a description inside the documented cap: {} chars",
        description.len()
    );
}

/// proves: every-agent-in-its-own-format@e98552 -- the operator's
/// §8.6 decision made mechanical: the generated integrations serve
/// more than one agent, each artefact in the place and shape its own
/// tool documents (judged by a foreign parser, never by our memory),
/// a project gets nothing belonging to an agent it did not name, and
/// no advice inside a generated text leads nowhere.
#[test]
fn every_agent_in_its_own_format() {
    // Claude alone: its own skill lands, and the vendor-neutral home
    // of an agent this project never named is never born.
    let dir = agents_project("only-claude", Some("[\"claude\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let claude = fs::read_to_string(dir.join(CLAUDE_SKILL))
        .unwrap_or_else(|e| panic!("the skill of the named agent stands: {e}\n{out}"));
    judge_skill(&claude, CLAUDE_SKILL);
    assert!(
        !dir.join(".agents").exists(),
        "nothing of an agent this project never named:\n{out}"
    );

    // Cursor alone: the same skill in the standard home, and no
    // .claude directory at all.
    let dir = agents_project("only-cursor", Some("[\"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let shared = fs::read_to_string(dir.join(SHARED_SKILL))
        .unwrap_or_else(|e| panic!("the skill of the named agent stands: {e}\n{out}"));
    judge_skill(&shared, SHARED_SKILL);
    assert!(
        !dir.join(".claude").exists(),
        "nothing of an agent this project never named:\n{out}"
    );
    // "The shared document and the CI file land always" -- said of
    // every configuration, so judged where it is not trivial: here
    // claude is not named at all (review 0024 R-4).
    assert!(
        dir.join("AGENTS.md").is_file() && dir.join(WORKFLOW).is_file(),
        "what every agent reads lands whoever was named:\n{out}"
    );

    // Both named: both homes, the shared document, and the two files
    // byte for byte the same -- one template, two homes.
    let dir = agents_project("both", Some("[\"claude\", \"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    assert!(
        dir.join("AGENTS.md").is_file(),
        "the shared document:\n{out}"
    );
    let claude = fs::read_to_string(dir.join(CLAUDE_SKILL)).unwrap();
    let shared = fs::read_to_string(dir.join(SHARED_SKILL)).unwrap();
    assert_eq!(claude, shared, "one template, two homes -- byte for byte");

    // No key at all: exactly what the project got before this wave.
    let dir = agents_project("silent", None);
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    assert!(
        dir.join(CLAUDE_SKILL).is_file() && !dir.join(".agents").exists(),
        "the default is the old behaviour, not a silent change:\n{out}"
    );

    // An empty list is not an answer: refused aloud, nothing written
    // -- not even the shared document -- and the exit code is red,
    // because a refusal with a green exit is a half-truth.
    let dir = agents_project("empty", Some("[]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_ne!(code, 0, "an empty choice is refused:\n{out}");
    assert!(
        !dir.join("AGENTS.md").exists(),
        "the refusal happens before the first write:\n{out}"
    );

    // A name this release does not know -- codex is postponed by the
    // operator's word -- is caught like any typo, and the word names
    // the ones it does know.
    let dir = agents_project("unknown", Some("[\"codex\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_ne!(code, 0, "an unknown agent is refused:\n{out}");
    assert!(
        out.contains("claude") && out.contains("cursor"),
        "the refusal names the agents it knows:\n{out}"
    );
    assert!(
        !dir.join("AGENTS.md").exists(),
        "nothing is written before the names are judged:\n{out}"
    );

    // A skill a person edited by hand is never trampled: the file is
    // wholly ours, so the digest judges all of it.
    let dir = agents_project("edited", Some("[\"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let path = dir.join(SHARED_SKILL);
    let mine = format!(
        "{}\nA line I wrote myself.\n",
        fs::read_to_string(&path).unwrap()
    );
    fs::write(&path, &mine).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_ne!(code, 0, "a hand-edited skill is refused aloud:\n{out}");
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        mine,
        "and not one byte of the person's file is touched"
    );

    // No advice that leads nowhere: every `keel <word>` named in any
    // generated text is a command the binary really takes.
    let dir = agents_project("advice", Some("[\"claude\", \"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let mut named: Vec<String> = Vec::new();
    for rel in ["AGENTS.md", CLAUDE_SKILL, SHARED_SKILL, WORKFLOW] {
        let text = fs::read_to_string(dir.join(rel)).unwrap();
        for word in advice(&text) {
            if !named.contains(&word) {
                named.push(word);
            }
        }
    }
    assert!(
        named.len() >= 3,
        "the artefacts do name the commands: {named:?}"
    );
    for word in &named {
        let (answer, _) = keel(&[word, "/keel-no-such-directory"]);
        assert!(
            !answer.contains("unknown command"),
            "the artefacts name {word:?}, and the binary does not know it:\n{answer}"
        );
    }

    // The other half of the release, judged by the same courts: the
    // Ukrainian templates are what this very repository generates,
    // and until review 0024 R-1 no parser had ever read them.
    let dir = agents_project_in("ukrainian", Some("[\"claude\", \"cursor\"]"), "uk");
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands in Ukrainian:\n{out}");
    let uk_claude = fs::read_to_string(dir.join(CLAUDE_SKILL)).unwrap();
    let uk_shared = fs::read_to_string(dir.join(SHARED_SKILL)).unwrap();
    judge_skill(&uk_claude, CLAUDE_SKILL);
    judge_skill(&uk_shared, SHARED_SKILL);
    assert_eq!(
        uk_claude, uk_shared,
        "one template, two homes -- in both languages"
    );
    let mut uk_named: Vec<String> = Vec::new();
    for rel in ["AGENTS.md", CLAUDE_SKILL, SHARED_SKILL, WORKFLOW] {
        let text = fs::read_to_string(dir.join(rel)).unwrap();
        for word in advice(&text) {
            if !uk_named.contains(&word) {
                uk_named.push(word);
            }
        }
    }
    assert!(
        uk_named.len() >= 3,
        "the Ukrainian artefacts name commands too: {uk_named:?}"
    );
    for word in &uk_named {
        let (answer, _) = keel(&[word, "/keel-no-such-directory"]);
        assert!(
            !answer.contains("unknown command"),
            "the Ukrainian artefacts name {word:?}, and the binary does not know it:\n{answer}"
        );
    }

    // An agent left the list, the person deleted its file, the agent
    // came back: the skill does not return, because a deleted file
    // with a standing record is a decision (0022 R-2). The wave says
    // so aloud now (review 0024 R-6) -- and the way out it names is
    // judged here, because advice that does not work is the defect
    // 0022 R-2 punished.
    let dir = agents_project("returning", Some("[\"claude\", \"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    fs::remove_file(dir.join(SHARED_SKILL)).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a deleted artefact is a decision, not a gap:\n{out}"
    );
    assert!(
        !dir.join(SHARED_SKILL).exists(),
        "nothing is written back over a person's decision:\n{out}"
    );
    // The named way out: remove its line in [generated] as well.
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let pruned: String = config
        .lines()
        .filter(|line| !line.contains(SHARED_SKILL))
        .map(|line| format!("{line}\n"))
        .collect();
    assert_ne!(pruned, config, "the record of the deleted artefact stood");
    fs::write(dir.join("keel.toml"), &pruned).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "and then the way out works:\n{out}");
    assert!(
        dir.join(SHARED_SKILL).is_file(),
        "the skill is born again by the two steps the word names:\n{out}"
    );
}

/// The hook configs of wave 0025, each in its own tool's home.
const CLAUDE_HOOKS: &str = ".claude/settings.json";
const CURSOR_HOOKS: &str = ".cursor/hooks.json";

/// Read a generated file and parse it with a REAL JSON parser: a file
/// that exists to be read by a parser must be read by one (review
/// 0023 R-1, now for JSON).
fn json_of(dir: &Path, rel: &str) -> serde_json::Value {
    let text = fs::read_to_string(dir.join(rel)).unwrap_or_else(|e| panic!("{rel} stands: {e}"));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("{rel} is not JSON: {e}\n{text}"))
}

/// proves: hook-speaks-the-next-step@08213d -- the second half of the
/// operator's §8.6 decision: the tool speaks BEFORE the work, through
/// each agent's own session hook, in the answer shape that agent
/// documents; a file of someone else's settings is never written over
/// -- the word carries the snippet instead; and the command a hook
/// calls is one the binary really takes.
#[test]
fn hook_speaks_the_next_step() {
    // Claude alone: its own settings file is born, and it is JSON
    // with exactly the branch Claude Code documents.
    let dir = agents_project("hook-claude", Some("[\"claude\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let settings = json_of(&dir, CLAUDE_HOOKS);
    let group = &settings["hooks"]["SessionStart"][0];
    assert!(
        group["matcher"]
            .as_str()
            .is_some_and(|m| m.contains("startup")),
        "the matcher names the sources of a session start: {group}"
    );
    let handler = &group["hooks"][0];
    assert_eq!(
        handler["type"].as_str(),
        Some("command"),
        "the documented handler type: {handler}"
    );
    let command = handler["command"].as_str().unwrap_or_default();
    assert!(
        command.contains("keel next"),
        "the hook asks the tool for the one step: {handler}"
    );
    assert!(
        command.contains("\"${CLAUDE_PROJECT_DIR}\""),
        "and the project variable is QUOTED -- a space in a real path would \
         otherwise split the command, which is the school of v1: {handler}"
    );
    assert!(
        handler["timeout"].as_u64().is_some(),
        "and it carries a ceiling: {handler}"
    );
    assert!(
        !dir.join(CURSOR_HOOKS).exists(),
        "nothing of an agent this project never named:\n{out}"
    );

    // Cursor alone: its own hooks file, its own shape -- version 1
    // and a sessionStart entry whose only field is command.
    let dir = agents_project("hook-cursor", Some("[\"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let hooks = json_of(&dir, CURSOR_HOOKS);
    assert_eq!(hooks["version"].as_u64(), Some(1), "their version: {hooks}");
    let entry = &hooks["hooks"]["sessionStart"][0];
    let command = entry["command"].as_str().unwrap_or_default();
    assert!(
        command.contains("keel next") && command.contains("--for cursor"),
        "the hook asks for the step in Cursor's own answer shape: {entry}"
    );
    assert!(
        entry.as_object().is_some_and(|o| o.len() == 1),
        "a hook entry of theirs carries command and nothing invented: {entry}"
    );
    assert!(
        !dir.join(CLAUDE_HOOKS).exists(),
        "nothing of an agent this project never named:\n{out}"
    );

    // Every `keel <word>` the hook configs name is a command the
    // binary really takes (the law of advice, wave 0024).
    let dir = agents_project("hook-advice", Some("[\"claude\", \"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    for rel in [CLAUDE_HOOKS, CURSOR_HOOKS] {
        let text = fs::read_to_string(dir.join(rel)).unwrap();
        let named = advice(&text);
        assert!(!named.is_empty(), "{rel} names a command: {text}");
        for word in &named {
            let (answer, _) = keel(&[word, "/keel-no-such-directory"]);
            assert!(
                !answer.contains("unknown command"),
                "{rel} names {word:?}, and the binary does not know it:\n{answer}"
            );
        }
    }

    // The step in each tool's answer shape: Cursor takes context only
    // as JSON, and the field is additional_context. `next` reads the
    // project's state through its adapter, so the sandbox gets a
    // crate of its own -- a real project, the school of 0005-0024.
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"sandbox\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "");
    let (wrapped, code) = keel(&["next", "--for", "cursor", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the step is said for cursor:\n{wrapped}");
    let envelope: serde_json::Value = serde_json::from_str(wrapped.trim())
        .unwrap_or_else(|e| panic!("the answer for cursor is JSON: {e}\n{wrapped}"));
    let context = envelope["additional_context"].as_str().unwrap_or_default();
    assert!(
        !context.is_empty(),
        "and it carries the step where Cursor reads it: {envelope}"
    );
    let (plain, code) = keel(&["next", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "and plainly for claude:\n{plain}");
    let (for_claude, code) = keel(&["next", "--for", "claude", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "named or not, the same step:\n{for_claude}");
    assert_eq!(
        for_claude, plain,
        "Claude Code takes plain stdout, so the shape is the plain step"
    );
    assert_eq!(
        context, plain,
        "and both tongues say the SAME step, not a piece of it"
    );
    // A hook that speaks, speaks always. Right after `keel init`
    // there is no wave yet and the step refuses; a broken keel.toml
    // refuses even earlier, in the config court. In both states the
    // answer must still be VALID JSON for Cursor and the exit must
    // stay green -- an exit code of 2 means "block the action" there.
    for (name, config) in [
        ("hook-fresh", "lang = \"en\"\nadapter = \"rust\"\n"),
        ("hook-broken", "lang = \"en\"\nadapter = [broken\n"),
    ] {
        let bare = sandbox(name);
        write(&bare, "keel.toml", config);
        git(&bare, &["init", "-q", "-b", "main"]);
        let (said, code) = keel(&["next", "--for", "cursor", bare.to_str().unwrap()]);
        assert_eq!(code, 0, "{name}: a refusing hook does not block:\n{said}");
        let envelope: serde_json::Value = serde_json::from_str(said.trim())
            .unwrap_or_else(|e| panic!("{name}: the answer is still JSON: {e}\n{said}"));
        assert!(
            envelope["additional_context"]
                .as_str()
                .is_some_and(|c| c.contains("keel")),
            "{name}: and it carries the refusal as the word the agent needs: {envelope}"
        );
        // The plain step keeps its own behaviour, untouched.
        let (_, plain_code) = keel(&["next", bare.to_str().unwrap()]);
        assert!(
            plain_code == 0 || plain_code == 2,
            "{name}: plain next answers as it always did, not as a hook"
        );
    }

    let (refusal, code) = keel(&["next", "--for", "clod", dir.to_str().unwrap()]);
    assert_ne!(code, 0, "an unknown agent is refused:\n{refusal}");
    assert!(
        refusal.contains("claude") && refusal.contains("cursor"),
        "and the word names the agents it knows:\n{refusal}"
    );

    // A file of someone else's SETTINGS: not one byte is written,
    // and the word must carry advice that WORKS. Review 0025 R-1
    // measured the old advice taken literally: a whole document
    // handed to a person who already has a file gives invalid JSON
    // two ways out of three, and the third silently eats their own
    // hooks. So the word names the key and hands the entries, and
    // this is where that is judged -- by performing the advice.
    let dir = agents_project("hook-guest", Some("[\"cursor\"]"));
    let mine = "{\n  \"version\": 1,\n  \"hooks\": {\n    \"beforeShellExecution\": [\n      { \"command\": \"./guard.sh\" }\n    ],\n    \"sessionStart\": [\n      { \"command\": \"./mine.sh\" }\n    ]\n  }\n}\n";
    write(&dir, CURSOR_HOOKS, mine);
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_ne!(code, 0, "a stranger's hooks are not written over:\n{out}");
    assert_eq!(
        fs::read_to_string(dir.join(CURSOR_HOOKS)).unwrap(),
        mine,
        "and not one byte of them moves"
    );
    assert!(
        !out.contains("delete the file"),
        "the word does not advise deleting a person's own file:\n{out}"
    );
    assert!(
        out.contains("\"hooks\""),
        "the word names the KEY the entries belong under:\n{out}"
    );

    // The entries in the word, taken out of it and merged under the
    // key it names -- the advice, performed.
    let snippet = {
        let first = out.find('{').expect("the word carries entries");
        let last = out.rfind('}').expect("the word carries entries");
        &out[first..=last]
    };
    let entries: serde_json::Value = serde_json::from_str(snippet)
        .unwrap_or_else(|e| panic!("the entries are JSON of their own: {e}\n{snippet}"));
    let mut theirs: serde_json::Value = serde_json::from_str(mine).unwrap();
    for (event, ours) in entries.as_object().expect("entries are an object") {
        let home = theirs["hooks"].as_object_mut().expect("their hooks object");
        match home.get_mut(event) {
            Some(standing) => standing
                .as_array_mut()
                .expect("their event is an array")
                .extend(ours.as_array().expect("ours is an array").iter().cloned()),
            None => {
                home.insert(event.clone(), ours.clone());
            }
        }
    }
    let merged = serde_json::to_string_pretty(&theirs).unwrap();
    let judged: serde_json::Value = serde_json::from_str(&merged)
        .unwrap_or_else(|e| panic!("the advice performed gives JSON: {e}\n{merged}"));
    assert_eq!(
        judged["version"].as_u64(),
        Some(1),
        "their own version survives the advice: {merged}"
    );
    assert!(
        merged.contains("./guard.sh"),
        "their own guard survives the advice -- this is the harm R-1 measured: {merged}"
    );
    assert!(
        merged.contains("./mine.sh"),
        "and their own session hook survives it too: {merged}"
    );
    assert!(
        merged.contains("keel next --for cursor"),
        "and ours is there beside theirs: {merged}"
    );

    // The entries are, byte for byte, a part of what would have been
    // written -- not a reworded cousin of it (review 0025 R-11).
    let twin = agents_project("hook-twin", Some("[\"cursor\"]"));
    let (twin_out, twin_code) = keel(&["init", twin.to_str().unwrap()]);
    assert_eq!(twin_code, 0, "the twin lands:\n{twin_out}");
    let born = fs::read_to_string(twin.join(CURSOR_HOOKS)).unwrap();
    assert!(
        born.contains(snippet),
        "the entries in the word stand byte for byte inside the born file:\nword: {snippet}\nborn: {born}"
    );

    // Our own hook config, edited by hand: refused, untouched.
    let dir = agents_project("hook-edited", Some("[\"cursor\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    let path = dir.join(CURSOR_HOOKS);
    let edited = fs::read_to_string(&path)
        .unwrap()
        .replace("sessionStart", "stop");
    fs::write(&path, &edited).unwrap();
    let (out, code) = keel(&["update", dir.to_str().unwrap()]);
    assert_ne!(
        code, 0,
        "a hand-edited hook config is refused aloud:\n{out}"
    );
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        edited,
        "and not one byte of the person's edit is touched"
    );

    // The tool's own validator, where it is at hand: a soft judge,
    // named as one (the operator's word: use their testing library).
    let dir = agents_project("hook-validate", Some("[\"claude\"]"));
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands:\n{out}");
    match Command::new("claude")
        .args(["plugin", "validate", ".claude/skills"])
        .current_dir(&dir)
        .output()
    {
        Ok(judged) => {
            let said = format!(
                "{}{}",
                String::from_utf8_lossy(&judged.stdout),
                String::from_utf8_lossy(&judged.stderr)
            );
            assert!(
                judged.status.success(),
                "the tool's own validator judges the generated skill:\n{said}"
            );
        }
        Err(e) => {
            // No binary at hand: said aloud, never painted green.
            eprintln!(
                "claude plugin validate not run ({e}) -- a soft judge, and this is its honest silence"
            );
        }
    }
}
