//! Scenario test of wave 0040: the frame takes the place and the
//! branch.
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
    let out = command
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn keel_at(cwd: &Path, args: &[&str], branch: Option<&str>) -> (String, String, i32) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_keel"));
    command
        .args(args)
        .current_dir(cwd)
        .env_remove("KEEL_BRANCH");
    if let Some(branch) = branch {
        command.env("KEEL_BRANCH", branch);
    }
    let out = command.output().unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// A project on a branch named like a wave, so the scope court has
/// something to say about which branch it is on.
fn project(name: &str) -> Sandbox {
    let dir = sandbox(name);
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base", "--no-verify"]);
    dir
}

/// proves: the-frame-takes-the-place-and-the-branch@1bc200 -- the
/// concept asks for both by name: `-C <dir>` says where to work, and
/// `--branch <name>` names the branch where git does not know it (CI
/// with a detached HEAD), because skipping the comparison in silence
/// is forbidden (sec. 4.10). Measured before the wave: `-C` existed
/// only inside, as an argument to git, and there was no `--branch`
/// flag at all -- only the KEEL_BRANCH variable the generated
/// workflow has to set.
#[test]
fn the_frame_takes_the_place_and_the_branch() {
    let dir = project("frameflags");
    let elsewhere = sandbox("elsewhere");

    // -C says where to work, from a directory that is not a project.
    let (out, err, code) = keel_at(
        &elsewhere,
        &["check", "-C", dir.to_str().unwrap(), "--json"],
        None,
    );
    assert_eq!(code, 0, "the named directory is judged:\n{out}{err}");
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["root"]
            .as_str()
            .is_some_and(|root| root.ends_with("frameflags")),
        "and it is the one that was named, not the current one:\n{out}"
    );

    // --branch names the branch where git hides it, and it is the
    // flag that wins over the variable: a person typing a flag means
    // it more than an environment left over from a hook.
    let (out, _, _) = keel_at(
        &dir,
        &["check", "--branch", "0001-a-wave", "--json"],
        Some("some-other-branch"),
    );
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    let report = package["report"].as_str().unwrap_or_default();
    assert!(
        report.contains("0001-a-wave"),
        "the named branch is the one judged:\n{report}"
    );
    assert!(
        !report.contains("some-other-branch"),
        "and the variable does not win over the flag:\n{report}"
    );

    // Where no flag is given the variable still stands, exactly as it
    // did before this wave -- the generated CI relies on it.
    let (out, _, _) = keel_at(&dir, &["check", "--json"], Some("0002-another"));
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["report"]
            .as_str()
            .is_some_and(|report| report.contains("0002-another")),
        "KEEL_BRANCH still names the branch where no flag does:\n{out}"
    );

    // A flag that takes a word and is given none is a refusal, not a
    // silent nothing.
    for args in [vec!["check", "-C"], vec!["check", "--branch"]] {
        let (_, err, code) = keel_at(&dir, &args, None);
        assert_eq!(code, 2, "{args:?} refuses:\n{err}");
        assert!(!err.is_empty(), "{args:?} says why:\n{err}");
    }

    // And an unknown flag is still a refusal that lists what this
    // command does take -- never a directory named "--jsonn".
    let (_, err, code) = keel_at(&dir, &["check", "--jsonn"], None);
    assert_eq!(code, 2, "an unknown flag refuses:\n{err}");
    assert!(
        err.contains("--json") && err.contains("-C"),
        "and names the flags every command takes:\n{err}"
    );
}
