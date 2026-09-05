//! Scenario test of wave 0037: the closing says what failed.

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

/// proves: the-closing-says-what-failed@35bb03 -- the bug audit (B6)
/// measured `keel close` running the battery three times, seeing red
/// and saying only that the wave is not closed: which test failed,
/// the court knew and did not say, so a person had to run the whole
/// battery again to learn what the court had just watched.
#[test]
fn the_closing_says_what_failed() {
    let dir = keel_sandbox("closingred");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{d}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    let rev = keel::rev::text_rev("тіло обіцянки\n");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    // One test that passes and one that fails, so the court has both
    // to tell apart.
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{}}\n\n#[test]\nfn falls_over() {{\n    panic!(\"навмисна поломка\");\n}}\n"
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);

    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(
        said.contains("falls_over"),
        "the court names the test it just watched fail (bug audit \
         B6):\n{said}"
    );
    assert!(
        !said.contains("holds_it"),
        "and does not drown it in the names of the green ones:\n{said}"
    );
    assert!(
        said.contains("червоний тест") && said.contains("у кожному бігу"),
        "a test that failed every time is called steadily red:\n{said}"
    );

    // A test that fails in some runs and not others is called flaky
    // by name -- which is the whole reason §7.13 runs the battery
    // three times. Review 0037 R-12 measured this word held by no
    // probe at all.
    let dir = keel_sandbox("closingflaky");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{d}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    let rev = keel::rev::text_rev("тіло обіцянки\n");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    // A counter on disk: the test fails on the first of the three
    // runs and passes afterwards.
    let counter = dir.join("runs.txt");
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{}}\n\n#[test]\nfn flaky_one() {{\n    let at = std::path::Path::new({:?});\n    let seen: u32 = std::fs::read_to_string(at)\n        .ok()\n        .and_then(|t| t.trim().parse().ok())\n        .unwrap_or(0);\n    std::fs::write(at, (seen + 1).to_string()).unwrap();\n    assert!(seen > 0, \"падаю лише першого разу\");\n}}\n",
            counter.to_str().unwrap()
        ),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);

    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        said.contains("хиткий тест") && said.contains("flaky_one"),
        "a test that failed in some runs and not others is called \
         flaky by name (§7.13):\n{said}"
    );
}
