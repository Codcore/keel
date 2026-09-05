//! Scenario test of wave 0047: rspec examples are read and run.
//!
//! These run a real `rspec` against real projects; where rspec is not
//! on the machine the probe stops aloud with the hand of wave 0044.
//! The ruby adapter's SECOND reading: `spec/**/*_spec.rb`, examples
//! named by rspec's own full description, selected by id and judged
//! from rspec's JSON.
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
    keel_with(dir, args, &[])
}

fn keel_with(dir: &Path, args: &[&str], envs: &[(&str, String)]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
        .envs(envs.iter().map(|(k, v)| (*k, v.as_str())))
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

const BODY: &str = "тіло обіцянки\n\n";

/// A ruby project in rspec's own standard shape: `.rspec` carrying
/// `--require spec_helper` (as `rspec --init` writes it), a helper in
/// `spec/`, a module in `lib/`, and the spec file as given.
fn project(name: &str, spec_body: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("spec")).unwrap();
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works\n    true\n  end\nend\n",
    )
    .unwrap();
    std::fs::write(dir.join(".rspec"), "--require spec_helper\n").unwrap();
    std::fs::write(
        dir.join("spec/spec_helper.rb"),
        "require \"toy\"\n\nRSpec.configure do |config|\n  config.color = false\nend\n",
    )
    .unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - lib/toy.rb\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("spec/toy_spec.rb"), spec_body).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

fn spec_file(rev: &str) -> String {
    format!(
        "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n    expect(Toy.works).to be(true)\n  end\nend\n"
    )
}

fn gate(dir: &Path) -> (String, i32) {
    gate_with(dir, &[])
}

