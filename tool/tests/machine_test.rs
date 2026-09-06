//! Scenario test of wave 0044: a probe without its tool stops aloud.
//!
//! Measured before this was written, on the tool's own CI: three
//! probes carry `assert!(have_mix(), ...)` while the head of their
//! own file says they "say so and stop rather than pretending" -- and
//! an assert is a failure, not a stop. The runner has no elixir, so
//! `keel close` there called two of wave 0042's scenarios unproven.
//!
//! This probe does not read that source; it RUNS those probes on a
//! machine without the tool, by handing them a PATH that has none.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use std::path::PathBuf;
use std::process::Command;

/// The compiled test binaries of this crate, found beside this one.
fn probe_binary(stem: &str) -> Option<PathBuf> {
    // This binary lives in `deps/` beside every other test binary
    // of the crate, so it is its own signpost -- no guessing at the
    // target directory, which a CARGO_TARGET_DIR moves.
    let deps = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let mut newest: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in std::fs::read_dir(deps).ok()?.flatten() {
        let path = entry.path();
        let name = path.file_name()?.to_str()?.to_string();
        if !name.starts_with(stem) || name.contains('.') {
            continue;
        }
        let when = entry.metadata().ok()?.modified().ok()?;
        if newest.as_ref().is_none_or(|(seen, _)| when > *seen) {
            newest = Some((when, path));
        }
    }
    newest.map(|(_, path)| path)
}

/// A PATH holding only what the probe legitimately needs, and NOT
/// the tongue's runner: the shape of a runner that simply does not
/// have elixir installed. Built rather than assumed, because on this
/// machine `mix` sits in `/usr/bin` beside everything else.
fn path_without(tool: &str, needs: &[&str]) -> common::Sandbox {
    // The one hand makes it, and the one hand takes it away when
    // this test ends (wave 0030): a probe that builds its own
    // directory in the shared temp is exactly the leak that court
    // exists to stop, and it holds this probe too.
    let bin = common::sandbox(&format!("nobin-{tool}"));
    // Looked for as a FILE on the real PATH. `command -v true`
    // answers with the shell's builtin -- a bare word, not a path --
    // and symlinking that leaves a dangling link, so the bare PATH
    // silently lacked what it was meant to carry.
    let path = std::env::var("PATH").unwrap_or_default();
    for want in needs {
        let Some(real) = path
            .split(':')
            .map(|dir| PathBuf::from(dir).join(want))
            .find(|candidate| candidate.is_file())
        else {
            panic!("{want} must be a real file on PATH for this probe to build its world");
        };
        std::os::unix::fs::symlink(&real, bin.join(want)).unwrap();
    }
    bin
}

