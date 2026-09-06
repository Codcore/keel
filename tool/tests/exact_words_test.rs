//! Scenario test of wave 0055: the words lead somewhere.
//!
//! Six words the final review of 2026-09-06 followed and found
//! nothing at the end of:
//!
//! * `keel method §6.3-б` -- "this generation's methodology has no
//!   such paragraph", over a paragraph standing in it (R-3);
//! * the run line `keel next` hands for an elixir test with a name
//!   past ASCII: `mix test --only 'test:test …'` answers "no test was
//!   executed", while the court itself runs `file:line` (R-12);
//! * a test file in latin-1: "stream did not contain valid UTF-8;
//!   instead: check the path and access permissions" -- advice for
//!   another fault entirely (R-16);
//! * an elixir test whose name carries an escaped quote: the reader
//!   stopped at the escape and refused the whole court (R-17);
//! * `keel close` in a stranger's project, measuring its disk "on
//!   this tree" -- keel's own tree, not the project's (R-8);
//! * the header `keel init` writes into keel.toml, which cites §2.9
//!   at a person who has not read the norm (R-5).
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

const BODY: &str = "тіло обіцянки\n";

/// A mix project on the branch of its wave, with the test file the
/// case needs.
fn elixir_project(name: &str, test_file: &str, worked: bool) -> common::Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"uk\"\nadapter = \"elixir\"\n");
    write(
        &dir,
        "mix.exs",
        "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\"]\nend\n",
    );
    write(
        &dir,
        "lib/toy.ex",
        "defmodule Toy do\n  def works, do: true\nend\n",
    );
    write(&dir, "test/test_helper.exs", "ExUnit.start()\n");
    write(&dir, "test/toy_test.exs", test_file);
    write(
        &dir,
        "keel/waves/0001-a-wave.md",
        &format!(
            "---\nscenarios:\n  it-works: {{covers: [functional.correctness]}}\ntransforms:\n  work:\n    implements: [it-works]\n    files: [lib/toy.ex]\n{}---\n\n## scenario: it-works\n\n{BODY}\n## transform: work\n\nтіло роботи\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "база"]);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    // The transform's own file, worked in and committed under its
    // slug where the case wants a finished wave (§4.4, §6.2); left
    // alone where it wants the step that hands out the run line.
    if worked {
        // The birth first: a green test nobody saw red proves
        // nothing (§6.3, §7.12), and this fixture is about a NAME,
        // so it walks the loop properly.
        git(
            &dir,
            &["commit", "-q", "--allow-empty", "-m", "red: it-works"],
        );
        write(
            &dir,
            "lib/toy.ex",
            "defmodule Toy do\n  @doc \"the work of this wave\"\n  def works, do: true\nend\n",
        );
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", "work: тіло роботи"]);
    }
    dir
}

fn elixir_project_unworked(name: &str, test_file: &str) -> common::Sandbox {
    elixir_project(name, test_file, false)
}

fn elixir_project_worked(name: &str, test_file: &str) -> common::Sandbox {
    elixir_project(name, test_file, true)
}

/// proves: the-words-lead-somewhere@cb3152 -- a word that points at
/// nothing is worse than silence: it costs the reader the walk.
#[test]
fn the_words_lead_somewhere() {
    let dir = keel_sandbox("wordsmethod");
    write(&dir, "keel.toml", "lang = \"uk\"\n");

    // -- a paragraph counted with a letter is served by its number ---
    for asked in ["§6.3-а", "§6.3-б"] {
        let (said, code) = keel(&dir, &["method", asked]);
        assert_eq!(code, 0, "`keel method {asked}` serves it:\n{said}");
        assert!(
            !said.contains("нема параграфа"),
            "and does not deny a paragraph that stands in the norm \
             (§6.3-а is the calling off of a started wave, §6.3-б the \
             rollback of a merged one):\n{said}"
        );
        assert!(
            said.contains(asked.trim_start_matches('§')),
            "the piece served is the one asked for:\n{said}"
        );
    }
    // And the paragraph is served whole, without swallowing its
    // neighbour: §6.3 itself still ends where its letters begin.
    let (plain, _) = keel(&dir, &["method", "§6.3"]);
    assert!(
        !plain.contains("Скасування початої хвилі"),
        "§6.3 is its own piece, and the lettered ones are theirs:\n{plain}"
    );

    // -- the config header speaks to a person, not to the norm -------
    let born = keel_sandbox("wordsinit");
    git(&born, &["init", "-q", "-b", "main"]);
    let (said, code) = keel(&born, &["init", "--no-ask", "--lang", "uk"]);
    assert_eq!(code, 0, "the frame stands:\n{said}");
    let config = fs::read_to_string(born.join("keel.toml")).unwrap();
    let header = config.lines().next().unwrap_or_default();
    assert!(
        header.contains("keel.toml"),
        "the header names the file:\n{header}"
    );
    assert!(
        header.contains("keel method"),
        "and where to read the vocabulary it speaks of -- a §-number \
         alone is a word that leads nowhere for a person who has not \
         read the norm:\n{header}"
    );

    // -- the run line of a tongue is the line the court itself runs --
    if common::machine_has("mix").ready() {
        let rev = keel::rev::text_rev(BODY);
        let dir = elixir_project_unworked(
            "wordsrunline",
            &format!(
                "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"тест додає число\" do\n    assert Toy.works()\n  end\nend\n"
            ),
        );
        let (said, code) = keel(&dir, &["next"]);
        assert_eq!(code, 0, "the step is given:\n{said}");
        assert!(
            said.contains("mix test test/toy_test.exs:"),
            "the run line is the one the court runs -- by file and line \
             (`--only` over a name past ASCII excludes everything, \
             measured):\n{said}"
        );
        assert!(
            !said.contains("--only"),
            "and not the one that answers \"no test was executed\":\n{said}"
        );

        // -- an escaped quote inside a name is part of the name ------
        let dir = elixir_project_worked(
            "wordsquote",
            &format!(
                "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it's \\\"quoted\\\"\" do\n    assert Toy.works()\n  end\nend\n"
            ),
        );
        let (said, code) = keel(&dir, &["check"]);
        assert!(
            !said.contains("тег не має тест-функції"),
            "a name with an escaped quote is a name, and the tag over it \
             is judged -- the reader stopped at the escape and refused \
             the whole court:\n{said}"
        );
        assert_eq!(code, 0, "so the tree is green:\n{said}");
    }

    // -- a file that is not UTF-8 is said to be that ------------------
    let dir = keel_sandbox("wordslatin");
    write(&dir, "keel.toml", "lang = \"uk\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub fn one() {}\n");
    fs::create_dir_all(dir.join("tests")).unwrap();
    // `é` in latin-1: one byte, 0xE9, which is no UTF-8 at all.
    fs::write(
        dir.join("tests/latin_test.rs"),
        b"// caf\xe9\n#[test]\nfn holds() {}\n".as_slice(),
    )
    .unwrap();
    let (said, _) = keel(&dir, &["check"]);
    assert!(
        said.contains("UTF-8"),
        "a file this reader cannot read as UTF-8 is named as that:\n{said}"
    );
    assert!(
        !said.contains("права доступу"),
        "and the advice is not for another fault entirely -- the file \
         opened and was read; its bytes are the trouble:\n{said}"
    );
    assert!(
        said.contains("latin_test.rs"),
        "with the file named:\n{said}"
    );

    // -- the price of the closing court is measured somewhere named --
    let dir = keel_sandbox("wordsprice");
    write(&dir, "keel.toml", "lang = \"uk\"\nadapter = \"rust\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&dir, "src/lib.rs", "pub fn one() {}\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "база"]);
    let (said, _) = keel(&dir, &["close"]);
    assert!(
        said.contains("ГіБ"),
        "the closing court names its price in disk:\n{said}"
    );
    assert!(
        !said.contains("зміряно на цьому дереві"),
        "and does not call a measurement taken on keel's own tree a \
         measurement of the project it is standing in:\n{said}"
    );
}
