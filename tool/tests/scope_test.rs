//! Scenario tests of wave 0004-scope-and-links, transform scope-checks:
//! chapter 4 judged against real git repositories built in sandboxes --
//! the tool asks git the same way it will in the field.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0004s-{}-{name}", std::process::id()));
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

/// git as the command of the system, exactly how the tool itself will
/// call it; sandbox commits need an identity and no signing, whatever
/// the host machine thinks.
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

/// Waves in these sandboxes obey §10.3 like any wave: a full
/// decisions block, minus the cuts the fixture covers itself.
fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

/// A wave whose one transform declares exactly the given files list
/// (yaml lines, six spaces deep).
fn wave_declaring(files_yaml: &str) -> String {
    format!(
        "---\nscenarios:\n  s:\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [s]\n    files:\n{files_yaml}{}---\n\n## Why\n\na scope sandbox wave\n\n## scenario: s\n\nbody\n",
        all_decided_except(&["functional.correctness"])
    )
}

/// proves: scope-both-ways@f77c1b -- holds §4.4-§4.6 and §4.8: on a
/// branch named as the wave, a touched file outside the declared list
/// and a declared file left untouched are both findings by name;
/// keel/ stays outside the comparison; and the report says which base
/// it took -- the merge-base with main, or, where main never existed,
/// the first commit of the branch.
#[test]
fn scope_both_ways() {
    let dir = sandbox("bothways");
    git(&dir, &["init", "-q", "-b", "main"]);
    write(&dir, "lib/a.txt", "one\n");
    write(&dir, "lib/b.txt", "two\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "base"]);

    git(&dir, &["checkout", "-q", "-b", "0005-scope-w"]);
    // The wave itself is born on its branch -- keel/ is exempt (§4.8),
    // or every wave would drift by existing.
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/a.txt\n      - lib/b.txt\n"),
    );
    write(&dir, "lib/a.txt", "one, changed\n"); // declared and touched
    write(&dir, "lib/c.txt", "drift\n"); // touched, never declared
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "work"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "both sides make the run red:\n{out}");
    assert!(
        out.contains("scope: branch \"0005-scope-w\""),
        "the comparison said aloud:\n{out}"
    );
    assert!(
        out.contains("merge-base with main"),
        "the base named:\n{out}"
    );
    assert!(
        out.contains("\"lib/c.txt\"") && out.contains("no transform"),
        "drift named by file (§4.6):\n{out}"
    );
    assert!(
        out.contains("\"lib/b.txt\" is untouched"),
        "the declared-yet-untouched file named (§4.4):\n{out}"
    );
    assert!(
        !out.contains("\"lib/a.txt\""),
        "declared and touched is silence:\n{out}"
    );
    assert!(
        !out.contains("\"keel/waves/0005-scope-w.md\""),
        "keel/ outside the comparison (§4.8):\n{out}"
    );

    // Where main never existed, the base falls back to the first
    // commit of the branch -- and the report says what it took.
    let dir = sandbox("firstbase");
    git(&dir, &["init", "-q", "-b", "0005-scope-w"]);
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/a.txt\n      - lib/b.txt\n"),
    );
    write(&dir, "lib/a.txt", "one\n");
    write(&dir, "lib/b.txt", "two\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "everything at once"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "declared files the branch never touched are findings:\n{out}"
    );
    assert!(
        out.contains("the first commit of the branch"),
        "the fallback base said aloud:\n{out}"
    );
    assert!(
        out.contains("\"lib/a.txt\" is untouched") && out.contains("\"lib/b.txt\" is untouched"),
        "both named:\n{out}"
    );

    // Second birth (work-check finding): a declared file with a
    // non-ASCII name must compare as itself -- git's default path
    // quoting ("\321\204...") must not turn an honest Ukrainian
    // filename into false drift plus false untouched.
    let dir = sandbox("cyrillic");
    git(&dir, &["init", "-q", "-b", "main"]);
    write(&dir, "lib/файл.txt", "one\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0005-scope-w"]);
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/файл.txt\n"),
    );
    write(&dir, "lib/файл.txt", "one, changed\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "work"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a declared and touched non-ASCII file is silence:\n{out}"
    );

    // Second birth (work-check finding): on a fresh clone the local
    // main often does not exist -- only origin/main does; the base
    // must come from there, not slide to the first commit and flood
    // the report with false drift.
    let dir = sandbox("originmain");
    git(&dir, &["init", "-q", "-b", "main"]);
    write(&dir, "lib/a.txt", "one\n");
    write(&dir, "lib/old.txt", "history\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0005-scope-w"]);
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/a.txt\n"),
    );
    write(&dir, "lib/a.txt", "one, changed\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "work"]);
    // The clone shape: origin/main knows the base, local main is gone.
    git(&dir, &["update-ref", "refs/remotes/origin/main", "main"]);
    git(&dir, &["branch", "-q", "-D", "main"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "the base comes from origin/main, no drift flood:\n{out}"
    );
    assert!(
        out.contains("merge-base with main"),
        "the base still named as main's:\n{out}"
    );

    // Second birth (review R-2): a rename with both names declared is
    // honest work -- and the verdict must not depend on whatever
    // diff.renames the host machine fancies.
    let dir = sandbox("renamed");
    git(&dir, &["init", "-q", "-b", "main"]);
    write(&dir, "lib/old.txt", "the text\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0005-scope-w"]);
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/old.txt\n      - lib/new.txt\n"),
    );
    git(&dir, &["mv", "lib/old.txt", "lib/new.txt"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "rename"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a declared rename is green -- the old name went, the new one came:\n{out}"
    );

    // Second birth (review R-3): "keel/ does not enter the comparison"
    // holds on the untouched side too -- a declared keel/ file the
    // branch never touched is not a finding.
    let dir = sandbox("keeldecl");
    git(&dir, &["init", "-q", "-b", "main"]);
    write(&dir, "lib/a.txt", "one\n");
    write(&dir, "keel/notes.md", "furniture\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0005-scope-w"]);
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/a.txt\n      - keel/notes.md\n"),
    );
    write(&dir, "lib/a.txt", "one, changed\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "work"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "keel/ stays outside, both ways (§4.8):\n{out}");
    assert!(
        !out.contains("\"keel/notes.md\""),
        "the declared keel/ file is not judged:\n{out}"
    );
}

