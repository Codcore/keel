//! Scenario test of wave 0039: the pin has a hand.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, sandbox};

use std::fs;
use std::path::Path;
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

fn pinned(name: &str, version: &str) -> Sandbox {
    let dir = sandbox(name);
    fs::write(
        dir.join("keel.toml"),
        format!("lang = \"en\"\nadapter = \"rust\"\nversion = \"{version}\"\n"),
    )
    .unwrap();
    dir
}

/// The installer, read as text: it is a shell script and this is a
/// Rust battery, so what can be judged here is that the hand exists
/// and takes what the verdict tells a person to give it. That border
/// is named rather than dressed up -- running it would clone the
/// repository over the network.
fn installer() -> String {
    // From the crate's own directory, not from a sandbox in /tmp and
    // not from a relative path that depends on where cargo was run.
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    fs::read_to_string(root.join("install.sh")).unwrap()
}

/// proves: the-pin-has-a-hand@37ae08 -- `keel version` over a mismatched
/// pin said the courts refuse until the pin and the binary meet, and
/// named no hand that makes them meet. install.sh took no version at
/// all: it cloned and built `main`, whatever keel.toml said. The
/// advice pointed at an action the tool could not do.
#[test]
fn the_pin_has_a_hand() {
    let dir = pinned("mismatch", "0.0.1-not-this-binary");
    let (said, code) = keel(&["version", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the lamp never refuses over a mismatch:\n{said}");
    assert!(
        said.contains("0.0.1-not-this-binary"),
        "it names the pin:\n{said}"
    );
    assert!(
        said.contains("KEEL_REF=\"0.0.1-not-this-binary\""),
        "and the command that fetches exactly that version, with the \
         version in it:\n{said}"
    );
    assert!(
        said.contains("install.sh"),
        "which is the hand the project already ships:\n{said}"
    );

    // A pin that IS this binary asks for nothing.
    let dir = pinned("held", env!("CARGO_PKG_VERSION"));
    let (said, code) = keel(&["version", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "a held pin is a green lamp:\n{said}");
    assert!(
        !said.contains("KEEL_REF"),
        "and asks for no fetching:\n{said}"
    );

    // The hand itself takes the version, by argument and by variable,
    // and refuses a ref that is not there instead of building main.
    let script = installer();
    assert!(
        script.contains("KEEL_REF"),
        "install.sh reads KEEL_REF:\n{script}"
    );
    assert!(
        script.contains("git -C \"$KEEL_HOME\" checkout"),
        "and checks the named ref out, rather than building whatever \
         main happens to be:\n{script}"
    );
    assert!(
        script.contains("keel: no such version"),
        "a ref that is not there refuses by name:\n{script}"
    );
}
