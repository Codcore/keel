//! Scenario test of wave 0026-init-asks: the wizard of `keel init`
//! and, above all, its silence -- a tool that asks where nobody is
//! listening hangs in CI, in a sandbox, in a pipe.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0026-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
    let out = command.arg("-C").arg(dir).args(args).output().unwrap();
    assert!(out.status.success(), "git {args:?}");
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

/// A bare project directory: git, and nothing else. No keel.toml --
/// this wave is about the birth of that file.
fn bare(name: &str) -> PathBuf {
    let dir = sandbox(name);
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// The config the wizard wrote, read back by a real TOML parser.
fn config_of(dir: &Path) -> toml::Value {
    let text = fs::read_to_string(dir.join("keel.toml")).expect("keel.toml stands");
    text.parse::<toml::Value>()
        .unwrap_or_else(|e| panic!("the config the wizard wrote is TOML: {e}\n{text}"))
}

/// proves: init-asks-only-when-it-can-hear@8ee7fb -- the operator's
/// §8.6 decision made mechanical, and its first clause is the silence:
/// no terminal, no question and no hang. The answers have a
/// question-free road, every one of them is judged by the vocabulary
/// of its own question, an existing config is never touched, and the
/// list of questions itself is judged as data -- because the drawing
/// of them cannot be played without a pty, and that is said aloud
/// rather than painted over.
#[test]
fn init_asks_only_when_it_can_hear() {
    // No terminal (which is how every test, every CI job and every
    // pipe runs keel): not one question, no hang, and the config is
    // born exactly as it was before this wave -- a commented
    // vocabulary, with nothing chosen quietly on the person's behalf.
    let dir = bare("silent");
    let (out, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands without a terminal:\n{out}");
    let text = fs::read_to_string(dir.join("keel.toml")).expect("keel.toml is born");
    for field in ["lang", "adapter", "mode"] {
        assert!(
            !text
                .lines()
                .any(|line| line.trim_start().starts_with(field)),
            "nothing is chosen for the person where nobody was asked: {field} stands uncommented\n{text}"
        );
    }
    assert!(
        text.contains("# lang") && text.contains("# mode"),
        "the vocabulary is there to read, as it always was:\n{text}"
    );

    // The question-free road: every answer given by a flag, and the
    // config born carrying them -- read back by a real TOML parser,
    // not by matching strings.
    let dir = bare("answered");
    let (out, code) = keel(&[
        "init",
        "--lang",
        "uk",
        "--adapter",
        "rust",
        "--mode",
        "soft",
        "--agents",
        "claude,cursor",
        "--no-hooks",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "the answered frame lands:\n{out}");
    let config = config_of(&dir);
    assert_eq!(config["lang"].as_str(), Some("uk"), "the answer is written");
    assert_eq!(config["adapter"].as_str(), Some("rust"));
    assert_eq!(config["mode"].as_str(), Some("soft"));
    assert_eq!(config["hooks"].as_bool(), Some(false));
    let agents: Vec<&str> = config["agents"]
        .as_array()
        .expect("agents is a list")
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(agents, vec!["claude", "cursor"], "both, in the order named");

    // And the tool READS what the wizard wrote: the skills of both
    // agents stand, the hook configs do not (that was the answer),
    // and a second run says so without refusing.
    assert!(
        dir.join(".claude/skills/keel/SKILL.md").is_file()
            && dir.join(".agents/skills/keel/SKILL.md").is_file(),
        "the agents answered for are served:\n{out}"
    );
    assert!(
        !dir.join(".claude/settings.json").exists() && !dir.join(".cursor/hooks.json").exists(),
        "and 'no hooks' means no hook configs at all -- a question whose \
         answer changes nothing is not a question:\n{out}"
    );
    let (again, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the config the wizard wrote reads clean:\n{again}");
    assert!(
        again.contains("keel.toml"),
        "and the frame names it as standing:\n{again}"
    );

    // An existing config is a fact (§7.9): not asked about, not read
    // for answers, not touched by a byte.
    let dir = bare("standing");
    let mine = "# my own config\nlang = \"en\"\nmode = \"manual\"\n";
    fs::write(dir.join("keel.toml"), mine).unwrap();
    let (out, code) = keel(&[
        "init",
        "--lang",
        "uk",
        "--mode",
        "soft",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "the frame lands around it:\n{out}");
    assert_eq!(
        fs::read_to_string(dir.join("keel.toml")).unwrap(),
        mine,
        "not one byte of a config that already stood"
    );

    // A value this release does not know is refused aloud, by the
    // vocabulary of its own question, before anything is written.
    for (flag, value, known) in [
        ("--mode", "bogus", "strict"),
        ("--lang", "klingon", "uk"),
        ("--agents", "clod", "cursor"),
        ("--adapter", "cobol", "rust"),
    ] {
        let dir = bare(&format!("unknown{}", flag.trim_start_matches('-')));
        let (out, code) = keel(&["init", flag, value, dir.to_str().unwrap()]);
        assert_ne!(code, 0, "{flag} {value} is refused:\n{out}");
        assert!(
            out.contains(known),
            "{flag}: and the word names what it does know ({known}):\n{out}"
        );
        assert!(
            !dir.join("keel.toml").exists(),
            "{flag}: nothing is written before the answers are judged:\n{out}"
        );
    }

    // The operator's law: several may be ticked, at least one must be.
    let dir = bare("nobody");
    let (out, code) = keel(&["init", "--agents", "", dir.to_str().unwrap()]);
    assert_ne!(code, 0, "an empty choice is not an answer:\n{out}");
    assert!(
        !dir.join("keel.toml").exists(),
        "and nothing is written for it:\n{out}"
    );

    // The list of questions, judged AS DATA -- the only way to judge
    // it without a pty, and the reason the list is data at all.
    let questions = keel::ask::questions();
    let names: Vec<&str> = questions.iter().map(|q| q.field).collect();
    assert_eq!(
        names,
        vec!["lang", "adapter", "mode", "agents", "hooks"],
        "five questions, exactly the ones the operator named"
    );
    let of = |field: &str| {
        questions
            .iter()
            .find(|q| q.field == field)
            .unwrap_or_else(|| panic!("{field} is asked"))
    };
    assert_eq!(of("lang").choices, vec!["uk", "en"]);
    assert_eq!(of("mode").choices, vec!["strict", "soft", "manual"]);
    assert_eq!(of("mode").default, Some("strict"));
    assert_eq!(of("agents").choices, vec!["claude", "cursor"]);
    assert!(
        of("agents").many && of("agents").at_least_one,
        "several may be ticked, at least one must be -- held by the question, \
         not by our discipline"
    );
    assert!(
        !of("lang").many && !of("mode").many,
        "and the single answers stay single"
    );
    assert!(
        of("adapter").skippable,
        "the code's language may be left unnamed -- a project of another tongue \
         is not refused, it simply waits for its own wave"
    );
}
