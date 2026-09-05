//! Scenario test of wave 0038: the adapter is chosen by name.

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

fn keel(dir: &Path, command: &str) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args([command, dir.to_str().unwrap()])
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

fn decided() -> String {
    let mut out = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        out.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    out
}

/// A project whose config names whatever adapter the case wants.
fn project(name: &str, adapter: Option<&str>) -> common::Sandbox {
    let dir = keel_sandbox(name);
    let line = match adapter {
        Some(named) => format!("lang = \"uk\"\nadapter = \"{named}\"\n"),
        None => "lang = \"uk\"\n".to_string(),
    };
    std::fs::write(dir.join("keel.toml"), line).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - README.md\n{}---\n\n## transform: tidy\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("README.md"), "проєкт\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: the-adapter-is-chosen-by-name@ac3adc -- the operator read
/// the README and asked where the other languages were. There was
/// one adapter, cargo, and it was there so keel could judge itself:
/// `rust_adapter()` was asked in eleven places across ten modules,
/// and every one of them meant "is this project Rust", not "which
/// language is this".
#[test]
fn the_adapter_is_chosen_by_name() {
    // The two names this release knows.
    for named in ["rust", "cargo", "ruby"] {
        let dir = project("known", Some(named));
        let (said, code) = keel(&dir, "check");
        assert_eq!(
            code, 0,
            "`adapter = \"{named}\"` is a name this release knows:\n{said}"
        );
        assert!(
            !said.contains("червоне"),
            "and naming it is not a finding:\n{said}"
        );
    }

    // A name it does not know is a REFUSAL that lists what it does --
    // never a silent skip of the language-shaped courts.
    let dir = project("unknown", Some("kotlin"));
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 2, "an unknown adapter is refused:\n{said}");
    assert!(
        said.contains("kotlin") && said.contains("rust") && said.contains("ruby"),
        "and the refusal names what was asked for and what is \
         known:\n{said}"
    );

    // No adapter at all: the courts that need one are skipped, and
    // the verdict says which -- as before this wave.
    let dir = project("none", None);
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 0, "a project without an adapter is judged:\n{said}");
    assert!(
        said.contains("адаптер") && said.contains("не званий") || said.contains("не названий"),
        "and the skipped courts are named aloud:\n{said}"
    );
}
