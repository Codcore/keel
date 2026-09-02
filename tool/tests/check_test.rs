//! Scenario tests of waves 0001 and 0002 that promise command
//! behaviour: we run the real binary -- the report and the exit code,
//! not a function.
//!
//! proves tags -- revisions per §5.3-§5.4, computed by hand (bootstrap).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0001c-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) {
    fs::write(dir.join(rel), text).unwrap();
}

fn keel(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// proves: check-reports-every-file@327c12 -- holds §7.9 and lesson
/// 4: the broken is named, neighbours are checked, the unchecked is
/// named aloud.
#[test]
fn check_reports_every_file() {
    let dir = sandbox("report");
    write(
        &dir,
        "keel/waves/0002-ok.md",
        "---\ntransforms:\n  tidy: {chore: \"лад у документах\", files: [README.md]}\n---\n",
    );
    write(
        &dir,
        "keel/contracts/session-run.md",
        "---\nmodule: Session\nexports: [\"run()\"]\n---\n",
    );
    write(&dir, "keel/contracts/broken.md", "---\nmodule: X\n");

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "a refusal in a document is a finding, exit 1; report:\n{out}"
    );
    for f in [
        "keel/waves/0002-ok.md",
        "keel/contracts/session-run.md",
        "keel/contracts/broken.md",
    ] {
        assert!(out.contains(f), "the report must name {f}:\n{out}");
    }
    assert!(
        out.contains("not closed"),
        "the broken one named with a reason:\n{out}"
    );
    assert!(
        out.contains("checked by this floor"),
        "the report names its own limits:\n{out}"
    );
    assert!(
        out.contains("not yet checked"),
        "the unchecked named aloud:\n{out}"
    );
    assert!(
        out.contains("revisions (§5)"),
        "revisions among the unchecked:\n{out}"
    );
    assert!(
        out.contains("links (chapter 3"),
        "links among the unchecked:\n{out}"
    );
    assert!(
        out.contains("contracts (§7.6)"),
        "contracts among the unchecked:\n{out}"
    );

    // Without the broken file -- exit 0; honesty about the unchecked stays.
    fs::remove_file(dir.join("keel/contracts/broken.md")).unwrap();
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "report:\n{out}");
    assert!(
        out.contains("not yet checked"),
        "green does not hide its own limits:\n{out}"
    );
}

/// proves: missing-keel-dir-refuses@01149b -- holds §9.7: a refusal
/// carries its reason and what to do instead.
#[test]
fn missing_keel_dir_refuses() {
    let dir = std::env::temp_dir().join(format!("keel-0001c-{}-nokeel", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let (_out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 2,
        "a refusal of the command itself -- exit 2; stderr:\n{err}"
    );
    assert!(err.contains("keel/"), "names what is missing:\n{err}");
    assert!(err.contains("create"), "says what to do instead:\n{err}");
}

/// proves: missing-config-defaults@cdc248 -- holds contract
/// tool-config: a default does not pass itself off as read, neither
/// for a missing file nor for a missing field.
#[test]
fn missing_config_defaults() {
    let dir = sandbox("nocfg");
    write(
        &dir,
        "keel/waves/0003-w.md",
        "---\ntransforms:\n  t: {chore: \"tidy\", files: [a]}\n---\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("no keel.toml"), "defaults said aloud:\n{out}");
    assert!(
        out.contains("defaults"),
        "named as defaults, not as read:\n{out}"
    );

    // The file exists, the lang field does not: still said aloud,
    // never printed as if lang had been read (review finding Z-1).
    let dir = sandbox("nolang-field");
    write(&dir, "keel.toml", "# no lang here\n");
    write(
        &dir,
        "keel/waves/0003-w.md",
        "---\ntransforms:\n  t: {chore: \"tidy\", files: [a]}\n---\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("lang not set"),
        "a defaulted field is named, not passed off as read:\n{out}"
    );
    assert!(
        !out.contains("lang = en)"),
        "must not print the very line an explicit lang = en gets:\n{out}"
    );
}

/// proves: output-follows-lang@fc3a8f -- holds contract tool-config:
/// the report and the refusals follow lang.
#[test]
fn output_follows_lang() {
    // lang = "en" -- an English report.
    let dir = sandbox("lang-en");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/waves/0004-w.md",
        "---\ntransforms:\n  t: {chore: \"tidy\", files: [a]}\n---\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("header reads"),
        "an English report for lang=en:\n{out}"
    );
    assert!(
        out.contains("lang = en"),
        "the config named in the report:\n{out}"
    );

    // And a refusal follows lang too, not only the green lines (Z-4b).
    write(&dir, "keel/contracts/broken.md", "---\nmodule: X\n");
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("not closed"), "the refusal in English:\n{out}");

    // lang = "uk" -- a Ukrainian report, and the refusal in Ukrainian too.
    let dir = sandbox("lang-uk");
    write(&dir, "keel.toml", "lang = \"uk\"\n");
    write(
        &dir,
        "keel/waves/0004-w.md",
        "---\ntransforms:\n  t: {chore: \"tidy\", files: [a]}\n---\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("шапка читається"),
        "a Ukrainian report for lang=uk:\n{out}"
    );

    write(&dir, "keel/contracts/broken.md", "---\nmodule: X\n");
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("не закрита"),
        "the refusal in the project language:\n{out}"
    );
}

/// proves: plural-forms-correct@b2c52a -- holds the concept's
/// "Output languages": plurals by CLDR rules, not ifs.
#[test]
fn plural_forms_correct() {
    for (n, expect) in [
        (1usize, "1 документ,"),
        (2, "2 документи,"),
        (5, "5 документів,"),
    ] {
        let dir = sandbox(&format!("plural-{n}"));
        write(&dir, "keel.toml", "lang = \"uk\"\n");
        for i in 1..=n {
            write(
                &dir,
                &format!("keel/waves/000{i}-w.md"),
                "---\ntransforms:\n  t: {chore: \"tidy\", files: [a]}\n---\n",
            );
        }
        let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
        assert_eq!(code, 0, "{out}");
        assert!(out.contains(expect), "the plural for {n}:\n{out}");
    }
}

/// proves: contract-refs-verified@07e476 -- holds §5.1/§7.3: a
/// recorded revision is checked against the current text; divergence
/// names both revisions and the §5.6 caveat.
#[test]
fn contract_refs_verified() {
    let dir = sandbox("refs");
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: Anchor\nexports: [\"run()\"]\n---\n\nprose\n",
    );
    let good = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();

    // A matching reference is fine and the checked line now claims §7.3.
    write(
        &dir,
        "keel/waves/0005-good.md",
        &format!(
            "---\nscenarios:\n  s: {{proves: anchor@{good}}}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\n---\n\n## scenario: s\n\nbody\n"
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("§7.3"), "the checked line claims §7.3:\n{out}");
    assert!(
        !out.contains("contract revisions"),
        "contract revisions must not sit in the unchecked line anymore:\n{out}"
    );

    // A stale reference is a finding naming both revisions and §5.6.
    write(
        &dir,
        "keel/waves/0006-stale.md",
        "---\nscenarios:\n  s: {proves: anchor@beef00}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\n---\n\n## scenario: s\n\nbody\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a stale revision is a finding:\n{out}");
    assert!(out.contains("beef00"), "names the recorded revision:\n{out}");
    assert!(out.contains(&good), "names the current revision:\n{out}");
    assert!(out.contains("§5.6"), "says the closed-wave caveat aloud:\n{out}");
}
