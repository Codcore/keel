//! Scenario test of wave 0043: a red test holds the wave open.
//!
//! The closing court runs the battery three times (§7.13), watches a
//! test fail, names it -- and, before this wave, closed the wave
//! anyway with rc 0. Measured identically in rust, ruby and elixir,
//! which is why the probe plays all three: the hole is in the court,
//! not in an adapter.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

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

fn closing(dir: &Path) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
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

fn have(tool: &str) -> bool {
    Command::new(tool)
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
}

/// A wave with one chore transform and no promises: it closes light
/// (§2.11), which is exactly the state that used to swallow the red.
fn wave_file(dir: &Path) {
    let mut decisions = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        decisions.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"робота без обіцянок\"\n    files:\n      - lib\n{decisions}---\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "init"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

/// The whole scenario, in the three tongues this release leads.
///
/// proves: a-red-test-holds-the-wave-open@80e2b5
#[test]
fn a_red_test_holds_the_wave_open() {
    // --- rust ---
    let dir = keel_sandbox("redrust");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(
        dir.join("tests/toy_test.rs"),
        "#[test]\nfn nobody_claims_me() {\n    assert!(false, \"red and unclaimed\");\n}\n",
    )
    .unwrap();
    wave_file(&dir);
    settle(&dir);
    let (said, code) = closing(&dir);
    assert!(
        said.contains("червоний тест") && said.contains("nobody_claims_me"),
        "the court names what it watched fail:\n{said}"
    );
    // The footer, and the NUMBERS in it. Both were free to say
    // anything: the reviewer's M02 removed the footer entirely and
    // M09 zeroed the count, and the battery noticed neither.
    assert!(
        said.contains("батарея бачила червоне: 1"),
        "the footer says how many, and one fell:\n{said}"
    );
    assert!(
        said.contains("червона (1)"),
        "and the wave's own line carries the same number:\n{said}"
    );
    assert_ne!(
        code, 0,
        "and a court that saw red does not close the wave:\n{said}"
    );
    assert!(
        !said.contains("закрита"),
        "no verdict above reads as closure:\n{said}"
    );

    // --- ruby ---
    if have("ruby") {
        let dir = keel_sandbox("redruby");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
        std::fs::write(
            dir.join("lib/toy.rb"),
            "class Toy\n  def works\n    true\n  end\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("test/toy_test.rb"),
            "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_nobody_claims_me\n    assert false, \"red and unclaimed\"\n  end\nend\n",
        )
        .unwrap();
        wave_file(&dir);
        settle(&dir);
        let (said, code) = closing(&dir);
        assert!(
            said.contains("червоний тест") && said.contains("test_nobody_claims_me"),
            "ruby's red is named too:\n{said}"
        );
        assert_ne!(code, 0, "and holds the wave open in ruby:\n{said}");
    }

    // --- elixir ---
    if have("mix") {
        let dir = keel_sandbox("redelixir");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("test")).unwrap();
        std::fs::write(
            dir.join("keel.toml"),
            "lang = \"uk\"\nadapter = \"elixir\"\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("mix.exs"),
            "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\n  def application, do: []\nend\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("lib/toy.ex"),
            "defmodule Toy do\n  def works, do: true\nend\n",
        )
        .unwrap();
        std::fs::write(dir.join("test/test_helper.exs"), "ExUnit.start()\n").unwrap();
        std::fs::write(
            dir.join("test/toy_test.exs"),
            "defmodule ToyTest do\n  use ExUnit.Case\n\n  test \"nobody claims me\" do\n    assert 1 == 2\n  end\nend\n",
        )
        .unwrap();
        wave_file(&dir);
        settle(&dir);
        let (said, code) = closing(&dir);
        assert!(
            said.contains("червоний тест") && said.contains("nobody claims me"),
            "elixir's red is named too:\n{said}"
        );
        assert_ne!(code, 0, "and holds the wave open in elixir:\n{said}");
    }
}

/// A wave file with one scenario, one transform and one review.
fn claiming_wave(dir: &Path, rev: &str, cancelled: Option<&str>) {
    let mut decisions = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            decisions.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    let off = cancelled
        .map(|why| format!("cancelled: \"{why}\"\n"))
        .unwrap_or_default();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\n{off}scenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{decisions}---\n\n## scenario: it-holds\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    let _ = rev;
}

/// A rust sandbox with the given test file, ready for `keel close`.
fn rust_project(name: &str, tests: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(dir.join("tests/toy_test.rs"), tests).unwrap();
    dir
}

const BODY: &str = "тіло обіцянки\n\n";

