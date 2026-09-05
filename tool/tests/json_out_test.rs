//! Scenario test of wave 0040: every reading command answers in JSON.
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

fn keel(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// A project with one wave whose header is broken on purpose, so the
/// document court has something real to be red about.
fn project(name: &str) -> Sandbox {
    let dir = sandbox(name);
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  a-promise:\n    covers: [no.such.cut]\ntransforms: {}\n---\n\n## Why\n\nтіло\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base", "--no-verify"]);
    dir
}

/// proves: every-reading-command-answers-in-json@42b771 -- every harness
/// keel itself writes had to parse prose to learn how many findings
/// there were, and the prose comes in two languages. The concept
/// promised the other road twice and the code never had it.
#[test]
fn every_reading_command_answers_in_json() {
    let dir = project("jsonout");

    // check: the findings a harness needs, as fields -- which file,
    // and why -- with the summary's own numbers beside them.
    let (out, _, code) = keel(&["check", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(code, 1, "the sandbox really is red:\n{out}");
    assert_eq!(
        package["structured"], true,
        "and the package says so:\n{out}"
    );
    let findings = package["findings"].as_array().expect("findings is a list");
    assert!(!findings.is_empty(), "the findings are listed:\n{out}");
    assert!(
        findings
            .iter()
            .all(|one| one["file"].is_string() && one["reason"].is_string()),
        "each with the file that is wrong and why:\n{out}"
    );
    assert_eq!(
        package["summary"]["findings"].as_u64().map(|n| n as usize),
        Some(findings.len()),
        "and the summary counts exactly those:\n{out}"
    );
    assert!(
        package["summary"]["documents"].is_u64() && package["limits"].is_array(),
        "with the documents walked and what was not judged:\n{out}"
    );
    assert!(
        findings.iter().any(|one| one["file"]
            .as_str()
            .is_some_and(|file| file.contains("0001-a-wave"))),
        "the broken wave is named by its path, so a harness need not \
         split a sentence in either tongue:\n{out}"
    );

    // close and status carry their own number rather than a count a
    // harness would have to read out of prose.
    let (out, _, _) = keel(&["close", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["blockers"].is_u64() || package["refusal"].is_object(),
        "close says how many blockers, or refuses as a package:\n{out}"
    );
    let (out, _, _) = keel(&["status", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["refusals"].is_u64() || package["refusal"].is_object(),
        "status says how many refusals, or refuses as a package:\n{out}"
    );

    // version answers what a harness pinning a release actually asks.
    let (out, _, _) = keel(&["version", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["running"].is_string(),
        "version names the binary that ran:\n{out}"
    );
    assert!(
        package["pin"].is_null(),
        "and the pin, null where there is none rather than an empty \
         string pretending to be one (NEW-CONCEPT: a number nobody \
         gave is empty, not zero):\n{out}"
    );

    // cuts hands over the list itself, not a page to scrape.
    let (out, _, _) = keel(&["cuts", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        package["cuts"].as_array().map(Vec::len),
        Some(40),
        "cuts gives the forty by name:\n{out}"
    );

    // And the promise that keeps every existing script working: the
    // plain road did not move a byte.
    for command in [
        "check", "close", "status", "map", "review", "version", "cuts", "rev",
    ] {
        let (plain, plain_err, plain_code) = keel(&[command, dir.to_str().unwrap()]);
        let (again, again_err, again_code) = keel(&[command, dir.to_str().unwrap()]);
        assert_eq!(plain, again, "{command} is steady");
        assert_eq!(plain_err, again_err, "{command} is steady on stderr");
        assert_eq!(plain_code, again_code, "{command} is steady in its code");
        let (_, _, json_code) = keel(&[command, "--json", dir.to_str().unwrap()]);
        assert_eq!(
            plain_code, json_code,
            "and both roads of {command} leave with the same code"
        );
    }
}
