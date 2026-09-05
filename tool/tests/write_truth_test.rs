//! Scenario test of wave 0037: a write that lies is red.
//!
//! Its own file: `rev_write_test.rs` belongs to wave 0016 and holds
//! the tag of `stale-refs-rewritten`. This wave overwrote it once,
//! which is exactly the disappearance §7.15 forbids -- caught by the
//! §5.6 count falling by seven.

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

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn decided(except: &str) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != except {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// proves: a-write-that-lies-is-red@6ee6d7 -- the bug audit (B5)
/// measured `keel rev --write` printing a red line, then the words
/// "nothing drifts", and exiting ZERO. CI sees the zero and drives
/// on: a hand that failed reported success.
#[test]
fn a_write_that_lies_is_red() {
    // A record left standing on purpose (§5.6: a closed wave keeps
    // the revision that was true when it closed) must not be followed
    // by "nothing drifted". The bug audit (B5) measured exactly that
    // shape: a report contradicting itself two lines apart.
    let dir = project("kept");
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nстарі слова\n",
    )
    .unwrap();
    let old_rev = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();
    let rev = keel::rev::text_rev("тіло обіцянки\n");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{}}\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    proves: anchor@{old_rev}\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided("functional.correctness")
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "the wave, closed, and its contract"],
    );
    // The contract moves on; the closed wave keeps the revision that
    // was true in history (§5.6).
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nнові слова\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "chore: the contract moved on"],
    );

    let (said, code) = keel(&dir, &["rev", "--write"]);
    assert!(
        said.contains("лишаю її записи") && said.contains("0001-a-wave"),
        "the write says what it left standing, and whose (§5.6):\n{said}"
    );
    assert!(
        !said.contains("нічого не розійшлось"),
        "and does not claim all is well two lines under a record it \
         just said it left drifted (bug audit B5):\n{said}"
    );
    assert!(
        said.contains("розійш") && said.contains("§5.6"),
        "it says plainly that something DID drift and why it stays \
         (§5.6), instead of the flat denial the audit found:\n{said}"
    );
    assert_eq!(
        code, 0,
        "leaving a closed wave's record is lawful, so the code is \
         zero -- and now the words agree with it:\n{said}"
    );

    // And the exit code itself, measured -- review 0037 R-5 showed
    // the named limit was wrong: rights DO hold, if one is not root.
    // A directory the hand cannot write into is the shape of the
    // audit's B5: a red line, and a zero under it.
    let dir = project("cannotwrite");
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    proves: anchor@beef00\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided("functional.correctness")
        ),
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let waves = dir.join("keel/waves");
        let mut perms = std::fs::metadata(&waves).unwrap().permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&waves, perms).unwrap();
        // Root ignores the mode, so as root the hand is run as
        // nobody -- and ONLY as root. A non-root caller cannot drop
        // privileges at all: `setpriv` spawns, fails with its own
        // "setgroups failed: Operation not permitted", and exits 127,
        // while `Command::output()` still returns `Ok` -- so this
        // probe used to judge SETPRIV'S message as keel's, on every
        // runner that is not root (wave 0044). Where the caller is
        // already not root, no dropping is needed: the mode above
        // holds by itself.
        // Asked by BEHAVIOUR, and asked without touching the hand
        // under test: can THIS caller write into a 0555 directory?
        // Where it cannot, the mode already holds and no dropping is
        // needed -- which is just as well, since a non-root caller
        // cannot drop privileges at all (wave 0044). The probe under
        // test is never used as the question, because a trial run
        // would write the very record the case is about.
        let trial = waves.join(".keel-can-i-write");
        let ignores_the_mode = std::fs::write(&trial, b"").is_ok();
        let _ = std::fs::remove_file(&trial);
        let out = if !ignores_the_mode {
            Command::new(env!("CARGO_BIN_EXE_keel"))
                .args(["rev", "--write", dir.to_str().unwrap()])
                .output()
        } else {
            Command::new("setpriv")
                .args([
                    "--reuid=65534",
                    "--regid=65534",
                    "--clear-groups",
                    env!("CARGO_BIN_EXE_keel"),
                    "rev",
                    "--write",
                    dir.to_str().unwrap(),
                ])
                .output()
        };
        let mut perms = std::fs::metadata(&waves).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&waves, perms).unwrap();
        match out {
            Ok(out) if String::from_utf8_lossy(&out.stderr).contains("setpriv:") => {
                // The helper itself did not run. That is an ABSENCE
                // of a verdict, never a verdict (wave 0044).
                eprintln!(
                    "the unwritable case is skipped: the helper left with {} -- \
                     {} -- these are ITS words, not keel's",
                    out.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&out.stderr).trim()
                );
            }
            Ok(out) => {
                let said = format!(
                    "{}{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                assert!(
                    said.contains("червоне"),
                    "the hand says what stopped it:\n{said}"
                );
                assert_ne!(
                    out.status.code().unwrap_or(-1),
                    0,
                    "and the exit code says the same -- a hand that \
                     failed does not report success (bug audit \
                     B5):\n{said}"
                );
                assert!(
                    !said.contains("нічого не розійшлось"),
                    "nor does it deny the drift it just printed:\n{said}"
                );
            }
            Err(why) => eprintln!(
                "the unwritable case is skipped: setpriv is not here ({why}) -- \
                 said aloud rather than pretended"
            ),
        }
    }

    // Where there is genuinely nothing to write, zero is honest.
    let dir = project("quiet");
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided("functional.correctness")
        ),
    )
    .unwrap();
    let (said, code) = keel(&dir, &["rev", "--write"]);
    assert_eq!(code, 0, "nothing to write is a green answer:\n{said}");
    assert!(
        said.contains("нічого не розійшлось"),
        "and it says so:\n{said}"
    );

    // And a write that lands is green too, with its count.
    let dir = project("lands");
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    let rev = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    proves: anchor@beef00\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    contracts: [anchor@beef00]\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided("functional.correctness")
        ),
    )
    .unwrap();
    let (said, code) = keel(&dir, &["rev", "--write"]);
    assert_eq!(code, 0, "a write that landed is green:\n{said}");
    assert!(
        said.contains(&rev),
        "and it names the revision it wrote:\n{said}"
    );
}
