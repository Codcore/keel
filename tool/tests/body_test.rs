//! Scenario tests of wave 0011-header-vs-body, transform
//! body-court: §7.7's other half -- the set of names in the header
//! equals the set of section headings in the body, both ways.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0011b-{}-{name}", std::process::id()));
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a for the body sandbox\"\n"));
        }
    }
    block
}

/// proves: body-matches-header@13b919 -- holds §7.7 both ways: a
/// header transform with no body section is a finding by name; a
/// body section declared by no header entry is an orphan finding by
/// name; matching sets are silence; and the "not yet checked" line
/// has left the report -- §7.8's border stands in its place.
#[test]
fn body_matches_header() {
    let dir = sandbox("mismatch");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/waves/0090-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n  silent-t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## scenario: stray\n\nan orphan scenario section\n\n## transform: t\n\nthe work of t\n\n## transform: stray-t\n\nan orphan transform section\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "mismatched sets are findings (§7.7):\n{out}");
    assert!(
        out.contains("\"silent-t\"") && out.contains("no body section"),
        "the bodiless transform is named:\n{out}"
    );
    assert!(
        out.contains("\"stray\"") && out.contains("orphan"),
        "the orphan scenario section is named:\n{out}"
    );
    assert!(
        out.contains("\"stray-t\"") && out.contains("orphan"),
        "the orphan transform section is named:\n{out}"
    );

    // Matching sets: silence -- and the unchecked line is gone for
    // good, §7.8's border standing in its place.
    let dir = sandbox("matched");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/waves/0091-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "matching sets are silence:\n{out}");
    assert!(
        !out.contains("not yet checked"),
        "the unchecked line has left the report (§7.7 was its last row):\n{out}"
    );
    assert!(
        out.contains("green form is not yet meaning"),
        "§7.8's border stands in its place:\n{out}"
    );
}

/// proves: body-matches-header@13b919 -- the second birth out of
/// review 0011 (R-2/R-6/R-7): the scenario side of §7.7 runs
/// without an adapter too, so "both ways" is true for every
/// project; a heading that spells the very word without its space
/// is a finding, not free prose; and a duplicated transform
/// section is not guessed between.
#[test]
fn body_matches_header_second_birth() {
    let dir = sandbox("adapterless");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/waves/0092-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\n  ghost: {{covers: [performance.capacity]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n\n## transform:x\n\na near-miss heading\n\n## transform: t\n\nthe work of t\n\n## transform: t\n\nthe work of t again\n",
            all_decided_except(&["functional.correctness", "performance.capacity"])
        ),
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 1,
        "all three defects are findings without an adapter too:\n{out}"
    );
    assert!(
        out.contains("ghost"),
        "the declared scenario with no body refuses adapter-free (R-2):\n{out}"
    );
    assert!(
        out.contains("transform:x") && out.contains("not recognised"),
        "the near-miss heading is named, not free prose (R-6):\n{out}"
    );
    assert!(
        out.contains("\"t\"") && out.contains("more than once"),
        "the duplicated transform section is not guessed between (R-7):\n{out}"
    );
}
