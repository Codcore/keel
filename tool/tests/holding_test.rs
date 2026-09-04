//! Scenario tests of wave 0010-contracts-hold, transform
//! holding-floor: §7.6's form court -- promised signatures compared
//! as collapsed text against the module's source; the incomparable
//! is a word aloud, never green.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;

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

/// proves: exports-held@ec9457 -- holds §7.6/§2.9: a promised
/// signature diverged in code is a finding carrying the promised
/// text; a vanished unit is a finding by name; a matching contract
/// is silence; and where there is nothing to compare with -- no
/// cargo adapter, a deeper module path -- the report says "no one
/// compared the form" aloud instead of green.
#[test]
fn exports_held() {
    let dir = keel_sandbox("form");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "src/lib.rs",
        "pub mod alpha;\npub mod beta;\npub mod deep;\n",
    );
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n\npub fn drifted(x: u64) -> u64 {\n    x\n}\n",
    );
    write(
        &dir,
        "src/beta.rs",
        "pub fn beta_keeps(word: &str) -> String {\n    word.to_string()\n}\n",
    );
    write(&dir, "src/deep/mod.rs", "pub mod inner;\n");
    write(
        &dir,
        "src/deep/inner.rs",
        "pub fn hidden() -> bool {\n    true\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn drifted(x: u32) -> u32\"\n  - \"pub fn vanished() -> bool\"\n---\n\nThe alpha promise.\n",
    );
    write(
        &dir,
        "keel/contracts/toy-beta.md",
        "---\nmodule: toy::beta\nexports:\n  - \"pub fn beta_keeps(word: &str) -> String\"\n---\n\nThe beta promise, held.\n",
    );
    write(
        &dir,
        "keel/contracts/toy-deep.md",
        "---\nmodule: toy::deep::inner\nexports:\n  - \"pub fn hidden() -> bool\"\n---\n\nA module two levels down, and held.\n",
    );

    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "diverged and vanished are findings:\n{out}");
    assert!(
        out.contains("pub fn drifted(x: u32) -> u32") && out.contains("does not match"),
        "the diverged signature carries the promised text (§2.9):\n{out}"
    );
    assert!(
        out.contains("\"vanished\"") && out.contains("no such unit"),
        "the vanished unit is a finding by name:\n{out}"
    );
    assert!(
        !out.lines()
            .any(|l| l.contains("toy-beta") && l.contains("no such unit")),
        "the held contract is silence:\n{out}"
    );
    assert!(
        !out.lines()
            .any(|l| l.contains("beta_keeps") && l.contains("does not match")),
        "the held signature is not a finding:\n{out}"
    );
    assert!(
        !out.lines()
            .any(|l| l.contains("toy-deep") && (l.contains("§7.6") || l.contains("hidden"))),
        "a module two levels down is compared like any other, and a \
         held promise there is silence (wave 0035 R-5; before it, the \
         court waved every deep path through unlooked-at):\n{out}"
    );
    assert!(
        out.contains("signatures checked:"),
        "the form court counts aloud:\n{out}"
    );

    // No adapter named: nothing to compare with -- the word again,
    // and no finding invented.
    let dir = keel_sandbox("no-adapter");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n---\n\nThe promise with no one to compare it.\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "nothing to compare is not a finding:\n{out}");
    assert!(
        out.contains("no one compared the form") && out.contains("no adapter"),
        "the skip is said aloud with its reason:\n{out}"
    );
}

/// proves: exports-held@ec9457 -- the second birth out of review
/// 0010 (R-1a/R-3/R-6), riding exports-held:
/// green must mean the form stands -- a short-form promise is not
/// satisfied by a longer neighbour's name, a signature surviving
/// only in a comment is vanished, the verdict words tell divergence
/// from disappearance apart; and on a plan branch the form court
/// does not run at all (§8.3) -- said aloud, not judged.
#[test]
fn exports_held_second_birth() {
    // Short form against a longer neighbour: `pub fn run` with only
    // run_all in the code is a vanished unit, never green (F1); and
    // the words say "no such unit", not "does not match" (F2/R-6).
    let dir = keel_sandbox("boundary");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod gamma;\n");
    write(
        &dir,
        "src/gamma.rs",
        "pub fn run_all(x: u32) -> u32 {\n    x\n}\n// pub fn ghost(x: u32) -> u32 -- only remembered here\n",
    );
    write(
        &dir,
        "keel/contracts/toy-gamma.md",
        "---\nmodule: toy::gamma\nexports:\n  - \"pub fn run\"\n  - \"pub fn ghost(x: u32) -> u32\"\n---\n\nShort and remembered promises.\n",
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "both green lies are findings now:\n{out}");
    assert!(
        out.contains("\"run\"") && out.contains("no such unit"),
        "the short form is vanished, not green and not 'does not match' (R-3/R-6):\n{out}"
    );
    assert!(
        out.contains("\"ghost\"") && out.contains("no such unit"),
        "a promise surviving only in a comment is vanished (R-3):\n{out}"
    );

    // A plan branch runs no form court (§8.3): exports may grow
    // ahead of the code there, and the skip is a word aloud.
    let dir = keel_sandbox("planbranch");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod alpha;\n");
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn planned_ahead() -> bool\"\n---\n\nThe plan grows the promise ahead of the code (§4.9).\n",
    );
    git2(&dir, &["init", "-q", "-b", "plan/0070-x"]);
    git2(&dir, &["add", "."]);
    git2(&dir, &["commit", "-q", "-m", "the plan rides its branch"]);
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "a plan branch is not red for growing exports (§8.3):\n{out}"
    );
    assert!(
        out.contains("plan branch") && out.contains("form court does not run"),
        "the skip is said aloud (§8.3):\n{out}"
    );
}

