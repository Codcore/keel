//! Scenario test of wave 0035: a contract names a unit that exists.

mod common;

use common::keel_sandbox;
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

fn project(name: &str, module: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn do_it() {}\n").unwrap();
    std::fs::write(
        dir.join("keel/contracts/probe.md"),
        format!("---\nmodule: {module}\nexports: [do_it]\n---\n\nтекст контракту\n"),
    )
    .unwrap();
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();
    dir
}

/// proves: a-contract-names-a-unit-that-exists@63e3f8 -- the bug
/// audit measured a contract with `module: /etc/passwd` reporting
/// "signatures checked: 1" and zero findings: a single segment made
/// the court read <crate>/src/lib.rs whatever the field said. So
/// renaming a module silently disarmed §7.6 along with every
/// signature under it -- the mirror of §7.15, where the same
/// disappearance is never forgiven to a test.
#[test]
fn a_contract_names_a_unit_that_exists() {
    // A module that is not there is a finding, and the finding says
    // where it looked.
    for module in ["totally-bogus-name", "/etc/passwd", "keel::nowhere"] {
        let dir = project("missing", module);
        let (said, _) = keel(&["check", dir.to_str().unwrap()]);
        assert!(
            said.contains("червоне") && said.contains("probe"),
            "a contract naming `{module}` is a finding:\n{said}"
        );
        assert!(
            said.contains("src/"),
            "and the finding says where the module was looked for:\n{said}"
        );
        assert!(
            !said.contains("сигнатур звірено: 1"),
            "and its signatures are not counted as checked:\n{said}"
        );
    }

    // The real one still passes, and is counted.
    let dir = project("present", "toy");
    let (said, _) = keel(&["check", dir.to_str().unwrap()]);
    assert!(
        said.contains("сигнатур звірено: 1"),
        "a contract naming the crate itself is compared as before:\n{said}"
    );
}
