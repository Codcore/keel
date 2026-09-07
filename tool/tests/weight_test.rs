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

    // The same contract, written with a leading `./` -- one file, one
    // weight (wave 0057; review R-4 measured this fix without a
    // probe, and a mutant that read the raw spelling passed the whole
    // battery). Before the normalisation `./keel/contracts/…` slipped
    // past the rule and the wave called itself light.
    let dir = project("full-dot");
    git(&dir, &["checkout", "-q", "-b", "0004-d-wave"]);
    std::fs::write(
        dir.join("keel/contracts/fresh.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0004-d-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - ./src/lib.rs\n      - ./keel/contracts/fresh.md\n{}---\n\n## transform: work\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn e() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "work: a chore with a dotted contract"],
    );
    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага повна"),
        "a contract written as ./keel/contracts/… is the same contract \
         (§6.8):\n{said}"
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
    // Two chores and nothing else: full by the count alone (§6.8), so
    // a weight that stopped counting transforms would turn this
    // fixture green (review 0052 R-14).
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
            "---\nscenarios:\n  gone:\n    covers: []\n    withdrawn: \"знято до старту\"\ntransforms:\n  one:\n    chore: \"перша\"\n    files:\n      - src/lib.rs\n  two:\n    chore: \"друга\"\n    files:\n      - README.md\n{}---\n\n## scenario: gone\nбуло\n\n## transform: one\nтіло\n\n## transform: two\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "merge: wave 0008 plan"]);
    git(&dir, &["checkout", "-q", "-b", "0008-h-wave"]);
    // Two transforms, two commits under their slugs (§6.2, wave 0052);
    // the withdrawn promise keeps the wave off §2.11's "chores alone".
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn i() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "one: the work"]);
    std::fs::write(dir.join("README.md"), "текст\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "two: the work"]);
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
            "---\nscenarios:\n  gone:\n    covers: []\n    withdrawn: \"знято до старту\"\ntransforms:\n  one:\n    chore: \"перша\"\n    files:\n      - src/lib.rs\n  two:\n    chore: \"друга\"\n    files:\n      - README.md\n{}---\n\n## scenario: gone\nбуло\n\n## transform: one\nтіло\n\n## transform: two\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "plan: wave 0007"]);
    git(&dir, &["checkout", "-q", "-b", "0007-g-wave"]);
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn h() {}\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "one: the work"]);
    std::fs::write(dir.join("README.md"), "текст\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "two: the work"]);
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("народився на цій самій гілці"),
        "the lawful two-PR sequence of §8.1 is not a finding:\n{said}"
    );
}