/// proves: one-new-in-counted@327a30 -- holds §4.1: `one new in
/// <dir>/` counts strictly. Zero new files in the directory is a
/// finding, two is a finding naming both, exactly one is silence --
/// the count is fixed, there is no glob liberty.
#[test]
fn one_new_in_counted() {
    let branch_with = |name: &str, files_yaml: &str, extra: &[(&str, &str)]| -> PathBuf {
        let dir = sandbox(name);
        git(&dir, &["init", "-q", "-b", "main"]);
        write(&dir, "lib/seed.txt", "seed\n");
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-q", "-m", "base"]);
        git(&dir, &["checkout", "-q", "-b", "0005-scope-w"]);
        write(
            &dir,
            "keel/waves/0005-scope-w.md",
            &wave_declaring(files_yaml),
        );
        for (rel, text) in extra {
            write(&dir, rel, text);
        }
        git(&dir, &["add", "."]);
        git(&dir, &["commit", "-q", "-m", "work"]);
        dir
    };
    let one_line = "      - one new in priv/migrations/\n";

    // Zero new files where exactly one was promised.
    let dir = branch_with("newzero", one_line, &[]);
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "zero is a finding:\n{out}");
    assert!(
        out.contains("no new file appeared in \"priv/migrations/\""),
        "the empty promise named:\n{out}"
    );

    // Exactly one -- silence, and the whole run is green end to end.
    let dir = branch_with(
        "newone",
        one_line,
        &[("priv/migrations/001.sql", "create table t;\n")],
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "exactly one new file passes:\n{out}");
    assert!(
        out.contains("scope: branch \"0005-scope-w\""),
        "compared, not skipped:\n{out}"
    );
    assert!(
        !out.contains("priv/migrations/001.sql"),
        "the matched file is no drift:\n{out}"
    );

    // Two new files where exactly one was promised.
    let dir = branch_with(
        "newtwo",
        one_line,
        &[
            ("priv/migrations/001.sql", "create table t;\n"),
            ("priv/migrations/002.sql", "drop table t;\n"),
        ],
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "two is a finding:\n{out}");
    assert!(
        out.contains("more than one new file in \"priv/migrations/\""),
        "the broken count named:\n{out}"
    );
    assert!(
        out.contains("priv/migrations/001.sql") && out.contains("priv/migrations/002.sql"),
        "both files named:\n{out}"
    );

    // Second birth (review R-1): §4.1 says "need two -- write two
    // lines". Two lines over the same directory and two new files
    // must meet in green; today every line judges the full count and
    // the truthful author gets two findings and no green state at all.
    let two_lines =
        "      - one new in priv/migrations/\n      - one new in priv/migrations/\n";
    let dir = branch_with(
        "newtwolines",
        two_lines,
        &[
            ("priv/migrations/001.sql", "create table t;\n"),
            ("priv/migrations/002.sql", "drop table t;\n"),
        ],
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "two lines, two files -- the counts meet:\n{out}");

    // And a mismatch against several lines is one counted finding.
    let dir = branch_with(
        "newtwoone",
        two_lines,
        &[("priv/migrations/001.sql", "create table t;\n")],
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "one file against two lines is a finding:\n{out}");
    assert!(
        out.contains("promise 2 new files in \"priv/migrations/\"")
            && out.contains("the branch adds 1"),
        "the counts named:\n{out}"
    );
}

