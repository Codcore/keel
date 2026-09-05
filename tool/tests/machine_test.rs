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
fn path_without(tool: &str, needs: &[&str]) -> PathBuf {
    let bin = std::env::temp_dir().join(format!("keel-{}-nobin-{tool}", std::process::id()));
    let _ = std::fs::remove_dir_all(&bin);
    std::fs::create_dir_all(&bin).unwrap();
    for want in needs {
        let Ok(found) = Command::new("sh")
            .args(["-c", &format!("command -v {want}")])
            .output()
        else {
            continue;
        };
        let real = String::from_utf8_lossy(&found.stdout).trim().to_string();
        if real.is_empty() {
            continue;
        }
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
    let bare = path_without("mix", &["git", "sh"]);
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

    let out = Command::new(&probe)
        .args(["--test-threads", "1"])
        .env("PATH", &bare_path)
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
        said.contains("mix"),
        "and it says aloud what it lacked, so a person reading the log \
         knows what was not judged:\n{said}"
    );
}
