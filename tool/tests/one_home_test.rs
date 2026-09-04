//! Scenario test of wave 0035: a scenario name belongs to one wave.

mod common;

use common::keel_sandbox;
use std::process::Command;

const WAVE: &str = "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-works\nтіло обіцянки\n\n## transform: work\nтіло роботи\n";

/// proves: a-scenario-name-belongs-to-one-wave@e10ffa -- the bug
/// audit copied a wave under a new number and `keel close` called
/// BOTH closed, though the second had no test of its own. A test tag
/// is a bare name, so the machine has no way to know whose promise
/// it proves. The norm never says the slugs are unique either
/// (methodology audit С-9).
#[test]
fn a_scenario_name_belongs_to_one_wave() {
    let dir = keel_sandbox("onehome");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-first.md"), WAVE).unwrap();
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&dir)
        .status()
        .unwrap();

    let said = check(&dir);
    assert!(
        !said.contains("it-works") || !said.contains("червоне"),
        "one wave with the name is nobody's business:\n{said}"
    );

    // The same name in a second wave: the tag cannot say whose it is.
    std::fs::write(dir.join("keel/waves/0002-second.md"), WAVE).unwrap();
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("it-works"),
        "two waves with one scenario name is a finding:\n{said}"
    );
    assert!(
        said.contains("0001-first") && said.contains("0002-second"),
        "and it names both homes, so the fix is obvious:\n{said}"
    );

    // And a namesake WITHDRAWN in the second wave does not forgive
    // the living one its tags. Review 0035 R-4: the courts kept the
    // withdrawn names in one set keyed by the bare name, so a single
    // `withdrawn:` line anywhere silenced every tag of the living
    // promise -- stale and orphan alike -- and the verdict said
    // nothing at all.
    std::fs::write(
        dir.join("keel/waves/0002-second.md"),
        "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\n    withdrawn: \"згорнуто в першу\"\ntransforms:\n  work:\n    chore: \"прибирання\"\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-works\nстаре тіло\n\n## transform: work\nтіло роботи\n",
    )
    .unwrap();
    // The tag floor reads `tests/*.rs` of a rust crate, so the
    // fixture becomes one.
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        "/// proves: it-works@beef00\n#[test]\nfn holds_it() {}\n",
    )
    .unwrap();
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("it-works"),
        "a tag on the LIVING promise is judged even where a namesake \
         is withdrawn elsewhere:\n{said}"
    );
    assert!(
        said.contains("beef00"),
        "and the finding shows the tag's stale revision:\n{said}"
    );

    // Three homes are not two. Review 0035 R-11: the words said
    // "lives in two waves" and then listed three, so the sentence
    // argued with its own list.
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::write(dir.join("keel/waves/0002-second.md"), WAVE).unwrap();
    std::fs::write(dir.join("keel/waves/0003-third.md"), WAVE).unwrap();
    let said = check(&dir);
    assert!(
        said.contains("0003-third") && !said.contains("двох хвилях"),
        "three homes are named, and no number argues with the \
         list:\n{said}"
    );

    // And the contracts share the namespace: a bare tag cannot say
    // whether `x@rev` holds a promise's revision or a contract's
    // (review 0035 R-17).
    std::fs::remove_file(dir.join("keel/waves/0002-second.md")).unwrap();
    std::fs::remove_file(dir.join("keel/waves/0003-third.md")).unwrap();
    std::fs::write(
        dir.join("keel/contracts/it-works.md"),
        "---\nmodule: toy\nexports: [\"pub fn it()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("it-works"),
        "a contract wearing a promise's name is a finding:\n{said}"
    );
    assert!(
        said.contains("в одному просторі"),
        "and the words say why -- one namespace for both:\n{said}"
    );
}

fn check(dir: &std::path::Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}
