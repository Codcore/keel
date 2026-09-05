//! Scenario test of wave 0039: the block names what really holds it.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

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

/// A project born by `keel init` with the two knobs set as asked.
fn born(name: &str, mode: &str, hooks: bool) -> Sandbox {
    let dir = sandbox(name);
    git(&dir, &["init", "-q", "-b", "main"]);
    let mut args = vec![
        "init".to_string(),
        dir.to_str().unwrap().to_string(),
        "--lang".to_string(),
        "en".to_string(),
        "--mode".to_string(),
        mode.to_string(),
        "--agents".to_string(),
        "claude".to_string(),
        "--no-ask".to_string(),
    ];
    args.push(if hooks { "--hooks" } else { "--no-hooks" }.to_string());
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    let (said, code) = keel(&borrowed);
    assert_eq!(code, 0, "the frame lands ({mode}, hooks={hooks}):\n{said}");
    dir
}

fn block(dir: &Path) -> String {
    let agents = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    let skill = fs::read_to_string(dir.join(".claude/skills/keel/SKILL.md")).unwrap();
    format!("{agents}\n{skill}")
}

/// The rule paragraph alone -- the last one the block carries, which
/// is the sentence about what holds the commit court.
///
/// Review 0039 R-2 of the wave's own review: assertions used to be
/// made against the WHOLE block, so "it names `keel close`" was
/// satisfied by the loop's general command list sixteen lines above,
/// and could never fail. What is claimed about the paragraph is
/// measured on the paragraph.
fn rule_paragraph(dir: &Path) -> String {
    let agents = fs::read_to_string(dir.join("AGENTS.md")).unwrap();
    let inside = agents
        .split("<!-- keel:end -->")
        .next()
        .expect("the block is fenced");
    inside
        .trim_end()
        .rsplit("\n\n")
        .next()
        .expect("the block has a last paragraph")
        .trim()
        .to_string()
}

/// proves: the-block-names-what-holds-it@0cdc16 -- `keel init --no-hooks
/// --mode strict` printed "the git hook is not installed" and two
/// lines later wrote into AGENTS.md that a machine holds the red
/// birth here through the commit-msg hook. There was no hook.
/// `rule_for()` asked `mode` and never `hooks`, so it knew one of
/// four truths -- and told it in the text an agent reads first.
#[test]
fn the_block_names_what_holds_it() {
    // The hook really stands: the machine may be named.
    let dir = born("strictwith", "strict", true);
    assert!(
        dir.join(".git/hooks/commit-msg").is_file(),
        "the hook is installed when the project asked for it"
    );
    let said = block(&dir);
    assert!(
        said.contains("commit-msg hook"),
        "and the rule names it:\n{said}"
    );

    // And on a machine where that hook is NOT there -- a colleague's
    // clone, a CI runner, git clones no hooks -- the check says so,
    // because the paragraph cannot: it is compared by digest across
    // every machine, so it states the project's answer and this row
    // states this machine (review 0039 R-1).
    std::fs::remove_file(dir.join(".git/hooks/commit-msg")).unwrap();
    let (said, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "a missing hook is not a finding:\n{said}");
    assert!(
        said.contains("no commit-msg hook of ours stands in this clone"),
        "the check names the machine the block cannot know:\n{said}"
    );
    assert!(
        said.contains("keel hook"),
        "and what to run about it:\n{said}"
    );

    // The hook does NOT stand. The paragraph must not promise it.
    // All THREE modes without hooks -- review 0039 R-11: the wave
    // said "four combinations", and mode has three values times two
    // knob positions, which is six. Two were never played.
    for (name, mode) in [
        ("strictno", "strict"),
        ("softno", "soft"),
        ("manualno", "manual"),
    ] {
        let dir = born(name, mode, false);
        assert!(
            !dir.join(".git/hooks/commit-msg").is_file(),
            "no hook was installed for {mode} + hooks = false"
        );
        let said = block(&dir);
        assert!(
            !said.contains("commit-msg hook"),
            "the rule does not name a hook that is not there ({mode}):\n{said}"
        );
        // Measured on the paragraph itself, not on the block around
        // it, and positively as well as negatively: a paragraph that
        // both promises a machine and denies one passed the negative
        // grep alone.
        let rule = rule_paragraph(&dir);
        assert!(
            rule.contains("no commit judgement runs here"),
            "it says plainly that nothing judges the commits ({mode}):\n{rule}"
        );
        assert!(
            rule.contains("held by people"),
            "and who holds the two rules instead ({mode}):\n{rule}"
        );
        for promise in [
            "hook that holds",
            "holds here",
            "a machine holds",
            "by machine, so",
        ] {
            assert!(
                !rule.contains(promise),
                "and promises no machine of any shape -- \"{promise}\" \
                 ({mode}):\n{rule}"
            );
        }
        assert!(
            rule.contains("`keel close`") && rule.contains("`keel check`"),
            "while naming, in this very paragraph, what still judges by \
             machine ({mode}):\n{rule}"
        );
    }

    // And the two remaining combinations with a hook installed: soft
    // names the machine as a warning, manual claims none even where
    // one stands. Six in all, which is what the two knobs make.
    for (name, mode, names_the_hook) in
        [("softwith", "soft", false), ("manualwith", "manual", false)]
    {
        let dir = born(name, mode, true);
        assert!(
            dir.join(".git/hooks/commit-msg").is_file(),
            "the hook is installed for {mode} + hooks = true"
        );
        let rule = rule_paragraph(&dir);
        assert_eq!(
            rule.contains("commit-msg hook"),
            names_the_hook,
            "the {mode} paragraph names the hook only if it means the \
             machine holds the rule:\n{rule}"
        );
        assert!(
            !rule.contains("no commit judgement runs here"),
            "and does not claim there is no court where a hook \
             stands ({mode}):\n{rule}"
        );
    }
}

