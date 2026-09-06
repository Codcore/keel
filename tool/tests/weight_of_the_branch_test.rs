//! Scenario test of wave 0052: the weight is read from the branch too.
//!
//! The global review of 2026-09-06 (methodology R-3, R-4, R-5) measured
//! the weight blind to the branch: a light wave whose branch changed a
//! contract rode to one PR; a wave of two chore transforms was called
//! light by status and full by nobody; a chore wave on a branch main
//! never saw was "closed by the fact of merge" -- citing §2.11 where
//! §6.5 speaks.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn keel(dir: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
        .output()
        .unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

fn decisions_except(covered: &[&str]) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// A rust crate: Cargo.toml, src/lib.rs, tests/ -- and the frame of
/// the methodology with the wave text given.
fn crate_with(name: &str, adapter: &str, wave_text: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), wave_text).unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    dir
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

fn chore_wave(transforms: &[(&str, &str)]) -> String {
    let mut t = String::from("transforms:\n");
    for (name, file) in transforms {
        t.push_str(&format!(
            "  {name}:\n    chore: \"прибирання\"\n    files:\n      - {file}\n"
        ));
    }
    let mut body = String::new();
    for (name, _) in transforms {
        body.push_str(&format!("## transform: {name}\nтіло\n\n"));
    }
    format!("---\n{t}{}---\n\n{body}", decisions_except(&[]))
}

/// proves: the-weight-is-read-from-the-branch-too@326203
#[test]
fn the_weight_is_read_from_the_branch_too() {
    // --- a light wave whose branch changes a contract is full, and
    // said so before the PR ---
    let dir = crate_with(
        "weightcontract",
        "rust",
        &chore_wave(&[("tidy", "src/lib.rs")]),
    );
    std::fs::write(
        dir.join("keel/contracts/ext.md"),
        "---\nverify: \"true\"\n---\n\nчужа обіцянка\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        format!(
            "lang = \"uk\"\nadapter = \"rust\"\n\n[trust]\n\"true\" = \"{}\"\n",
            keel::trust::fingerprint("true")
        ),
    )
    .unwrap();
    std::fs::remove_file(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    std::fs::remove_file(dir.join("keel/reviews/0001-a-wave.md")).unwrap();
    settle(&dir);
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        chore_wave(&[("tidy", "src/lib.rs")]),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/ext.md"),
        "---\nverify: \"true\"\n---\n\nчужа обіцянка, ЗМІНЕНА на гілці легкої хвилі\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn works() -> bool { true }\n// tidy\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "tidy: work and a contract change",
        ],
    );
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains("§6.8") && said.contains("ext"),
        "a contract changed on a light wave's branch is a finding naming the \
         contract and the paragraph:\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        !said.contains("легка хвиля їде в свій один PR"),
        "and `next` does not lead a contract change to one PR:\n{said}"
    );

    // --- two chore transforms: a wave of chores alone must be light,
    // and light means one transform (§2.11) ---
    let dir = crate_with(
        "weightchores",
        "rust",
        &chore_wave(&[("tidy", "src/lib.rs"), ("more", "Cargo.toml")]),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains("§2.11"),
        "a wave of two chores is a finding by the paragraph:\n{said}"
    );
    assert_eq!(code, 1, "and the check is red:\n{said}");

    // --- "closed by the fact of merge" only where the fact stands:
    // the wave file at main ---
    let dir = crate_with(
        "weightmerge",
        "rust",
        &chore_wave(&[("tidy", "src/lib.rs")]),
    );
    std::fs::remove_file(dir.join("keel/waves/0001-a-wave.md")).unwrap();
    std::fs::remove_file(dir.join("keel/reviews/0001-a-wave.md")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "root"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        chore_wave(&[("tidy", "src/lib.rs")]),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "--no-verify", "-m", "tidy: work"]);
    let (said, _) = keel(&dir, &["status"]);
    assert!(
        said.contains("закриється фактом merge") && said.contains("§6.5"),
        "on its own branch the chore wave WILL close by merge, and the \
         paragraph is §6.5:\n{said}"
    );
    assert!(
        !said.contains("закрита фактом merge"),
        "and it is not called closed before the merge:\n{said}"
    );
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        !said.contains("закрита (легка)"),
        "close says the same before the merge:\n{said}"
    );
    assert_eq!(
        code, 0,
        "and blocks nothing -- the merge is the closure:\n{said}"
    );
    git(&dir, &["checkout", "-q", "main"]);
    git(
        &dir,
        &[
            "merge",
            "-q",
            "--no-ff",
            "--no-verify",
            "-m",
            "merge",
            "0001-a-wave",
        ],
    );
    let (said, _) = keel(&dir, &["status"]);
    assert!(
        said.contains("закрита фактом merge (§6.5)"),
        "at main the fact stands, and the wave is closed by it:\n{said}"
    );
}
