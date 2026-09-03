//! Scenario tests of waves 0022-generated-block (the block and its
//! boundary) and 0023-generated-many (the table of artefacts, and
//! files wholly ours): what is generated is refreshed, what a hand
//! has touched is refused, and what a person removed stays
//! removed.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0022b-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

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

fn project(name: &str) -> PathBuf {
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