fn git2(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// proves: plan-window-forgiven@a68ef9 -- holds §6.5 and §6.7 (0010
/// review R-1b): a contract whose exports grew ahead of the code,
/// held only by a wave with no tag on any live scenario, is not
/// judged for form -- the approved-not-started window is lawful and
/// said aloud by name; the wave's first tag brings the court back.
#[test]
fn plan_window_forgiven() {
    let dir = keel_sandbox("window");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod alpha;\n");
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn planned_ahead() -> bool\"\n---\n\nThe plan grew this promise ahead of the code (§4.9).\n",
    );
    let contract_rev = keel::rev::contract_rev(&dir.join("keel/contracts/toy-alpha.md")).unwrap();
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            decided.push_str(&format!("  {cut}: \"n/a for the window sandbox\"\n"));
        }
    }
    write(
        &dir,
        "keel/waves/0080-w.md",
        &format!(
            "---\nscenarios:\n  s:\n    proves: toy-alpha@{contract_rev}\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [s]\n    contracts: [toy-alpha@{contract_rev}]\n    files: [src/alpha.rs]\n{decided}---\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nthe work ahead.\n",
        ),
    );

    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the approved-not-started window is lawful, not red (§6.5):\n{out}"
    );
    assert!(
        out.contains("toy-alpha") && out.contains("approved, not started"),
        "the window is said aloud by name:\n{out}"
    );

    // The wave's first tag brings the court back.
    let s_rev = keel::rev::text_rev("body of s\n");
    write(
        &dir,
        "tests/t_test.rs",
        &format!("/// proves: s@{s_rev}\n#[test]\nfn holds_s() {{}}\n"),
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "the first tag brings the form court back:\n{out}");
    assert!(
        out.contains("\"planned_ahead\"") && out.contains("no such unit"),
        "the grown promise is judged again once the wave starts:\n{out}"
    );
}

