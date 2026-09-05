//! Scenario test of wave 0039: the pin has a hand.
//!
//! The hand is a shell script, and this is a Rust battery -- so the
//! probe RUNS it rather than reading it. Review 0039 R-5: it used to
//! search install.sh for substrings, and two mutations that gut the
//! scenario (never read KEEL_REF; check out origin/HEAD instead of
//! the named ref) passed the whole battery, because a comment in the
//! script's head contains the word.
//!
//! What is faked here, and named rather than hidden: `cargo`. The
//! probe puts a stub on PATH that writes the binary install.sh
//! expects, so the run costs milliseconds instead of a release
//! build. Everything else -- the clone, the fetch, the checkout, the
//! refusals -- is the real script against a real git repository.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, sandbox};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn keel(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
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

fn git(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
    let out = command
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn installer_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("install.sh")
}

/// A repository shaped like keel's: a `tool/Cargo.toml` so the layout
/// guard passes, one tag on an older commit, and a newer tip.
struct World {
    dir: PathBuf,
    repo: PathBuf,
    home: PathBuf,
    bin: PathBuf,
    stub: PathBuf,
    tagged: String,
    tip: String,
}

fn world(name: &str) -> (Sandbox, World) {
    let dir = sandbox(name);
    let repo = dir.join("source");
    fs::create_dir_all(repo.join("tool")).unwrap();
    fs::write(repo.join("tool/Cargo.toml"), "[package]\nname = \"keel\"\n").unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "the tagged one"]);
    let tagged = git(&repo, &["rev-parse", "HEAD"]);
    git(&repo, &["tag", "v2.0.0"]);
    fs::write(repo.join("tool/newer.txt"), "later\n").unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "the tip"]);
    let tip = git(&repo, &["rev-parse", "HEAD"]);

    // The stub cargo: it writes the binary install.sh copies out, and
    // that binary answers `--version` as the real one does.
    let stub = dir.join("stub");
    fs::create_dir_all(&stub).unwrap();
    let home = dir.join("home");
    fs::write(
        stub.join("cargo"),
        "#!/bin/sh\nout=\"$KEEL_HOME/tool/target/release\"\nmkdir -p \"$out\"\nprintf '#!/bin/sh\\necho \"keel 2.0.0 stub\"\\n' > \"$out/keel\"\nchmod +x \"$out/keel\"\n",
    )
    .unwrap();
    let mut perms = fs::metadata(stub.join("cargo")).unwrap().permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        perms.set_mode(0o755);
    }
    fs::set_permissions(stub.join("cargo"), perms).unwrap();

    let world = World {
        dir: dir.to_path_buf(),
        repo,
        home,
        bin: dir.join("bin"),
        stub,
        tagged,
        tip,
    };
    (dir, world)
}

/// One run of the real install.sh. Returns (what it said, exit code).
fn install(world: &World, git_ref: Option<&str>) -> (String, i32) {
    let path = format!(
        "{}:{}",
        world.stub.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let mut command = Command::new("sh");
    command
        .arg(installer_path())
        .current_dir(&world.dir)
        .env("PATH", path)
        .env("KEEL_REPO", &world.repo)
        .env("KEEL_HOME", &world.home)
        .env("KEEL_BIN", &world.bin)
        .env("KEEL_HOME_FOR_STUB", &world.home)
        .env("KEEL_REF", git_ref.unwrap_or(""));
    // The stub reads KEEL_HOME to know where to write.
    command.env("KEEL_HOME", &world.home);
    let out = command.output().unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

fn head_of(world: &World) -> String {
    git(&world.home, &["rev-parse", "HEAD"])
}

/// proves: the-pin-has-a-hand@37ae08 -- `keel version` over a mismatched
/// pin said the courts refuse until the pin and the binary meet, and
/// named no hand that makes them meet. install.sh took no version at
/// all: it cloned and built `main`, whatever keel.toml said. The
/// advice pointed at an action the tool could not do.
#[test]
fn the_pin_has_a_hand() {
    // -- what the lamp says ------------------------------------------
    let dir = sandbox("mismatch");
    fs::write(
        dir.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\nversion = \"0.0.1-not-this-binary\"\n",
    )
    .unwrap();
    let (said, code) = keel(&["version", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the lamp never refuses over a mismatch:\n{said}");
    assert!(
        said.contains("KEEL_REF=\"0.0.1-not-this-binary\""),
        "it names the command that fetches exactly that version, with \
         the version already in it:\n{said}"
    );
    assert!(
        said.contains("install.sh"),
        "which is the hand the project ships:\n{said}"
    );
    // Review 0039 R-3: the pin field holds a version (`0.1.0`) and the
    // repository's tags are refs (`v0.8.9`). The lamp must not imply
    // the two are one word.
    assert!(
        said.contains("git ref") || said.contains("tag"),
        "and says the version must be a ref this repository carries, \
         not merely the number in keel.toml:\n{said}"
    );

    let dir = sandbox("held");
    fs::write(
        dir.join("keel.toml"),
        format!(
            "lang = \"en\"\nadapter = \"rust\"\nversion = \"{}\"\n",
            env!("CARGO_PKG_VERSION")
        ),
    )
    .unwrap();
    let (said, code) = keel(&["version", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "a held pin is a green lamp:\n{said}");
    assert!(!said.contains("KEEL_REF"), "and asks for nothing:\n{said}");

    // -- what the hand does ------------------------------------------
    let (_keep, w) = world("pinhand");

    // Named version: exactly that ref lands, not the tip.
    let (said, code) = install(&w, Some("v2.0.0"));
    assert_eq!(code, 0, "the named version installs:\n{said}");
    assert_eq!(
        head_of(&w),
        w.tagged,
        "and the clone stands at the tag's own commit, not at the tip \
         -- a hand that ignores the name silently builds main, which \
         is the bug this scenario exists for:\n{said}"
    );
    assert_ne!(w.tagged, w.tip, "the fixture really has two commits");

    // Unnamed: the tip, and no death coming back from the pin.
    let (said, code) = install(&w, None);
    assert_eq!(code, 0, "the unpinned run comes back from a pin:\n{said}");
    assert_eq!(head_of(&w), w.tip, "and lands on the tip:\n{said}");

    // Review 0039 R-2: the SECOND ordinary run is the one the script's
    // own head calls updating, and it died on `checkout -`.
    let (_keep2, fresh) = world("secondrun");
    let (said, code) = install(&fresh, None);
    assert_eq!(code, 0, "a first ordinary run:\n{said}");
    let (said, code) = install(&fresh, None);
    assert_eq!(
        code, 0,
        "and the second one, which is how it updates:\n{said}"
    );

    // A ref that is not there refuses by name and installs nothing.
    let (_keep3, missing) = world("noref");
    let (said, code) = install(&missing, Some("v99.0.0"));
    assert_eq!(code, 1, "an unknown version refuses:\n{said}");
    assert!(
        said.contains("v99.0.0"),
        "by the name that was asked for:\n{said}"
    );
    assert!(
        !missing.bin.join("keel").is_file(),
        "and nothing was installed:\n{said}"
    );
}
