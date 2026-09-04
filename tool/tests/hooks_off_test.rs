//! Scenario test of wave 0037: the answers are obeyed.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn init(dir: &Path, args: &[&str]) -> String {
    let mut all: Vec<&str> = vec!["init"];
    all.extend_from_slice(args);
    all.push(dir.to_str().unwrap());
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

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::remove_dir_all(dir.join("keel")).ok();
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// proves: the-answers-are-obeyed@67cc0f -- review 0035 named it and
/// set it aside: `keel init --no-hooks` writes `hooks = false` into
/// keel.toml and installs `.git/hooks/commit-msg` all the same. A
/// question whose answer changes nothing is not a question.
#[test]
fn the_answers_are_obeyed() {
    // The answer is no: no git hook is written, and the report says
    // so rather than falling silent.
    let dir = project("nohooks");
    let said = init(&dir, &["--no-ask", "--no-hooks", "--lang", "uk"]);
    assert!(
        !dir.join(".git/hooks/commit-msg").exists(),
        "`--no-hooks` means no commit-msg hook (§9.3):\n{said}"
    );
    assert!(
        said.contains("hooks = false") || said.contains("hooks"),
        "and the report says the answer was obeyed, not nothing:\n{said}"
    );

    // The answer is yes: the hook is there, as before.
    let dir = project("withhooks");
    let said = init(&dir, &["--no-ask", "--hooks", "--lang", "uk"]);
    assert!(
        dir.join(".git/hooks/commit-msg").exists(),
        "the default road is untouched:\n{said}"
    );

    // A hook that is already ours is not swept away by a later "no":
    // removing what a person may rely on is not this command's to do.
    let dir = project("already");
    init(&dir, &["--no-ask", "--hooks", "--lang", "uk"]);
    assert!(
        dir.join(".git/hooks/commit-msg").exists(),
        "the hook stands"
    );
    let said = init(&dir, &["--no-ask", "--no-hooks", "--lang", "uk"]);
    assert!(
        dir.join(".git/hooks/commit-msg").exists(),
        "an installed hook is left where it is:\n{said}"
    );
    assert!(
        said.contains("hooks"),
        "and the report says it is still there and no longer \
         maintained:\n{said}"
    );
}
