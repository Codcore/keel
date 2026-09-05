//! Scenario test of wave 0042: a tongue that tells the two apart
//! says so.
//!
//! These run a real `mix` against real projects. Where mix is not on
//! the machine the probe says so and stops rather than pretending --
//! a green test over a language nobody ran is exactly the fixture
//! more convenient than reality that review 0041 named.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

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

fn have_mix() -> bool {
    Command::new("mix")
        .arg("--version")
        .output()
        .is_ok_and(|out| out.status.success())
}

const BODY: &str = "тіло обіцянки\n\n";

/// A mix project: mix.exs, a module, and a test file as given.
fn project(name: &str, test_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"elixir\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("lib/toy")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("mix.exs"),
        "defmodule Toy.MixProject do\n  use Mix.Project\n  def project do\n    [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\n  end\n  def application, do: []\nend\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("lib/toy.ex"),
        "defmodule Toy do\n  def works, do: true\nend\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("lib/toy/bar.ex"),
        "defmodule Toy.Bar do\n  def works(a, b), do: a + b\nend\n",
    )
    .unwrap();
    std::fs::write(dir.join("test/test_helper.exs"), "ExUnit.start()\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - lib/toy.ex\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("test/toy_test.exs"), test_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn test_file(rev: &str) -> String {
    format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert Toy.works()\n  end\nend\n"
    )
}

/// proves: a-tongue-that-tells-the-two-apart-says-so@4a6eee -- sec. 7.12
/// was written for the case where an adapter CAN tell a failure from
/// a broken build. Ruby cannot: both exit 1. Elixir can: 2 on a
/// failure, 1 on a compilation error, 0 green. Dragging ruby's border
/// along would be as untrue as leaving one unsaid.
#[test]
fn a_tongue_that_tells_the_two_apart_says_so() {
    assert!(have_mix(), "this probe runs a real mix; it is not on PATH");
    let rev = keel::rev::text_rev(BODY);

    // A file that does not compile is a refusal aloud, not a red test
    // and not a green battery.
    let broken = "defmodule ToyTest do\n  use ExUnit.Case\n  test \"it works\" do\n";
    let dir = project("exbroken", broken);
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a file that does not compile never closes:\n{said}"
    );
    assert!(
        !said.contains("закрита"),
        "and the verdict does not read as closure:\n{said}"
    );

    // And the border ruby needed is NOT printed here, because it is
    // not true of this project.
    let dir = project("exborder", &test_file(&rev));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a whole elixir project is judged:\n{said}");
    // Ruby's row says the two are NOT told apart; this one says they
    // are. The words overlap, so the assertion names the half that
    // differs rather than a fragment both share.
    assert!(
        !said.contains("ruby не відрізняє"),
        "the check does not carry ruby's border into a tongue that \
         does not have it:\n{said}"
    );
    assert!(
        said.contains("ця мова відрізняє") && said.contains("1 не скомпілювалось"),
        "it says the opposite, with the codes it measured:\n{said}"
    );

    // A test inside a `describe` block: ExUnit puts the block's name
    // in front of it, so `--only 'test:test <bare name>'` matches
    // NOTHING there. Measured, and it is why the reader tracks the
    // block rather than naming a border it could have hidden behind.
    let grouped = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  describe \"the group\" do\n    # proves: it-works@{rev}\n    test \"inside\" do\n      assert Toy.works()\n    end\n  end\nend\n"
    );
    let dir = project("exdescribe", &grouped);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a tagged test inside a describe is read:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "and counted:\n{said}"
    );
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, "work: тіло\n").unwrap();
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
        "and it really runs -- a name mix cannot select would have \
         been \"not run\", never green:\n{said}"
    );
    assert!(
        said.contains("робота проходить"),
        "so the work passes over a test that truly ran:\n{said}"
    );

    // R-9: and the other half of that sentence, which the red birth
    // held and the green commit dropped. A tag over a name ExUnit
    // does not know is "NOT RUN" -- never "work passes". `if false
    // do` is the honest way to build it: the reader sees the
    // declaration and takes the name, ExUnit registers nothing, and
    // `--only` leaves with 1 and "no test was executed" (measured).
    let ghostly = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  if false do\n    # proves: it-works@{rev}\n    test \"it works\" do\n      assert Toy.works()\n    end\n  end\n\n  test \"other\" do\n    assert true\n  end\nend\n"
    );
    let dir = project("exnotrun", &ghostly);
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, "work: тіло\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert_ne!(
        out.status.code().unwrap_or(-1),
        0,
        "work over a test that never ran does not pass the gate:\n{said}"
    );
    assert!(
        !said.contains("робота проходить"),
        "and \"not run\" is not read as green:\n{said}"
    );
}

