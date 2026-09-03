//! Scenario tests of wave 0020-ignore-reminded, transform
//! ignore-row: the frame speaks of the adapter's build directory --
//! it advises, and never writes a file of the project's.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0020i-{}-{name}", std::process::id()));
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

/// A git repository with a keel.toml naming the given adapter (or
/// none at all), ready for the frame.
fn project(name: &str, adapter: Option<&str>) -> PathBuf {
    let dir = sandbox(name);
    let mut config = String::from("lang = \"en\"\n");
    if let Some(adapter) = adapter {
        config.push_str(&format!("adapter = \"{adapter}\"\n"));
    }
    write(&dir, "keel.toml", &config);
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// proves: ignore-reminded@99f4d6 -- holds the third gift of the
/// first field: the frame that lays the methodology says aloud
/// what its own adapter will build into. No .gitignore -- start one
/// with the build directory; a .gitignore without the rule -- add
/// exactly this line; the rule standing -- said so; no adapter of
/// this release -- no directory to name. In every state the file is
/// neither created nor changed by a byte, and the advice never
/// reddens the frame.
#[test]
fn ignore_reminded() {
    // No .gitignore at all: the row advises starting one, and the
    // frame writes no such file.
    let dir = project("nofile", Some("rust"));
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
    assert!(
        out.contains("ignore rules") && out.contains("no .gitignore") && out.contains("target/"),
        "the row advises starting a .gitignore with the build directory:\n{out}"
    );
    assert!(
        !dir.join(".gitignore").exists(),
        "the frame advises and writes no file of the project's"
    );

    // A .gitignore without the rule: the row names the exact line,
    // and the file stays byte for byte what it was.
    let dir = project("norule", Some("rust"));
    let before = "# mine\n*.log\n";
    write(&dir, ".gitignore", before);
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
    assert!(
        out.contains("ignore rules") && out.contains("add exactly") && out.contains("target/"),
        "the row names the exact line to add:\n{out}"
    );
    assert_eq!(
        fs::read_to_string(dir.join(".gitignore")).unwrap(),
        before,
        "the project's file is not touched by a byte (school 0014)"
    );

    // The rule already standing -- with a slash and without it, as
    // git reads both.
    for (name, rule) in [("slash", "target/"), ("bare", "target")] {
        let dir = project(name, Some("rust"));
        write(&dir, ".gitignore", &format!("# mine\n{rule}\n"));
        let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
        let out = format!("{out}{err}");
        assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
        assert!(
            out.contains("ignore rules") && out.contains("stands"),
            "the standing rule is said to stand (written \"{rule}\"):\n{out}"
        );
    }

    // No adapter of this release: there is no build directory to
    // name, and the row says that instead of advising a guess.
    for (name, adapter) in [("noadapter", None), ("foreign", Some("elixir"))] {
        let dir = project(name, adapter);
        let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
        let out = format!("{out}{err}");
        assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
        assert!(
            out.contains("ignore rules") && out.contains("no adapter"),
            "with no adapter of this release there is no directory to name:\n{out}"
        );
        assert!(
            !out.contains("target"),
            "no build directory is guessed for a language this release does not serve:\n{out}"
        );
    }
}
