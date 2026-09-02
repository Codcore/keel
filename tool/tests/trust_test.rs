//! Scenario tests of wave 0008-command-trust: the TOFU court of
//! §7.16/§2.8 -- a command from repository files is trusted only by
//! its recorded fingerprint -- and `keel trust`, the recording hand.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.
//! Expected fingerprints are computed OUTSIDE the tool (python:
//! collapse whitespace runs, trim, sha256, first 12 hex), so the
//! recipe is pinned from the outside, the §5.3 school.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0008t-{}-{name}", std::process::id()));
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

/// A wave answering every cut, so only the trust court speaks.
fn quiet_wave() -> String {
    let mut block = String::from(
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\ndecisions:\n",
    );
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            block.push_str(&format!("  {cut}: \"n/a for the trust sandbox\"\n"));
        }
    }
    block.push_str("---\n\n## scenario: s\n\nbody of s\n");
    block
}

/// Fingerprints computed outside the tool (see the header note).
const CARGO_TEST_FP: &str = "0031fd18feb5";
const CURL_FP: &str = "eb2d2a5d8061";

/// proves: untrusted-command-red@7f4796 -- holds §7.16: a command
/// from repository files -- a contract's verify or the project's ci
/// -- with no matching fingerprint in [trust] is a finding carrying
/// the command's text and the hint to record trust; ci written empty
/// is "undecided"; the same commands with matching fingerprints are
/// silence.
#[test]
fn untrusted_command_red() {
    // New commands, no [trust] at all: both named, red exit.
    let dir = sandbox("untrusted");
    write(&dir, "keel.toml", "ci = \"cargo test\"\n");
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel/contracts/ext-health.md",
        "---\nverify: \"curl -fsS https://svc.example/health\"\n---\n\nA foreign promise: the service answers (§2.8).\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "untrusted commands are findings:\n{out}");
    assert!(
        out.contains("\"cargo test\"") && out.contains("not trusted"),
        "the ci command named with its text (§7.16):\n{out}"
    );
    assert!(
        out.contains("\"curl -fsS https://svc.example/health\""),
        "the verify command named with its text:\n{out}"
    );
    assert!(
        out.contains("keel trust"),
        "the finding hints how to record the trust:\n{out}"
    );

    // ci written empty: undecided, said as its own finding.
    let dir = sandbox("empty-ci");
    write(&dir, "keel.toml", "ci = \"\"\n");
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "an empty ci is undecided, a finding:\n{out}");
    assert!(
        out.contains("written empty"),
        "the undecided ci named aloud:\n{out}"
    );
    assert!(
        out.contains("\"none\""),
        "the way out named: decide, or say none aloud:\n{out}"
    );

    // The same commands with matching fingerprints: silence.
    let dir = sandbox("trusted");
    write(
        &dir,
        "keel.toml",
        &format!(
            "ci = \"cargo test\"\n\n[trust]\n\"cargo test\" = \"{CARGO_TEST_FP}\"\n\"curl -fsS https://svc.example/health\" = \"{CURL_FP}\"\n"
        ),
    );
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel/contracts/ext-health.md",
        "---\nverify: \"curl -fsS https://svc.example/health\"\n---\n\nA foreign promise: the service answers (§2.8).\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "matching fingerprints are silence:\n{out}");
    assert!(
        !out.contains("not trusted"),
        "no trust finding over a matching fingerprint:\n{out}"
    );
}