/// The same rule the other way round: what keel says in someone
/// else's project is about that project. `keel check` used to close a
/// green verdict with a note out of keel's OWN queue ("a contract
/// naming a module that does not exist should be a finding off the
/// plan branch"), which is keel's rung and not the reader's, and was
/// closed by wave 0035 besides. And sec. 4.13 told every reader that
/// the ban on merging `spike/*` is held by "the check on the PR" --
/// measured on spike/probe, `keel check` exits 0 and only notes it;
/// `keel close` is what refuses.
#[test]
fn what_keel_says_in_a_stranger_s_project_is_about_that_project() {
    let dir = born("stranger", "strict", true);
    // A project with a CLOSED wave, so `check` reaches its green
    // "what next" line at all. Review 0039 R-6: the sandbox used to
    // have no waves, so check printed "create the first wave" and the
    // line under judgement was never printed -- the assertion could
    // not fail, and a mutation restoring keel's own backlog note
    // passed the whole battery.
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    let mut decisions = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        decisions.push_str(&format!("  {cut}: \"not about this sandbox\"\n"));
    }
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios: {{}}\ntransforms:\n  work:\n    chore: \"the frame alone\"\n    files:\n      - README.md\n{decisions}---\n\n## Why\n\nтіло\n\n## transform: work\n\nтіло роботи\n"
        ),
    )
    .unwrap();
    fs::write(dir.join("README.md"), "a project\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "commit",
            "-q",
            "-m",
            "base",
            "--no-verify",
        ],
    );
    let (said, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "a whole project is judged green:\n{said}");
    let next = said
        .lines()
        .find(|line| line.starts_with("next step:"))
        .unwrap_or_else(|| panic!("the green verdict says what to do next:\n{said}"))
        .to_string();
    assert!(
        !next.contains("plan branch") && !next.contains("review 0022"),
        "and what it says is about the reader's project, not keel's own \
         queue:\n{next}"
    );
    assert!(
        next.contains("keel next") || next.contains("plan the next wave"),
        "it points at the reader's own next move:\n{next}"
    );

    // The norm names the court that really holds the ban -- asked of
    // the binary itself (`keel method`), not of a path on disk: a
    // probe that reads a file beside it judges whatever tree it was
    // run from, which is not this one.
    let (paragraph, code) = keel(&["method", "§4.13", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the paragraph is served:\n{paragraph}");
    assert!(
        paragraph.contains("keel close"),
        "sec. 4.13 names the court that refuses the merge:\n{paragraph}"
    );
    assert!(
        !paragraph.contains("the check on a PR"),
        "and no longer credits the court that stays green there:\n{paragraph}"
    );
}

/// The English methodology stands in two files -- `docs/en/` and the
/// root copy a reader meets beside the README -- and nothing judged
/// the second (review 0039 R-12): a mutation that rolled back sec.
/// 4.13 in the root copy alone passed the whole battery, while the
/// same rollback in either of the other two texts reddened it.
#[test]
fn the_root_methodology_is_the_same_text() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let carried = fs::read_to_string(root.join("docs/en/METHODOLOGY-V2.md")).unwrap();
    let beside = fs::read_to_string(root.join("METHODOLOGY.md")).unwrap();

    // The heads differ on purpose -- one records the revision it was
    // translated from, the other says it is a copy. The body is one
    // text, and it starts where the constitution does.
    let body = |text: &str| {
        text.split_once("\n## Constitution")
            .map(|(_, tail)| tail.to_string())
            .unwrap_or_else(|| panic!("the text carries a constitution"))
    };
    let (carried, beside) = (body(&carried), body(&beside));
    assert_eq!(
        carried.len(),
        beside.len(),
        "the root copy is the same length as the text it copies"
    );
    if carried != beside {
        let at = carried
            .char_indices()
            .zip(beside.chars())
            .find(|((_, a), b)| a != b)
            .map(|((at, _), _)| at)
            .unwrap_or(0);
        let from = at.saturating_sub(120);
        panic!(
            "the root METHODOLOGY.md and docs/en/METHODOLOGY-V2.md carry \
             one body, and they differ around byte {at}:\n\
             docs/en: …{}…\n\
             root:    …{}…",
            &carried[from..(at + 120).min(carried.len())],
            &beside[from..(at + 120).min(beside.len())]
        );
    }
}
