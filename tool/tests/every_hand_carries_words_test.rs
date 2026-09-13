//! Scenario test of wave 0071: a red battery carries the words that
//! made it red -- on every road keel drives a foreign runner down.
//!
//! Issues #49 and #52. The author of #52 re-ran a flaky test about
//! twenty times by hand and never once saw the failure keel had seen:
//! the court names the test and keeps nothing of what it said.
//!
//! One probe per road, on purpose. This session found two class
//! defects at exactly these seams -- latin letters in elixir under an
//! empty locale, ASCII escaping in pytest -- and a change made in the
//! shared part breaks, in silence, whichever road nobody measured.

mod common;

use common::{keel_sandbox, machine_has};
use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
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

fn closing(dir: &Path) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
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

/// A chore wave with no promises, its branch having done the work.
fn frame(dir: &Path, adapter: &str, touched: &str) {
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        d.push_str(&format!(
            "  {cut}: \"не про цю пісочницю, вона грає інше\"\n"
        ));
    }
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\ntransforms:\n  work:\n    chore: \"робота без обіцянок\"\n    files:\n      - {touched}\n{d}---\n\n## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    common::did_the_work(dir);
}

/// The words a failing test said, which the court must carry: a
/// marker no keel line could produce by itself.
const MARK: &str = "the-assertion-that-fell";

