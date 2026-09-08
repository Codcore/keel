//! Scenario test of wave 0055: the courts agree on one tree.
//!
//! Five places where two courts read one tree and said different
//! things (final review 2026-09-06):
//!
//! * a cargo target with `harness = false` whose runner prints a
//!   green block and leaves with 1 -- the belt that refuses it was
//!   held by code alone, and mutant M04, which lifts the belt,
//!   survived the whole battery (tests R-1);
//! * pytest leaving 1 with every test passed -- a session hook's
//!   own failure -- read as a green battery, where cargo's hand
//!   refuses (tests R-2);
//! * `#[should_panic]`, whose verdict line cargo writes as `test
//!   it_panics - should panic ... ok`: the reader kept the whole of
//!   it as the name, so `keel close` said the battery ran no test of
//!   that name while the gate was green (bugs R-9);
//! * a `verify` command §7.16 does not trust: `keel check` red,
//!   `keel close` "closed, no blockers", exit 0 (bugs R-13);
//! * the tags of a wave called off: `keel check` reddened them as
//!   orphans, though §6.3-a puts a called-off wave outside judgement
//!   whole (methodology R-2).
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

fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!(
                "  {cut}: \"n/a, бо ця пісочниця грає інший розріз\"\n"
            ));
        }
    }
    block
}

const BODY: &str = "body of s\n";

/// A crate on the branch of wave 0009-w, with one promise proven by
/// `tests/t_test.rs`, the review committed, and whatever else the
/// case needs.
fn project(name: &str, manifest_tail: &str, test_body: &str) -> Sandbox {
    let dir = keel_sandbox(name);
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        &format!(
            "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\n[lib]\npath = \"src/lib.rs\"\n{manifest_tail}"
        ),
    );
    write(&dir, "src/lib.rs", "pub fn one() {}\n");
    let rev = keel::rev::text_rev(BODY);
    write(
        &dir,
        "tests/t_test.rs",
        &format!("/// proves: s@{rev}\n#[test]\nfn holds_s() {{ {test_body} }}\n"),
    );
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\n{BODY}\n## transform: t\n\nthe work\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    write(&dir, "keel/reviews/0009-w.md", "# Review\n\nok\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the trunk"]);
    git(&dir, &["checkout", "-q", "-b", "0009-w"]);
    dir
}

