//! Scenario tests of wave 0009-reviewer-package: the §9.9 package
//! assembled by the machine -- Why, scenarios with revisions,
//! caveats, chore reasons, the full branch diff, and the three
//! mandatory lists (§4.6 drift, §10.7 map, §5.7 impact).
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0009r-{}-{name}", std::process::id()));
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
            block.push_str(&format!("  {cut}: \"n/a for the review sandbox\"\n"));
        }
    }
    block
}

/// proves: review-package-built@a76d98 -- holds §9.9: on a branch
/// named as a wave the package carries every part under its own
/// header -- the Why verbatim, each scenario with its revision, the
/// caveat paragraphs, the chore reasons, the full branch diff --
/// and any other branch is a refusal aloud (§8.2).
#[test]
fn review_package_built() {
    let dir = sandbox("package");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the base stands"]);
    git(&dir, &["checkout", "-q", "-b", "0060-w"]);
    write(
        &dir,
        "keel/waves/0060-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n  tidy:\n    chore: \"the docs put in order\"\n    files: [README.md]\n{}---\n\n## Why\n\nwhy the wave rides, verbatim words\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work of t.\n\nZasterezhennia: this corner we deliberately leave, aloud.\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the wave rides its branch"]);

    let (out, err, code) = keel(&["review", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the package assembles:\n{out}");
    assert!(
        out.contains("why the wave rides, verbatim words"),
        "the Why rides verbatim:\n{out}"
    );
    let s_rev = keel::rev::text_rev("body of s\n");
    assert!(
        out.contains(&format!("s@{s_rev}")),
        "the scenario carries its revision:\n{out}"
    );
    assert!(
        out.contains("Zasterezhennia: this corner we deliberately leave, aloud."),
        "the caveat paragraph rides in the package:\n{out}"
    );
    assert!(
        out.contains("the docs put in order"),
        "the chore reason is named:\n{out}"
    );
    assert!(
        out.contains("diff --git") && out.contains("0060-w.md"),
        "the full branch diff rides along:\n{out}"
    );

    // Any other branch: a refusal aloud -- the package does not
    // guess which wave it is for.
    git(&dir, &["checkout", "-q", "main"]);
    let (out, err, code) = keel(&["review", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "no wave branch -- no package:\n{out}");
    assert!(
        out.contains("not named as a wave"),
        "the refusal says why and hints:\n{out}"
    );
}

