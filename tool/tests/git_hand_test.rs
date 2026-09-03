//! Scenario tests of wave 0021-git-speaks-to-the-project, transform
//! one-git-hand: every court judges the project it was pointed at,
//! never the repository that spawned it.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

/// What a git hook hands its children -- the probe's own sandboxes
/// must be as deaf to it as the tool is (school 0020).
const GIT_ENV: [&str; 10] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
    "GIT_CEILING_DIRECTORIES",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
];

mod common;

use common::{Sandbox, keel_sandbox};

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
    for name in GIT_ENV {
        command.env_remove(name);
    }
    let out = command
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Everything a git hook can leave pointing at another repository:
/// the ten names the tool forgets, plus the numbered pair that
/// carries `git -c`. The probe sets them all -- eight of the ten
/// were held by nothing before (review 0021 R-1).
fn hook_env(foreign: &Path, judged: &Path) -> Vec<(String, String)> {
    let git = foreign.join(".git");
    vec![
        ("GIT_DIR".into(), git.display().to_string()),
        ("GIT_WORK_TREE".into(), foreign.display().to_string()),
        ("GIT_COMMON_DIR".into(), git.display().to_string()),
        (
            "GIT_INDEX_FILE".into(),
            git.join("index").display().to_string(),
        ),
        (
            "GIT_OBJECT_DIRECTORY".into(),
            git.join("objects").display().to_string(),
        ),
        (
            "GIT_ALTERNATE_OBJECT_DIRECTORIES".into(),
            git.join("objects").display().to_string(),
        ),
        ("GIT_PREFIX".into(), String::new()),
        (
            "GIT_CEILING_DIRECTORIES".into(),
            judged.display().to_string(),
        ),
        (
            "GIT_CONFIG_PARAMETERS".into(),
            format!("'core.hooksPath={}'", foreign.join("evil-hooks").display()),
        ),
        ("GIT_CONFIG_COUNT".into(), "1".into()),
        ("GIT_CONFIG_KEY_0".into(), "core.hooksPath".into()),
        (
            "GIT_CONFIG_VALUE_0".into(),
            foreign.join("evil-numbered").display().to_string(),
        ),
    ]
}

fn run(args: &[&str], env: &[(String, String)]) -> (String, i32) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_keel"));
    command.args(args);
    for name in GIT_ENV {
        command.env_remove(name);
    }
    command.env_remove("GIT_CONFIG_KEY_0");
    command.env_remove("GIT_CONFIG_VALUE_0");
    for (name, value) in env {
        command.env(name, value);
    }
    let out = command.output().unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

/// The judged project: a crate, a git repository on the wave's own
/// branch, and one wave whose scenario carries no tag at all -- so
/// the closure court must stay red.
fn project(name: &str) -> Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    // The scenario is proven by a tag and a green test; the review
    // report is what is missing -- so the wave is IN PROGRESS, and
    // its lack blocks the merge (a wave with no tag at all would be
    // a plan, and a plan is not red, §6.5).
    let rev = keel::rev::text_rev("body of never-proven\n");
    // The project's own test writes down which repository it was
    // run in: the battery must run in this project's world, not in
    // the one that spawned keel (review 0021 R-3).
    write(
        &dir,
        "tests/steady_test.rs",
        &format!(
            "/// proves: never-proven@{rev}\n#[test]\nfn steady() {{\n    let out = std::process::Command::new(\"git\")\n        .args([\"rev-parse\", \"--absolute-git-dir\"])\n        .current_dir(env!(\"CARGO_MANIFEST_DIR\"))\n        .output()\n        .unwrap();\n    std::fs::write(\n        std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"seen-git.txt\"),\n        out.stdout,\n    )\n    .unwrap();\n}}\n"
        ),
    );
    write(
        &dir,
        "keel/waves/0030-unproven.md",
        &format!(
            "---\nscenarios:\n  never-proven: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [never-proven]\n    files: [src/lib.rs]\n{}---\n\n## Why\n\nwhy words\n\n## scenario: never-proven\n\nbody of never-proven\n\n## transform: t\n\nthe work of t\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    git(&dir, &["init", "-q", "-b", "0030-unproven"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "state"]);
    dir
}

/// A stranger's repository, whole and quiet: whatever the courts do,
/// not one byte of it may change.
fn foreign(name: &str) -> Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(&dir, "README.md", "the stranger\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the stranger's own state"]);
    dir
}

/// Every file of a tree with its bytes -- the honest way to say
/// "not one byte changed".
fn snapshot(root: &Path) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if let Ok(bytes) = fs::read(&path) {
                out.push((
                    path.strip_prefix(root).unwrap().display().to_string(),
                    bytes,
                ));
            }
        }
    }
    out.sort();
    out
}