fn gate_with(dir: &Path, envs: &[(&str, String)]) -> (String, i32) {
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
    let msg = dir.join("COMMIT_EDITMSG");
    std::fs::write(&msg, "work: тіло\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["gate", msg.to_str().unwrap(), dir.to_str().unwrap()])
        .envs(envs.iter().map(|(k, v)| (*k, v.as_str())))
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

fn reviewed(dir: &Path) {
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "review"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

fn walk(root: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.file_name().is_some_and(|n| n == ".git") {
                continue;
            }
            out.push(path.display().to_string());
            if path.is_dir() {
                stack.push(path);
            }
        }
    }
    out
}

/// proves: rspec-examples-are-read-and-run@31f651
#[test]
fn rspec_examples_are_read_and_run() {
    if !common::machine_has("rspec").ready() {
        return;
    }
    let rev = keel::rev::text_rev(BODY);

    // The tag is read from spec/*_spec.rb over `it "…" do`, and the
    // project is judged whole -- the second reading of the same
    // adapter.
    let dir = project("rsread", &spec_file(&rev));
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "an rspec project is judged whole:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "and the tag over `it \"…\"` is read:\n{said}"
    );

    // The gate runs exactly that example -- by the ID rspec gives it
    // in a dry run, never by its name as a string -- and reads the
    // state from rspec's JSON.
    let (said, code) = gate(&dir);
    assert_eq!(
        code, 0,
        "the work passes over an example rspec ran green:\n{said}"
    );
    assert!(said.contains("робота проходить"), "and says so:\n{said}");

    // A tagged example that FAILS: rspec leaves with 1, the same code
    // as a load error, and the verdict is the JSON state `failed` --
    // "падає", not "broken", not "did not run".
    let dir = project(
        "rsred",
        &format!(
            "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n    expect(Toy.works).to be(false)\n  end\nend\n"
        ),
    );
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "work over a red example does not pass:\n{said}");
    assert!(
        said.contains("падає") && said.contains("Toy works"),
        "and it is called a failing example, by rspec's full description:\n{said}"
    );
    assert!(
        !said.contains("SyntaxError") && !said.contains("не виконав"),
        "not a broken build, not \"did not run\":\n{said}"
    );

    // Pending -- `pending` in the body, `xit`, `skip:` -- is "did not
    // run" (§7.12): rspec's state is `pending`, and a dry run would
    // even call it passed, so the state comes from the real run only.
    for (name, line, inside) in [
        (
            "rspending",
            "it \"works\" do",
            "pending \"later\"\n    expect(Toy.works).to be(false)",
        ),
        (
            "rsxit",
            "xit \"works\" do",
            "expect(Toy.works).to be(false)",
        ),
        (
            "rsskip",
            "it \"works\", skip: \"reason\" do",
            "expect(Toy.works).to be(false)",
        ),
    ] {
        let dir = project(
            name,
            &format!(
                "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  {line}\n    {inside}\n  end\nend\n"
            ),
        );
        let (said, code) = gate(&dir);
        assert_ne!(code, 0, "{name}: a pending example proves nothing:\n{said}");
        assert!(
            said.contains("не виконав жодного тесту"),
            "{name}: and it is \"did not run\", not green and not red:\n{said}"
        );
    }

    // An example rspec never registers: no id in the dry run, and
    // "did not run" -- though rspec leaves with 0 and says
    // `0 examples`.
    let dir = project(
        "rsnotrun",
        &format!(
            "RSpec.describe Toy do\n  if false\n    # proves: it-works@{rev}\n    it \"works\" do\n      expect(Toy.works).to be(true)\n    end\n  end\n\n  it \"other\" do\n    expect(true).to be(true)\n  end\nend\n"
        ),
    );
    let (said, code) = gate(&dir);
    assert_ne!(
        code, 0,
        "work over an example that never ran does not pass:\n{said}"
    );
    assert!(
        said.contains("не виконав жодного тесту"),
        "and \"did not run\" is said, never green:\n{said}"
    );

    // A SyntaxError in the MODULE: rspec leaves with 1, the same code
    // as a failed example. The court says "broken" with ruby's own
    // words, read from the JSON rspec still writes, and never "red
    // example" -- in the gate and in the battery alike.
    let dir = project("rsbroken", &spec_file(&rev));
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works( = true\nend\n",
    )
    .unwrap();
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "a broken module is not a green example:\n{said}");
    assert!(
        said.contains("SyntaxError") && said.contains("toy.rb"),
        "and the refusal carries ruby's own words and the place:\n{said}"
    );
    assert!(
        !said.contains("падає") && !said.contains("червоний тест"),
        "a build that broke is a REFUSAL, not a red example:\n{said}"
    );
    let dir = project("rsbrokenclose", &spec_file(&rev));
    std::fs::write(
        dir.join("lib/toy.rb"),
        "module Toy\n  def self.works( = true\nend\n",
    )
    .unwrap();
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a battery over a file that did not load does not close:\n{said}"
    );
    assert!(
        said.contains("SyntaxError") && !said.contains("червоний тест"),
        "and it is a refusal with ruby's words, not a red test:\n{said}"
    );

    // A GREEN close: the tag and the battery meet on one key
    // (`spec/toy_spec`), and the wave closes. With SPEC_OPTS in the
    // environment -- rspec reads it, and a `--tag` there filters the
    // run away (measured), so the adapter drops it -- and nothing is
    // written into the project: no `.rspec_status`, no cache.
    let dir = project("rsgreen", &spec_file(&rev));
    reviewed(&dir);
    let before = walk(&dir);
    let (said, code) = keel_with(
        &dir,
        &["close"],
        &[(
            "SPEC_OPTS",
            "--format documentation --tag nothing".to_string(),
        )],
    );
    assert_eq!(code, 0, "a proven rspec wave closes:\n{said}");
    assert!(
        said.contains("0001-a-wave: закрита"),
        "and says so:\n{said}"
    );
    let after = walk(&dir);
    let written: Vec<&String> = after.iter().filter(|p| !before.contains(p)).collect();
    assert!(
        written.is_empty(),
        "close leaves the project as it found it: {written:?}"
    );

    // Two examples of ONE full description under the GATE, the red
    // one first: the court reads every id the dry run named, and a
    // red among them is red -- the rule review 0046 R-2 asked to be
    // one in both courts (review 0047 R-5 found no probe held it
    // here).
    let dir = project(
        "rstwice",
        &format!(
            "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n    expect(1).to eq(2)\n  end\n\n  it \"works\" do\n    expect(1).to eq(1)\n  end\nend\n"
        ),
    );
    let (said, code) = gate(&dir);
    assert_ne!(
        code, 0,
        "a red among two examples of one name is red:\n{said}"
    );
    assert!(said.contains("падає"), "and said so:\n{said}");

    // rspec that STARTED and left without its JSON -- an `abort` in
    // spec_helper -- is not rspec being absent: the refusal carries
    // what it said, not "put rspec on PATH" (review 0047 R-9).
    let dir = project("rsabort", &spec_file(&rev));
    std::fs::write(
        dir.join("spec/spec_helper.rb"),
        "abort \"no database here\"\n",
    )
    .unwrap();
    let (said, code) = gate(&dir);
    assert_ne!(code, 0, "a helper that aborts proves nothing:\n{said}");
    assert!(
        said.contains("no database here") && !said.contains("PATH"),
        "and the refusal carries rspec's own words, not advice about PATH:\n{said}"
    );

    // The battery: the roll AND the verdicts from rspec's JSON. A
    // second example the reader did not tag still exists for the
    // court, together with its failure, named by rspec's full
    // description; an example inside `describe`/`context` is named
    // through its groups; a pending one is neither green nor red;
    // two examples of ONE full description are one key, red if
    // either is (the failing one first, so a court that let the last
    // win would call it green).
    let dir = project(
        "rsbattery",
        &format!(
            "RSpec.describe Toy do\n  # proves: it-works@{rev}\n  it \"works\" do\n    expect(Toy.works).to be(true)\n  end\n\n  it \"nobody claims me\" do\n    expect(1).to eq(2)\n  end\n\n  it \"not now\" do\n    pending \"later\"\n    expect(1).to eq(2)\n  end\n\n  describe \"#works\" do\n    context \"when grouped\" do\n      it \"is inside\" do\n        expect(true).to be(true)\n      end\n    end\n  end\n\n  it \"twice\" do\n    expect(1).to eq(2)\n  end\n\n  it \"twice\" do\n    expect(1).to eq(1)\n  end\nend\n"
        ),
    );
    reviewed(&dir);
    let (said, code) = keel(&dir, &["close"]);
    assert!(
        said.contains("батарея: 4 тестів"),
        "the battery counts what rspec RAN -- the grouped one and the two \
         `twice` as ONE included; not the pending one:\n{said}"
    );
    assert!(
        !said.contains("not now"),
        "a pending example is neither green nor red, so it is not named:\n{said}"
    );
    assert!(
        said.contains("червоний тест") && said.contains("Toy nobody claims me"),
        "and names the red one by rspec's full description:\n{said}"
    );
    assert!(
        said.contains("Toy twice"),
        "one red among two examples of one description is a red one:\n{said}"
    );
    assert_ne!(code, 0, "so a red battery does not close:\n{said}");
}

