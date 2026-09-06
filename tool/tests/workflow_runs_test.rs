//! Scenario test of wave 0039: the generated CI runs where it is
//! born.
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

fn born(name: &str, adapter: Option<&str>) -> Sandbox {
    let dir = sandbox(name);
    git(&dir, &["init", "-q", "-b", "main"]);
    let mut args: Vec<String> = vec![
        "init".to_string(),
        dir.to_str().unwrap().to_string(),
        "--lang".to_string(),
        "en".to_string(),
        "--no-ask".to_string(),
    ];
    if let Some(adapter) = adapter {
        args.push("--adapter".to_string());
        args.push(adapter.to_string());
    }
    let borrowed: Vec<&str> = args.iter().map(String::as_str).collect();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&borrowed)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "the frame lands:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    dir
}

fn workflow(dir: &Path) -> String {
    fs::read_to_string(dir.join(".github/workflows/keel.yml")).unwrap()
}

/// proves: the-generated-ci-runs-where-it-is-born@2eee71 -- the frame
/// wrote a workflow that calls `keel` without installing it and said
/// so in a comment. A project that ran `keel init` and pushed got
/// `keel: command not found` -- a red CI that is not about its work,
/// on the very first push, from a file the frame itself had written.
#[test]
fn the_generated_ci_runs_where_it_is_born() {
    let text = workflow(&born("ciborn", Some("rust")));

    // The tool arrives before the courts that call it.
    let Some(install) = text.find("install.sh") else {
        panic!("a step that puts keel on PATH:\n{text}");
    };
    let Some(first_court) = text.find("keel check") else {
        panic!("the documents court:\n{text}");
    };
    assert!(
        install < first_court,
        "and it arrives BEFORE the first court that calls keel:\n{text}"
    );

    // It is a step with its own name, so a failure to install is not
    // a `command not found` in the middle of a judgement.
    assert!(
        text.contains("- name: the tool itself"),
        "the installing step carries its own name:\n{text}"
    );

    // And it says what it costs and how to replace it, rather than
    // leaving a reader to find out on a runner.
    for word in [
        "builds it from source",
        "git and cargo",
        "replace this step",
    ] {
        assert!(
            text.contains(word),
            "the file says \"{word}\" about the step it added:\n{text}"
        );
    }

    // Both courts stand, and sec. 4.13's ban needs both of them.
    assert!(
        text.contains("keel check") && text.contains("keel close"),
        "both courts are in the workflow (sec. 4.13):\n{text}"
    );

    // A project that named no adapter still gets the tool installed:
    // the courts that need no language still run.
    let text = workflow(&born("cinone", None));
    assert!(
        text.contains("install.sh"),
        "the installing step does not depend on a language:\n{text}"
    );

    // Review 0039 R-4: a project that PINS a version got a step that
    // installed `main` and then a refusal from every court two lines
    // below, because the pin and the binary differ. The generator
    // knows the pin, so the step carries it.
    let dir = born("cipin", Some("rust"));
    let (said, code) = keel(&[
        "setup",
        dir.to_str().unwrap(),
        "--version",
        "pin",
        "--no-ask",
    ]);
    assert_eq!(code, 0, "the pin is written:\n{said}");
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let pinned = config
        .lines()
        .find_map(|line| line.strip_prefix("version = "))
        .expect("keel.toml carries the pin")
        .trim()
        .trim_matches('"')
        .to_string();
    let text = workflow(&dir);
    assert!(
        text.contains(&format!("KEEL_REF: \"{pinned}\"")),
        "and the installing step fetches exactly that version:\n{text}"
    );
    let install = text.find("KEEL_REF").unwrap();
    let court = text.find("keel check").unwrap();
    assert!(install < court, "before any court runs:\n{text}");

    // And a project that keeps its own step is told how to keep it
    // without being nagged by `keel update` forever (R-10).
    assert!(
        text.contains("delete this") && text.contains("[generated]"),
        "the file says how an edited copy is kept for good:\n{text}"
    );
}

/// proves: the-generated-close-knows-its-branch@159b6e -- on a
/// pull_request event actions/checkout leaves a detached HEAD, and the
/// generated file named the branch for `keel check` alone: `keel
/// close` counted the blockers of "its own" wave, found no branch,
/// and left with "no blockers", exit 0, over a wave in progress
/// (global review 2026-09-06, bugs cut R-4).
#[test]
fn the_generated_close_knows_its_branch() {
    let text = workflow(&born("cibranch", Some("rust")));
    let Some(close_step) = text.find("- name: the closure court") else {
        panic!("the closure court step:\n{text}");
    };
    let Some(close_run) = text[close_step..].find("run: keel close") else {
        panic!("the closure court runs keel close:\n{text}");
    };
    assert!(
        text[close_step..close_step + close_run].contains("KEEL_BRANCH"),
        "the close step carries the branch the way the check step does:\n{text}"
    );
    // The first step's comment tells the truth since wave 0048: the
    // installer takes a published release where one exists, and
    // builds from source otherwise.
    assert!(
        !text.contains("There is no released binary yet"),
        "the comment no longer denies the release road:\n{text}"
    );
    assert!(
        text.contains("builds it from source"),
        "and still says what happens without one:\n{text}"
    );

    // This repository's own generated file was rewritten by `keel
    // update`, so both courts here carry the branch too.
    let own = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../.github/workflows/keel.yml"
    ))
    .unwrap();
    assert_eq!(
        own.matches("KEEL_BRANCH").count(),
        2,
        "keel's own workflow names the branch for check and for close"
    );

    // And close over a detached HEAD with the branch named counts the
    // blockers of that wave -- the wave is in work: proven, but no
    // review beside it.
    let dir = sandbox("cidetached");
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    fs::create_dir_all(dir.join("keel/reviews")).unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    let body = "тіло обіцянки\n\n";
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{d}---\n\n## scenario: it-works\n{body}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    let rev = keel::rev::text_rev(body);
    fs::write(
        dir.join("tests/toy_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(toy::works());\n}}\n"),
    )
    .unwrap();
    git(&dir, &["-c", "user.email=keel@test", "-c", "user.name=keel-test", "init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["-c", "user.email=keel@test", "-c", "user.name=keel-test", "commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "--detach"]);
    let named = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
        .env("KEEL_BRANCH", "0001-a-wave")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&named.stdout),
        String::from_utf8_lossy(&named.stderr)
    );
    assert!(
        said.contains("блокери хвилі цієї гілки 0001-a-wave"),
        "with the branch named, the detached close counts that wave's \
         blockers:\n{said}"
    );
    assert_ne!(named.status.code(), Some(0), "and leaves red:\n{said}");
}
