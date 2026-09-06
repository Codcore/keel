//! Scenario test of wave 0051: what is not code is not read.
//!
//! The global review of 2026-09-06 (bugs cut R-15, R-23, R-26) measured
//! the readers reading text as code: a `// proves:` line inside a raw
//! string became a tag over the next `fn`; a `## scenario:` inside a
//! fenced block of a wave's body became an orphan section; a tag with
//! uppercase hex was read and then called stale against its own
//! lowercase; a BOM before a first-line tag made the tag invisible --
//! and silent. Each case is judged here beside its clean form.
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

fn check(dir: &Path) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
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

/// A rust crate under the methodology, on main, with the wave file as
/// given (or the plain one) and the test file as given.
fn crate_with(name: &str, wave_tail: &str, test_text: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness" {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    std::fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{d}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n{wave_tail}"
        ),
    )
    .unwrap();
    std::fs::write(dir.join("tests/w_test.rs"), test_text).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: what-is-not-code-is-not-read@864706
#[test]
fn what_is_not_code_is_not_read() {
    let rev = keel::rev::text_rev(BODY);

    // --- a tag-shaped line inside a raw string is text ---
    let raw = format!(
        "const FIXTURE: &str = r#\"\n// proves: it-works@abcdef\nfn not_a_test() {{}}\n\"#;\n\n/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(toy::works() && !FIXTURE.is_empty());\n}}\n"
    );
    let tags = keel::tags::scan_text(Path::new("tests/w_test.rs"), &raw).unwrap();
    assert_eq!(
        tags.iter().map(|t| t.test.as_str()).collect::<Vec<_>>(),
        vec!["it_works"],
        "the tag inside the raw string is not a tag (bugs R-15)"
    );
    let dir = crate_with("notcoderaw", "", &raw);
    let (said, code) = check(&dir);
    assert_eq!(code, 0, "and the check is green over it:\n{said}");
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "one tag read, the real one:\n{said}"
    );

    // --- `## scenario:` inside a fenced block of the body is text ---
    let dir = crate_with(
        "notcodefence",
        "\nПриклад файлу хвилі:\n\n```markdown\n## scenario: example-in-a-fence\n\nце текст прикладу, не секція\n```\n",
        &format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{}}\n"),
    );
    let (said, code) = check(&dir);
    assert!(
        !said.contains("секція-сирота"),
        "a section inside a fence is not an orphan (bugs R-23):\n{said}"
    );
    assert_eq!(code, 0, "and the check is green:\n{said}");

    // --- hex is a number: uppercase compares equal ---
    let upper = rev.to_uppercase();
    let dir = crate_with(
        "notcodehex",
        "",
        &format!("/// proves: it-works@{upper}\n#[test]\nfn it_works() {{}}\n"),
    );
    let (said, code) = check(&dir);
    assert!(
        !said.contains("тримає it-works@"),
        "an uppercase revision is the same revision (bugs R-26):\n{said}"
    );
    assert_eq!(code, 0, "and the check is green:\n{said}");
    assert!(keel::rev::matches(&upper, &rev), "and `matches` says so");

    // --- a BOM before a first-line tag hides nothing ---
    let dir = crate_with(
        "notcodebom",
        "",
        &format!("\u{feff}/// proves: it-works@{rev}\n#[test]\nfn it_works() {{}}\n"),
    );
    let (said, code) = check(&dir);
    assert!(
        said.contains("тегів тестів звірено: 1"),
        "the tag behind a BOM is read (bugs R-26):\n{said}"
    );
    assert_eq!(code, 0, "and the check is green:\n{said}");
}
