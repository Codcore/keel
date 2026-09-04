//! Scenario test of wave 0036: the weight comes from the file.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

/// git with an identity of its own: review 0036 R-11 measured all
/// five probes of this wave failing on a machine with no global
/// git config -- a fresh CI container, that is -- while the
/// twenty-one older ones, which pass `-c user.email`, held.
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

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    // `status` is the eye of the stages and needs the adapter.
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: the-weight-comes-from-the-file@5194b3 -- §6.8 states the
/// rule exactly (one transform, no contracts, nothing withdrawn) and
/// nothing computed it. The norm audit (В-2) and the conformance
/// audit (ВАЖКА-6) both landed on the same consequence: a chore with
/// a NEW CONTRACT calls itself light and rides in on one PR --
/// without the second human look §6.8 demands for exactly that case.
#[test]
fn the_weight_comes_from_the_file() {
    // A light wave: one transform, no contract, nothing withdrawn.
    // It may ride one branch, and the weight is said aloud.
    let dir = project("light");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            {
                let mut d = String::from("decisions:\n");
                for cut in keel::graph::cuts() {
                    if *cut != "functional.correctness" {
                        d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
                    }
                }
                d
            }
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn b() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: the light wave"]);

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага легка"),
        "the weight is derived and said aloud (§6.8) -- in the weight \
         line's own words, since review 0036 R-1 found these asserts \
         satisfied by an older line counting by another rule:\n{said}"
    );
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("повна хвиля"),
        "a light wave riding one branch is lawful:\n{said}"
    );

    // A wave that grows a CONTRACT is full whatever it calls itself,
    // and a full wave born on its own work branch never had the
    // plan PR §6.8 asks for -- that is the finding.
    let dir = project("full");
    git(&dir, &["checkout", "-q", "-b", "0002-b-wave"]);
    std::fs::write(
        dir.join("keel/contracts/fresh.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0002-b-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n      - keel/contracts/fresh.md\n{}---\n\n## transform: work\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn c() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "work: a chore with a new contract"],
    );

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага повна"),
        "a wave that grows a contract is full, whatever it calls \
         itself (§6.8):\n{said}"
    );
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 1, "and riding one branch is a finding:\n{said}");
    assert!(
        said.contains("0002-b-wave") && said.contains("двох"),
        "the finding names the wave and says what is missing -- the \
         two human looks §6.8 asks for:\n{said}"
    );

    // Withdrawing a promise makes a wave full too: a promise dying is
    // exactly the risk §6.8 wants two people to see.
    let dir = project("withdrawn");
    git(&dir, &["checkout", "-q", "-b", "0003-c-wave"]);
    std::fs::write(
        dir.join("keel/waves/0003-c-wave.md"),
        format!(
            "---\nscenarios:\n  gone:\n    covers: [functional.correctness]\n    withdrawn: \"згорнуто\"\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: gone\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn d() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: a withdrawal"]);

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага повна"),
        "a wave that withdraws a promise is full (§6.8):\n{said}"
    );
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 1, "and riding one branch is a finding:\n{said}");

    // Two transforms make a wave full -- the FIRST clause of §6.8,
    // which review 0036 R-3 (M8) measured held by nothing at all.
    let dir = project("twotransforms");
    git(&dir, &["checkout", "-q", "-b", "0004-d-wave"]);
    std::fs::write(
        dir.join("keel/waves/0004-d-wave.md"),
        format!(
            "---\ntransforms:\n  one:\n    chore: \"перша\"\n    files:\n      - src/lib.rs\n  two:\n    chore: \"друга\"\n    files:\n      - README.md\n{}---\n\n## transform: one\nтіло\n\n## transform: two\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn e() {}\n").unwrap();
    std::fs::write(dir.join("README.md"), "текст\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: two transforms"]);
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 1, "two transforms make a wave full (§6.8):\n{said}");
    assert!(
        said.contains("0004-d-wave") && said.contains("двох"),
        "and the finding names the wave:\n{said}"
    );

    // `one new in keel/contracts/` grows a contract just as much as
    // naming the file does -- review 0036 R-4 measured exactly the
    // hole the Why calls closed sailing through as light.
    let dir = project("onenewin");
    git(&dir, &["checkout", "-q", "-b", "0005-e-wave"]);
    std::fs::write(
        dir.join("keel/waves/0005-e-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n      - one new in keel/contracts/\n{}---\n\n## transform: work\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/grown.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn f() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "work: a contract through one new in"],
    );
    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага повна"),
        "`one new in keel/contracts/` grows a contract too (§6.8, \
         §4.1):\n{said}"
    );
    // And `close` obeys the same weight: review 0036 R-2 measured it
    // calling such a wave light and green, which is the second human
    // look §6.8 buys being handed back.
    let (said, code) = keel(&dir, "close");
    assert!(
        code != 0 || !said.contains("закрита"),
        "a full wave is not closed by merge alone, whatever it \
         carries (§6.8, §9.9):\n{said}"
    );

    // Leaning on a contract is not changing one: chapter 3's own
    // vocabulary calls `contracts:` "what the work leans on", and
    // review 0036 R-9 measured a lawful light wave turned red by it.
    let dir = project("leaning");
    std::fs::write(
        dir.join("keel/contracts/keeper.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "chore: a contract to lean on"],
    );
    let rev = keel::rev::contract_rev(&dir.join("keel/contracts/keeper.md")).unwrap();
    git(&dir, &["checkout", "-q", "-b", "0006-f-wave"]);
    std::fs::write(
        dir.join("keel/waves/0006-f-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    contracts: [keeper@{rev}]\n    files:\n      - src/lib.rs\n{}---\n\n## transform: work\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn g() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: leaning on a contract"]);
    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага легка"),
        "leaning on a contract changes none, so the wave stays light \
         (§6.8, chapter 3):\n{said}"
    );
    let (said, code) = keel(&dir, "check");
    assert_eq!(
        code, 0,
        "and a lawful light wave is not turned red by it:\n{said}"
    );

    // A full wave whose file was merged into the trunk BEFORE the
    // work began -- the ordinary shape of §8.1 once the plan PR has
    // landed -- is not a finding either. Review 0036 R-3 (M14)
    // measured the "born in this very diff" clause held by nothing:
    // removing it left the battery green.
    let dir = project("planlanded");
    std::fs::write(
        dir.join("keel/waves/0008-h-wave.md"),
        format!(
            "---\ntransforms:\n  one:\n    chore: \"перша\"\n    files:\n      - src/lib.rs\n  two:\n    chore: \"друга\"\n    files:\n      - README.md\n{}---\n\n## transform: one\nтіло\n\n## transform: two\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "merge: wave 0008 plan"]);
    git(&dir, &["checkout", "-q", "-b", "0008-h-wave"]);
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn i() {}\n").unwrap();
    std::fs::write(dir.join("README.md"), "текст\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "one: the work"]);
    let (said, code) = keel(&dir, "check");
    assert_eq!(
        code, 0,
        "a full wave whose plan landed in the trunk before the work \
         is lawful (§8.1):\n{said}"
    );
    assert!(
        !said.contains("народився на цій самій гілці"),
        "and it is not accused of riding one branch:\n{said}"
    );

    // A full wave whose file came from its OWN plan branch is the
    // lawful §8.1 sequence, not a finding -- review 0036 R-10
    // measured it accused, with an instead telling the author to do
    // what they had already done.
    let dir = project("twoprs");
    git(&dir, &["checkout", "-q", "-b", "plan/0007-g-wave"]);
    std::fs::write(
        dir.join("keel/waves/0007-g-wave.md"),
        format!(
            "---\ntransforms:\n  one:\n    chore: \"перша\"\n    files:\n      - src/lib.rs\n  two:\n    chore: \"друга\"\n    files:\n      - README.md\n{}---\n\n## transform: one\nтіло\n\n## transform: two\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "plan: wave 0007"]);
    git(&dir, &["checkout", "-q", "-b", "0007-g-wave"]);
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn h() {}\n").unwrap();
    std::fs::write(dir.join("README.md"), "текст\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "one: the work"]);
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("народився на цій самій гілці"),
        "the lawful two-PR sequence of §8.1 is not a finding:\n{said}"
    );
}