/// proves: courts-deaf-to-the-environment@429407 -- holds what the
/// 0020 review measured and the wave repairs: a git hook hands its
/// children GIT_DIR and its kin, those outrank -C, and a court that
/// inherits them judges the repository that spawned it. Under that
/// environment every court must still judge the project it was
/// given -- above all the closure court, which greened on an
/// unproven wave -- and the stranger's repository must not change
/// by a byte.
#[test]
fn courts_deaf_to_the_environment() {
    let dir = project("judged");
    let stranger = foreign("stranger");
    let before = snapshot(&stranger);
    let env = hook_env(&stranger, &dir);
    let clean: Vec<(String, String)> = Vec::new();
    let path = dir.to_str().unwrap();

    // Every read-only court: the word under a hook's environment is
    // the word without it, byte for byte. This is the assertion that
    // holds every one of the ten names that can change git's answer
    // (review 0021 R-1, R-2: the old probe held two names and three
    // courts, and a broken tool satisfied its check and plan asserts).
    for args in [
        vec!["check", path],
        vec!["status", path],
        vec!["next", path],
        vec!["review", path],
        vec!["rev", path],
        vec!["map", path],
        vec!["version", path],
        vec!["trust", path],
    ] {
        let (plain, plain_code) = run(&args, &clean);
        let (hooked, hooked_code) = run(&args, &env);
        assert_eq!(
            plain, hooked,
            "{:?} says the same word whatever repository the environment names",
            args[0]
        );
        assert_eq!(
            plain_code, hooked_code,
            "{:?} answers with the same exit code",
            args[0]
        );
    }

    // The closure court, the worst damage the 0020 review measured:
    // red on this project's unproven wave, and the same word either
    // way. It also runs the battery -- so the project's own test
    // must have run in the project's world (R-3).
    let (plain, plain_code) = run(&["close", path], &clean);
    let (hooked, hooked_code) = run(&["close", path], &env);
    assert_eq!(plain_code, 1, "close is red on the unproven wave:\n{plain}");
    assert_eq!(
        hooked_code, 1,
        "close stays red under a stranger's environment:\n{hooked}"
    );
    assert_eq!(plain, hooked, "close says the same word either way");
    assert!(
        hooked.contains("0030-unproven") && hooked.contains("review"),
        "close names this project's wave and its lack:\n{hooked}"
    );
    let seen = fs::read_to_string(dir.join("seen-git.txt")).unwrap_or_default();
    assert!(
        seen.trim().starts_with(dir.to_str().unwrap()),
        "the battery ran in this project's world, not the stranger's (R-3): {seen:?}"
    );

    // The commit judgement keeps its subject.
    write(&dir, ".git/COMMIT_MSG_PROBE", "red: never-proven\n");
    let msg = dir.join(".git/COMMIT_MSG_PROBE");
    let (plain, _) = run(&["gate", msg.to_str().unwrap(), path], &clean);
    let (hooked, _) = run(&["gate", msg.to_str().unwrap(), path], &env);
    assert_eq!(plain, hooked, "the gate judges the same commit either way");
    assert!(
        hooked.contains("never-proven"),
        "the gate keeps this project's scenario:\n{hooked}"
    );

    // The planning hand counts THIS project's numbers: 0031 is free
    // here and taken in the stranger, whose branch says so.
    let (out, code) = run(&["plan", "0031-twin", path], &env);
    assert_eq!(
        code, 0,
        "a number free in this project is free, whatever the stranger holds:\n{out}"
    );
    fs::remove_file(dir.join("keel/waves/0031-twin.md")).unwrap();

    // `git -c` reaches a child by two roads -- the packed
    // GIT_CONFIG_PARAMETERS and the numbered COUNT/KEY/VALUE -- and
    // both point core.hooksPath at the stranger here. The frame's
    // own writing hand proves both are forgotten: the hook lands in
    // this project, and the stranger grows no hooks directory.
    let fresh = project("hookpath");
    let (out, code) = run(
        &["hook", fresh.to_str().unwrap()],
        &hook_env(&stranger, &fresh),
    );
    assert_eq!(code, 0, "the hook is installed:\n{out}");
    assert!(
        fresh.join(".git/hooks/commit-msg").is_file(),
        "the hook lands in the project the tool was given:\n{out}"
    );
    assert!(
        !stranger.join("evil-hooks").exists() && !stranger.join("evil-numbered").exists(),
        "neither road of `git -c` moves the frame's hand:\n{out}"
    );

    // And the stranger is exactly as it was -- not one byte.
    assert_eq!(
        snapshot(&stranger),
        before,
        "no court writes into the repository of the environment"
    );
}

/// The one home judged by a run, not by prose (review 0021 R-6): a
/// raw git call anywhere but inside the hand would let the next
/// wave leak the environment again in silence.
#[test]
fn the_only_raw_git_call_is_the_hand() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut raw = Vec::new();
    for entry in fs::read_dir(&src).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&path).unwrap();
            let count = text.matches("Command::new(\"git\")").count();
            if count > 0 {
                raw.push((
                    path.file_name().unwrap().to_string_lossy().into_owned(),
                    count,
                ));
            }
        }
    }
    assert_eq!(
        raw,
        vec![("scope.rs".to_string(), 1usize)],
        "git is called by one hand only -- scope::git_at (§ the wave's own promise)"
    );
}
