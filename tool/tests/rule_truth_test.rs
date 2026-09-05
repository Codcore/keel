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

    // The hook does NOT stand. The paragraph must not promise it.
    for (name, mode) in [("strictno", "strict"), ("softno", "soft")] {
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
        assert!(
            said.contains("no commit judgement runs here"),
            "it says plainly that nothing judges the commits ({mode}):\n{said}"
        );
        assert!(
            said.contains("keel close"),
            "and names what does still judge ({mode}):\n{said}"
        );
    }

    // `manual` was already honest, and stays so with a hook present.
    let dir = born("manualwith", "manual", true);
    let said = block(&dir);
    assert!(
        !said.contains("commit-msg hook"),
        "manual claims no hook even where one stands:\n{said}"
    );
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
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
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
    assert_eq!(code, 0, "a fresh project is judged whole:\n{said}");
    assert!(
        !said.contains("plan branch") || !said.contains("does not exist"),
        "the verdict does not hand a stranger keel's own backlog:\n{said}"
    );

    // The norm names the court that really holds the ban.
    let uk = fs::read_to_string("../keel/docs/uk/METHODOLOGY-V2.md")
        .or_else(|_| fs::read_to_string("docs/uk/METHODOLOGY-V2.md"))
        .unwrap();
    let paragraph = uk
        .split("**§4.13.**")
        .nth(1)
        .expect("sec. 4.13 exists")
        .split("**§")
        .next()
        .unwrap()
        .to_string();
    assert!(
        paragraph.contains("keel close"),
        "sec. 4.13 names the court that refuses the merge:\n{paragraph}"
    );
}
