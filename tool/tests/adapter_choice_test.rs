//! Scenario test of wave 0038: the adapter is chosen by name.

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

fn decided() -> String {
    let mut out = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        out.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    out
}

/// A project whose config names whatever adapter the case wants.
fn project(name: &str, adapter: Option<&str>) -> common::Sandbox {
    let dir = keel_sandbox(name);
    let line = match adapter {
        Some(named) => format!("lang = \"uk\"\nadapter = \"{named}\"\n"),
        None => "lang = \"uk\"\n".to_string(),
    };
    std::fs::write(dir.join("keel.toml"), line).unwrap();
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - README.md\n{}---\n\n## transform: tidy\nтіло роботи\n",
            decided()
        ),
    )
    .unwrap();
    std::fs::write(dir.join("README.md"), "проєкт\n").unwrap();
    // The skeleton the named adapter needs to exist at all: a crate
    // for cargo, a lib and a test directory for ruby. Missing those
    // is the adapter's own refusal, not a question of choosing one.
    match adapter {
        Some("rust") | Some("cargo") => {
            std::fs::write(
                dir.join("Cargo.toml"),
                "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            )
            .unwrap();
            std::fs::create_dir_all(dir.join("src")).unwrap();
            std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
        }
        Some("ruby") => {
            std::fs::create_dir_all(dir.join("lib")).unwrap();
            std::fs::create_dir_all(dir.join("test")).unwrap();
            std::fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        }
        _ => {}
    }
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: the-adapter-is-chosen-by-name@007ef7 -- the operator read
/// the README and asked where the other languages were. There was
/// one adapter, cargo, and it was there so keel could judge itself:
/// `rust_adapter()` was asked in eleven places across ten modules,
/// and every one of them meant "is this project Rust", not "which
/// language is this".
#[test]
fn the_adapter_is_chosen_by_name() {
    // The two names this release knows.
    for named in ["rust", "cargo", "ruby"] {
        let dir = project("known", Some(named));
        let (said, code) = keel(&dir, "check");
        assert_eq!(
            code, 0,
            "`adapter = \"{named}\"` is a name this release knows:\n{said}"
        );
        assert!(
            !said.contains("червоне"),
            "and naming it is not a finding:\n{said}"
        );
        // And the name really turns the language-shaped courts ON --
        // review 0038 R-12: this probe used to measure only an exit
        // code and the absence of a word, so a `ruby` that quietly
        // lost every language court stayed green under it.
        assert!(
            said.contains("тегів тестів звірено:"),
            "the tag court ran for \"{named}\", it did not stand \
             down:\n{said}"
        );
        assert!(
            !said.contains("теги тестів не звірялись"),
            "and it does not say it stood down for \"{named}\":\n{said}"
        );
    }

    // A name it does not know is a FINDING that lists what it does --
    // never a silent skip of the language-shaped courts. Not a
    // refusal: a project that named a language keel cannot lead yet
    // still gets every court that does not need one.
    let dir = project("unknown", Some("kotlin"));
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 1, "an unknown adapter is a finding:\n{said}");
    assert!(
        said.contains("kotlin") && said.contains("rust") && said.contains("ruby"),
        "and the refusal names what was asked for and what is \
         known:\n{said}"
    );

    // No adapter at all: the courts that need one are skipped, and
    // the verdict says which -- as before this wave.
    let dir = project("none", None);
    let (said, code) = keel(&dir, "check");
    assert_eq!(code, 0, "a project without an adapter is judged:\n{said}");
    // Review 0038 R-17: this used to read
    // `contains("адаптер") && contains("не званий") || contains("не названий")`
    // -- and "не званий" is in no dictionary of this tool, so by
    // Rust's own precedence the first two conjuncts were dead and the
    // assert held one thing while looking like two. Both courts that
    // stand down are named, each by its own words.
    assert!(
        said.contains("теги тестів не звірялись: adapter у keel.toml не названий"),
        "the tag court says it stood down, and why:\n{said}"
    );
    assert!(
        said.contains("1 річ не перевірено"),
        "and the summary counts that stand-down rather than reading \
         as a clean green:\n{said}"
    );
}

/// A Rust project keeps its Rust reading (review 0038 R-3): `#`
/// opens a comment in ruby and fences a raw string in Rust, and the
/// second tongue must not redden the first.
#[test]
fn a_rust_file_is_read_as_rust() {
    let dir = keel_sandbox("rusthash");
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    // A Rust probe that builds a ruby fixture: the `#` lines are a
    // string's content, not this file's comments.
    std::fs::write(
        dir.join("tests/fixture_test.rs"),
        "#[test]\nfn a_fixture() {\n    let fixture = r#\"\n# proves: nothing@aaaaaa\nname: toy\n\"#;\n    assert!(!fixture.is_empty());\n}\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    let (said, code) = keel(&dir, "check");
    assert_eq!(
        code, 0,
        "a Rust project with a ruby fixture in a raw string is not a \
         finding:\n{said}"
    );
    assert!(
        !said.contains("nothing@aaaaaa") && !said.contains("адаптер відмовив"),
        "and the tag court is neither fooled by it nor killed by it:\n{said}"
    );
}