/// proves: a-probe-without-its-tool-stops-aloud@413d37
#[test]
fn a_probe_without_its_tool_stops_aloud() {
    let Some(probe) = probe_binary("elixir_border_test-") else {
        panic!("the elixir probe binary was not found beside this one");
    };
    // `true` travels with it deliberately: a runner without elixir
    // still has coreutils, and a PATH so bare that EVERY probe of the
    // machine fails would let a hand that asks the wrong question
    // pass for one that asks the right one.
    let bare = path_without("mix", &["git", "sh", "true"]);
    let bare_path = bare.display().to_string();
    // It really is absent under that PATH -- otherwise this proves
    // nothing at all.
    let there = Command::new("sh")
        .args(["-c", "command -v mix"])
        .env("PATH", &bare_path)
        .output()
        .unwrap();
    assert!(
        !there.status.success(),
        "the bare PATH must not carry mix, or this probe measures \
         nothing"
    );

    // `--nocapture`, or the harness swallows the very sentence this
    // probe exists to read: without it the assertion below was
    // satisfied by a TEST NAME that happens to contain "mix", and a
    // mutation that made the skip name the wrong tool survived.
    // Off a declared runner: the child must not inherit this
    // machine's `CI`, or the skip it plays turns into the fall of
    // wave 0053 (the same trap review 0053 R-1 found in the probe of
    // the runner itself; measured under `CI=true` before the merge).
    let out = Command::new(&probe)
        .args(["--test-threads", "1", "--nocapture"])
        .env("PATH", &bare_path)
        .env_remove("CI")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "a probe whose runner is not on the machine does not go RED: \
         it has judged nothing, and a red says it judged and found \
         fault:\n{said}"
    );
    assert!(
        said.contains("skipped, and said aloud") && said.contains("`mix` is not on this machine"),
        "and it says aloud WHAT it lacked, in the skip itself and not \
         merely in a test's name, so a person reading the log knows \
         what was not judged:\n{said}"
    );
    // On a declared runner the same lack is a fall by name (wave
    // 0053): a runner that promised the whole battery and has no
    // elixir is the runner's fault, and the log names the tool.
    let out = Command::new(&probe)
        .args(["--test-threads", "1", "--nocapture"])
        .env("PATH", &bare_path)
        .env("CI", "true")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "under CI the probe without its tool goes red instead of skipping:\n{said}"
    );
    assert!(
        said.contains("on a declared runner (CI is set)") && said.contains("`mix`"),
        "and the fall names the runner's lack:\n{said}"
    );

    // The fifth tongue's probe, under a PATH without node (review
    // 0046 R-11): the same hand, the same word, naming `node`.
    let Some(probe) = probe_binary("javascript_border_test-") else {
        panic!("the javascript probe binary was not found beside this one");
    };
    let bare = path_without("node", &["git", "sh", "true"]);
    let bare_path = bare.display().to_string();
    let there = Command::new("sh")
        .args(["-c", "command -v node"])
        .env("PATH", &bare_path)
        .output()
        .unwrap();
    assert!(
        !there.status.success(),
        "the bare PATH must not carry node, or this probe measures nothing"
    );
    let out = Command::new(&probe)
        .args(["--test-threads", "1", "--nocapture"])
        .env("PATH", &bare_path)
        .env_remove("CI")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        out.status.code().unwrap_or(-1),
        0,
        "a javascript probe without node does not go red:\n{said}"
    );
    assert!(
        said.contains("`node` is not on this machine"),
        "and names node as what it lacked:\n{said}"
    );
}

/// The two halves review 0044 found unjudged: a tool that IS on PATH
/// and will not run is not the same as one that is absent (m20), and
/// a machine that HAS the tool must be JUDGED, not skipped (m22) --
/// a skip that fires where everything is installed is a way of not
/// judging at all, which is worse than the red it replaced.
#[test]
fn the_two_answers_a_machine_can_give() {
    // Present and working: judge.
    assert!(
        common::machine_has("git").ready(),
        "git is here and runs, so the case is judged"
    );

    // Absent: skip, and the skip names it.
    let bin = common::sandbox("machine-answers");
    match common::machine_has("no-such-tool-413d37") {
        common::Machine::Has => panic!("a tool that does not exist is not here"),
        common::Machine::Lacks(why) => assert!(
            why.contains("no-such-tool-413d37") && why.contains("not on this machine"),
            "the skip names what is missing: {why}"
        ),
    }

    // On PATH and will NOT run -- a shim that leaves with 3. The
    // difference matters: "absent" is a machine's shape, "will not
    // run" is a machine's fault, and a person reading the log needs
    // to know which one stopped the court.
    let shim = bin.join("brokentool");
    std::fs::write(&shim, "#!/bin/sh\nexit 3\n").unwrap();
    let mut perms = std::fs::metadata(&shim).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
    std::fs::set_permissions(&shim, perms).unwrap();
    // Asked by its own path, not through PATH: the first cut set the
    // process's PATH around the question, and the tests of this
    // binary run in parallel threads -- `set_var` races every other
    // thread's `Command`, and the closing court of wave 0050 caught
    // this probe red in one of its three runs (§7.13 doing its job).
    let answer = common::machine_has(shim.to_str().unwrap());
    match answer {
        common::Machine::Has => panic!("a tool that exits 3 did not answer"),
        common::Machine::Lacks(why) => assert!(
            why.contains("left with 3") && why.contains("will not run"),
            "a tool on PATH that will not run is told from an absent \
             one, and both are told from a verdict: {why}"
        ),
    }
}