/// proves: a-red-battery-carries-the-words-that-made-it-red@671975
#[test]
fn a_red_battery_carries_the_words_that_made_it_red() {
    // Which roads this run actually measured. A probe gated on the
    // machine having a runner is silent about what it skipped, and
    // this session paid for that twice: four rspec sandboxes were
    // green here and red on a runner, for two different reasons, and
    // nobody could see they had not run. So the roads walked are
    // printed, and the count is asserted at the end.
    let mut walked: Vec<&str> = vec!["cargo"];
    // --- cargo ----------------------------------------------------
    let dir = keel_sandbox("wordscargo");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        format!("#[test]\nfn it_falls() {{\n    assert!(false, \"{MARK}\");\n}}\n"),
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_ne!(code, 0, "a red battery holds the wave open:\n{said}");
    assert!(
        said.contains("it_falls"),
        "cargo: the court names the test (it has since v1.0.0):\n{said}"
    );
    assert!(
        said.contains(MARK),
        "cargo: and carries the words that made it red. Naming the \
         test without them is issue #52: its author re-ran the test \
         about twenty times and never saw the failure keel saw:\n{said}"
    );

    // --- a green battery still says nothing extra -----------------
    let dir = keel_sandbox("wordsquiet");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        "#[test]\nfn it_stands() {\n    assert!(true);\n}\n",
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_eq!(code, 0, "a green battery closes:\n{said}");
    assert!(
        !said.contains("stdout") && !said.contains("панік"),
        "and carries no quotation at all -- the words belong to a red \
         verdict, not to every verdict:\n{said}"
    );

    // --- minitest -------------------------------------------------
    if machine_has("ruby").ready() {
        walked.push("minitest");
        let dir = keel_sandbox("wordsminitest");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("test")).unwrap();
        fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        fs::write(
            dir.join("test/toy_test.rb"),
            format!(
                "require \"minitest/autorun\"\n\nclass ToyTest < Minitest::Test\n  def test_it_falls\n    flunk \"{MARK}\"\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "minitest: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "minitest: and carries the words. This is the road issue \
             #49 was reported from -- rails and minitest:\n{said}"
        );
    }

    // --- rspec ----------------------------------------------------
    if machine_has("rspec").ready() {
        walked.push("rspec");
        let dir = keel_sandbox("wordsrspec");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("spec")).unwrap();
        fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        fs::write(
            dir.join("spec/toy_spec.rb"),
            format!(
                "RSpec.describe \"Toy\" do\n  it \"falls\" do\n    raise \"{MARK}\"\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "rspec: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "rspec: and carries the words -- from its own JSON, which \
             is the one road where the raw voice does not exist:\n{said}"
        );
    }

    // --- elixir ---------------------------------------------------
    if machine_has("mix").ready() {
        walked.push("elixir");
        let dir = keel_sandbox("wordselixir");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("test")).unwrap();
        fs::write(
            dir.join("mix.exs"),
            "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\nend\n",
        )
        .unwrap();
        fs::write(dir.join("lib/toy.ex"), "defmodule Toy do\nend\n").unwrap();
        fs::write(dir.join("test/test_helper.exs"), "ExUnit.start()\n").unwrap();
        fs::write(
            dir.join("test/toy_test.exs"),
            format!(
                "defmodule ToyTest do\n  use ExUnit.Case\n\n  test \"it falls\" do\n    flunk(\"{MARK}\")\n  end\nend\n"
            ),
        )
        .unwrap();
        frame(&dir, "elixir", "lib/toy.ex");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "elixir: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "elixir: and carries the words:\n{said}"
        );
    }

    // --- javascript -----------------------------------------------
    if machine_has("node").ready() {
        walked.push("node");
        let dir = keel_sandbox("wordsnode");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("test")).unwrap();
        fs::write(dir.join("package.json"), "{\n  \"name\": \"toy\"\n}\n").unwrap();
        fs::write(dir.join("src/toy.js"), "module.exports = {};\n").unwrap();
        fs::write(
            dir.join("test/toy.test.js"),
            format!(
                "const {{ test }} = require('node:test');\nconst assert = require('node:assert');\n\ntest('it falls', () => {{\n  assert.fail('{MARK}');\n}});\n"
            ),
        )
        .unwrap();
        frame(&dir, "javascript", "src/toy.js");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "node: a red battery holds the wave:\n{said}");
        assert!(said.contains(MARK), "node: and carries the words:\n{said}");
    }

    // --- python ---------------------------------------------------
    if machine_has("pytest").ready() {
        walked.push("pytest");
        let dir = keel_sandbox("wordspytest");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("tests")).unwrap();
        fs::write(dir.join("src/toy.py"), "def a():\n    return True\n").unwrap();
        fs::write(
            dir.join("tests/test_toy.py"),
            format!("def test_it_falls():\n    assert False, \"{MARK}\"\n"),
        )
        .unwrap();
        frame(&dir, "python", "src/toy.py");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "pytest: a red battery holds the wave:\n{said}");
        assert!(
            said.contains(MARK),
            "pytest: and carries the words:\n{said}"
        );
    }

    println!("roads measured: {}", walked.join(", "));
    // ...and the COUNT is asserted, which the comment above promised
    // and the code did not do (review 0071 R4-9): the list is built
    // again here, from the machine's own inventory, and the two must
    // agree. A sandbox that stops early, or a gate that stops
    // matching its block, now shows as a difference instead of a
    // shorter list nobody reads.
    let mut ready: Vec<&str> = vec!["cargo"];
    if machine_has("ruby").ready() {
        ready.push("minitest");
    }
    if machine_has("rspec").ready() {
        ready.push("rspec");
    }
    if machine_has("mix").ready() {
        ready.push("elixir");
    }
    if machine_has("node").ready() {
        ready.push("node");
    }
    if machine_has("pytest").ready() {
        ready.push("pytest");
    }
    assert_eq!(
        walked, ready,
        "every road this machine can run must have been walked -- a \
         probe that skipped one would pass in silence, which is how \
         four rspec sandboxes stayed green here and red on a runner"
    );
}

