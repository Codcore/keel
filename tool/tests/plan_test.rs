//! Scenario tests of wave 0013-planning-skeletons, transform
//! skeleton-hand: `keel plan` and `keel new contract` hand the form
//! of a plan -- deliberately red scaffolding, the §8.2 branches, the
//! §10.2 author's pass and the §8.5/§8.8 number court -- and never
//! the content.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0013p-{}-{name}", std::process::id()));
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

/// proves: plan-skeleton-born@11d9f8 -- holds §10.2/§8.2/§8.5/§8.8:
/// the born skeleton is a full-form wave file said aloud to be
/// deliberately red (check leads by the §3.3 refusal), the print
/// names the branches and the forty-cut author's pass, a name
/// without a number refuses, a taken number refuses with the next
/// free one, a taken branch number refuses too, and nothing existing
/// is ever overwritten.
#[test]
fn plan_skeleton_born() {
    let dir = sandbox("wavebirth");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    let (out, err, code) = keel(&["plan", "0100-first-steps", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a birth is an answer:\n{out}");
    assert!(
        dir.join("keel/waves/0100-first-steps.md").is_file(),
        "the skeleton file is born:\n{out}"
    );
    assert!(
        out.contains("keel/waves/0100-first-steps.md") && out.contains("deliberately red"),
        "the print names the file and says the scaffolding is red (§3.3):\n{out}"
    );
    assert!(
        out.contains("plan/0100-first-steps") && out.contains("\"0100-first-steps\""),
        "the §8.2 branches are named -- plan and work:\n{out}"
    );
    assert!(
        out.contains("forty cuts"),
        "the §10.2 author's pass is the reminder:\n{out}"
    );
    assert!(
        out.contains("keel check"),
        "the next word points at the fullness court (§8.3):\n{out}"
    );

    // The born skeleton is deliberately red: check leads by §3.3.
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "the skeleton is red until it is a plan:\n{out}");
    assert!(
        out.contains("leans on nothing"),
        "check leads the author by the §3.3 refusal:\n{out}"
    );

    // Nothing existing is ever overwritten.
    let (out, err, code) = keel(&["plan", "0100-first-steps", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "an existing file refuses:\n{out}");
    assert!(
        out.contains("already exists"),
        "the refusal says the file stands:\n{out}"
    );

    // A taken number refuses with the next free one (§8.8).
    let (out, err, code) = keel(&["plan", "0100-other-name", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "a taken number refuses:\n{out}");
    assert!(
        out.contains("0101"),
        "the next free number rides in the instead (§8.8):\n{out}"
    );

    // A name without a number refuses (§8.5).
    let (out, err, code) = keel(&["plan", "no-number-here", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "a name without a number refuses:\n{out}");
    assert!(
        out.contains("starts with no number"),
        "the §8.5 word is said:\n{out}"
    );

    // A crooked slug refuses (§1.2).
    let (out, err, code) = keel(&["plan", "0102_Bad_Slug", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "a crooked slug refuses (§1.2):\n{out}");

    // A number taken by a branch refuses too (§8.8 reads all
    // branches, not only main).
    let dir = sandbox("branchnumber");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(&dir, "seed.txt", "seed\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "seed"]);
    git(&dir, &["branch", "0200-taken-elsewhere"]);
    let (out, err, code) = keel(&["plan", "0200-mine", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 2, "a branch holds the number (§8.8):\n{out}");
    assert!(
        out.contains("0201"),
        "the next free number is counted across branches:\n{out}"
    );
}
