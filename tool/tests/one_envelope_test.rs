//! Scenario test of wave 0040: one envelope for every command.
//!
//! The output is judged by a real JSON parser, never by matching
//! substrings (the school of wave 0025).
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

/// A project whose name carries what naive string-building breaks on:
/// a quote, a backslash and Cyrillic.
fn project(name: &str) -> Sandbox {
    let dir = sandbox(name);
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// proves: one-envelope-for-every-command@3a699f -- the concept promises
/// a CLI contract twice: every reading command has `--json` and the
/// output is a self-sufficient package. Measured before the wave: the
/// word appears seven times in the sources, every one about the
/// format of a hook config.
#[test]
fn one_envelope_for_every_command() {
    let dir = project("envelope");

    // Every reading command answers, and the outer shape is one.
    for command in [
        "check", "close", "status", "next", "map", "review", "version", "cuts", "rev",
    ] {
        let (out, _, code) = keel(&[command, "--json", dir.to_str().unwrap()]);
        let package: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|error| {
            panic!("`keel {command} --json` prints one JSON object: {error}\n{out}")
        });
        assert_eq!(
            package["keel"], 1,
            "the envelope names its own version ({command}):\n{out}"
        );
        assert_eq!(
            package["command"], command,
            "and the command it answers for:\n{out}"
        );
        assert_eq!(
            package["exit"], code,
            "and the exit code it left with ({command}):\n{out}"
        );
        assert_eq!(
            package["ok"],
            code == 0,
            "and whether that is green ({command}):\n{out}"
        );
        assert!(
            package["root"].is_string() && package["lang"].is_string(),
            "and where it looked, in which tongue ({command}):\n{out}"
        );
        assert!(
            package["report"].is_string(),
            "and the whole prose report, so nothing is lost ({command}):\n{out}"
        );
    }

    // A refusal is the same envelope, not a bare stderr.
    let (out, _, code) = keel(&["check", "--json", "/keel-no-such-directory"]);
    let package: serde_json::Value = serde_json::from_str(&out)
        .unwrap_or_else(|error| panic!("a refusal is a package too: {error}\n{out}"));
    assert_eq!(package["exit"], code, "with the code it left with:\n{out}");
    assert_eq!(package["ok"], false, "and green is false:\n{out}");
    assert!(
        package["refusal"]["reason"].is_string() && package["refusal"]["instead"].is_string(),
        "carrying the reason and what to do instead (NEW-CONCEPT, the CLI contract):\n{out}"
    );

    // The package is built by a real serializer: a path with a quote,
    // a backslash and Cyrillic in it stays one object.
    let awkward = sandbox("лапка\"і\\слеш");
    fs::write(awkward.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let (out, _, _) = keel(&["version", "--json", awkward.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap_or_else(|error| {
        panic!("an awkward path does not break the package: {error}\n{out}")
    });
    assert!(
        package["root"]
            .as_str()
            .is_some_and(|root| root.contains('"') && root.contains('\\')),
        "and the path comes back whole, escaped rather than mangled:\n{out}"
    );

    // Without the flag, nothing changed: the prose road is the road
    // every existing script and probe already walks.
    let (prose, _, prose_code) = keel(&["version", dir.to_str().unwrap()]);
    assert!(
        serde_json::from_str::<serde_json::Value>(&prose).is_err(),
        "the plain road is still prose:\n{prose}"
    );
    let (_, _, json_code) = keel(&["version", "--json", dir.to_str().unwrap()]);
    assert_eq!(
        prose_code, json_code,
        "and both roads leave with the same code"
    );
}