/// The rest of the promise, which the first cut of this wave made
/// and did not judge (review 0043 R-11): the flaky test, the tag
/// over a name the runner does not know, a wave of an earlier
/// generation, a wave called off, and a plan branch.
///
/// proves: a-red-test-holds-the-wave-open@80e2b5
#[test]
fn the_shapes_a_red_test_comes_in() {
    let rev = keel::rev::text_rev(BODY);

    // A test that falls in ONE of the three runs is not a green one.
    // Sec. 7.13 runs three times precisely so this is visible, and
    // the file it writes into is the sandbox's own.
    let dir = rust_project(
        "redflaky",
        "use std::path::Path;\n#[test]\nfn sometimes_red() {\n    let mark = Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"run-2\");\n    if mark.exists() {\n        return;\n    }\n    let first = Path::new(env!(\"CARGO_MANIFEST_DIR\")).join(\"run-1\");\n    if first.exists() {\n        std::fs::write(&mark, \"\").unwrap();\n        panic!(\"red on the second run only\");\n    }\n    std::fs::write(&first, \"\").unwrap();\n}\n",
    );
    wave_file(&dir);
    settle(&dir);
    let (said, code) = closing(&dir);
    assert!(
        said.contains("хиткий тест") && said.contains("sometimes_red"),
        "a flaky test is named AS flaky -- not folded in with the \
         reds, and not left out:\n{said}"
    );
    assert_ne!(code, 0, "and holds the wave open like any red:\n{said}");

    // A tag over a name the runner does not know is "did not run"
    // (§7.12) -- and this wave does not turn that into a red.
    let dir = rust_project(
        "rednotrun",
        &format!("/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{}}\n"),
    );
    claiming_wave(&dir, &rev, None);
    settle(&dir);
    let (said, code) = closing(&dir);
    assert!(
        !said.contains("червоний тест"),
        "a green battery names no red:\n{said}"
    );
    assert_eq!(code, 0, "and a proven wave closes:\n{said}");

    // A red a scenario of this branch's own wave DOES claim is one
    // blocker, not two (review 0043 R-5).
    let dir = rust_project(
        "redclaimed",
        &format!(
            "/// proves: it-holds@{rev}\n#[test]\nfn holds_it() {{\n    assert!(false, \"claimed and red\");\n}}\n"
        ),
    );
    claiming_wave(&dir, &rev, None);
    settle(&dir);
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", "--json", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let json = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        json.contains("\"blockers\": 1") || json.contains("\"blockers\":1"),
        "one test fell, so one blocker -- not the same fall counted \
         twice:\n{json}"
    );

    // And the other direction of the same number: one UNCLAIMED red
    // is one blocker too, not two (the reviewer's M13 doubled it and
    // nothing noticed).
    let dir = rust_project(
        "redcount",
        "#[test]\nfn nobody_claims_me() {\n    assert!(false, \"red and unclaimed\");\n}\n",
    );
    wave_file(&dir);
    settle(&dir);
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", "--json", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let json = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(
        json.contains("\"blockers\": 1") || json.contains("\"blockers\":1"),
        "one unclaimed red is one blocker:\n{json}"
    );

    // A PLAN branch merges as a plan (§6.6), and a red somewhere in
    // the tree must not shut that gate: sec. 8.3's own words are
    // that a gate always shut stops being read (review 0043 R-7).
    let dir = rust_project(
        "redplan",
        "#[test]\nfn nobody_claims_me() {\n    assert!(false, \"red and unclaimed\");\n}\n",
    );
    claiming_wave(&dir, &rev, None);
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "init"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = closing(&dir);
    assert!(
        said.contains("план-PR") || said.contains("затверджена"),
        "the plan's own footer still says what kind of PR this is:\n{said}"
    );

    // A wave of an EARLIER generation keeps "closed": its promises
    // were proven at its time, and today's red is not its lack.
    let dir = rust_project(
        "redpast",
        "#[test]\nfn nobody_claims_me() {\n    assert!(false, \"red and unclaimed\");\n}\n",
    );
    wave_file(&dir);
    std::fs::rename(
        dir.join("keel/waves/0001-a-wave.md"),
        dir.join("keel/waves/0001-old-wave.md"),
    )
    .unwrap();
    std::fs::rename(
        dir.join("keel/reviews/0001-a-wave.md"),
        dir.join("keel/reviews/0001-old-wave.md"),
    )
    .unwrap();
    wave_file(&dir);
    settle(&dir);
    let (said, code) = closing(&dir);
    assert!(
        said.contains("0001-old-wave") && said.contains("закрита"),
        "a wave closed in an earlier generation keeps its verdict:\n{said}"
    );
    assert!(
        said.contains("0001-a-wave: НЕ закривається"),
        "and only the branch's own wave is held open:\n{said}"
    );
    assert_ne!(code, 0, "the red is a blocker all the same:\n{said}");

    // A wave CALLED OFF has nothing to prove, and the reason is the
    // point of the line (review 0043 R-6).
    let dir = rust_project(
        "redcancelled",
        "#[test]\nfn nobody_claims_me() {\n    assert!(false, \"red and unclaimed\");\n}\n",
    );
    claiming_wave(&dir, &rev, Some("передумали"));
    settle(&dir);
    let (said, code) = closing(&dir);
    assert!(
        said.contains("скасована") && said.contains("передумали"),
        "the called-off wave keeps its verdict and its reason:\n{said}"
    );
    assert_ne!(
        code, 0,
        "and the red still holds the tree, said in the footer:\n{said}"
    );
}
