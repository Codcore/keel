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

/// The frame run with a foreign repository in the environment --
/// exactly what a git hook hands its children (GIT_DIR and friends).
fn keel_under_hook_env(args: &[&str], git_dir: &Path, work_tree: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .env("GIT_DIR", git_dir)
        .env("GIT_WORK_TREE", work_tree)
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
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
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// The same, with the crate one level down -- keel's own shape, and
/// the one where cargo writes a .gitignore of its own beside it.
fn nested_project(name: &str) -> PathBuf {
    let dir = sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "tool/Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "tool/src/lib.rs", "");
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// proves: ignore-reminded@3d57a9 -- holds the third gift of the
/// first field: the frame says aloud what its own adapter will
/// build into, and git itself judges the rule -- so a nested
/// .gitignore (the one cargo writes beside a crate) counts, and a
/// rule living only in .git/info/exclude is named for what it is:
/// present, but not travelling with the repository. Six states,
/// six words; no ignore file is ever created or changed by a byte,
/// and the advice never reddens the frame.
#[test]
fn ignore_reminded() {
    // No .gitignore at all: the row advises starting one, and the
    // frame writes no such file.
    let dir = project("nofile", Some("rust"));
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
    assert!(
        out.contains("ignore rules") && out.contains("add exactly") && out.contains("target/"),
        "with nothing ignoring it, the row names the exact line to add:\n{out}"
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

    // The rule already standing in the root file -- with a slash and
    // without it, as git reads both.
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

    // The rule in the crate's own .gitignore, one level down -- the
    // file cargo writes itself. Reading only the root would raise a
    // false alarm here; git knows better (caught by dogfooding keel
    // on keel, whose crate lives in tool/).
    let dir = nested_project("nested");
    write(&dir, "tool/.gitignore", "/target\n");
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
    assert!(
        out.contains("ignore rules") && out.contains("stands"),
        "a nested .gitignore counts -- git judges the rule, not a guess:\n{out}"
    );
    assert!(
        out.contains("tool/.gitignore"),
        "the row names whose file gave the rule:\n{out}"
    );

    // The rule living only in .git/info/exclude: present for this
    // clone, gone for everyone else -- said aloud, with the advice
    // repeated (the first field's R-4 school).
    let dir = project("excluded", Some("rust"));
    write(&dir, ".git/info/exclude", "target/\n");
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
    assert!(
        out.contains("does not travel") && out.contains("target/"),
        "an exclude-only rule is named for what it is, advice and all:\n{out}"
    );

    // No git at all: the rule is not judged, and the frame lands on.
    let dir = sandbox("gitless");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");
    let (out, err, _) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("ignore rules") && out.contains("not judged"),
        "with git silent the rule is not judged, and the frame keeps landing:\n{out}"
    );

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

    // Run from inside a git hook, where git hands its children
    // GIT_DIR and GIT_WORK_TREE of ITS repository: the row must
    // still judge the project it was pointed at, never the
    // repository that happened to spawn it (the same school as the
    // adapter's isolation from an inherited CARGO_TARGET_DIR).
    let foreign = sandbox("foreign-repo");
    git(&foreign, &["init", "-q", "-b", "main"]);
    write(&foreign, ".gitignore", "target/\n");
    let dir = project("hookenv", Some("rust"));
    let out = keel_under_hook_env(
        &["init", dir.to_str().unwrap()],
        &foreign.join(".git"),
        &foreign,
    );
    assert!(
        out.contains("add exactly"),
        "the row judges the project it was given, not the repository in the environment:\n{out}"
    );
}
