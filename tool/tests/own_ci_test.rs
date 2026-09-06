//! Scenario test of wave 0053: the own CI runs the same battery.
//!
//! The global review of 2026-09-06 (tests R-10) measured this
//! repository's own CI narrower than the courts: `tool-ci.yml` ran
//! `cargo test` without ruby, elixir, python or node on the runner,
//! so eleven probes skipped themselves in silence; and the generated
//! `keel.yml` installed the tool from upstream main through
//! install.sh, so a branch was judged by a binary that was not its
//! own.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use std::path::Path;

fn repo_file(rel: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// A layout framed by `keel init`, and the workflow it was given.
fn framed(name: &str, files: &[(&str, &str)]) -> String {
    let dir = common::sandbox(name);
    let git = |args: &[&str]| {
        let out = std::process::Command::new("git")
            .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
            .args(args)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(out.status.success(), "git {args:?}");
    };
    git(&["init", "-q", "-b", "main"]);
    for (rel, text) in files {
        let path = dir.join(rel);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_keel"))
        .args([
            "init",
            dir.to_str().unwrap(),
            "--lang",
            "uk",
            "--adapter",
            "rust",
            "--no-ask",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "the frame lands in {name}:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read_to_string(dir.join(".github/workflows/keel.yml")).unwrap()
}

/// proves: the-own-ci-runs-the-same-battery@b4c9e0
#[test]
fn the_own_ci_runs_the_same_battery() {
    // --- tool-ci.yml sets up every tongue the courts judge before
    // the battery runs ---
    let ci = repo_file(".github/workflows/tool-ci.yml");
    let battery = ci.find("cargo test").expect("the battery step is there");
    for (tool, marker) in [
        ("ruby", "ruby/setup-ruby"),
        ("rspec", "gem install rspec"),
        ("elixir", "erlef/setup-beam"),
        ("python", "actions/setup-python"),
        ("pytest", "pip install pytest"),
        ("node", "actions/setup-node"),
    ] {
        let at = ci
            .find(marker)
            .unwrap_or_else(|| panic!("tool-ci.yml sets up {tool} ({marker}):\n{ci}"));
        assert!(
            at < battery,
            "{tool} is set up before the battery runs:\n{ci}"
        );
    }

    // --- under a declared runner a missing tool is a fall by name,
    // not a skip: the battery of the runner equals the battery of the
    // courts ---
    // Both halves set the variable themselves and put it back as it
    // was: the first reading asked the environment's own CI for the
    // "off the runner" half, and on the runner -- where CI is always
    // set -- the probe of the runner fell on its own first clause
    // (review 0053 R-1).
    // SAFETY: this binary runs one test, and the variable is set for
    // the length of one call each time.
    let declared = std::env::var_os("CI");
    unsafe { std::env::remove_var("CI") };
    let lacking = common::machine_has("keel-no-such-tool-anywhere");
    assert!(
        !lacking.ready(),
        "off the runner a missing tool is a skip said aloud"
    );
    unsafe { std::env::set_var("CI", "true") };
    let fell =
        std::panic::catch_unwind(|| common::machine_has("keel-no-such-tool-anywhere").ready());
    match &declared {
        Some(value) => unsafe { std::env::set_var("CI", value) },
        None => unsafe { std::env::remove_var("CI") },
    }
    let words = fell
        .err()
        .map(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
                .unwrap_or_default()
        })
        .unwrap_or_else(|| {
            panic!("on the runner (CI set) a missing tool fails the probe instead of skipping it")
        });
    // BY NAME, which is what makes a red runner readable at all: the
    // first reading held only that it fell (final review 2026-09-06,
    // tests R-6).
    assert!(
        words.contains("keel-no-such-tool-anywhere") && words.contains("CI"),
        "and the fall names the tool that was missing and why it is a \
         fall here:\n{words}"
    );

    // --- keel's own workflow judges the branch by the branch's own
    // binary, built from the tree, not fetched from upstream main ---
    let workflow = repo_file(".github/workflows/keel.yml");
    assert!(
        workflow.contains("cargo build") && workflow.contains("tool/Cargo.toml"),
        "keel.yml builds the tool from the checked-out tree:\n{workflow}"
    );
    assert!(
        !workflow.contains("install.sh"),
        "and does not fetch another tree's binary:\n{workflow}"
    );
    // The toolchain this project pins is named BEFORE the build that
    // uses it (review 0053 R-12: the pin step stood after the courts).
    let pinned = workflow
        .find("rustup toolchain install")
        .expect("the workflow names the toolchain this project pins");
    let built = workflow.find("cargo build").unwrap();
    assert!(
        pinned < built,
        "the toolchain is named before the tool is built with it:\n{workflow}"
    );
    // The tool's own repository is known by two marks, and only by
    // both (review 0053 R-7): a stranger's crate that merely shares
    // the name, and a stranger's tree that merely carries an
    // install.sh, both get the installer step.
    let crate_named = |name: &str| {
        format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n")
    };
    let by_name = framed(
        "ownbyname",
        &[("Cargo.toml", &crate_named("keel")), ("src/lib.rs", "")],
    );
    assert!(
        by_name.contains("install.sh") && !by_name.contains("cargo build"),
        "a crate that merely shares the name gets the installer step:\n{by_name}"
    );
    let by_file = framed(
        "ownbyfile",
        &[
            ("Cargo.toml", &crate_named("toy")),
            ("src/lib.rs", ""),
            ("install.sh", "#!/bin/sh\n"),
        ],
    );
    assert!(
        by_file.contains("install.sh") && !by_file.contains("cargo build"),
        "a tree that merely carries an install.sh gets the installer step:\n{by_file}"
    );
    let both = framed(
        "ownboth",
        &[
            ("tool/Cargo.toml", &crate_named("keel")),
            ("tool/src/lib.rs", ""),
            ("install.sh", "#!/bin/sh\n"),
        ],
    );
    assert!(
        both.contains("cargo build --manifest-path tool/Cargo.toml")
            && !both.contains("install.sh"),
        "both marks together: the tool builds from its own tree:\n{both}"
    );
    // The file stands as the tool left it: its recorded digest is
    // the file's own.
    let config = repo_file("keel.toml");
    let recorded = config
        .lines()
        .find(|l| l.contains("\".github/workflows/keel.yml\""))
        .and_then(|l| l.split('"').nth(3))
        .expect("the workflow's digest is recorded");
    assert_eq!(
        recorded,
        keel::generated::digest(&workflow),
        "the recorded digest is the file's own"
    );
}
