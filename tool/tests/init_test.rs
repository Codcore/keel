//! Scenario tests of wave 0014-field-frame, transform frame-hand:
//! `keel init` sets the methodology's frame in one move -- and
//! never tramples anything that already stands.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0014i-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
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

/// proves: init-births-the-frame@4e449f -- holds the NEW-CONCEPT
/// frame row and §8.7: one move births the three keel/ directories
/// (each with .gitkeep), keel.toml with the commented §2.9
/// vocabulary enabling nothing, and the commit-msg hook by gate's
/// hand; every piece is its own "born" line, the tail reminds of
/// §8.7 and leads to keel plan, and check reads the born frame
/// without refusals.
#[test]
fn init_births_the_frame() {
    let dir = sandbox("cleanbirth");
    git(&dir, &["init", "-q", "-b", "main"]);
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a clean birth is green:\n{out}");
    for piece in ["keel/waves", "keel/contracts", "keel/reviews"] {
        assert!(
            dir.join(piece).join(".gitkeep").is_file(),
            "the directory {piece} is born with .gitkeep:\n{out}"
        );
    }
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        config.contains("# lang") && config.contains("# adapter") && config.contains("# mode"),
        "the config carries the commented vocabulary, enabling nothing:\n{config}"
    );
    assert!(
        !config.lines().any(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with('#')
        }),
        "nothing is enabled -- defaults stay with config's words:\n{config}"
    );
    let hook = fs::read_to_string(dir.join(".git/hooks/commit-msg")).unwrap();
    assert!(
        hook.contains("keel gate"),
        "the commit-msg hook is gate's own:\n{hook}"
    );
    assert!(
        out.contains("born") && out.contains("keel.toml") && out.contains("commit-msg"),
        "every piece is its own line by name:\n{out}"
    );
    assert!(
        out.contains("squash") && out.contains("rebase"),
        "the §8.7 reminder rides the tail -- the button holds the rule:\n{out}"
    );
    assert!(
        out.contains("keel plan"),
        "the tail leads onwards to the first wave:\n{out}"
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the born frame reads without refusals:\n{out}");
}
