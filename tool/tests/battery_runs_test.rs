//! Scenario tests of wave 0019-ci-and-battery, transform
//! battery-runs: the closure battery runs several times (§7.13) and
//! a test is green only when green in every run.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

#[allow(unused_imports)]
use common::{Sandbox, keel_sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn keel(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

/// proves: battery-several-runs@d15536 -- holds §7.13 in the
/// closure court: the battery runs three times and the row says so;
/// a stable wave closes; a test green in the first run and red in
/// the next -- state that survived its test -- is a lack named with
/// its count of green runs, never blessed by the one green; a test
/// red in every run stays "red".
#[test]
fn battery_several_runs() {
    let dir = keel_sandbox("runs");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    );
    write(&dir, "src/lib.rs", "");

    // 0020-stable: one steady scenario, always green.
    write(
        &dir,
        "keel/waves/0020-stable.md",
        &format!(
            "---\nscenarios:\n  steady: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [steady]\n    files: [src/lib.rs]\n{}---\n\n## scenario: steady\n\nbody of steady\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let steady = keel::rev::text_rev("body of steady\n");
    write(
        &dir,
        "tests/steady_test.rs",
        &format!("/// proves: steady@{steady}\n#[test]\nfn holds_steady() {{}}\n"),
    );
    write(&dir, "keel/reviews/0020-stable.md", "# Рецензія\n\nok\n");

    // 0021-leaky: a test whose state survives it -- green once, red
    // after -- and a test red in every run.
    write(
        &dir,
        "keel/waves/0021-leaky.md",
        &format!(
            "---\nscenarios:\n  leaky: {{covers: [functional.correctness]}}\n  always-red: {{covers: [performance.capacity]}}\ntransforms:\n  t:\n    implements: [leaky, always-red]\n    files: [src/lib.rs]\n{}---\n\n## scenario: leaky\n\nbody of leaky\n\n## scenario: always-red\n\nbody of always-red\n",
            all_decided_except(&["functional.correctness", "performance.capacity"])
        ),
    );
    let leaky = keel::rev::text_rev("body of leaky\n");
    let red = keel::rev::text_rev("body of always-red\n");
    write(
        &dir,
        "tests/leaky_test.rs",
        &format!(
            "/// proves: leaky@{leaky}\n#[test]\nfn leaks() {{\n    let marker = std::path::Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"leak.marker\");\n    assert!(!marker.exists(), \"state survived its test\");\n    std::fs::write(&marker, \"\").unwrap();\n}}\n\n/// proves: always-red@{red}\n#[test]\nfn holds_red() {{ assert!(false); }}\n"
        ),
    );
    write(&dir, "keel/reviews/0021-leaky.md", "# Рецензія\n\nok\n");

    let (out, err, _) = keel(&["close", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("battery:") && out.contains("3 runs"),
        "the battery row says how many runs (§7.13):\n{out}"
    );
    assert!(
        out.contains("0020-stable: closed"),
        "a stable wave closes under several runs:\n{out}"
    );
    assert!(
        out.contains("0021-leaky: in progress"),
        "the leaky wave is not blessed by its one green run:\n{out}"
    );
    assert!(
        out.contains("\"leaky\"") && out.contains("green in 1 of 3 runs"),
        "the leaky test is a lack named with its count of green runs (§7.13):\n{out}"
    );
    assert!(
        out.contains("\"always-red\"") && out.contains("is red"),
        "a test red in every run stays red:\n{out}"
    );
}
