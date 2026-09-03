//! Scenario tests of wave 0016-drifted-records, transform
//! rewrite-hand: `keel rev --write` rewrites the drifted records of
//! OPEN waves onto the current contract revisions -- by name, header
//! surgery only -- and leaves the closed to history's court (§5.6).
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0016w-{}-{name}", std::process::id()));
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

/// proves: stale-refs-rewritten@d9b9cd -- holds the NEW-CONCEPT
/// rev --write row and §5.6: the open wave's proves and contracts
/// records are rewritten onto the current revision by name, old and
/// new in the line; section bodies stay untouched; the closed wave
/// stays byte-identical with the "leaving" word; a second run says
/// nothing has drifted; the rewritten file still reads through the
/// strict parser.
#[test]
fn stale_refs_rewritten() {
    let dir = sandbox("drift");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    let v1_text = "---\nmodule: toy\nexports:\n  - \"pub fn one()\"\n---\n\nthe first promise\n";
    write(&dir, "keel/contracts/c.md", v1_text);
    let v1 = keel::rev::text_rev(v1_text);
    write(
        &dir,
        "keel/waves/0300-open.md",
        &format!(
            "---\nscenarios:\n  s: {{proves: c@{v1}, covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    contracts: [c@{v1}]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t\n"
        ),
    );
    write(
        &dir,
        "keel/waves/0301-done.md",
        &format!(
            "---\nscenarios:\n  s2: {{proves: c@{v1}, covers: [functional.completeness]}}\ntransforms:\n  t:\n    implements: [s2]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s2\n\nbody of s2\n"
        ),
    );
    let s2_rev = keel::rev::text_rev("body of s2\n");
    write(
        &dir,
        "tests/s2_test.rs",
        &format!("/// proves: s2@{s2_rev}\n#[test]\nfn holds_s2() {{}}\n"),
    );
    write(&dir, "keel/reviews/0301-done.md", "# Рецензія\n\nok\n");

    // The contract grows: the records of 0300-open drift.
    let v2_text = format!("{v1_text}\nand a grown one\n");
    write(&dir, "keel/contracts/c.md", &v2_text);
    let v2 = keel::rev::text_rev(&v2_text);
    let closed_before = fs::read_to_string(dir.join("keel/waves/0301-done.md")).unwrap();

    let (out, err, code) = keel(&["rev", "--write", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the rewrite is an answer:\n{out}");
    let open = fs::read_to_string(dir.join("keel/waves/0300-open.md")).unwrap();
    assert!(
        open.contains(&format!("c@{v2}")) && !open.contains(&format!("c@{v1}")),
        "both records of the open wave now hold the current revision:\n{open}"
    );
    assert!(
        open.contains("body of s") && open.contains("the work of t"),
        "the surgery never touches a section body:\n{open}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("keel/waves/0301-done.md")).unwrap(),
        closed_before,
        "the closed wave stays byte-identical (§5.6):\n{out}"
    );
    assert!(
        out.contains("0300-open") && out.contains(&v1) && out.contains(&v2),
        "the rewrite is named with the old and the new revision:\n{out}"
    );
    assert!(
        out.contains("0301-done") && out.contains("leaving"),
        "the closed wave gets the leaving word, never a byte:\n{out}"
    );

    // The rewritten file still reads strictly, and a second run
    // finds nothing drifted.
    let (out, err, code) = keel(&["rev", "--write", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the second run is green:\n{out}");
    assert!(
        out.contains("nothing has drifted"),
        "the second run says nothing has drifted:\n{out}"
    );
    let (out, err, _) = keel(&["rev", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        !out.contains("червоне") && !out.contains("red"),
        "the rewritten header reads through the strict parser:\n{out}"
    );
}
