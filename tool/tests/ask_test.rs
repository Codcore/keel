//! Scenario test of wave 0026-init-asks: the wizard of `keel init`
//! and, above all, its silence -- a tool that asks where nobody is
//! listening hangs in CI, in a sandbox, in a pipe.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

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
    assert!(out.status.success(), "git {args:?}");
}

/// Every call of the tool in this scenario is BOUNDED IN TIME, and
/// that is the point: the wave's first promise is that the wizard
/// never asks where nobody listens, and the failure mode of breaking
/// it is not a red assert but a battery that hangs forever. Review
/// 0026 R-1 measured exactly that -- a mutant without the terminal
/// guard hung for the full 25 seconds on a machine that HAS a
/// controlling terminal, while this sandbox merely happened to lack
/// /dev/tty. A promise about hanging is judged with a clock.
fn keel_within(args: &[&str], limit: Duration) -> (String, i32) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        match child.try_wait().unwrap() {
            Some(_) => break,
            None => {
                if start.elapsed() > limit {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!(
                        "keel {args:?} did not finish within {limit:?} -- it is waiting for an \
                         answer nobody is there to give"
                    );
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        }
    }
    let out = child.wait_with_output().unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

/// The same, with the bound this scenario always wants: twenty
/// seconds is a hundredfold what any of these runs takes.
fn keel(args: &[&str]) -> (String, i32) {
    keel_within(args, Duration::from_secs(20))
}

/// A bare project directory: git, and nothing else. No keel.toml --
/// this wave is about the birth of that file.
fn bare(name: &str) -> Sandbox {
    let dir = sandbox(name);
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// The shape of keel.toml, as a foreign parser sees it.
#[derive(serde::Deserialize)]
struct Written {
    lang: Option<String>,
    adapter: Option<String>,
    mode: Option<String>,
    agents: Option<Vec<String>>,
    hooks: Option<bool>,
}

/// The config the wizard wrote, read back by a REAL TOML parser and
/// typed -- not matched as strings (the school of 0023 R-1).
fn config_of(dir: &Path) -> Written {
    let text = fs::read_to_string(dir.join("keel.toml")).expect("keel.toml stands");
    toml::from_str(&text)
        .unwrap_or_else(|e| panic!("the config the wizard wrote is TOML: {e}\n{text}"))
}

/// proves: init-asks-only-when-it-can-hear@5bf1f3 -- the operator's
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
    assert_eq!(config.lang.as_deref(), Some("uk"), "the answer is written");
    assert_eq!(config.adapter.as_deref(), Some("rust"));
    assert_eq!(config.mode.as_deref(), Some("soft"));
    assert_eq!(config.hooks, Some(false));
    assert_eq!(
        config.agents.as_deref(),
        Some(["claude".to_string(), "cursor".to_string()].as_slice()),
        "both, in the order named"
    );

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

    // The tool's own courts read what the wizard wrote, and they
    // read it in the language chosen: `keel check` is green on that
    // project and `keel status` speaks Ukrainian back (review 0026
    // R-6: the scenario claimed status also names the mode, and it
    // does not -- the clause now says what is true, and the probe
    // drives it instead of nobody driving it).
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"sandbox\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "");
    let (judged, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "the config the wizard wrote passes its own court:\n{judged}"
    );
    let (state, code) = keel(&["status", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "and the state can be read:\n{state}");
    assert!(
        state.contains("де ми"),
        "in the language chosen, not the default one:\n{state}"
    );

    // --no-ask is a road the contract names, so the probe drives it
    // (review 0026 R-7: it was the one counterfactual that stayed
    // green, for want of an assert).
    let dir = bare("silenced");
    let (out, code) = keel(&["init", "--no-ask", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the silenced frame lands:\n{out}");
    let text = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        !text
            .lines()
            .any(|line| line.trim_start().starts_with("lang")),
        "--no-ask writes the plain defaults, choosing nothing:\n{text}"
    );

    // The skip a terminal offers has the same spelling on the command
    // line (review 0026 R-11: "-" was refused there while being
    // offered here).
    let dir = bare("skipped");
    let (out, code) = keel(&[
        "init",
        "--adapter",
        "-",
        "--agents",
        "cursor",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 0,
        "a project of another tongue is not refused:\n{out}"
    );
    let config = config_of(&dir);
    assert_eq!(config.adapter, None, "the field is left unwritten");
    assert_eq!(config.agents, Some(vec!["cursor".to_string()]));

    // A setting given twice is a typo, not a choice (R-17), and two
    // paths are a typo too (R-12) -- both refused, neither guessed.
    let dir = bare("twice");
    let (out, code) = keel(&[
        "init",
        "--lang",
        "uk",
        "--lang",
        "en",
        dir.to_str().unwrap(),
    ]);
    assert_ne!(code, 0, "the same setting twice is refused:\n{out}");
    assert!(
        !dir.join("keel.toml").exists(),
        "and nothing is written for it:\n{out}"
    );
    let one = bare("path-one");
    let two = bare("path-two");
    let (out, code) = keel(&["init", one.to_str().unwrap(), two.to_str().unwrap()]);
    assert_ne!(code, 0, "two paths are refused, not guessed:\n{out}");
    assert!(
        !one.join("keel.toml").exists() && !two.join("keel.toml").exists(),
        "and neither of them is touched:\n{out}"
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
    let after = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        after.starts_with(mine),
        "the person's own lines stand first and unchanged:\n{after}"
    );
    assert!(
        after.contains("lang = \"en\"") && after.contains("mode = \"manual\""),
        "and the answers on the command line do NOT overwrite what stood:\n{after}"
    );
    assert!(
        !after.contains("lang = \"uk\"") && !after.contains("mode = \"soft\""),
        "not once, not appended, not anywhere:\n{after}"
    );
    // What DOES grow is the [generated] record -- the hand of wave
    // 0022, not the wizard's, and it only ever adds its own section.
    assert!(
        after == mine || after[mine.len()..].trim_start().starts_with("[generated]"),
        "and the only thing added is the generated record:\n{after}"
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
        vec![
            "lang", "adapter", "mode", "agents", "hooks", "version", "ci", "trust"
        ],
        "eight questions: the five the operator named, and the three \
         wave 0032 added -- version, ci and trust were listed as a \
         limit in the plan of wave 0026 and stood unlifted for six waves"
    );
    let of = |field: &str| {
        questions
            .iter()
            .find(|q| q.field == field)
            .unwrap_or_else(|| panic!("{field} is asked"))
    };
    // The vocabulary of a question is the release's own vocabulary,
    // in its own order -- one home, not a second list to drift.
    assert_eq!(of("lang").choices, keel::config::LANGUAGES.to_vec());
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
    // And the interactive road itself, played by a machine on a real
    // pty. The wave first claimed this could not be played without
    // one and left it to the dogfood; a pty is a few lines of any
    // standard library, I played it by hand, and so the claim was
    // too weak. Now the machine plays it.
    let dir = bare("terminal");
    let spoken = format!("{} init {}", env!("CARGO_BIN_EXE_keel"), dir.display());
    let mut session = rexpect::spawn(&spoken, Some(20_000)).expect("a pty of our own");
    for (asked, answer) in [
        ("human language", "\r"),
        ("language is the code", "\r"),
        ("commit court", "\r"),
        // The agents are a MultiSelect: space ticks, enter takes.
        ("agents", " \r"),
        ("session hooks", "\r"),
        // The three wave 0032 added. The ci command is free text
        // (its list is a suggestion), so it is typed, not picked.
        ("pin the keel version", "\r"),
        ("what command does CI run", "\r"),
        ("record trust", "\r"),
    ] {
        session
            .exp_string(asked)
            .unwrap_or_else(|e| panic!("the wizard asks about {asked:?}: {e}"));
        session.send(answer).expect("the answer is typed");
        session.flush().expect("and reaches the wizard");
    }
    session
        .exp_eof()
        .expect("and the wizard finishes, never hangs");
    let config = config_of(&dir);
    assert_eq!(
        config.lang.as_deref(),
        Some("en"),
        "the default taken by pressing enter is written as a choice"
    );
    assert_eq!(config.mode.as_deref(), Some("strict"));
    assert_eq!(
        config.agents,
        Some(vec!["claude".to_string()]),
        "the ticked agent, and only it"
    );
    assert_eq!(config.hooks, Some(true));
    assert!(
        dir.join(".claude/settings.json").is_file(),
        "and the answers are acted on, not just written"
    );

    assert!(
        of("adapter").skippable,
        "the code's language may be left unnamed -- a project of another tongue \
         is not refused, it simply waits for its own wave"
    );
}
