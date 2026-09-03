//! Scenario tests of wave 0017-adapter-named-by-language, transform
//! language-name: the adapter is named by the project's language --
//! rust is canonical, cargo an accepted synonym said aloud.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0017a-{}-{name}", std::process::id()));
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

/// proves: adapter-named-by-language@139959 -- holds the concept's
/// letter and the operator's decision of 2026-09-03: rust is the
/// canonical adapter name and every adapter-needing court accepts
/// it; the old spelling cargo still works and keel check says the
/// synonym aloud with the canonical name beside it; an unknown
/// adapter refuses with rust in the instead; and the init
/// scaffolding recommends rust.
#[test]
fn adapter_named_by_language() {
    // rust is canonical: the courts run as they did with cargo.
    let dir = sandbox("rustname");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "rust is the canonical adapter name:\n{out}");
    assert!(
        out.contains("counted:"),
        "the stage eye runs under the language name:\n{out}"
    );
    let (out, err, code) = keel(&["next", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the step hand runs under the language name:\n{out}"
    );

    // cargo still works -- and check says the synonym aloud.
    let dir = sandbox("synonym");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the old spelling still works -- no config breaks silently:\n{out}"
    );
    let (out, err, _) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("synonym") && out.contains("\"rust\""),
        "check says the synonym aloud with the canonical name (§9.7):\n{out}"
    );

    // An unknown adapter refuses with the canonical name to reach for.
    let dir = sandbox("unknown");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"elixir\"\n");
    crate_files(&dir);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "an unknown adapter refuses aloud:\n{out}");
    assert!(
        out.contains("\"rust\""),
        "the instead names the canonical language name:\n{out}"
    );

    // The init scaffolding recommends the language name.
    let dir = sandbox("initword");
    git(&dir, &["init", "-q", "-b", "main"]);
    let (_, _, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the frame lands");
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        config.contains("# adapter = \"rust\""),
        "the scaffolding recommends rust, the concept's letter:\n{config}"
    );
}