/// The shapes `classify` must tell apart, played without a project on
/// disk. Unlike ruby, this tongue answers with distinct codes -- and
/// the one code that carries two meanings (1 is both "did not
/// compile" and "--only matched nothing") is told apart by the text.
#[test]
fn what_mix_said_and_how_it_left() {
    use keel::adapter::Outcome;

    assert!(matches!(
        keel::elixir::classify("2 tests, 0 failures", 0),
        Outcome::Green
    ));
    assert!(matches!(
        keel::elixir::classify("2 tests, 1 failure", 2),
        Outcome::Failed
    ));
    assert!(matches!(
        keel::elixir::classify(
            "The --only option was given to \"mix test\" but no test was executed",
            1
        ),
        Outcome::NotRun
    ));
    assert!(matches!(
        keel::elixir::classify(
            "== Compilation error in file test/toy_test.exs ==\n** (TokenMissingError) missing terminator",
            1
        ),
        Outcome::BuildBroken(_)
    ));
}

/// The shapes review 0042 measured and the wave had not: a doctest
/// `mix new` writes itself, a test after a describe block, a name
/// with an escaped quote, an attribute between the tag and its test,
/// an unread `.exs`, and the compiler's own words in a refusal.
#[test]
fn the_battery_believes_mix_and_not_the_source() {
    assert!(have_mix(), "this probe runs a real mix; it is not on PATH");
    let rev = keel::rev::text_rev(BODY);

    let closing = |dir: &Path| {
        std::fs::write(
            dir.join("keel/reviews/0001-a-wave.md"),
            "# Рецензія\n\nok\n",
        )
        .unwrap();
        git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
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
    };

    // R-2, and the worst of it: a `doctest` is what `mix new` writes
    // into every generated test file, and no reader of ours was ever
    // going to see it. Its failure vanished, and the wave closed.
    let with_doctest = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n  doctest Toy\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert Toy.works()\n  end\nend\n"
    );
    let dir = project("exdoctest", &with_doctest);
    std::fs::write(
        dir.join("lib/toy.ex"),
        "defmodule Toy do\n  @doc \"\"\"\n      iex> Toy.works()\n      :not_this\n  \"\"\"\n  def works, do: true\nend\n",
    )
    .unwrap();
    let (said, _) = closing(&dir);
    assert!(
        said.contains("червоний тест") && said.contains("Toy.works/0"),
        "mix counted the doctest, so the court names it red -- by the \
         name mix gave it:\n{said}"
    );
    // The border, measured and not painted over: naming a red test
    // that no scenario claims does not, in any tongue, hold the wave
    // open. Ruby does the same (BACKLOG: «незаявлений червоний тест»).
    // Wave 0042 promised that mix is read; who a red test binds is
    // older law than this wave, and not its to change.

    // R-1: a test AFTER a describe block belongs to no block, and the
    // group used to leak to the end of the module -- so a scenario was
    // called proven by a test that had just failed.
    let after = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  describe \"a group\" do\n    test \"inside\" do\n      assert true\n    end\n  end\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert 1 == 2\n  end\nend\n"
    );
    let dir = project("exafter", &after);
    let (said, code) = closing(&dir);
    assert_ne!(
        code, 0,
        "a red test after a describe block keeps the wave open:\n{said}"
    );
    assert!(
        said.contains("it works") && !said.contains("a group it works"),
        "and it is named as mix names it, not with a group it is not \
         in:\n{said}"
    );

    // R-2 again: a name with an escaped quote is a name mix runs.
    let quoted = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert Toy.works()\n  end\n\n  test \"he said \\\"no\\\"\" do\n    assert 1 == 2\n  end\nend\n"
    );
    let dir = project("exquoted", &quoted);
    let (said, _) = closing(&dir);
    assert!(
        said.contains("батарея: 2 тестів"),
        "the battery counts what mix ran, both of them -- an escaped \
         quote in a name is a name:\n{said}"
    );
    assert!(
        said.contains("червоний тест") && said.contains("he said"),
        "and the red one is named as mix names it:\n{said}"
    );

    // R-7: `@tag :slow` is an everyday ExUnit line, and it stood
    // between a tag and its test.
    let tagged = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  @tag :slow\n  test \"it works\" do\n    assert Toy.works()\n  end\nend\n"
    );
    let dir = project("exattr", &tagged);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "an attribute does not orphan the tag:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "and the tag is read:\n{said}"
    );

    // R-6: an unread `.exs` is named, as ruby's are.
    let dir = project("exunread", &test_file_of(&rev));
    std::fs::create_dir_all(dir.join("test/support")).unwrap();
    std::fs::write(
        dir.join("test/support/helpers.exs"),
        "# proves: it-works@aaaaaa\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "an unread file is a limit, not a finding:\n{said}");
    assert!(
        said.contains("helpers.exs"),
        "and the verdict names the file it did not read:\n{said}"
    );

    // R-8: a measured fact is not a thing left unchecked.
    assert!(said.contains("зміряно:"), "the border is said:\n{said}");
    let unchecked: usize = said
        .split("речі не перевірено")
        .next()
        .and_then(|head| head.rsplit(", ").next())
        .and_then(|word| word.trim().parse().ok())
        .unwrap_or(0);
    assert!(
        !said.contains("2 речі не перевірено") || unchecked == 2,
        "and the count of unchecked things does not swallow it:\n{said}"
    );

    // R-16: the thin half of the form court, and the ghost inside a
    // `@moduledoc`. Both measured here rather than asserted from the
    // dictionary: the promise is that the tongue SAYS its borders,
    // and a border said but untrue is worse than one left unsaid.
    let dir = project("exghost", &test_file_of(&rev));
    std::fs::write(
        dir.join("lib/toy.ex"),
        "defmodule Toy do\n  @moduledoc \"\"\"\n      def ghost(a, b)\n  \"\"\"\n  def works, do: true\nend\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: Toy\nexports:\n  - \"def ghost(a, b)\"\n---\n\nПривид у документації.\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    // Wave 0042 measured this holding and named the border; wave 0043
    // took the border away -- what lives only inside a `@moduledoc`
    // is not source in any tongue. The case stays here because it is
    // elixir's shape of it, and a court once wrong is worth watching.
    assert_ne!(code, 0, "the ghost does not hold:\n{said}");
    assert!(
        said.contains("lib/toy.ex"),
        "and the finding names the file it looked in:\n{said}"
    );
    assert!(
        said.contains("сигнатур звірено: 1"),
        "the form court DID compare it -- the refusal is a verdict, \
         not a court that skipped (review 0043 R-14: this assertion \
         was dropped when the case was turned round):\n{said}"
    );
    assert!(
        said.contains("не пише типів"),
        "and the border that DOES stand is still said: elixir writes \
         no types, so green form is less than meaning:\n{said}"
    );

    // The reviewer's mutation list, and its sharp reading: "every
    // thing the court DOES is judged; every thing it SAYS TO A
    // PERSON is not". M26, M27 and M30 are that list's remainder.

    // M26 -- the generated CI step. A ruby project once got a
    // workflow with no battery at all (review 0038 R-9); an elixir
    // one must get mix's line, not cargo's.
    let dir = project("exci", &test_file_of(&rev));
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"elixir\"\nci = \"github\"\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["update"]);
    assert_eq!(code, 0, "the workflow is written:\n{said}");
    let flow = std::fs::read_to_string(dir.join(".github/workflows/keel.yml")).unwrap();
    assert!(
        flow.contains("mix test") && !flow.contains("cargo test"),
        "and it carries mix's battery, not cargo's:\n{flow}"
    );

    // M27 -- the line a person is told to type. The same lesson,
    // one command over: `keel next` hands out the tongue's own way
    // to run exactly one test.
    let dir = project("exnext", &test_file_of(&rev));
    git(&dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let (said, _) = keel(&dir, &["next"]);
    assert!(
        said.contains("mix test --only 'test:test it works'"),
        "next hands the tongue's own run line:\n{said}"
    );
    assert!(
        !said.contains("cargo test"),
        "and not another tongue's:\n{said}"
    );

    // M30 -- the battery map is keyed by FILE and name, and the file
    // half must be real: two files, the same test name in both, one
    // red. Keyed loosely, the green one answers for the red.
    let two = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert 1 == 2\n  end\nend\n"
    );
    let dir = project("exkeys", &two);
    std::fs::write(
        dir.join("test/other_test.exs"),
        "defmodule OtherTest do\n  use ExUnit.Case\n\n  test \"it works\" do\n    assert true\n  end\nend\n",
    )
    .unwrap();
    let (said, _) = closing(&dir);
    assert!(
        said.contains("червоний тест") && said.contains("toy_test"),
        "a red test is named with the file it is in -- a same-named \
         green one elsewhere does not answer for it:\n{said}"
    );

    // M28's real shape, found by playing the mutation the reviewer
    // left standing: the guard in `quoted_after` was never the
    // question. `@doc """ … """` holding an example written in the
    // language it documents is the most ordinary elixir there is,
    // and the reader had no idea heredocs existed -- so a file `mix
    // test` runs GREEN was refused, and a phantom `test "..."` line
    // inside a heredoc could be taken for a declaration.
    let herdoc = format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  @doc \"\"\"\n      test \"an example\" is the shape we use\n  \"\"\"\n  test \"it works\" do\n    assert Toy.works()\n  end\nend\n"
    );
    let dir = project("exheredoc", &herdoc);
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "a doc example does not orphan the tag above it -- mix runs \
         this file green:\n{said}"
    );
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "and the tag holds its own test, not the prose:\n{said}"
    );
    let (said, _) = closing(&dir);
    assert!(
        !said.contains("an example"),
        "no line inside a heredoc is ever taken for a declaration:\n{said}"
    );

    // R-5: the refusal carries the compiler's own words, which is
    // what the scenario promises -- not the banner above them.
    let dir = project(
        "exwords",
        "defmodule ToyTest do\n  use ExUnit.Case\n  test \"it works\" do\n",
    );
    let (said, _) = closing(&dir);
    assert!(
        said.contains("** ("),
        "the refusal carries what the compiler said, not only that it \
         could not compile:\n{said}"
    );
}

fn test_file_of(rev: &str) -> String {
    format!(
        "defmodule ToyTest do\n  use ExUnit.Case\n\n  # proves: it-works@{rev}\n  test \"it works\" do\n    assert Toy.works()\n  end\nend\n"
    )
}
