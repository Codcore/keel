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

/// What a git hook hands its children -- the probe's own sandboxes
/// must be as deaf to it as the frame is, or a run from inside the
/// gate judges someone else's repository (and an empty
/// GIT_CONFIG_PARAMETERS makes git itself complain).
const GIT_ENV: [&str; 10] = [
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_COMMON_DIR",
    "GIT_INDEX_FILE",
    "GIT_OBJECT_DIRECTORY",
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_PREFIX",
    "GIT_CEILING_DIRECTORIES",
    "GIT_CONFIG_PARAMETERS",
    "GIT_CONFIG_COUNT",
];

fn git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    for name in GIT_ENV {
        command.env_remove(name);
    }
    let out = command
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

fn keel_with_env(args: &[&str], envs: &[(&str, &str)]) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_keel"));
    command.args(args);
    for (name, value) in envs {
        command.env(name, value);
    }
    let out = command.output().unwrap();
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

/// proves: ignore-reminded@8e811e -- holds the third gift of the
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

    // No adapter named at all: there is no build directory to name,
    // and the row says that instead of advising a guess. (A NAMED
    // adapter this release does not serve gets its own word -- see
    // the R-8 case below.)
    let dir = project("noadapter", None);
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the advice never reddens the frame:\n{out}");
    assert!(
        out.contains("ignore rules") && out.contains("no adapter"),
        "with no adapter named there is no directory to name:\n{out}"
    );
    assert!(
        !out.contains("target"),
        "no build directory is guessed where no adapter is named:\n{out}"
    );

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
    // And the frame WRITES where it was pointed: the hook lands in
    // this project, never in the repository that spawned keel.
    assert!(
        dir.join(".git/hooks/commit-msg").is_file(),
        "the hook lands in the project the frame was given:\n{out}"
    );
    assert!(
        !foreign.join(".git/hooks/commit-msg").exists(),
        "no byte is written into the repository of the environment:\n{out}"
    );

    // `git -c` travels to children through GIT_CONFIG_PARAMETERS,
    // and a hook's own -c must not rewrite our answer either
    // (review 0020 R-10): with a config forced through it, the row
    // still judges the project's own files.
    let forced = sandbox("forced-rules");
    write(&forced, "rules", "target/\n");
    let dir = project("configparams", Some("rust"));
    let out = keel_with_env(
        &["init", dir.to_str().unwrap()],
        &[(
            "GIT_CONFIG_PARAMETERS",
            &format!("'core.excludesFile={}'", forced.join("rules").display()),
        )],
    );
    assert!(
        out.contains("add exactly"),
        "a config forced through the environment does not rewrite the verdict (R-10):\n{out}"
    );

    // A broken keel.toml is not "no adapter": the rule is simply
    // not judged, and the reason is said (review 0020 R-2).
    let dir = project("brokenconfig", Some("rust"));
    write(&dir, ".gitignore", "target/\n");
    write(
        &dir,
        "keel.toml",
        "lang = \"en\"\nadapter = \"rust\"\nbroken = = =\n",
    );
    let (out, err, _) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("ignore rules") && out.contains("not judged"),
        "a broken config leaves the rule unjudged, with its reason (R-2):\n{out}"
    );
    assert!(
        !out.contains("no adapter"),
        "a broken config is never read as an unnamed adapter (R-2):\n{out}"
    );

    // A rule that git obeys only because a config points at the
    // file -- global or local -- travels with nobody (R-5): the
    // path may be relative and look like a file of the tree.
    let dir = project("localexcludes", Some("rust"));
    write(&dir, "extra-rules", "target/\n");
    git(&dir, &["config", "core.excludesFile", "extra-rules"]);
    let (out, err, _) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("does not travel") && out.contains("target/"),
        "a rule named by core.excludesFile does not travel, whatever its path (R-5):\n{out}"
    );

    // The sharpest corner of the same truth: a file that IS named
    // .gitignore and does lie in the tree, but which git obeys only
    // because core.excludesFile points at it. The file travels; the
    // config that gives it force does not -- so a fresh clone would
    // ignore nothing.
    let dir = project("configgitignore", Some("rust"));
    write(&dir, "extra/.gitignore", "target/\n");
    git(&dir, &["config", "core.excludesFile", "extra/.gitignore"]);
    let (out, err, _) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("does not travel"),
        "a .gitignore obeyed only through core.excludesFile does not travel (R-5):\n{out}"
    );

    // The same for a global config of the person's machine.
    let home = sandbox("globalhome");
    write(&home, "ignore-rules", "target/\n");
    write(
        &home,
        "gitconfig",
        &format!(
            "[core]\n\texcludesFile = {}\n",
            home.join("ignore-rules").display()
        ),
    );
    let dir = project("globalexcludes", Some("rust"));
    let out = keel_with_env(
        &["init", dir.to_str().unwrap()],
        &[(
            "GIT_CONFIG_GLOBAL",
            home.join("gitconfig").to_str().unwrap(),
        )],
    );
    assert!(
        out.contains("does not travel"),
        "a rule from the person's global config travels with nobody (R-5, R-6):\n{out}"
    );

    // No crate to name a build directory by -- several at the first
    // level: said aloud, never guessed (R-6).
    let dir = sandbox("manycrates");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    for name in ["one", "two"] {
        write(
            &dir,
            &format!("{name}/Cargo.toml"),
            "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
        );
    }
    git(&dir, &["init", "-q", "-b", "main"]);
    let (out, err, _) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("ignore rules") && out.contains("no crate"),
        "with no single crate the frame says so instead of guessing (R-6):\n{out}"
    );

    // A named adapter this release does not serve is called by its
    // name -- never "not named" (R-8, the 0017 R-3 school).
    let dir = project("namedforeign", Some("elixir"));
    let (out, err, _) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("\"elixir\"") && !out.contains("no adapter"),
        "a named adapter is named, not called absent (R-8):\n{out}"
    );
}
