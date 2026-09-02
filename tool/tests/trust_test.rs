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

/// proves: trust-line-stale-red@9cc58d -- holds §7.16 on the
/// records' side: a [trust] line no live command answers to is a
/// door opened in advance (change or withdrawal does not inherit
/// trust); a crooked fingerprint over a live command is a finding
/// with the advice to rewrite; and ci = "none" is a lawful refusal
/// aloud, not a finding.
#[test]
fn trust_line_stale_red() {
    let dir = sandbox("stale");
    write(
        &dir,
        "keel.toml",
        &format!(
            "ci = \"none\"\n\n[trust]\n\"rm -rf old\" = \"{CARGO_TEST_FP}\"\n\"echo ok\" = \"000000000000\"\n"
        ),
    );
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel/contracts/ext-echo.md",
        "---\nverify: \"echo ok\"\n---\n\nA foreign promise: the echo answers (§2.8).\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "stale trust lines are findings:\n{out}");
    assert!(
        out.contains("\"rm -rf old\"") && out.contains("door opened in advance"),
        "the orphaned line is a door opened in advance:\n{out}"
    );
    assert!(
        out.contains("\"echo ok\"") && out.contains("crooked"),
        "the crooked fingerprint over a live command named:\n{out}"
    );
    assert!(
        out.contains("keel trust"),
        "the crooked line's advice: rewrite through keel trust:\n{out}"
    );
    assert!(
        !out.contains("\"none\""),
        "ci = none is a refusal aloud, not a finding (§7.16):\n{out}"
    );
    // Aloud means aloud (review R-5): the refusal stands in the
    // count line, not only in the absence of a finding.
    assert!(
        out.contains("refusal aloud: none"),
        "the none refusal is said in the count line:\n{out}"
    );

    // The withdrawal side of the door: a withdrawn contract's verify
    // is no live command (§2.12), so its trust line orphans too.
    let dir = sandbox("withdrawn");
    write(
        &dir,
        "keel.toml",
        &format!("[trust]\n\"echo ok\" = \"{}\"\n", "7d10fced96b3"),
    );
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel/contracts/ext-echo.md",
        "---\nverify: \"echo ok\"\nwithdrawn: \"the service is gone\"\n---\n\nWas a foreign promise (§2.8); withdrawn per §2.12.\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 1,
        "a withdrawn verify leaves its trust line orphaned:\n{out}"
    );
    assert!(
        out.contains("door opened in advance"),
        "the withdrawn contract's line is the same door:\n{out}"
    );

    // Second birth (review R-4): no door verdict over rubble. A
    // broken contract may hide the very command the record answers
    // to -- with unread documents the trust court does not judge,
    // and says so instead of inventing doors.
    let dir = sandbox("rubble");
    write(
        &dir,
        "keel.toml",
        "[trust]\n\"echo ok\" = \"7d10fced96b3\"\n",
    );
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel/contracts/ext-echo.md",
        "---\nverify \"echo ok\"\n---\n\nThe colon is gone -- the header does not read.\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "the broken document is the finding:\n{out}");
    assert!(
        !out.contains("door opened in advance"),
        "no door verdict over rubble (R-4):\n{out}"
    );
    assert!(
        out.contains("not judged"),
        "the skipped trust court says so aloud:\n{out}"
    );
}

