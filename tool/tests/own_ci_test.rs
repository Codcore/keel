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
    let lacking = common::machine_has("keel-no-such-tool-anywhere");
    assert!(
        !lacking.ready(),
        "off the runner a missing tool is a skip said aloud"
    );
    // SAFETY: this binary runs one test, and the variable is set for
    // the length of one call.
    unsafe { std::env::set_var("CI", "true") };
    let fell =
        std::panic::catch_unwind(|| common::machine_has("keel-no-such-tool-anywhere").ready());
    unsafe { std::env::remove_var("CI") };
    assert!(
        fell.is_err(),
        "on the runner (CI set) a missing tool fails the probe by name instead of skipping it"
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
