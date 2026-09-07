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
    // The stub writes where the installer builds. Wave 0041 moved that
    // from $KEEL_HOME to $KEEL_HOME/source, because each version now
    // has a home of its own -- the fixture follows the layout; not one
    // assertion of this probe changed.
    fs::write(
        stub.join("cargo"),
        "#!/bin/sh\nout=\"$KEEL_HOME/source/tool/target/release\"\nmkdir -p \"$out\"\nprintf '#!/bin/sh\\necho \"keel 2.0.0 stub\"\\n' > \"$out/keel\"\nchmod +x \"$out/keel\"\n",
    )
    .unwrap();
    // And a `curl` that never leaves the machine (wave 0050, global
    // review tests cut R-1): this world had no KEEL_RELEASES and no
    // shim, so after wave 0048 the installer walked its release road
    // to github.com twice per battery run -- a verdict that depended
    // on the network. A `file://` address goes to the real curl, any
    // other is logged and refused, and `install()` holds the log
    // empty.
    let real_curl = ["/usr/bin/curl", "/bin/curl", "/usr/local/bin/curl"]
        .into_iter()
        .find(|c| Path::new(c).is_file())
        .unwrap_or("/usr/bin/curl");
    fs::write(
        stub.join("curl"),
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do case \"$a\" in http://*|https://*) echo \"$a\" >> '{}'; exit 22;; esac; done\nexec {real_curl} \"$@\"\n",
            dir.join("curl-http.log").display()
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for tool in ["cargo", "curl"] {
            let mut perms = fs::metadata(stub.join(tool)).unwrap().permissions();
            perms.set_mode(0o755);
            fs::set_permissions(stub.join(tool), perms).unwrap();
        }
    }

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

/// One run of the real install.sh, the ref handed as KEEL_REF.
fn install(world: &World, git_ref: Option<&str>) -> (String, i32) {
    install_as(world, git_ref, false)
}

/// The same run with the ref handed POSITIONALLY -- the form the lamp
/// advises since wave 0057 (`sh install.sh <pin>`). Review 0057 R-13:
/// the probe asserted the new words while every run still went
/// through the variable, so the advised spelling was never played.
fn install_positional(world: &World, git_ref: &str) -> (String, i32) {
    install_as(world, Some(git_ref), true)
}

fn install_as(world: &World, git_ref: Option<&str>, positional: bool) -> (String, i32) {
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
        .env(
            "KEEL_REF",
            if positional {
                ""
            } else {
                git_ref.unwrap_or("")
            },
        );
    if let (true, Some(git_ref)) = (positional, git_ref) {
        command.arg(git_ref);
    }
    // The stub reads KEEL_HOME to know where to write.
    command.env("KEEL_HOME", &world.home);
    // No release stands anywhere this world can reach: an empty
    // `file://` directory, so the release road of wave 0048 ends here
    // and the source road is taken -- and the curl shim holds that
    // no other address was ever asked for.
    let releases = world.dir.join("no-releases");
    fs::create_dir_all(&releases).unwrap();
    command.env("KEEL_RELEASES", format!("file://{}", releases.display()));
    let out = command.output().unwrap();
    assert!(
        !world.dir.join("curl-http.log").exists(),
        "no road of this world walks to the network:\n{}",
        fs::read_to_string(world.dir.join("curl-http.log")).unwrap_or_default()
    );
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
    // The clone lives in `source` since wave 0041.
    git(&world.home.join("source"), &["rev-parse", "HEAD"])
}

/// proves: the-pin-has-a-hand@aac589 -- `keel version` over a mismatched
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
    // The promise is a COMMAND that works and carries the version
    // (§2.3 of the scenario), not one spelling of it: wave 0057
    // measured `sh install.sh <pin>` and `KEEL_REF="<pin>" sh
    // install.sh` to be the same road (the positional argument IS
    // KEEL_REF), and the lamp now hands the shorter form.
    assert!(
        said.contains("sh install.sh '0.0.1-not-this-binary'"),
        "it names the command that fetches exactly that version, with \
         the version already in it -- QUOTED, because the pin is a \
         stranger's string and `x; echo` in it made two commands of \
         one (review 0057 R-10):\n{said}"
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

    // The spelling the lamp advises -- the ref handed positionally,
    // which install.sh reads as KEEL_REF (review 0057 R-13).
    let (said, code) = install_positional(&w, "v2.0.0");
    assert_eq!(code, 0, "the advised spelling installs:\n{said}");
    assert!(
        said.contains("v2.0.0"),
        "and names the version it took:\n{said}"
    );

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

/// proves: the-advice-leads-with-the-road-that-works@1a48d6 -- the
/// lamp said one road for every pin: "KEEL_REF takes a git ref of
/// this repository BY NAME and builds from source -- nothing checks
/// a checksum there". Measured on keel 1.0.0 against a `file://`
/// release: `KEEL_REF="1.0.0" sh install.sh` fetched the published
/// archive and verified its sha256, exactly as `sh install.sh 1.0.0`
/// does -- the pin's own SHAPE picks the road, and the words named
/// the wrong one for every version pin (queue after 0055, bugs R-26).
#[test]
fn the_advice_leads_with_the_road_that_works() {
    // -- a pin shaped like a version: the release road -------------
    let dir = sandbox("advice-version");
    fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"rust\"\nversion = \"9.9.9\"\n",
    )
    .unwrap();
    let (said, code) = keel(&["version", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the lamp never refuses over a mismatch:\n{said}");
    assert!(
        said.contains("sh install.sh '9.9.9'"),
        "the shortest form of the road that works is what a person is \
         handed:\n{said}"
    );
    assert!(
        said.contains("реліз") && said.contains(".sha256"),
        "and the road is named for what it is -- a published release \
         whose checksum is verified:\n{said}"
    );
    assert!(
        !said.contains("checksum там не звіряє ніхто"),
        "never the sentence of the OTHER road: this pin's road does \
         check one:\n{said}"
    );
    // The promise is that the road LEADS, not that it is mentioned
    // (review 0057 R-2: two mutants that swapped the rows passed the
    // whole battery). The command comes first, its road next, the
    // border last -- a warning before the thing it warns about is
    // the very shape this wave took away.
    let command_at = said.find("sh install.sh").expect("the command");
    let road_at = said.find("дорога цього піна").expect("the road");
    let border_at = said.find("межа:").expect("the border");
    assert!(
        command_at < road_at && road_at < border_at,
        "the rows stand in the order a person reads them -- command, \
         road, border:\n{said}"
    );
    assert!(
        said[road_at..border_at]
            .find("реліз")
            .is_some_and(|release| {
                said[road_at..border_at]
                    .find("KEEL_REF")
                    .is_none_or(|other| release < other)
            }),
        "and the road line leads with the release, not with the form \
         that builds from source:\n{said}"
    );

    // -- a pin that is a ref: the road from source ------------------
    let dir = sandbox("advice-ref");
    fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"rust\"\nversion = \"my-branch\"\n",
    )
    .unwrap();
    let (said, code) = keel(&["version", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the lamp never refuses over a mismatch:\n{said}");
    assert!(
        said.contains("git ref") && said.contains("checksum там не звіряє ніхто"),
        "a pin no release can answer takes the road from source, and \
         the words say what that road does not check:\n{said}"
    );
    assert!(
        !said.contains("опублікований реліз, його архів"),
        "and the release road is not offered for a name no release \
         carries:\n{said}"
    );
}