/// proves: a-red-battery-carries-the-words-that-made-it-red@671975 --
/// the REPORT's own guards, each on a tree where it decides something.
///
/// Round four measured five of six mutants surviving the full battery
/// (review 0071 R4-1), and named the reason exactly: every sandbox of
/// the probe above holds ONE red test in ONE file. On that shape the
/// guard this round added -- the voice said once per file -- cannot
/// be seen at all, because saying it once and saying it under each of
/// one red test are the same text. Measured: twelve files of one red
/// each gave 275 lines and twelve assertions with the guard and
/// without it.
///
/// So each tree here is built to make ONE guard decide.
#[test]
fn the_report_says_each_voice_once_and_from_the_right_run() {
    // --- two reds in ONE file: the guard's own shape ---------------
    let dir = keel_sandbox("wordstwo");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        "#[test]\nfn first_falls() {\n    assert!(false, \"THE-FIRST-ASSERTION\");\n}\n\n#[test]\nfn second_falls() {\n    assert!(false, \"THE-SECOND-ASSERTION\");\n}\n",
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_ne!(code, 0, "two reds hold the wave open:\n{said}");
    assert_eq!(
        said.matches("доки біг \"toy_test\"").count(),
        1,
        "the runner's voice belongs to the FILE and is said ONCE, \
         however many of that file's tests fell. What that buys is \
         this heading: the de-duplication below folds identical \
         blocks anyway, so without the guard one file of ten reds is \
         announced as \"these 10 files\" and named ten times over \
         (measured, round five):\n{said}"
    );
    assert!(
        said.contains("THE-FIRST-ASSERTION") && said.contains("THE-SECOND-ASSERTION"),
        "and one block still carries both assertions, because the \
         block is what the runner said while that file ran:\n{said}"
    );

    // --- a steady red is quoted from its LAST fall ----------------
    let dir = keel_sandbox("wordslast");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        format!("#[test]\nfn it_falls() {{\n    assert!(false, \"{MARK}\");\n}}\n"),
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_ne!(code, 0, "the red holds the wave open:\n{said}");
    assert!(
        said.contains("(біг 3 із 3)") && !said.contains("(біг 1 із 3)"),
        "a steady red is quoted from the LAST run that saw it fall, \
         and the run NUMBER is that run's own -- three copies of one \
         text tell a reader nothing:\n{said}"
    );

    // --- the voice of a target is that target's, not the run's ----
    //
    // Review 0071 R2-1 measured what a per-RUN voice costs on keel's
    // own tree: 1065 lines spoken, the one red block at line 659, and
    // a window of forty and forty keeping neither. So the split is by
    // cargo's own `running N tests` boundary -- and a tree with a
    // GREEN target beside the red one is where that shows.
    let dir = keel_sandbox("wordstargets");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/a_test.rs"),
        "#[test]\nfn a_stands() {\n    assert!(true);\n}\n",
    )
    .unwrap();
    fs::write(
        dir.join("tests/b_test.rs"),
        "#[test]\nfn b_falls() {\n    assert!(false, \"THE-RED-TARGET-SPOKE\");\n}\n",
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_ne!(code, 0, "the red target holds the wave open:\n{said}");
    assert!(
        said.contains("THE-RED-TARGET-SPOKE"),
        "the red target's words are carried:\n{said}"
    );
    assert!(
        !said.contains("a_stands"),
        "and the green target's block is not -- the voice is cut by \
         cargo's own boundary, so a report over many targets does not \
         spend its ceiling on the ones that stood:\n{said}"
    );

    // --- a flaky test makes its FILE flaky ------------------------
    //
    // The file's voice is one block, so steady-or-flaky is a question
    // about the file and must be asked of every red test in it. Asked
    // of whichever test the map held first, the answer was decided by
    // the alphabet: `a_steady` sorts before `z_flaky`, the file was
    // called steady, and the words of the flaky run vanished (review
    // 0071 R4-4).
    let dir = keel_sandbox("wordsflaky");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    // The counter lives in a sandbox of its own, not in a directory
    // this probe builds by hand: every probe takes the one hand, and
    // `sandbox_test` holds that (review 0030 R-2). It has to stand
    // apart from the tree under judgement -- a file inside it would
    // be a file the branch touched and did not declare.
    let counter_home = common::Sandbox::new("wordsflakycounter");
    let counter = counter_home.join("runs.txt");
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        format!(
            "#[test]\nfn a_steady() {{\n    assert!(false, \"THE-STEADY-ONE\");\n}}\n\n\
             #[test]\nfn z_flaky() {{\n    \
             let path = std::path::Path::new(\"{}\");\n    \
             let n: u32 = std::fs::read_to_string(path).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0) + 1;\n    \
             std::fs::write(path, n.to_string()).unwrap();\n    \
             assert!(n == 2, \"FLAKY-FELL-ON-RUN-{{n}}\");\n}}\n",
            counter.display()
        ),
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    drop(counter_home);
    assert_ne!(code, 0, "the reds hold the wave open:\n{said}");
    assert!(
        said.contains("FLAKY-FELL-ON-RUN-1") && said.contains("FLAKY-FELL-ON-RUN-3"),
        "one flaky test makes the FILE flaky, and a flaky file is \
         quoted from every run that saw it fall -- three different \
         assertions on one file is the most valuable thing anyone can \
         say about flakiness, and it is the whole of issue #52:\n{said}"
    );

    // --- and what the run said outside any one target -------------
    //
    // libtest captures the test's own `eprintln!` into the stdout
    // block; a CHILD process it starts writes to the real stderr and
    // escapes that. Splitting the voice by target dropped stderr
    // altogether, and these words -- which had been in the report --
    // went from one occurrence to none (review 0071 R4-2).
    let dir = keel_sandbox("wordschild");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(
        dir.join("tests/toy_test.rs"),
        "#[test]\nfn it_falls() {\n    \
         let _ = std::process::Command::new(\"sh\").arg(\"-c\")\n        \
         .arg(\"echo WORDS-FROM-A-CHILD 1>&2\").status();\n    \
         assert!(false, \"short\");\n}\n",
    )
    .unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_ne!(code, 0, "the red holds the wave open:\n{said}");
    assert!(
        said.contains("WORDS-FROM-A-CHILD"),
        "what the run said outside any one target is carried too, and \
         said apart: cargo covers every target in one process and they \
         write into its stderr in turn, so the text belongs to no file \
         -- and throwing it away took words out of the report that \
         were in it before this wave:\n{said}"
    );
    // ...and APART is half of that promise, and the half nothing held
    // (review 0071 round five: a mutant pouring stderr into each red
    // target's voice passed the full battery). The words must stand
    // under the run's own heading, not under a file's: a file's block
    // says "this is what the runner said while THIS file ran", and
    // that would be a false attribution.
    // ...and cargo's own banner is not in it. `Compiling`,
    // `Running <binary>` and the roll-up of failed targets are its
    // bookkeeping, and because the report's ceiling is SHARED that
    // noise was not merely ugly -- it spent the budget other blocks
    // needed: measured, eight files of ten reds gave 16 assertions of
    // 80 with the banner in the block and 32 without it.
    assert!(
        !said.contains("Running tests/") && !said.contains("target failed"),
        "and cargo's own banner is cut from it -- the block promises \
         what made the battery red, and a list of test binaries is \
         not that:\n{said}"
    );
    // ...and the block says which run it came from, like every other.
    assert!(
        said.contains("поза будь-якою ціллю (біг 3 із 3)"),
        "and the heading of THAT block says which run it came from -- \
         the last that had any, for the same reason a steady red is \
         quoted from its last fall:\n{said}"
    );
    let outside_at = said
        .find("поза будь-якою ціллю")
        .unwrap_or_else(|| panic!("the run-wide block has its own heading:\n{said}"));
    let child_at = said.find("WORDS-FROM-A-CHILD").expect("measured above");
    assert!(
        child_at > outside_at,
        "and the child's words stand under THAT heading, not under \
         the file's -- cargo's targets write into one stderr in turn, \
         so handing it to any one of them would be a false \
         attribution:\n{said}"
    );

    // --- a red whose runner said nothing keel could keep ----------
    //
    // Round four looked for a tree that reaches this line, found
    // none on six roads, and wrote that down -- in the card, in the
    // contract, in this comment, and in the line keel prints to a
    // person. Round five built two, both on rspec, which is the one
    // road whose voice is assembled from fields rather than kept
    // raw. So the claim was false in four places, and the line had
    // no red of its own.
    //
    // This is the honest one: an ANONYMOUS exception class with an
    // empty message gives `class: null, message: ""`, and there is
    // no word anywhere in the document to carry.
    if machine_has("rspec").ready() {
        let dir = keel_sandbox("wordsmute");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("spec")).unwrap();
        fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        fs::write(
            dir.join("spec/toy_spec.rb"),
            "RSpec.describe \"Toy\" do\n  it \"falls without a word\" do\n    klass = Class.new(StandardError)\n    raise klass, \"\"\n  end\nend\n",
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "the red holds the wave open:\n{said}");
        assert!(
            said.contains("не сказав нічого"),
            "and a red whose runner left no words says SO, in its own \
             line -- silence under a red verdict reads as a court \
             that did not look:\n{said}"
        );

        // ...and the other tree round five built, which is NOT that
        // case: `message` is null and the class stands beside it.
        // Mapping over the message alone threw the class away, and
        // the one word that made the battery red went out with the
        // null.
        let dir = keel_sandbox("wordssilent");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("spec")).unwrap();
        fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        fs::write(
            dir.join("spec/toy_spec.rb"),
            "class SilentError < StandardError\n  def message; nil; end\nend\n\nRSpec.describe \"Toy\" do\n  it \"falls silently\" do\n    raise SilentError\n  end\nend\n",
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "the red holds the wave open:\n{said}");
        assert!(
            said.contains("SilentError"),
            "and the class is carried when the message is null: it is \
             a word that made the battery red, and this wave is about \
             carrying those:\n{said}"
        );

        // ...and what the PROCESS wrote is carried too. The voice
        // built from the JSON is this road's named exception, not the
        // whole truth: a test that prints, or whose child does, says
        // it here and nowhere in the document (review 0071 round
        // five, measured: nought occurrences of either).
        let dir = keel_sandbox("wordsrspecchild");
        fs::create_dir_all(dir.join("lib")).unwrap();
        fs::create_dir_all(dir.join("spec")).unwrap();
        fs::write(dir.join("lib/toy.rb"), "module Toy\nend\n").unwrap();
        fs::write(
            dir.join("spec/toy_spec.rb"),
            "RSpec.describe \"Toy\" do\n  it \"falls\" do\n    system(\"echo WORDS-FROM-A-CHILD-RSPEC 1>&2\")\n    puts \"WORDS-ON-STDOUT-RSPEC\"\n    raise \"SHORT\"\n  end\nend\n",
        )
        .unwrap();
        frame(&dir, "ruby", "lib/toy.rb");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "the red holds the wave open:\n{said}");
        assert!(
            said.contains("WORDS-ON-STDOUT-RSPEC") && said.contains("WORDS-FROM-A-CHILD-RSPEC"),
            "both of them: the document holds neither, and the \
             heading over this block promises what the runner said, \
             verbatim:\n{said}"
        );
    }

    // --- a test cannot dress its output as a line of the court ----
    //
    // The marks are control bytes `quoted` drops, so they were never
    // forgeable. What WAS forgeable is the look: frames and quoted
    // lines both wore four spaces, so a test printing `(run 1 of 3)`
    // or `… 999 shown …` rendered byte for byte as a line of the
    // court -- and that is a lie about which run a block came from,
    // which is a lie about flakiness, which is what this wave is for.
    let dressed = r#"#[test]