/// What rspec is HANDED: a dry run with `--format json --out <file
/// outside the project>`, then the run of exactly the id that dry run
/// named -- never the name as a string, never `-e` (a substring
/// match) -- read off a shim named `rspec` that logs its argv and
/// answers with rspec's own JSON shapes.
#[test]
fn what_rspec_is_handed() {
    let rev = keel::rev::text_rev(BODY);
    let dir = project("rsshim", &spec_file(&rev));
    let bin = dir.join("shim");
    std::fs::create_dir_all(&bin).unwrap();
    let log = dir.join("argv.log");
    // The shim: log argv (one line per run), find the `--out` path,
    // and write the JSON a dry run would -- TWO examples, so that the
    // run of "exactly the id" is a claim with teeth (review 0047 R-5,
    // mutation M1: running every id of the file passed a shim of one)
    // -- and the JSON a run would: whatever was asked for, passed.
    std::fs::write(
        bin.join("rspec"),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{log}'\nout=''\nprev=''\ndry=''\nfor a in \"$@\"; do\n  if [ \"$prev\" = \"--out\" ]; then out=\"$a\"; fi\n  if [ \"$a\" = \"--dry-run\" ]; then dry=yes; fi\n  prev=\"$a\"\ndone\nif [ -n \"$dry\" ]; then\n  printf '%s' '{{\"examples\":[{{\"id\":\"./spec/toy_spec.rb[1:1]\",\"description\":\"works\",\"full_description\":\"Toy works\",\"status\":\"passed\"}},{{\"id\":\"./spec/toy_spec.rb[1:2]\",\"description\":\"other\",\"full_description\":\"Toy other\",\"status\":\"passed\"}}],\"summary\":{{\"example_count\":2,\"failure_count\":0,\"pending_count\":0,\"errors_outside_of_examples_count\":0}},\"summary_line\":\"2 examples, 0 failures\"}}' > \"$out\"\nelse\n  printf '%s' '{{\"examples\":[{{\"id\":\"./spec/toy_spec.rb[1:1]\",\"description\":\"works\",\"full_description\":\"Toy works\",\"status\":\"passed\"}}],\"summary\":{{\"example_count\":1,\"failure_count\":0,\"pending_count\":0,\"errors_outside_of_examples_count\":0}},\"summary_line\":\"1 example, 0 failures\"}}' > \"$out\"\nfi\n",
            log = log.display()
        ),
    )
    .unwrap();
    let mut perms = std::fs::metadata(bin.join("rspec")).unwrap().permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut perms, 0o755);
    std::fs::set_permissions(bin.join("rspec"), perms).unwrap();
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let (said, code) = gate_with(&dir, &[("PATH", path)]);
    assert_eq!(code, 0, "the shim's JSON is read as green:\n{said}");
    let logged = std::fs::read_to_string(&log).unwrap();
    let runs: Vec<&str> = logged.lines().collect();
    assert_eq!(runs.len(), 2, "a dry run, then the run:\n{logged}");
    assert!(
        runs[0].contains("--dry-run")
            && runs[0].contains("--format json")
            && runs[0].contains("--out ")
            && runs[0].contains("--no-color")
            && runs[0].ends_with("spec/toy_spec.rb"),
        "the dry run asks for JSON into a file, colourless, over the file:\n{logged}"
    );
    assert!(
        !runs[0].contains("Toy works") && !runs[1].contains("Toy works") && !logged.contains("-e "),
        "the name never goes into a command as a string:\n{logged}"
    );
    assert!(
        runs[1].ends_with("spec/toy_spec.rb[1:1]")
            && !runs[1].contains("[1:2]")
            && !runs[1].contains("--dry-run"),
        "the run is of exactly the id the dry run named for the tag, and not \
         the other example of the file:\n{logged}"
    );
    let out_path = runs[0]
        .split_whitespace()
        .skip_while(|w| *w != "--out")
        .nth(1)
        .expect("an --out path");
    assert!(
        !Path::new(out_path).starts_with(&*dir),
        "the JSON file lives outside the project: {out_path}"
    );
}
