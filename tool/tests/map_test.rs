//! Scenario tests of wave 0007-quality-map, transform map-command:
//! the quality map of §10.7 -- every cut mapped to what closes it or
//! how it is decided -- drawn through the binary.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0007m-{}-{name}", std::process::id()));
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a for the map sandbox\"\n"));
        }
    }
    block
}

/// proves: map-drawn-per-wave@90e3aa -- holds §10.7/§9.9: on a
/// branch named as a wave the map draws that wave's forty rows in
/// vocabulary order -- covered cuts name the scenario with an honest
/// proof word, decided cuts quote the reason verbatim, a dead
/// cover's cut shows the answer that remains -- and the first line
/// names which map this is and why.
#[test]
fn map_drawn_per_wave() {
    let dir = sandbox("perwave");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "keel/waves/0040-w.md",
        &format!(
            "---\nscenarios:\n  proven-one: {{covers: [functional.correctness]}}\n  bare-one: {{covers: [performance.capacity]}}\n  gone:\n    covers: [security.integrity]\n    withdrawn: \"folded\"\ntransforms:\n  t:\n    implements: [proven-one, bare-one]\n    files: [src/lib.rs]\n{}---\n\n## scenario: proven-one\n\nbody of proven-one\n\n## scenario: bare-one\n\nbody of bare-one\n\n## scenario: gone\n\nold body\n",
            all_decided_except(&["functional.correctness", "performance.capacity"])
        ),
    );
    let proven_rev = keel::rev::text_rev("body of proven-one\n");
    write(
        &dir,
        "tests/t_test.rs",
        &format!("/// proves: proven-one@{proven_rev}\n#[test]\nfn holds_proven() {{}}\n"),
    );
    git(&dir, &["init", "-q", "-b", "0040-w"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the wave rides its branch"]);

    let (out, err, code) = keel(&["map", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the map draws:\n{out}");
    assert!(
        out.contains("0040-w"),
        "the first lines name which map this is:\n{out}"
    );
    assert!(
        out.contains("functional.correctness")
            && out.contains("\"proven-one\"")
            && out.contains("proven"),
        "a covered cut names its scenario with the proof word:\n{out}"
    );
    assert!(
        out.contains("performance.capacity")
            && out.contains("\"bare-one\"")
            && out.contains("not yet proven"),
        "an untagged scenario gets the honest word, not green:\n{out}"
    );
    assert!(
        out.contains("n/a for the map sandbox"),
        "a decided cut quotes the reason verbatim:\n{out}"
    );
    assert!(
        out.contains("security.integrity") && !out.contains("\"gone\""),
        "a dead cover's cut shows the answer that remains (§2.12):\n{out}"
    );
    // Forty rows: every cut of the vocabulary appears.
    for cut in keel::graph::cuts() {
        assert!(out.contains(cut), "the map has a row for {cut}:\n{out}");
    }
}
