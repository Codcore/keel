//! Scenario tests of wave 0018-version-pin, transform pin-court:
//! the keel.toml version field pins the tool -- the wrong binary
//! never judges a project, and the keel version lamp answers even
//! where the courts refuse.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0018v-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
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

fn crate_files(dir: &Path) {
    write(
        dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(dir, "src/lib.rs", "");
}

/// proves: version-pin@a1646e -- holds the concept's letter
/// (NEW-CONCEPT, Distribution: the project pins the tool's exact
/// version, the pin moves as a diff line): a matching pin leaves
/// every court working and the lamp says it holds; a foreign pin
/// turns every court into a refusal aloud carrying both names
/// while the lamp still answers with both; no pin leaves the
/// courts as they were and the lamp advises pinning the running
/// version; the init scaffolding recommends the same pin.
#[test]
fn version_pin() {
    let running = env!("CARGO_PKG_VERSION");

    // A matching pin: the courts run as before, the lamp says held.
    let dir = sandbox("held");
    write(
        &dir,
        "keel.toml",
        &format!("lang = \"en\"\nadapter = \"rust\"\nversion = \"{running}\"\n"),
    );
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a matching pin never blocks a court:\n{out}");
    assert!(
        out.contains("counted:"),
        "the stage eye runs under a held pin:\n{out}"
    );
    let (out, err, code) = keel(&["version", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the lamp answers green:\n{out}");
    assert!(
        out.contains(running) && out.contains("held"),
        "the lamp names the running version and the held pin:\n{out}"
    );

    // A foreign pin: every court refuses aloud with both names --
    // the wrong binary never judges -- while the lamp still answers.
    let dir = sandbox("foreign");
    write(
        &dir,
        "keel.toml",
        "lang = \"en\"\nadapter = \"rust\"\nversion = \"9.9.9\"\n",
    );
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 2,
        "a foreign pin is a refusal, never a silent run:\n{out}"
    );
    assert!(
        out.contains("9.9.9") && out.contains(running),
        "the refusal carries both names -- the pinned and the running:\n{out}"
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 2,
        "every config-reading court inherits the refusal:\n{out}"
    );
    let (out, err, code) = keel(&["version", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the lamp answers where the courts refuse -- diagnostics never die:\n{out}"
    );
    assert!(
        out.contains("9.9.9") && out.contains(running) && out.contains("NOT this binary"),
        "the lamp names both versions and the verdict:\n{out}"
    );

    // No version field: the courts run as they always did, the lamp
    // advises the pin.
    let dir = sandbox("nopin");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "no pin changes nothing for the courts:\n{out}");
    let (out, err, code) = keel(&["version", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the lamp answers green without a pin:\n{out}");
    assert!(
        out.contains("not set") && out.contains(running),
        "the lamp says no pin is set and advises the running version:\n{out}"
    );

    // The init scaffolding recommends pinning the running version.
    let dir = sandbox("initpin");
    git(&dir, &["init", "-q", "-b", "main"]);
    let (_, _, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands");
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        config.contains(&format!("# version = \"{running}\"")),
        "the scaffolding recommends the running version as the pin:\n{config}"
    );
}

/// proves: version-pin@a1646e -- the second birth out of review
/// 0018 (R-1, R-3, R-4): exactness is the heart of the pin, so a
/// NEAR foreign pin -- same length, same first character, one
/// character apart from the running version -- must refuse like a
/// far one (the reviewer's first-character counterfeit survived
/// the battery); the lamp quotes the pin so a trailing space or an
/// empty pin is visible to the eye; and the refusal's instead names
/// the order of moving the pin forward -- the new binary first,
/// whose own gate passes the new pin.
#[test]
fn version_pin_second_birth() {
    let running = env!("CARGO_PKG_VERSION");
    let mut near = running.to_string();
    let last = near.pop().unwrap();
    near.push(if last == '9' { '8' } else { '9' });

    // R-1: the near foreign pin refuses in the court with both
    // names, and the lamp names it as not this binary.
    let dir = sandbox("near");
    write(
        &dir,
        "keel.toml",
        &format!("lang = \"en\"\nadapter = \"rust\"\nversion = \"{near}\"\n"),
    );
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 2,
        "a pin one character apart is as foreign as any (R-1):\n{out}"
    );
    assert!(
        out.contains(&near) && out.contains(running),
        "the refusal carries both names, near as they are (R-1):\n{out}"
    );
    // R-4: the instead names the order of moving the pin forward.
    assert!(
        out.contains("new binary"),
        "the instead says the new binary comes first (R-4):\n{out}"
    );
    let (out, err, code) = keel(&["version", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the lamp answers on a near foreign pin:\n{out}");
    assert!(
        out.contains("NOT this binary"),
        "the lamp calls the near pin what it is (R-1):\n{out}"
    );
    // R-3: the lamp quotes the pin -- the eye sees its exact bytes.
    assert!(
        out.contains(&format!("\"{near}\"")),
        "the lamp quotes the pin (R-3):\n{out}"
    );

    // R-3 on the held side too: the quoted pin beside "held".
    let dir = sandbox("heldquoted");
    write(
        &dir,
        "keel.toml",
        &format!("lang = \"en\"\nadapter = \"rust\"\nversion = \"{running}\"\n"),
    );
    crate_files(&dir);
    let (out, err, code) = keel(&["version", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the lamp answers on a held pin:\n{out}");
    assert!(
        out.contains(&format!("\"{running}\"")) && out.contains("held"),
        "the held pin is quoted as well (R-3):\n{out}"
    );
}
