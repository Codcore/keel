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
    // The path in the words is the path that was looked at. Review
    // 0035 R-12: asking only for "src/" let the message name any
    // file at all -- a lie about where the tool looked would have
    // passed, and for `/etc/passwd` it printed `src//etc/passwd.rs`
    // while `Path::join` had gone to `/etc/passwd.rs`.
    for (module, looked) in [
        ("totally-bogus-name", "src/totally-bogus-name.rs"),
        ("keel::nowhere", "src/nowhere.rs"),
        ("toy::alpha::beta", "src/alpha/beta.rs"),
    ] {
        let dir = project("missing", module);
        let (said, _) = keel(&["check", dir.to_str().unwrap()]);
        assert!(
            said.contains("червоне") && said.contains("probe"),
            "a contract naming `{module}` is a finding:\n{said}"
        );
        assert!(
            said.contains(looked),
            "and the finding says where the module was looked for \
             ({looked}):\n{said}"
        );
        assert!(
            !said.contains("сигнатур звірено: 1"),
            "and its signatures are not counted as checked:\n{said}"
        );
    }

    // A name that leads out of the crate is a finding too -- and this
    // one bites hardest (review 0035 R-2): `Path::join` with an
    // absolute component throws away everything before it, so the
    // court used to read a file outside the project and call the
    // contract held. Both names below point at a real file holding a
    // real `do_it`, so without the guard the verdict goes green.
    let dir = project("escapes", "toy");
    std::fs::write(dir.join("elsewhere.rs"), "pub fn do_it() {}\n").unwrap();
    let outside = dir.join("elsewhere");
    for module in [outside.to_str().unwrap(), "toy::..::elsewhere"] {
        std::fs::write(
            dir.join("keel/contracts/probe.md"),
            format!("---\nmodule: {module}\nexports: [do_it]\n---\n\nтекст контракту\n"),
        )
        .unwrap();
        let (said, _) = keel(&["check", dir.to_str().unwrap()]);
        assert!(
            said.contains("червоне") && said.contains("probe"),
            "a contract naming `{module}` is a finding:\n{said}"
        );
        assert!(
            said.contains("за його межі"),
            "and the finding says the name leads out of the crate:\n{said}"
        );
        assert!(
            !said.contains("сигнатур звірено: 1"),
            "and nothing outside the crate is counted as compared:\n{said}"
        );
    }

    // The real one still passes, and is counted.
    let dir = project("present", "toy");
    let (said, _) = keel(&["check", dir.to_str().unwrap()]);
    assert!(
        said.contains("сигнатур звірено: 1"),
        "a contract naming the crate itself is compared as before:\n{said}"
    );

    // And so does a module in a directory of its own: `src/a/mod.rs`
    // is as lawful a place as `src/a.rs`, and calling it missing was
    // the same false verdict the other way round (review 0035 R-5).
    let dir = project("as-dir", "toy::inner");
    std::fs::create_dir_all(dir.join("src/inner")).unwrap();
    std::fs::write(dir.join("src/inner/mod.rs"), "pub fn do_it() {}\n").unwrap();
    let (said, _) = keel(&["check", dir.to_str().unwrap()]);
    assert!(
        said.contains("сигнатур звірено: 1"),
        "a module living in src/inner/mod.rs is found and compared:\n{said}"
    );
}