/// proves: trust-recorded@b6bb57 -- holds §7.16's recording hand:
/// `keel trust` writes the fingerprints of untrusted commands as
/// [trust] lines, keeps the rest of keel.toml -- comments included
/// -- rewrites a crooked line of a live command, says what it did;
/// a second run says nothing is new and does not touch the file;
/// and check after the recording is silence. Without keel.toml the
/// command refuses aloud.
#[test]
fn trust_recorded() {
    let dir = sandbox("record");
    write(
        &dir,
        "keel.toml",
        "# the project's own gate\nci = \"cargo test\"\n",
    );
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel/contracts/ext-health.md",
        "---\nverify: \"curl -fsS https://svc.example/health\"\n---\n\nA foreign promise: the service answers (§2.8).\n",
    );

    let (out, err, code) = keel(&["trust", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the trust records:\n{out}");
    assert!(
        out.contains("\"cargo test\"") && out.contains(CARGO_TEST_FP),
        "the recorded ci line said aloud with its fingerprint:\n{out}"
    );
    assert!(
        out.contains("\"curl -fsS https://svc.example/health\"") && out.contains(CURL_FP),
        "the recorded verify line said aloud:\n{out}"
    );
    let toml = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        toml.contains("# the project's own gate"),
        "the comments live -- surgery only on [trust]:\n{toml}"
    );
    assert!(
        toml.contains(&format!("\"cargo test\" = \"{CARGO_TEST_FP}\""))
            && toml.contains(&format!(
                "\"curl -fsS https://svc.example/health\" = \"{CURL_FP}\""
            )),
        "the trust lines landed:\n{toml}"
    );

    // The second run: nothing new, the file untouched.
    let before = fs::read(dir.join("keel.toml")).unwrap();
    let (out, err, code) = keel(&["trust", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a quiet second run:\n{out}");
    assert!(
        out.contains("nothing new"),
        "nothing new said aloud:\n{out}"
    );
    assert_eq!(
        before,
        fs::read(dir.join("keel.toml")).unwrap(),
        "no writing when there is nothing to record"
    );

    // And the court is silent now.
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "check after the recording is silence:\n{out}");

    // A crooked line of a live command is rewritten by the run.
    let crooked = toml.replace(CARGO_TEST_FP, "000000000000");
    fs::write(dir.join("keel.toml"), &crooked).unwrap();
    let (out, err, code) = keel(&["trust", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the crooked line is rewritten:\n{out}");
    let toml = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        toml.contains(&format!("\"cargo test\" = \"{CARGO_TEST_FP}\""))
            && !toml.contains("000000000000"),
        "the fingerprint is true again:\n{toml}"
    );

    // Without keel.toml: a refusal aloud, nothing invented.
    let dir = sandbox("no-config");
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    let (out, err, code) = keel(&["trust", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "no config -- a refusal, not an invention:\n{out}");
    assert!(
        out.contains("nowhere to prepare"),
        "the refusal says why:\n{out}"
    );

    // Second birth (review R-1/R-2/R-3): the surgery must not
    // understand less TOML than the strict parser reads. A commented
    // [trust] header, a literal key and a collapse-twin are all
    // valid TOML -- after the run the file must still parse, the
    // command must stand as ONE canonical line, and the header
    // comment must live. Corruption with a success report is the
    // one forbidden outcome.
    let dir = sandbox("hard-toml");
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel.toml",
        &format!(
            "ci = \"cargo test\"\n\n[trust]  # recorded by hand\n'cargo test' = \"000000000000\"\n\"cargo  test\" = \"{CARGO_TEST_FP}\"\n"
        ),
    );
    let (out, err, code) = keel(&["trust", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the surgery understands its own TOML:\n{out}");
    let toml = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        toml.contains("# recorded by hand"),
        "the header comment lives:\n{toml}"
    );
    assert_eq!(
        toml.matches(CARGO_TEST_FP).count(),
        1,
        "one canonical line for one command -- twins consolidated:\n{toml}"
    );
    assert!(
        !toml.contains("000000000000"),
        "the crooked twin is gone:\n{toml}"
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the file still parses and the command is trusted:\n{out}");

    // A control character inside a verify command (hostile or
    // accidental) must not become a dead config: the line is written
    // escaped, reads back, and the command stands trusted.
    let dir = sandbox("bell");
    write(&dir, "keel.toml", "ci = \"none\"\n");
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(
        &dir,
        "keel/contracts/ext-bell.md",
        "---\nverify: \"run \u{0007} bell\"\n---\n\nA foreign promise with a bell inside (§2.8).\n",
    );
    let (out, err, code) = keel(&["trust", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a control character does not kill the run:\n{out}");
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the config parses after the bell and the command is trusted:\n{out}"
    );
    assert!(
        !out.contains("does not parse"),
        "no dead config with a success report behind it:\n{out}"
    );

    // CRLF endings survive the surgery: the promise is byte for
    // byte outside the [trust] block, on Windows files too.
    let dir = sandbox("crlf");
    write(&dir, "keel/waves/0050-w.md", &quiet_wave());
    write(&dir, "keel.toml", "# gate\r\nci = \"cargo test\"\r\n");
    let (out, err, code) = keel(&["trust", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the CRLF file is recorded too:\n{out}");
    let toml = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        toml.contains("# gate\r\n"),
        "the untouched line keeps its CRLF ending:\n{toml:?}"
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the CRLF config parses and is trusted:\n{out}");
}
