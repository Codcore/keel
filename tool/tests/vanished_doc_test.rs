//! Scenario test of wave 0036: a document does not vanish.

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

fn check(dir: &Path) -> String {
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

const CONTRACT: &str = "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n";

/// A project with a wave and a contract, committed on main.
fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(dir.join("keel/contracts/old-name.md"), CONTRACT).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0002-b-wave"]);
    dir
}

/// proves: a-document-does-not-vanish@519d48 -- the conformance
/// audit (ВАЖКА-3) measured §4.12 held by nothing: a wave or
/// contract file can simply be deleted, taking every promise in it
/// with it, and `renamed_from` -- the only lawful move the paragraph
/// allows -- is parsed and never read by anything.
#[test]
fn a_document_does_not_vanish() {
    // A deleted contract is a finding that names the slug.
    let dir = project("vanished");
    git(&dir, &["rm", "-q", "keel/contracts/old-name.md"]);
    git(&dir, &["commit", "-q", "-m", "chore: gone"]);
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("old-name"),
        "a deleted contract is a finding by name (§4.12):\n{said}"
    );

    // A deleted WAVE file just as much: every promise in it dies with
    // the file, and §2.12 says a promise dies by `withdrawn`.
    let dir = project("vanishedwave");
    git(&dir, &["rm", "-q", "keel/waves/0001-a-wave.md"]);
    git(&dir, &["commit", "-q", "-m", "chore: gone"]);
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("0001-a-wave"),
        "a deleted wave file is a finding by name (§4.12):\n{said}"
    );

    // Unless a living document claims the inheritance: then the
    // departure is a line in the diff, not an error.
    let dir = project("renamed");
    git(&dir, &["rm", "-q", "keel/contracts/old-name.md"]);
    std::fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    std::fs::write(
        dir.join("keel/contracts/new-name.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\nrenamed_from: old-name\n---\n\nтіло контракту\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "chore: renamed"]);
    let said = check(&dir);
    assert!(
        !said.contains("зник із гілки") && !said.contains("old-name"),
        "a rename that says so is lawful (§4.12):\n{said}"
    );

    // Two documents claiming one inheritance is a finding: the old
    // name cannot lead to both.
    let dir = project("twoheirs");
    git(&dir, &["rm", "-q", "keel/contracts/old-name.md"]);
    std::fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    for heir in ["heir-one", "heir-two"] {
        std::fs::write(
            dir.join(format!("keel/contracts/{heir}.md")),
            "---\nmodule: toy\nexports: [\"pub fn a()\"]\nrenamed_from: old-name\n---\n\nтіло контракту\n",
        )
        .unwrap();
    }
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "chore: two heirs"]);
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("heir-one") && said.contains("heir-two"),
        "two documents claiming one inheritance is a finding, and it \
         names both (§4.12):\n{said}"
    );

    // A move across directories is a finding: a wave is not a
    // contract, whatever the header says.
    let dir = project("crossmove");
    git(&dir, &["rm", "-q", "keel/contracts/old-name.md"]);
    std::fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    std::fs::write(
        dir.join("keel/waves/0003-c-wave.md"),
        "---\nrenamed_from: old-name\nscenarios:\n  it-holds-too:\n    covers: [performance.capacity]\ntransforms:\n  work:\n    implements:\n      - it-holds-too\n    files:\n      - src/lib.rs\n---\n\n## scenario: it-holds-too\nтіло обіцянки\n\n## transform: work\nтіло роботи\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "chore: moved across"]);
    let said = check(&dir);
    assert!(
        said.contains("червоне") && said.contains("old-name"),
        "a move across directories is a finding (§4.12):\n{said}"
    );

    // "Where there is no history or it is truncated, the court says
    // NOT CHECKED" -- the scenario's own words, and the answer this
    // wave wrote against reliability.fault-tolerance. Review 0036 R-6
    // measured a silent skip instead.
    let dir = project("truncated");
    git(&dir, &["rm", "-q", "keel/contracts/old-name.md"]);
    git(&dir, &["commit", "-q", "-m", "chore: gone"]);
    let shallow = keel_sandbox("truncatedclone");
    std::fs::remove_dir_all(&*shallow).unwrap();
    let out = Command::new("git")
        .args([
            "clone",
            "-q",
            "--depth",
            "1",
            "--branch",
            "0002-b-wave",
            &format!("file://{}", dir.display()),
            shallow.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "the shallow clone was made: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let said = check(&shallow);
    assert!(
        said.contains("не перевірено") && said.contains("§4.12"),
        "a truncated history is said aloud, not skipped (§4.10):\n{said}"
    );

    // And a trunk not called `main` gives no fork point at all: the
    // base fell back to the root commit, dragging up every deletion
    // ever made as a finding nobody can act on (review 0036 R-8).
    let dir = keel_sandbox("mastertrunk");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(dir.join("keel/contracts/ancient.md"), CONTRACT).unwrap();
    git(&dir, &["init", "-q", "-b", "master"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["rm", "-q", "keel/contracts/ancient.md"]);
    git(
        &dir,
        &["commit", "-q", "-m", "chore: long ago and lawfully"],
    );
    let said = check(&dir);
    assert!(
        !said.contains("зник із гілки"),
        "a project whose trunk is not called main is not accused of \
         every deletion in its history (§4.10 -- say it, do not \
         invent it):\n{said}"
    );
    assert!(
        said.contains("не перевірено"),
        "and the missing fork point is said aloud:\n{said}"
    );
}
