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
    // A quiet wave on main keeps keel/waves/ tracked, so the main
    // checkout still scans and the refusal speaks about the branch.
    write(
        &dir,
        "keel/waves/0059-base.md",
        &format!(
            "---\nscenarios:\n  base: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [base]\n    files: [src/lib.rs]\n{}---\n\n## scenario: base\n\nbody of base\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
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

/// proves: review-lists-drawn@36b795 -- holds the three mandatory
/// lists of §9.9: drift names the file added to scope after the
/// anchor and the anchor itself aloud (§4.6); the quality map is
/// the wave view (§10.7); contract impact names the old and new
/// revisions and every holder with its recorded revision (§5.7);
/// and where git does not testify, the list says a word instead of
/// posing as empty.
#[test]
fn review_lists_drawn() {
    let dir = sandbox("lists");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/contracts/ext-c.md",
        "---\nverify: \"echo c\"\n---\n\nold words of the promise\n",
    );
    let old_rev =
        keel::rev::text_rev(&fs::read_to_string(dir.join("keel/contracts/ext-c.md")).unwrap());
    write(
        &dir,
        "keel/waves/0061-old.md",
        &format!(
            "---\nscenarios:\n  old-s:\n    proves: ext-c@{old_rev}\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [old-s]\n    contracts: [ext-c@{old_rev}]\n    files: [src/lib.rs]\n{}---\n\n## scenario: old-s\n\nbody of old-s\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the base stands"]);
    git(&dir, &["checkout", "-q", "-b", "0062-w"]);
    write(
        &dir,
        "keel/waves/0062-w.md",
        &format!(
            "---\nscenarios:\n  new-s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [new-s]\n    files: [src/a.rs]\ndecisions:\n  functional.completeness: \"overridden below\"\n{}---\n\n## Why\n\nthe second wave\n\n## scenario: new-s\n\nbody of new-s\n",
            {
                let mut block = String::new();
                for cut in keel::graph::cuts() {
                    if *cut != "functional.correctness" && *cut != "functional.completeness" {
                        block.push_str(&format!("  {cut}: \"n/a for the review sandbox\"\n"));
                    }
                }
                block
            }
        ),
    );
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "the wave file is born"]);
    // After the anchor: a file joins the scope, and the held
    // contract's text changes.
    let grown = fs::read_to_string(dir.join("keel/waves/0062-w.md"))
        .unwrap()
        .replace("files: [src/a.rs]", "files: [src/a.rs, src/b.rs]");
    write(&dir, "keel/waves/0062-w.md", &grown);
    write(
        &dir,
        "keel/contracts/ext-c.md",
        "---\nverify: \"echo c\"\n---\n\nnew words of the promise\n",
    );
    let new_rev =
        keel::rev::text_rev(&fs::read_to_string(dir.join("keel/contracts/ext-c.md")).unwrap());
    git(&dir, &["add", "."]);
    git(
        &dir,
        &["commit", "-q", "-m", "scope grows, the contract changes"],
    );

    let (out, err, code) = keel(&["review", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the lists assemble:\n{out}");
    // Drift (§4.6): the added file and the anchor aloud.
    assert!(
        out.contains("src/b.rs"),
        "the drifted file is named:\n{out}"
    );
    assert!(
        !out.lines()
            .any(|l| l.contains("src/a.rs") && l.contains("drift")),
        "the planned file is not drift:\n{out}"
    );
    assert!(out.contains("anchor"), "the anchor is named aloud:\n{out}");
    // The quality map (§10.7): the wave view rides inside.
    assert!(
        out.contains("0062-w") && out.contains("functional.completeness"),
        "the map rides in the package:\n{out}"
    );
    // Impact (§5.7): old and new revisions, every holder by name
    // with its recorded revision.
    assert!(
        out.contains("ext-c") && out.contains(&old_rev) && out.contains(&new_rev),
        "the changed contract with both revisions:\n{out}"
    );
    assert!(
        out.contains("0061-old") && out.contains("old-s"),
        "the holders named:\n{out}"
    );

    // Where git does not testify: a word, not an empty list. A
    // truncated history cannot prove the anchor is the true first
    // commit of the wave file.
    fs::write(
        dir.join(".git/shallow"),
        "0000000000000000000000000000000000000000\n",
    )
    .unwrap();
    let (out, err, code) = keel(&["review", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the package still assembles:\n{out}");
    assert!(
        out.contains("not verified"),
        "the drift says the word instead of posing as empty:\n{out}"
    );
}