/// proves: plan-window-forgiven@a68ef9 -- the second birth out of
/// review 0011 (R-1/R-9): a wave with every scenario withdrawn is
/// not a plan -- the window never opens for the dead, the court
/// stays; and a namesake tag from a foreign wave does not slam the
/// window on a lawful plan -- started is judged by this wave's own
/// scenario revision.
#[test]
fn plan_window_forgiven_second_birth() {
    // Dead holder: all scenarios withdrawn -- no window, ever.
    let dir = keel_sandbox("deadwave");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod alpha;\n");
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn grown_and_orphaned() -> bool\"\n---\n\nGrown by a wave that then died whole (§2.12).\n",
    );
    let contract_rev = keel::rev::contract_rev(&dir.join("keel/contracts/toy-alpha.md")).unwrap();
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        decided.push_str(&format!("  {cut}: \"n/a for the window sandbox\"\n"));
    }
    write(
        &dir,
        "keel/waves/0081-dead.md",
        &format!(
            "---\nscenarios:\n  gone:\n    proves: toy-alpha@{contract_rev}\n    withdrawn: \"folded\"\ntransforms:\n  t:\n    chore: \"the leftovers\"\n    contracts: [toy-alpha@{contract_rev}]\n    files: [src/alpha.rs]\n{decided}---\n\n## scenario: gone\n\nold body\n\n## transform: t\n\nleftover work\n",
        ),
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 1,
        "a dead wave is not a plan -- the court stays (R-1):\n{out}"
    );
    assert!(
        out.contains("\"grown_and_orphaned\"") && out.contains("no such unit"),
        "the grown promise is judged, not forgiven forever:\n{out}"
    );
    assert!(
        !out.contains("approved, not started"),
        "no window word over the dead:\n{out}"
    );

    // Namesake tag from a foreign wave: this wave's plan stays a
    // plan -- started is this wave's own revision, not a name match.
    let dir = keel_sandbox("namesake");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod alpha;\n");
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn planned_ahead() -> bool\"\n---\n\nGrown ahead by the plan (§4.9).\n",
    );
    let contract_rev = keel::rev::contract_rev(&dir.join("keel/contracts/toy-alpha.md")).unwrap();
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            decided.push_str(&format!("  {cut}: \"n/a for the window sandbox\"\n"));
        }
    }
    write(
        &dir,
        "keel/waves/0082-plan.md",
        &format!(
            "---\nscenarios:\n  s:\n    proves: toy-alpha@{contract_rev}\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [s]\n    contracts: [toy-alpha@{contract_rev}]\n    files: [src/alpha.rs]\n{decided}---\n\n## scenario: s\n\nthe planned body\n\n## transform: t\n\nthe work ahead\n",
        ),
    );
    write(
        &dir,
        "keel/waves/0083-other.md",
        &format!(
            // Its own name: wave 0035 forbids two live waves sharing
            // one scenario slug, since a test tag is bare and would
            // close both.
            "---\nscenarios:\n  other-s: {{covers: [performance.capacity]}}\ntransforms:\n  t:\n    implements: [other-s]\n    files: [src/lib.rs]\ndecisions:\n  functional.correctness: \"the other wave decides it\"\n{}---\n\n## scenario: other-s\n\na different body entirely\n\n## transform: t\n\nother work\n",
            {
                let mut block = String::new();
                for cut in keel::graph::cuts() {
                    if *cut != "functional.correctness" && *cut != "performance.capacity" {
                        block.push_str(&format!("  {cut}: \"n/a for the window sandbox\"\n"));
                    }
                }
                block
            }
        ),
    );
    let other_rev = keel::rev::text_rev("a different body entirely\n");
    write(
        &dir,
        "tests/t_test.rs",
        &format!("/// proves: other-s@{other_rev}\n#[test]\nfn holds_other_s() {{}}\n"),
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    // The tag of ANOTHER wave's scenario does not slam this wave's
    // window (R-9). It used to be a namesake -- the same slug in two
    // waves -- and wave 0035 made that a finding of its own, since a
    // bare tag cannot say whose promise it proves and one test then
    // closed both waves. The situation this guards is still real:
    // some other wave's tag must not touch this window. It is now
    // played with a name of its own, which is what the norm requires.
    assert_eq!(
        code, 0,
        "another wave's tag does not slam the window (R-9):\n{out}"
    );
    assert!(
        out.contains("approved, not started"),
        "the lawful plan keeps its window:\n{out}"
    );

    // What the rewrite above gave up, played the way the norm now
    // allows (review 0035 R-10): the guard of review 0011 R-1/R-9 is
    // that a wave is STARTED by a tag holding ITS OWN revision, not
    // by the bare name. The namesake in a second wave is a finding of
    // its own since wave 0035, so the case is played with one wave
    // and a tag whose revision is somebody else's: the wave is still
    // a plan, and the stale tag is said aloud beside it.
    let dir = keel_sandbox("foreignrev");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub mod alpha;\n");
    write(
        &dir,
        "src/alpha.rs",
        "pub fn stays(x: u32) -> u32 {\n    x\n}\n",
    );
    write(
        &dir,
        "keel/contracts/toy-alpha.md",
        "---\nmodule: toy::alpha\nexports:\n  - \"pub fn stays(x: u32) -> u32\"\n  - \"pub fn planned_ahead() -> bool\"\n---\n\nGrown ahead by the plan (§4.9).\n",
    );
    let contract_rev = keel::rev::contract_rev(&dir.join("keel/contracts/toy-alpha.md")).unwrap();
    let mut decided = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            decided.push_str(&format!("  {cut}: \"n/a for the window sandbox\"\n"));
        }
    }
    write(
        &dir,
        "keel/waves/0084-plan.md",
        &format!(
            "---\nscenarios:\n  s:\n    proves: toy-alpha@{contract_rev}\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [s]\n    contracts: [toy-alpha@{contract_rev}]\n    files: [src/alpha.rs]\n{decided}---\n\n## scenario: s\n\nthe planned body\n\n## transform: t\n\nthe work ahead\n",
        ),
    );
    write(
        &dir,
        "tests/t_test.rs",
        "/// proves: s@beef00\n#[test]\nfn holds_s() {}\n",
    );
    let (out, err, _) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert!(
        out.contains("approved, not started"),
        "a tag holding somebody else's revision does not start this \
         wave -- the window stands (review 0011 R-1/R-9):\n{out}"
    );
    assert!(
        out.contains("beef00"),
        "and the tag itself is said aloud, not swallowed:\n{out}"
    );
    assert!(
        !out.contains("\"planned_ahead\"") || !out.contains("no such unit"),
        "the promise grown ahead is not judged while the window \
         stands:\n{out}"
    );
}
