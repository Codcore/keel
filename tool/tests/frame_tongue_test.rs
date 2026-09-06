//! Scenario test of wave 0055: the generated frame carries the
//! tongue.
//!
//! Measured on the first day in a stranger's project (final review
//! 2026-09-06): the generated `keel.yml` of a python or elixir
//! project went checkout -> the tool -> `keel check` -> `keel close`
//! -> the battery, with no `setup-python` and no `pip install
//! pytest`, no `setup-beam` and no hex -- the tongue's own block was
//! written for the tool's own repository alone, so a stranger's
//! closing court ran a battery on a runner that had no runner (bugs
//! R-14). The Claude session hook called `keel next` without `--for
//! claude`, so a refusal of the config left with exit 2 on stderr and
//! never reached the agent, while cursor's got it (methodology R-1).
//! And the commit-msg hook was `exec keel gate "$1"`: a GUI client
//! runs hooks with its own PATH, and the whole court was `exec: keel:
//! not found` (bugs R-15).
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;

use std::fs;
use std::path::Path;
use std::process::Command;

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

/// A stranger's project of one tongue, with the file that tongue is
/// known by, and the frame written into it.
fn project(name: &str, adapter: &str, marker: &str, text: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    fs::write(
        dir.join("keel.toml"),
        format!("lang = \"en\"\nadapter = \"{adapter}\"\nci = \"github\"\n"),
    )
    .unwrap();
    fs::write(dir.join(marker), text).unwrap();
    fs::create_dir_all(dir.join("test")).unwrap();
    let (said, code) = keel(&dir, &["update"]);
    assert_eq!(code, 0, "{name}: the frame is written:\n{said}");
    dir
}

fn workflow(dir: &Path) -> String {
    fs::read_to_string(dir.join(".github/workflows/keel.yml")).unwrap()
}

/// proves: the-generated-frame-carries-the-tongue@0225fe -- the
/// frame a stranger's project gets must be able to run: the tongue
/// before the courts, the agent's hook speaking the agent's shape,
/// and the commit court reachable from a client with its own PATH.
#[test]
fn the_generated_frame_carries_the_tongue() {
    // -- the tongue stands before the courts that need it ------------
    for (name, adapter, marker, text, wanted) in [
        (
            "framepython",
            "python",
            "pyproject.toml",
            "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n",
            vec!["setup-python", "pytest"],
        ),
        (
            "frameelixir",
            "elixir",
            "mix.exs",
            "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\"]\nend\n",
            vec!["setup-beam", "mix local.hex"],
        ),
        (
            "frameruby",
            "ruby",
            "Gemfile",
            "source \"https://rubygems.org\"\n",
            vec!["setup-ruby"],
        ),
        (
            "framenode",
            "node",
            "package.json",
            "{\n  \"name\": \"toy\"\n}\n",
            vec!["setup-node"],
        ),
    ] {
        let dir = project(name, adapter, marker, text);
        let yml = workflow(&dir);
        for piece in &wanted {
            assert!(
                yml.contains(piece),
                "{name}: the frame puts this tongue on the runner \
                 (\"{piece}\") -- the closure court below runs its \
                 battery:\n{yml}"
            );
        }
        // And before the courts that run it, not after: a step that
        // installs the tongue under `keel close` has already lost.
        let tongue = wanted
            .iter()
            .map(|piece| yml.find(piece).unwrap_or(usize::MAX))
            .max()
            .unwrap();
        let court = yml.find("keel close").unwrap_or(0);
        assert!(
            tongue < court,
            "{name}: and it stands BEFORE the closure court, which \
             runs the battery:\n{yml}"
        );
    }

    // -- the agent's hook speaks the agent's shape --------------------
    let dir = project(
        "frameagent",
        "python",
        "pyproject.toml",
        "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n",
    );
    let settings = fs::read_to_string(dir.join(".claude/settings.json")).unwrap();
    assert!(
        settings.contains("--for claude"),
        "the Claude session hook asks for the step in the agent's own \
         shape, as cursor's does:\n{settings}"
    );

    // Which is what carries a refusal to the agent at all: a config
    // this release cannot read is the word an agent needs most, and
    // an exit of 2 on stderr is a word it never sees.
    fs::write(
        dir.join("keel.toml"),
        "lang = \"en\"\nadapter = \"python\"\nversion = \"9.9.9\"\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["next", "--for", "claude", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&out.stdout).into_owned();
    assert_eq!(
        out.status.code(),
        Some(0),
        "the hook's road comes back green, so the agent is handed the \
         words instead of a blocked step:\n{said}{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        said.contains("9.9.9"),
        "and the words are the refusal itself:\n{said}"
    );

    // -- the commit court is reachable from a client of its own PATH --
    let dir = keel_sandbox("framehook");
    fs::write(dir.join("keel.toml"), "lang = \"en\"\nadapter = \"rust\"\n").unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q", "-b", "main"]);
    let (said, code) = keel(&dir, &["hook"]);
    assert_eq!(code, 0, "the hook is installed:\n{said}");
    let hook = dir.join(".git/hooks/commit-msg");
    let text = fs::read_to_string(&hook).unwrap();
    assert!(
        text.contains("keel gate"),
        "it calls the commit court:\n{text}"
    );
    // The client's PATH is not a person's: a hook that finds the tool
    // only through PATH is a court that does not run.
    let msg = dir.join("MSG");
    fs::write(&msg, "the trunk\n").unwrap();
    let out = Command::new("sh")
        .arg(&hook)
        .arg(&msg)
        .current_dir(dir.path())
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !said.contains("not found"),
        "with no keel on PATH the hook still reaches the tool it was \
         installed by -- a GUI client runs hooks with its own PATH, \
         and `exec: keel: not found` is what a person saw:\n{said}"
    );
    assert!(
        said.contains("keel"),
        "and speaks in the tool's own voice:\n{said}"
    );
    assert_eq!(
        out.status.code(),
        Some(0),
        "a message outside the judgement passes, as it does on a \
         person's PATH:\n{said}"
    );
}