/// proves: scope-honest-when-unknown@7af274 -- holds the honesty rule
/// of the wave: a branch not named as any wave (like the bootstrap
/// session branch this tool is built on), or a directory git does not
/// serve, gets "scope not compared" with the reason aloud -- neither
/// a finding nor a red exit; green is not painted over the unverified.
#[test]
fn scope_honest_when_unknown() {
    // A branch that is no wave: the declared file is touched and a
    // stranger file too, yet nothing is judged -- only said.
    let dir = sandbox("notwave");
    git(&dir, &["init", "-q", "-b", "main"]);
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/a.txt\n"),
    );
    write(&dir, "lib/a.txt", "one\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "just-work"]);
    write(&dir, "lib/c.txt", "stranger\n");
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "work"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "not compared is not red:\n{out}");
    assert!(
        out.contains("scope not compared: branch \"just-work\""),
        "the deviation named with the branch:\n{out}"
    );
    assert!(
        !out.contains("is untouched") && !out.contains("no transform"),
        "no scope findings where scope was not judged:\n{out}"
    );

    // No git at all: the same honesty, the other reason.
    let dir = sandbox("nogit");
    write(
        &dir,
        "keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/a.txt\n"),
    );

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "no git is not red either:\n{out}");
    assert!(
        out.contains("scope not compared: git"),
        "the reason is git, said aloud:\n{out}"
    );

    // Second birth (review R-4): a keel root that is not the top of
    // its own git tree -- a subdirectory inside some parent repo --
    // must not be judged by the parent's branch: the parent's paths
    // and the root's declared names would never meet, and every
    // verdict would be a lie. Honesty instead: not compared, aloud.
    let parent = sandbox("parenttop");
    git(&parent, &["init", "-q", "-b", "main"]);
    write(
        &parent,
        "proj/keel/waves/0005-scope-w.md",
        &wave_declaring("      - lib/a.txt\n"),
    );
    write(&parent, "proj/lib/a.txt", "one\n");
    git(&parent, &["add", "."]);
    git(&parent, &["commit", "-q", "-m", "base"]);
    git(&parent, &["checkout", "-q", "-b", "0005-scope-w"]);
    write(&parent, "proj/lib/a.txt", "one, changed\n");
    git(&parent, &["add", "."]);
    git(&parent, &["commit", "-q", "-m", "work"]);

    let root = parent.join("proj");
    fs::create_dir_all(root.join("keel/contracts")).unwrap();
    let (out, _err, code) = keel(&["check", root.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a foreign git above the root judges nothing:\n{out}"
    );
    assert!(
        out.contains("scope not compared: git"),
        "the foreign tree named as a git reason, aloud:\n{out}"
    );
}
