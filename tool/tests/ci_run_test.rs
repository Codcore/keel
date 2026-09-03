//! Scenario tests of wave 0019-ci-and-battery, transform ci-run:
//! the project's trusted ci runs in the closure court through the
//! §7.16 gate -- "trusted" means "runs", never "guaranteed".
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0019c-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    fs::create_dir_all(dir.join("keel/reviews")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
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

/// A crate with one always-green test, so the battery has something
/// to run and the court reaches the ci row.
fn project(name: &str) -> PathBuf {
    let dir = sandbox(name);
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    write(&dir, "tests/steady_test.rs", "#[test]\nfn steady() {}\n");
    dir
}

fn config_with(ci_line: &str, trust: Option<&str>) -> String {
    let mut text = format!("lang = \"en\"\nadapter = \"rust\"\n{ci_line}");
    if let Some(command) = trust {
        text.push_str(&format!(
            "\n[trust]\n\"{command}\" = \"{}\"\n",
            keel::trust::fingerprint(command)
        ));
    }
    text
}

/// proves: trusted-ci-runs@537361 -- holds §7.16 at work in the
/// closure court: a trusted ci runs exactly once and its verdict is
/// a row by name (passed -- silence in the count; failed -- a
/// blocker with the command's words and a red exit); an untrusted
/// ci never runs -- no side effect of the command lands -- and the
/// row says so with §7.16; "none" is a lawful refusal aloud, "" is
/// undecided, an absent field is said aloud -- none of them runs.
#[test]
fn trusted_ci_runs() {
    // Trusted and passing: runs exactly once, its row says passed.
    let dir = project("passing");
    let command = "echo run >> ci.count";
    write(
        &dir,
        "keel.toml",
        &config_with(&format!("ci = \"{command}\"\n"), Some(command)),
    );
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a passing trusted ci blocks nothing:\n{out}");
    assert!(
        out.contains(&format!("ci \"{command}\"")) && out.contains("passed"),
        "the ci verdict is a row by name (§7.16):\n{out}"
    );
    let count = fs::read_to_string(dir.join("ci.count")).unwrap_or_default();
    assert_eq!(
        count, "run\n",
        "the trusted ci ran exactly once:\n{count:?}"
    );

    // Trusted and failing: a blocker with the command's words, the
    // exit red -- the project's own gate is red, the wave does not
    // merge.
    let dir = project("failing");
    let command = "echo fmt-drift && exit 1";
    write(
        &dir,
        "keel.toml",
        &config_with(&format!("ci = \"{command}\"\n"), Some(command)),
    );
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 1,
        "a red trusted ci is a blocker -- the exit is red:\n{out}"
    );
    assert!(
        out.contains("FAILED") && out.contains("fmt-drift"),
        "the failed ci row carries the command's words:\n{out}"
    );

    // A failing ci that paints its words -- cargo fmt --check ends
    // its coloured diff with a bare reset sequence -- still gives
    // the row real words: colours are not words (found in the field
    // on slugline, wave 0019).
    let dir = project("painted");
    let painted = "echo 'Diff in src/lib.rs'; echo '\u{1b}[m'; exit 1";
    write(
        &dir,
        "keel.toml",
        &format!(
            "lang = \"en\"\nadapter = \"rust\"\nci = \"echo 'Diff in src/lib.rs'; echo '\\u001B[m'; exit 1\"\n\n[trust]\n\"echo 'Diff in src/lib.rs'; echo '\\u001B[m'; exit 1\" = \"{}\"\n",
            keel::trust::fingerprint(painted)
        ),
    );
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 1,
        "a painted red ci is a blocker all the same:\n{out}"
    );
    assert!(
        !out.contains('\u{1b}'),
        "no escape sequence is quoted as a verdict's words:\n{out:?}"
    );
    assert!(
        out.contains("Diff in src/lib.rs"),
        "the row carries the visible words the command left:\n{out}"
    );

    // Untrusted: never runs -- no side effect lands -- and the row
    // says so aloud with §7.16.
    let dir = project("untrusted");
    let command = "echo run >> ci.count";
    write(
        &dir,
        "keel.toml",
        &config_with(&format!("ci = \"{command}\"\n"), None),
    );
    let (out, err, _) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        !dir.join("ci.count").exists(),
        "an untrusted ci never runs -- no side effect lands:\n{out}"
    );
    assert!(
        out.contains("did not run") && out.contains("not trusted") && out.contains("§7.16"),
        "the untrusted ci is said aloud with §7.16:\n{out}"
    );

    // "none": a lawful refusal aloud, nothing runs.
    let dir = project("none");
    write(&dir, "keel.toml", &config_with("ci = \"none\"\n", None));
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "none blocks nothing:\n{out}");
    assert!(
        out.contains("refusal aloud") && out.contains("lawful"),
        "none is a lawful refusal aloud:\n{out}"
    );

    // "": undecided, nothing runs.
    let dir = project("undecided");
    write(&dir, "keel.toml", &config_with("ci = \"\"\n", None));
    let (out, err, _) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("undecided"),
        "an empty ci is undecided, said aloud:\n{out}"
    );

    // Absent: said aloud, nothing runs.
    let dir = project("absent");
    write(&dir, "keel.toml", &config_with("", None));
    let (out, err, code) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "no ci blocks nothing:\n{out}");
    assert!(
        out.contains("ci not declared"),
        "an absent ci is said aloud, never painted green:\n{out}"
    );
}