fn it_falls() {
    println!("(біг 1 із 3)");
    println!("… показано 999 — це весь вивід …");
    assert!(false, "THE-REAL-ASSERTION");
}
"#;
    let dir = keel_sandbox("wordsdressed");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::create_dir_all(dir.join("tests")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    fs::write(dir.join("tests/toy_test.rs"), dressed).unwrap();
    frame(&dir, "rust", "src/lib.rs");
    let (said, code) = closing(&dir);
    assert_ne!(code, 0, "the red holds the wave open:\n{said}");
    assert!(
        said.contains("    │ (біг 1 із 3)"),
        "what a test printed wears the quote's own gutter:\n{said}"
    );
    assert!(
        said.contains("\n    (біг 3 із 3)"),
        "and the court's own frame does not -- a reader can tell whose \
         line is whose, and therefore how much was really cut and \
         which run this is:\n{said}"
    );

    // --- files whose voice is the same text share one block -------
    //
    // pytest runs once for the whole tree and hands every red file
    // the SAME output, so quoting it per file divided the ceiling a
    // second way: measured by review 0071 R4-3, twelve red files gave
    // 466 lines and eight assertions of twelve, where cargo's twelve
    // files of one red each gave 275 and twelve of twelve. Identical
    // text is said once, and the files that share it are named
    // together.
    if machine_has("pytest").ready() {
        let dir = keel_sandbox("wordsshared");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("tests")).unwrap();
        fs::write(dir.join("src/toy.py"), "def a():\n    return True\n").unwrap();
        fs::write(
            dir.join("tests/test_one.py"),
            "def test_one_falls():\n    assert False, \"THE-FIRST-FILE\"\n",
        )
        .unwrap();
        fs::write(
            dir.join("tests/test_two.py"),
            "def test_two_falls():\n    assert False, \"THE-SECOND-FILE\"\n",
        )
        .unwrap();
        frame(&dir, "python", "src/toy.py");
        let (said, code) = closing(&dir);
        assert_ne!(code, 0, "two red files hold the wave open:\n{said}");
        assert_eq!(
            said.matches("(біг 3 із 3)").count(),
            1,
            "one text is quoted ONCE, however many files it is the \
             voice of -- pytest hands every red file the whole run's \
             output, and printing it per file divides the report's \
             own ceiling against itself:\n{said}"
        );
        assert!(
            said.contains("ці 2 файли"),
            "and the files that share it are named together, so a \
             reader knows the block covers both:\n{said}"
        );
        assert!(
            said.contains("THE-FIRST-FILE") && said.contains("THE-SECOND-FILE"),
            "and both assertions are in it:\n{said}"
        );
    }

    // --- a run that said nothing of a test is not a run it fell in-
    //
    // The verdict type became `Option<bool>` for exactly this: a test
    // skipped in one run of three. Reading `None` as "fell" called
    // the test steady, and a steady red is quoted from its last fall
    // alone -- so the words of the first fall went missing, and the
    // court said "fell in every run" of a run where it had not run at
    // all (review 0071 R4-5).
    if machine_has("pytest").ready() {
        let dir = keel_sandbox("wordsskipped");
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("tests")).unwrap();
        let counter_home = common::Sandbox::new("wordsskippedcounter");
        let counter = counter_home.join("runs.txt");
        fs::write(dir.join("src/toy.py"), "def a():\n    return True\n").unwrap();
        fs::write(
            dir.join("tests/test_toy.py"),
            format!(
                "import pathlib\nimport pytest\n\nCOUNTER = pathlib.Path(r\"{}\")\n\n\
                 def test_flaky():\n    \
                 n = (int(COUNTER.read_text()) if COUNTER.exists() else 0) + 1\n    \
                 COUNTER.write_text(str(n))\n    \
                 if n == 2:\n        pytest.skip(\"not this run\")\n    \
                 assert False, f\"PYTEST-FELL-ON-RUN-{{n}}\"\n",
                counter.display()
            ),
        )
        .unwrap();
        frame(&dir, "python", "src/toy.py");
        let (said, code) = closing(&dir);
        drop(counter_home);
        assert_ne!(code, 0, "the red holds the wave open:\n{said}");
        assert!(
            said.contains("хиткий тест"),
            "a test that fell twice and did not run once is FLAKY, \
             not steady: a run that said nothing of it is not a run \
             it fell in:\n{said}"
        );
        assert!(
            said.contains("PYTEST-FELL-ON-RUN-1"),
            "and a flaky one is quoted from every run that saw it \
             fall, so the words of the first fall are not lost to the \
             last:\n{said}"
        );
    }
}