/// proves: chores-alone-may-carry-a-contract@8f652e -- §2.11 said a
/// wave whose transforms are all chores must be light; §6.8 said a
/// wave that changes a contract is full. The debt of the release --
/// three paragraphs of prose in two contracts, not one new promise --
/// was both, so no such wave could exist and `keel check` refused it
/// (measured 2026-09-07). The operator's decision of 2026-09-07:
/// §6.8 wins, and §2.11 gets that exception.
#[test]
fn chores_alone_may_carry_a_contract() {
    // Chores alone, and a contract among the files: lawful, and FULL.
    let dir = project("chore-contract");
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    std::fs::write(
        dir.join("keel/contracts/fresh.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  words:\n    chore: \"проза контракту\"\n    files:\n      - keel/contracts/fresh.md\n{}---\n\n## transform: words\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "words: the prose of a contract"],
    );

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага повна"),
        "a wave that changes a contract is full (§6.8):\n{said}"
    );
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("мусить бути легкою"),
        "and §2.11 does not forbid it: the exception is exactly this \
         case (the operator's decision of 2026-09-07):\n{said}"
    );
    let (said, _) = keel(&dir, "next");
    assert!(
        !said.contains("дай їй сценарій"),
        "the step stops asking for a promise a prose fix does not \
         have, and names the weight instead:\n{said}"
    );

    // The exception is NARROW: chores alone with two transforms and
    // no contract is still the finding §2.11 exists for.
    let dir = project("chore-two");
    git(&dir, &["checkout", "-q", "-b", "0002-b-wave"]);
    std::fs::write(
        dir.join("keel/waves/0002-b-wave.md"),
        format!(
            "---\ntransforms:\n  one:\n    chore: \"перша\"\n    files:\n      - src/lib.rs\n  two:\n    chore: \"друга\"\n    files:\n      - Cargo.toml\n{}---\n\n## transform: one\nтіло\n\n## transform: two\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\npub fn f() {}\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.1\"\nedition = \"2021\"\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "one: two chores and no contract"],
    );
    let (said, code) = keel(&dir, "check");
    assert!(
        said.contains("мусить бути легкою"),
        "big work without a single promise is still a reason to stop \
         and think (§2.11):\n{said}"
    );
    assert_eq!(code, 1, "and the branch is red for it:\n{said}");

    // The exception is not narrow along the OTHER axis, and the norm
    // now says so aloud (review 0058 R-4): one contract row makes a
    // wave of ANY number of chores full. Measured here, because the
    // choice to ask about the contract apart from `docs::heavy` --
    // where `Transforms` outranks `Contract` -- is exactly what this
    // case turns on, and a mutant reading the verdict of `heavy`
    // instead walked through the whole battery.
    let dir = project("chore-two-and-a-contract");
    git(&dir, &["checkout", "-q", "-b", "0003-c-wave"]);
    std::fs::write(
        dir.join("keel/contracts/fresh.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0003-c-wave.md"),
        format!(
            "---\ntransforms:\n  words:\n    chore: \"проза контракту\"\n    files:\n      - keel/contracts/fresh.md\n  more:\n    chore: \"друга\"\n    files:\n      - Cargo.toml\n{}---\n\n## transform: words\nтіло\n\n## transform: more\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.1\"\nedition = \"2021\"\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "words: two chores and a contract"],
    );
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("мусить бути легкою"),
        "two chores and a contract: the exception holds for a wave of any \
         size, and the court asks about the contract apart from the first \
         verdict of `heavy` (§2.11, §6.8):\n{said}"
    );
    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага повна"),
        "and such a wave is full:\n{said}"
    );

    // The NAME, not the spelling (wave 0057, review 0058 R-8): the
    // exception must read `./keel/contracts/x.md` as the contract it
    // is, or a leading dot puts the wave back in the dead end.
    let dir = project("chore-contract-dot");
    git(&dir, &["checkout", "-q", "-b", "0004-d-wave"]);
    std::fs::write(
        dir.join("keel/contracts/fresh.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/waves/0004-d-wave.md"),
        format!(
            "---\ntransforms:\n  words:\n    chore: \"проза контракту\"\n    files:\n      - ./keel/contracts/fresh.md\n{}---\n\n## transform: words\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "words: the prose of a contract"],
    );
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("мусить бути легкою"),
        "a dot does not hide the contract from the exception:\n{said}"
    );

    // And the mirror of the same reading (review 0058 R-2): a
    // DIFFERENT directory whose name merely starts with the letters
    // of this one is no contract to either rule. Before the readings
    // were made one, `docs::heavy` called such a wave full and §2.11
    // demanded it light -- the dead end this wave exists to close,
    // reopened by a row nobody would write on purpose.
    let dir = project("chore-contract-lookalike");
    git(&dir, &["checkout", "-q", "-b", "0005-e-wave"]);
    std::fs::create_dir_all(dir.join("keel/contractsfoo")).unwrap();
    std::fs::write(dir.join("keel/contractsfoo/x.md"), "не контракт\n").unwrap();
    std::fs::write(
        dir.join("keel/waves/0005-e-wave.md"),
        format!(
            "---\ntransforms:\n  words:\n    chore: \"прибирання\"\n    files:\n      - one new in keel/contractsfoo/\n{}---\n\n## transform: words\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "words: a directory of its own"],
    );
    let (said, _) = keel(&dir, "check");
    assert!(
        !said.contains("мусить бути легкою"),
        "one rule, one reading: a lookalike directory is no contract, so \
         the wave is simply light and lawful (§2.11, review 0058 R-2):\n{said}"
    );
    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("вага легка"),
        "and light is what its weight says, with no second rule calling \
         it full:\n{said}"
    );

    // The step's word in ENGLISH too (review 0058 R-1): a mutant that
    // put the pre-exception text back walked the whole battery in
    // both tongues. This sandbox is the other side of the exception
    // -- the branch changes a contract the wave does NOT name --
    // because that is where the step speaks at all.
    let dir = project("chore-contract-en");
    std::fs::write(dir.join("keel.toml"), "lang = \"en\"\nadapter = \"rust\"\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the tongue of this sandbox"]);
    git(&dir, &["checkout", "-q", "-b", "0006-f-wave"]);
    std::fs::write(
        dir.join("keel/waves/0006-f-wave.md"),
        format!(
            "---\ntransforms:\n  tidy:\n    chore: \"прибирання\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("keel/reviews/0006-f-wave.md"), "# Review\n\nok\n").unwrap();
    std::fs::write(
        dir.join("keel/contracts/ext.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nчужа обіцянка\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n// tidy\n").unwrap();
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "--no-verify",
            "-m",
            "tidy: work and a contract change",
        ],
    );
    let (said, _) = keel(&dir, "next");
    assert!(
        said.contains("name it among the files") && said.contains("§6.8") && said.contains("§2.11"),
        "the step in English names the same move -- name the contract among \
         the files, and the wave is full:\n{said}"
    );
    assert!(
        said.contains("plan apart and work apart"),
        "and the same price, two approvals:\n{said}"
    );
    assert!(
        !said.contains("give it a scenario"),
        "and no longer asks for a promise a prose fix does not have:\n{said}"
    );
}
