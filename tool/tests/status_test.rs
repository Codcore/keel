//! Scenario tests of wave 0012-loop-stages, transform stage-eye:
//! `keel status` tells where we stand -- a line per wave with weight
//! and structural stage, waiting plans by branch name, and the aloud
//! word that the battery did not run.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0012s-{}-{name}", std::process::id()));
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

/// proves: status-tells-where@760ecb -- holds §6.5/§6.8/§9.2: every
/// wave gets a line with its weight and structural stage (closed by
/// merge, approved-not-started, in progress with lacks by name), a
/// ready plan is named as awaiting its branch, the report says aloud
/// that the battery did not run, and the branch line says where we
/// ourselves stand.
#[test]
fn status_tells_where() {
    let dir = sandbox("threestages");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    // 0060-light: one chore transform, no scenarios, no contracts --
    // the light weight of §6.8, closed by the fact of merge.
    write(
        &dir,
        "keel/waves/0060-light.md",
        "---\ntransforms:\n  tidy:\n    chore: \"a tidy-up without a promise\"\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n",
    );
    // 0061-plan: a live scenario with no tag, depending on the light
    // wave -- approved, not started, and awaiting its start.
    write(
        &dir,
        "keel/waves/0061-plan.md",
        "---\ndepends_on: [0060-light]\nscenarios:\n  b: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [b]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: b\n\nbody of b\n",
    );
    // 0062-work: the scenario is proven by a matching tag, yet the
    // review report is missing -- in progress, the lack by name.
    write(
        &dir,
        "keel/waves/0062-work.md",
        "---\nscenarios:\n  a: {covers: [functional.completeness]}\ntransforms:\n  t:\n    implements: [a]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: a\n\nbody of a\n",
    );
    let a_rev = keel::rev::text_rev("body of a\n");
    write(
        &dir,
        "tests/a_test.rs",
        &format!("/// proves: a@{a_rev}\n#[test]\nfn holds_a() {{}}\n"),
    );

    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a readable project is a green overview:\n{out}");
    assert!(
        out.contains("0060-light") && out.contains("light") && out.contains("closed by merge"),
        "the light wave carries its weight and merge-fact stage (§6.8):\n{out}"
    );
    assert!(
        out.contains("0061-plan") && out.contains("approved, not started"),
        "the plan wave is approved and not started, never red (§6.5):\n{out}"
    );
    assert!(
        out.contains("awaits its start") && out.contains("the branch \"0061-plan\""),
        "the ready plan is named awaiting, with its branch name (§8.2):\n{out}"
    );
    assert!(
        out.contains("0062-work") && out.contains("in progress"),
        "the started wave is in progress:\n{out}"
    );
    assert!(
        out.contains("the review file"),
        "the lack is named next to the in-progress wave (§9.9):\n{out}"
    );
    assert!(
        out.contains("the battery was not run"),
        "the structural border is said aloud -- close and hook judge green (§9.2):\n{out}"
    );
    assert!(
        out.contains("git named no branch"),
        "a sandbox without git gets the honest branch word, never a guess:\n{out}"
    );
}

/// proves: status-tells-where@760ecb -- the second birth out of
/// review 0012 (R-2/R-6/R-9): a namesake tag holding another wave's
/// legal revision does not turn a plan into "in progress" nor hide
/// it from the awaiting list; a light wave on its own branch is not
/// painted "closed by merge" before any merge; and a branch named
/// after a wave whose document refused is said so, not "no wave".
#[test]
fn status_tells_where_second_birth() {
    // A closed wave and a plan wave sharing the scenario name "s":
    // the old tag is the old wave's proof, never the plan's start.
    let dir = sandbox("namesake");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    write(
        &dir,
        "keel/waves/0200-old.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\nbody of s\n",
    );
    let old_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/s_test.rs",
        &format!("/// proves: s@{old_rev}\n#[test]\nfn holds_s() {{}}\n"),
    );
    write(&dir, "keel/reviews/0200-old.md", "# Рецензія\n\nok\n");
    write(
        &dir,
        "keel/waves/0201-new.md",
        "---\ndepends_on: [0200-old]\nscenarios:\n  s: {covers: [functional.completeness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n\n## scenario: s\n\na different body of s\n",
    );
    let (out, err, _) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("0201-new — full, approved, not started"),
        "the namesake's foreign proof does not start the plan (R-2, 0011 R-9 school):\n{out}"
    );
    assert!(
        out.contains("awaits its start") && out.contains("the branch \"0201-new\""),
        "the plan with closed dependencies still awaits by name (R-2):\n{out}"
    );

    // A light wave on its own branch: no merge happened, so the line
    // does not claim its fact -- the wave rides, it is not closed.
    let dir = sandbox("lightown");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    write(
        &dir,
        "keel/waves/0900-l.md",
        "---\ntransforms:\n  tidy:\n    chore: \"a tidy-up without a promise\"\n    files: [src/lib.rs]\n---\n\n## Why\n\nwhy words\n",
    );
    git(&dir, &["init", "-q", "-b", "0900-l"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "state"]);
    let (out, err, _) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("0900-l — light, riding this branch"),
        "a light wave on its own branch rides, no merge fact is guessed (R-6):\n{out}"
    );
    assert!(
        !out.contains("light, closed by merge"),
        "no merge happened -- the closed-by-merge word would be a guess (R-6):\n{out}"
    );

    // A branch named after a wave whose document refused: the branch
    // line says that, never "named as no wave".
    let dir = sandbox("brokenbranch");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    write(&dir, "keel/waves/0101-broken.md", "no header at all\n");
    git(&dir, &["init", "-q", "-b", "0101-broken"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "state"]);
    let (out, err, code) = keel(&["status", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "the refusal row reddens the exit:\n{out}");
    assert!(
        out.contains("whose document refused"),
        "the branch is named as the broken wave's, not as no wave (R-9):\n{out}"
    );
}
