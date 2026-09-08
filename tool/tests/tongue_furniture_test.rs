//! Scenario test of wave 0055: furniture is not drift.
//!
//! Two measurements of the final review (2026-09-06), both on the
//! first day in a stranger's project: `keel init` leaves
//! `keel/contracts/.gitkeep` so the empty directory outlives git, and
//! all three courts then said the branch of a light wave "changes the
//! contract .gitkeep" and led to a full wave (bugs R-8); the first
//! `red:` commit through the hook builds the crate, and the
//! `Cargo.lock` the runner writes was "touched, and no transform of
//! the wave names it" (bugs R-20). A runner's droppings are the
//! tongue's furniture, like the frame's own files (§4.8), and the
//! same waits for mix.lock, package-lock.json and Gemfile.lock.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, keel_sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
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

fn all_decided() -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        block.push_str(&format!(
            "  {cut}: \"n/a, бо ця пісочниця грає інший розріз\"\n"
        ));
    }
    block
}

/// A project of one tongue with a light chore wave, on the wave's own
/// branch: the chore has tidied its one declared file, and whatever
/// else the world left is judged by the courts.
fn project(name: &str, adapter: &str, sources: &[(&str, &str)]) -> Sandbox {
    let dir = keel_sandbox(name);
    write(
        &dir,
        "keel.toml",
        &format!("lang = \"en\"\nadapter = \"{adapter}\"\n"),
    );
    write(&dir, "README.md", "# toy\n");
    for (path, text) in sources {
        write(&dir, path, text);
    }
    write(
        &dir,
        "keel/waves/0001-a-wave.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"the README says the day\"\n    files: [README.md]\n{}---\n\n## transform: tidy\n\nthe tidying\n",
            all_decided()
        ),
    );
    write(&dir, "keel/reviews/0001-a-wave.md", "# Review\n\nok\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the trunk"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    write(&dir, "README.md", "# toy, tidied\n");
    dir
}

fn commit(dir: &Path) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &["commit", "-q", "-m", "tidy: the README says the day"],
    );
}

/// What no court may say over a file the tongue's own runner -- or
/// the frame itself -- left behind.
fn not_judged(dir: &Path, furniture: &[&str], tongue: &str) {
    for (command, code_matters) in [("check", true), ("next", false), ("status", false)] {
        let (said, code) = keel(dir, &[command]);
        for piece in furniture {
            assert!(
                !said.contains(piece),
                "{tongue}: `keel {command}` judges \"{piece}\", which no \
                 transform names and no person wrote -- the tongue's \
                 runner and the frame leave it (§4.8):\n{said}"
            );
        }
        if code_matters {
            assert_eq!(
                code, 0,
                "{tongue}: and the branch is green over its furniture:\n{said}"
            );
        }
    }
}

/// proves: furniture-is-not-drift@ad1a99 -- `.gitkeep` under
/// keel/contracts/ made every court call a light wave's branch a
/// contract change, and the lock file the first build leaves was
/// drift; both greeted a stranger's project on its first day.
#[test]
fn furniture_is_not_drift() {
    // -- rust: the frame's .gitkeep and cargo's lock ------------------
    let dir = project(
        "furniturerust",
        "rust",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            ("src/lib.rs", "pub fn one() {}\n"),
        ],
    );
    // What `keel init` leaves so an empty directory outlives git, and
    // what the first build through the hook leaves.
    write(&dir, "keel/contracts/.gitkeep", "");
    write(&dir, "Cargo.lock", "version = 3\n");
    commit(&dir);
    not_judged(&dir, &["keel/contracts/.gitkeep", "Cargo.lock"], "rust");

    // The three other tongues, each with the lock its runner writes.
    for (name, adapter, sources, lock) in [
        (
            "furnitureelixir",
            "elixir",
            vec![(
                "mix.exs",
                "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\"]\nend\n",
            )],
            "mix.lock",
        ),
        (
            "furniturenode",
            "node",
            vec![("package.json", "{\n  \"name\": \"toy\"\n}\n")],
            "package-lock.json",
        ),
        (
            "furnitureruby",
            "ruby",
            vec![
                ("lib/toy.rb", "module Toy\nend\n"),
                ("Gemfile", "source \"https://rubygems.org\"\n"),
            ],
            "Gemfile.lock",
        ),
    ] {
        let dir = project(name, adapter, &sources);
        write(&dir, "keel/contracts/.gitkeep", "");
        write(&dir, lock, "left by the runner\n");
        commit(&dir);
        not_judged(&dir, &["keel/contracts/.gitkeep", lock], adapter);
    }

    // -- and the borders, so the rule is not "any file is furniture" --
    // A lock of ANOTHER tongue is nobody's furniture here: this
    // project runs cargo, and a mix.lock in it is a file a person put
    // there.
    let dir = project(
        "furnitureforeign",
        "rust",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            ("src/lib.rs", "pub fn one() {}\n"),
        ],
    );
    write(&dir, "mix.lock", "not this tongue's\n");
    commit(&dir);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 1, "a foreign lock file is drift:\n{said}");
    assert!(
        said.contains("mix.lock"),
        "named, as any undeclared file is (§4.6):\n{said}"
    );

    // And a real contract, changed on a light wave's branch, is still
    // the full wave's business (§6.8, §5.7) -- .gitkeep was never a
    // contract, but a document under keel/contracts/ is.
    let dir = project(
        "furniturecontract",
        "rust",
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            ("src/lib.rs", "pub fn one() {}\n"),
        ],
    );
    write(
        &dir,
        "keel/contracts/toy-thing.md",
        "---\nmodule: toy::thing\nexports: []\n---\n\nThe promise.\n",
    );
    commit(&dir);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 1,
        "a contract on a light wave's branch is a finding:\n{said}"
    );
    assert!(
        said.contains("toy-thing"),
        "by the contract's own name (§6.8):\n{said}"
    );
}
