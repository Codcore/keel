//! Scenario test of wave 0037: a green birth is named and proved.

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

/// The gate over one commit message, as the hook calls it.
fn gate(dir: &Path, message: &str) -> (String, i32) {
    let file = dir.join("MSG");
    std::fs::write(&file, message).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", file.to_str().unwrap(), dir.to_str().unwrap()])
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

/// A crate whose promise is proven by a test that is ALREADY GREEN --
/// the shape of a court over our own battery, which cannot be seen
/// failing without breaking the very thing it guards.
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
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  the-battery-is-judged:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - the-battery-is-judged\n    files:\n      - src/lib.rs\n{d}---\n\n## scenario: the-battery-is-judged\nтіло обіцянки\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    let rev = keel::rev::text_rev("тіло обіцянки\n");
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: the-battery-is-judged@{rev}\n#[test]\nfn holds_it() {{}}\n"),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    dir
}

/// proves: a-green-birth-is-named-and-proved@d5f734 -- the operator's
/// decision of 2026-09-04. §6.3 forbids a promise proven without
/// having been seen red, and a court over our own battery cannot be
/// seen red without breaking the thing it guards; the escape hatch
/// people actually used was to rename the work `chore`, which drops
/// the promise altogether. The exception is named instead, and it
/// costs work: the commit carries the mutant.
#[test]
fn a_green_birth_is_named_and_proved() {
    let dir = project("greenbirth");

    // Still refused when the message says nothing: the guarantee
    // stands, and the escape is not free.
    let (said, code) = gate(&dir, "red: the-battery-is-judged\n");
    assert_eq!(code, 1, "a green test is not a birth (§6.3):\n{said}");
    assert!(
        said.contains("mutant:"),
        "and the refusal names the way out, so nobody has to guess \
         it:\n{said}"
    );

    // With the mutant recorded, the birth passes -- named, narrow,
    // and paid for.
    let (said, code) = gate(
        &dir,
        "red: the-battery-is-judged\n\nmutant: прибрано перевірку тегу -> проба назвала \"тег не знайдено\"\n",
    );
    assert_eq!(
        code, 0,
        "a green birth carrying its mutant passes (§6.3):\n{said}"
    );
    assert!(
        said.contains("mutant") || said.contains("мутант"),
        "and the gate says which road it took:\n{said}"
    );

    // A mutant line with nothing on the right of the arrow is not a
    // mutant: the exception costs work, and an empty one costs none.
    for empty in [
        "red: the-battery-is-judged\n\nmutant:\n",
        "red: the-battery-is-judged\n\nmutant: щось зламав\n",
        "red: the-battery-is-judged\n\nmutant:  -> \n",
    ] {
        let (said, code) = gate(&dir, empty);
        assert_eq!(
            code, 1,
            "a mutant line must say WHAT was broken and HOW the probe \
             named it:\n{said}"
        );
    }

    // And `keel check` says such an exception aloud: it is named, not
    // silent, so a reviewer meets it.
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &[
            "commit",
            "-q",
            "-m",
            "red: the-battery-is-judged\n\nmutant: прибрано перевірку тегу -> проба назвала \"тег не знайдено\"",
        ],
    );
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        said.contains("the-battery-is-judged") && said.contains("мутант"),
        "the verdict names the exception and whose promise took it \
         (§6.3):\n{said}"
    );
    assert!(
        said.contains("не перевіряє") || said.contains("на слово"),
        "and says the machine does not check the mutant is real:\n{said}"
    );
}
