//! Scenario test of wave 0037: a started wave can be cancelled.

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

/// A wave whose BODY is missing the section its header promises --
/// the likeliest state of a wave someone abandoned half-way, and the
/// state review 0037 R-1 measured being judged despite the report
/// saying it was not.
fn half_written(cancelled: bool) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    let head = if cancelled {
        "cancelled: \"передумали: обіцянку закриє інша хвиля\"\n"
    } else {
        ""
    };
    format!(
        "---\n{head}scenarios:\n  it-holds:\n    proves: anchor@beef00\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n      - README.md\n{d}---\n\n## transform: work\nтіло роботи\n"
    )
}

fn wave(cancelled: bool) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    let head = if cancelled {
        "cancelled: \"передумали: обіцянку закриє інша хвиля\"\n"
    } else {
        ""
    };
    format!(
        "---\n{head}scenarios:\n  it-holds:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-holds\n    files:\n      - src/lib.rs\n      - README.md\n{d}---\n\n## scenario: it-holds\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
    )
}

fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(dir.join("README.md"), "проєкт\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    // Started: the wave file is written and some work is committed,
    // and then it is decided the wave will not be done.
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), wave(false)).unwrap();
    std::fs::write(
        dir.join("src/lib.rs"),
        "pub fn a() {}\npub fn half_done() {}\n",
    )
    .unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: half of it"]);
    dir
}

/// proves: a-started-wave-can-be-cancelled@8fa155 -- the operator's
/// decision of 2026-09-04. A wave begun and then dropped had no way
/// to die: promises can be withdrawn (§2.12) and the wave itself
/// could not, so `keel close` would call it unclosed for ever and the
/// honest ways out were to lie or to delete the file -- which §4.12
/// forbids, rightly.
#[test]
fn a_started_wave_can_be_cancelled() {
    // Started and abandoned: every court has something to say.
    let dir = project("started");
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 1, "an abandoned wave is judged as any other:\n{said}");
    assert!(
        said.contains("it-holds"),
        "its promise is judged, since nothing says otherwise:\n{said}"
    );

    // Called off by name, with a reason a person can read.
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), wave(true)).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: called off"]);

    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 0, "a cancelled wave is judged no more:\n{said}");
    assert!(
        !said.contains("it-holds"),
        "neither its promises nor its scope (§6):\n{said}"
    );
    assert!(
        said.contains("0001-a-wave") && said.contains("скасован"),
        "and the verdict says which wave was called off, not just \
         falling silent about it:\n{said}"
    );

    let (said, _) = keel(&dir, "status");
    assert!(
        said.contains("0001-a-wave") && said.contains("скасован") && said.contains("передумали"),
        "status names it cancelled and carries the reason \
         verbatim:\n{said}"
    );

    let (said, code) = keel(&dir, "close");
    assert_eq!(
        code, 0,
        "and the closing court stops calling it unclosed:\n{said}"
    );
    assert!(
        said.contains("0001-a-wave") && said.contains("скасован"),
        "saying so by name:\n{said}"
    );

    // And "outside judgement" means every court, not one of them.
    // Review 0037 R-1: the §7.7 court (header vs body) and the §7.3
    // one (contract records) went on judging, so the report said
    // "not judged" and gave a finding two lines apart. A wave
    // abandoned half-way is exactly the wave with a missing section
    // and a record left behind.
    let dir = project("halfwritten");
    std::fs::write(
        dir.join("keel/contracts/anchor.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту\n",
    )
    .unwrap();
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), half_written(false)).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: half written"]);
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 1, "half-written and not called off: judged:\n{said}");
    assert!(
        said.contains("it-holds") && said.contains("anchor"),
        "both the §7.7 lack and the §7.3 record are findings:\n{said}"
    );

    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), half_written(true)).unwrap();
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "work: and called off"]);
    let (said, code) = keel(&dir, "check");
    assert_eq!(
        code, 0,
        "called off: EVERY court lets it be, not just the graph \
         one (§6.3-a):\n{said}"
    );
    assert!(
        !said.contains("it-holds") && !said.contains("anchor@beef00"),
        "neither the missing section nor the drifted record is a \
         finding any more:\n{said}"
    );
    assert!(
        said.contains("не судиться") && said.contains("0001-a-wave"),
        "and the wave is named aloud, so silence does not look like a \
         wave nobody wrote:\n{said}"
    );

    // "Every court says so aloud" means the gate and the loop's hand
    // too. Review 0037 R-6: with the hook `keel init` installs by
    // default, a cancelled wave's branch could not take a commit at
    // all -- the gate judged it in silence. R-7: `next` handed out
    // "write the test" for a scenario of a wave nobody is doing.
    let msg = dir.join("MSG");
    for line in ["work: ще трохи", "red: it-holds"] {
        std::fs::write(&msg, format!("{line}\n")).unwrap();
        let out = Command::new(env!("CARGO_BIN_EXE_keel"))
            .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
            .output()
            .unwrap();
        let said = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            out.status.code().unwrap_or(-1),
            0,
            "the gate lets a cancelled wave's branch commit (§6.3-a):\n{said}"
        );
        assert!(
            said.contains("скасован") && said.contains("0001-a-wave"),
            "and says why, rather than judging in silence:\n{said}"
        );
    }
    std::fs::remove_file(&msg).ok();

    let (said, _) = keel(&dir, "next");
    assert!(
        said.contains("скасован") && !said.contains("red: it-holds"),
        "the loop's hand does not drive work on a wave nobody is \
         doing (§6.3-a):\n{said}"
    );

    // A cancellation with no reason is not a cancellation: the whole
    // point is that a person can read WHY.
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        wave(true).replace("передумали: обіцянку закриє інша хвиля", "   "),
    )
    .unwrap();
    let (said, code) = keel(&dir, "check");
    assert_eq!(
        code, 1,
        "an empty reason is a broken document (§7.9), not a \
         cancellation:\n{said}"
    );
    assert!(
        said.contains("cancelled") && said.contains("порожнє"),
        "and the finding says which field and why:\n{said}"
    );
}