/// proves: the-courts-agree-on-one-tree@73e245 -- one tree, two
/// courts, and five ways they answered differently: a runner cargo
/// could not read, a pytest that failed outside its tests, a name
/// cargo spells with its own suffix, a command trust does not let
/// run, and a wave called off.
#[test]
fn the_courts_agree_on_one_tree() {
    // -- a target the reader cannot read is not a green battery -----
    // `harness = false`: the runner is a plain binary, and this one
    // prints exactly what libtest would print for a green run, then
    // leaves with 1. Nothing red is there to read, and the exit says
    // the tree is not green: the court refuses rather than guessing.
    let dir = project(
        "oneverdictraw",
        "\n[[test]]\nname = \"raw\"\npath = \"tests/raw.rs\"\nharness = false\n",
        "",
    );
    write(
        &dir,
        "tests/raw.rs",
        "fn main() {\n    println!(\"running 1 test\");\n    println!(\"test holds_raw ... ok\");\n    println!(\"test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\");\n    std::process::exit(1);\n}\n",
    );
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "t: the work"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a battery whose runner left red with nothing red read does \
         not close a wave:\n{said}"
    );
    assert!(
        said.contains("cargo"),
        "and the refusal is the adapter's own, naming what it saw:\n{said}"
    );
    assert!(
        !said.contains("0009-w: closed"),
        "the wave is not called closed over it:\n{said}"
    );

    // -- `#[should_panic]` is one name in both courts ----------------
    // cargo writes the verdict line as `test it_panics - should panic
    // ... ok`; the tag names `it_panics`, and so must the battery.
    let dir = project("oneverdictpanic", "", "");
    let rev = keel::rev::text_rev(BODY);
    write(
        &dir,
        "tests/t_test.rs",
        &format!(
            "/// proves: s@{rev}\n#[test]\n#[should_panic]\nfn holds_s() {{ panic!(\"as promised\"); }}\n"
        ),
    );
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "t: the work"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(
        code, 0,
        "a promise proven by a test that must panic closes its \
         wave:\n{said}"
    );
    assert!(
        !said.contains("should panic"),
        "the battery keys the test by the name the tag carries, not by \
         cargo's own suffix:\n{said}"
    );
    assert!(
        said.contains("0009-w: closed"),
        "and the wave is closed:\n{said}"
    );

    // -- a command trust does not let run is a blocker, not a note ---
    let dir = project("oneverdicttrust", "", "");
    write(
        &dir,
        "keel/contracts/toy-thing.md",
        "---\nmodule: toy::thing\nverify: \"true\"\n---\n\nThe promise, proven by a command.\n",
    );
    write(&dir, "src/thing.rs", "pub fn thing() {}\n");
    write(&dir, "src/lib.rs", "pub mod thing;\n");
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "t: the work"]);
    let (check, check_code) = keel(&dir, &["check"]);
    assert_eq!(
        check_code, 1,
        "the documents court reddens over a command it does not \
         trust (§7.16):\n{check}"
    );
    let (said, _code) = keel(&dir, &["close"]);
    assert!(
        said.contains("did not run") && said.contains("keel trust"),
        "the closing court names the proof that did not run and the \
         hand that lets it run (§7.16):\n{said}"
    );
    // The counted line itself, not merely the absence of a footer:
    // the first cut of this wave was held only by what it did NOT
    // say, so a mutant that dropped the line entirely lived (review
    // 0055 R-6).
    assert!(
        said.contains("promises whose proof did not run: 1"),
        "and counts them on a line of its own:\n{said}"
    );
    assert!(
        !said.lines().any(|line| line.starts_with("no blockers")),
        "and does not say \"no blockers\" under it: the verdict of \
         distrust is check's (wave 0010 promised this court would not \
         duplicate it), and while it stands red the tree does not \
         merge -- a footer that says otherwise is the two courts \
         disagreeing over one tree:\n{said}"
    );

    // Trust recorded by hand, and both courts agree the other way.
    let (said, code) = keel(&dir, &["trust"]);
    assert_eq!(code, 0, "trust is recorded:\n{said}");
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "t: trust recorded"]);
    let (said, code) = keel(&dir, &["close"]);
    assert_eq!(code, 0, "with trust recorded the wave closes:\n{said}");
    assert!(
        said.lines().any(|line| line.starts_with("no blockers")),
        "and the footer is the plain one again:\n{said}"
    );

    // The project's own gate, distrusted, gets the same treatment:
    // the first cut took the footer away and put nothing in its
    // place, so this court said nothing at all about the tree it had
    // not judged (review 0055 R-6).
    let dir = project("oneverdictci", "", "");
    write(
        &dir,
        "keel.toml",
        "lang = \"en\"\nadapter = \"cargo\"\nci = \"true\"\n",
    );
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "t: the work"]);
    let (said, _code) = keel(&dir, &["close"]);
    assert!(
        said.contains("did not run") && said.contains("gate (ci)"),
        "an untrusted ci is named as the gate that did not judge this \
         tree:\n{said}"
    );
    assert!(
        !said.lines().any(|line| line.starts_with("no blockers")),
        "and no footer says otherwise:\n{said}"
    );

    // -- a project living in a subdirectory of a bigger repository --
    // `HEAD:<path>` is read from the top of the work tree, so the
    // report of a project in a subdirectory was invisible to this
    // court while standing in history (review 0055 R-3).
    let outer = keel_sandbox("oneverdictouter");
    git(&outer, &["init", "-q", "-b", "main"]);
    write(&outer, "README.md", "the outer repository\n");
    let inner = project("oneverdictinner", "", "");
    let sub = outer.join("sub");
    std::fs::create_dir_all(&sub).unwrap();
    for entry in std::fs::read_dir(inner.path()).unwrap().flatten() {
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let out = Command::new("cp")
            .arg("-r")
            .arg(entry.path())
            .arg(sub.join(&name))
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "the project is copied into the repository"
        );
    }
    git(&outer, &["add", "-A"]);
    git(&outer, &["commit", "-q", "-m", "the project rides inside"]);
    git(&outer, &["checkout", "-q", "-b", "0009-w"]);
    let (said, _code) = keel(&sub, &["close"]);
    assert!(
        !said.contains("is not in the branch's history"),
        "the report of a project in a subdirectory is read from that \
         project's own place in HEAD:\n{said}"
    );

    // -- the tags of a wave called off are outside judgement --------
    let dir = project("oneverdictoff", "", "");
    let rev = keel::rev::text_rev("body of gone\n");
    write(
        &dir,
        "keel/waves/0010-off.md",
        &format!(
            "---\ncancelled: \"the operator called it off\"\nscenarios:\n  gone: {{covers: [performance.capacity]}}\ntransforms:\n  t:\n    implements: [gone]\n    files: [src/lib.rs]\n{}---\n\n## scenario: gone\n\nbody of gone\n",
            all_decided_except(&["performance.capacity"])
        ),
    );
    write(
        &dir,
        "tests/gone_test.rs",
        &format!("/// proves: gone@{rev}\n#[test]\nfn holds_gone() {{}}\n"),
    );
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "t: the work"]);
    let (said, _code) = keel(&dir, &["check"]);
    // (The sandbox has other findings of its own -- a scope this
    // fixture does not declare, a birth it never committed -- so the
    // exit code says nothing here; the tag rows do.)
    assert!(
        !said.contains("no wave knows"),
        "a called-off wave is outside judgement whole (§6.3-a), so its \
         test's tag is no orphan of this court -- an orphan is a tag no \
         wave knows, and this wave knows it:\n{said}"
    );
    assert!(
        said.contains("not checked") && said.contains("holds_gone"),
        "it is said aloud among what was not judged, by the test's own \
         name -- never painted green:\n{said}"
    );
    assert!(
        said.contains("0010-off"),
        "and the wave called off is named as what was not judged \
         (§6.3-a):\n{said}"
    );

    // -- pytest that leaves red with nothing red in it ---------------
    if !common::machine_has("pytest").ready() {
        return;
    }
    let dir = keel_sandbox("oneverdictpytest");
    write(&dir, "keel.toml", "lang = \"en\"\nadapter = \"python\"\n");
    write(&dir, "src/toy.py", "def works():\n    return True\n");
    write(
        &dir,
        "pyproject.toml",
        "[project]\nname = \"toy\"\nversion = \"0.1.0\"\n\n[tool.pytest.ini_options]\npythonpath = [\"src\"]\ntestpaths = [\"tests\"]\n",
    );
    // A session hook that fails after every test passed: pytest
    // leaves with 1 and its report says `1 passed`.
    write(
        &dir,
        "tests/conftest.py",
        "def pytest_sessionfinish(session, exitstatus):\n    session.exitstatus = 1\n",
    );
    let rev = keel::rev::text_rev(BODY);
    write(
        &dir,
        "tests/test_toy.py",
        &format!(
            "from toy import works\n\n# proves: s@{rev}\ndef test_holds_s():\n    assert works()\n"
        ),
    );
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [src/toy.py]\n{}---\n\n## scenario: s\n\n{BODY}\n## transform: t\n\nthe work\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    write(&dir, "keel/reviews/0009-w.md", "# Review\n\nok\n");
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "the trunk"]);
    git(&dir, &["checkout", "-q", "-b", "0009-w"]);
    git(
        &dir,
        &["commit", "-q", "--allow-empty", "-m", "t: the work"],
    );
    let (said, code) = keel(&dir, &["close"]);
    assert_ne!(
        code, 0,
        "a runner that left red with nothing red in it is not a green \
         battery -- the same word cargo's hand gives:\n{said}"
    );
    assert!(
        !said.contains("0009-w: closed"),
        "and the wave does not close over it:\n{said}"
    );
}
