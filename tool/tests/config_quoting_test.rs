//! Scenario test of wave 0055: the config is written as TOML.
//!
//! `keel init --adapter ruby` wrote the tongue's own battery command
//! as a hint -- `# ci = "ruby -Itest -e 'Dir.glob("test/**/*_test.rb")
//! …'"` -- with quotes inside the quotes: uncommented, "TOML parse
//! error at line 5", and every court refused the config; `--ci` with
//! the same value wrote it ACTIVE, and init said "born from your
//! answers" over a file no parser reads (final review 2026-09-06,
//! bugs R-4; reproduced on f744252).
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;

use std::fs;
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

fn keel(dir: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
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

/// A ruby project with nothing but a module and a repository: the
/// frame is what init brings.
fn ruby_project(name: &str) -> common::Sandbox {
    let dir = sandbox(name);
    fs::create_dir_all(dir.join("lib")).unwrap();
    fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    dir
}

/// What no court may say of a config the frame itself wrote.
fn parses(said: &str, what: &str) {
    assert!(
        !said.contains("does not parse") && !said.contains("TOML parse error"),
        "{what}: the frame writes a TOML string the parser reads, \
         quotes and backslashes escaped:\n{said}"
    );
}

/// proves: the-config-is-written-as-toml@3473b5 -- the hint and
/// the active line were both `"{value}"` with the value pasted in
/// raw; ruby's battery command carries `"` of its own, and the file
/// stopped parsing the moment a person did what the header says:
/// "uncomment to enable".
#[test]
fn the_config_is_written_as_toml() {
    // The hint: the tongue's own battery command, commented, and
    // the header's own advice is to uncomment it.
    let dir = ruby_project("tomlhint");
    let (said, code) = keel(&dir, &["init", "--no-ask", "--adapter", "ruby"]);
    assert_eq!(code, 0, "the frame stands:\n{said}");
    let text = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let hint = text
        .lines()
        .find(|line| line.starts_with("# ci = "))
        .unwrap_or_else(|| panic!("init writes a ci hint for the tongue:\n{text}"));
    let enabled = text.replace(hint, hint.trim_start_matches("# "));
    fs::write(dir.join("keel.toml"), &enabled).unwrap();
    let (said, _) = keel(&dir, &["check"]);
    parses(&said, "the hint, uncommented");
    let config = keel::config::read(&dir).unwrap_or_else(|refusal| {
        panic!(
            "the config the frame wrote is read back whole: {}",
            refusal.reason
        )
    });
    assert_eq!(
        config.ci.as_deref(),
        Some(
            keel::config::Language::named("ruby")
                .unwrap()
                .battery_command()
        ),
        "and the value read back is the tongue's command, quotes and \
         all -- what the hint promised, not a mangled cousin:\n{enabled}"
    );

    // The active line: a value with quotes and a backslash, given by
    // flag. Init says what was born only when the parser agrees.
    let dir = ruby_project("tomlci");
    let value = "ruby -e \"puts 'a\\\\b'\"";
    let (said, code) = keel(
        &dir,
        &["init", "--no-ask", "--adapter", "ruby", "--ci", value],
    );
    assert_eq!(code, 0, "init with a quoted --ci stands:\n{said}");
    assert!(
        said.contains("born from your answers"),
        "and says the config was born from the answers:\n{said}"
    );
    let config = keel::config::read(&dir).unwrap_or_else(|refusal| {
        panic!(
            "a --ci value with quotes and a backslash is read back: {}",
            refusal.reason
        )
    });
    assert_eq!(
        config.ci.as_deref(),
        Some(value),
        "as exactly the value that was given:\n{}",
        fs::read_to_string(dir.join("keel.toml")).unwrap()
    );
    let (said, _) = keel(&dir, &["check"]);
    parses(&said, "the active ci line");

    // -- and the file is born in the tongue that was answered -------
    // With no config yet there is none to read, so the frame wrote
    // its header -- and its own report -- in English over `--lang uk`
    // (review 0055 R-14).
    let dir = ruby_project("tomluk");
    let (said, code) = keel(&dir, &["init", "--no-ask", "--lang", "uk"]);
    assert_eq!(code, 0, "the frame stands:\n{said}");
    assert!(
        said.contains("рама методики"),
        "the frame reports in the tongue just answered:\n{said}"
    );
    let text = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        text.lines()
            .next()
            .is_some_and(|line| line.contains("налаштування")),
        "and the header of the born config speaks it too:\n{text}"
    );

    // -- every name the config takes, the flag takes ----------------
    // `adapter = "node"` is a name this release reads, and the flag
    // refused it while the courts understood it (review 0055 R-14).
    for name in ["node", "js", "ts", "javascript", "rust", "cargo"] {
        let dir = ruby_project(&format!("tomlname{name}"));
        let (said, code) = keel(&dir, &["init", "--no-ask", "--adapter", name]);
        assert_eq!(
            code, 0,
            "`--adapter {name}` is taken, as keel.toml takes it:\n{said}"
        );
        let config = keel::config::read(&dir).expect("the config reads back");
        assert_eq!(
            config.adapter.as_deref(),
            Some(name),
            "and stands in the file as written"
        );
    }
}
