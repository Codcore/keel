//! Scenario tests of waves 0001 and 0002 that promise command
//! behaviour: we run the real binary -- the report and the exit code,
//! not a function.
//!
//! proves tags -- revisions per §5.3-§5.4, computed by hand (bootstrap).

mod common;

use common::{keel_sandbox, sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// Waves in these sandboxes must themselves obey §10.3 now that the
/// graph floor judges silence: a full decisions block, one line per
/// cut, minus any cuts the fixture covers itself.
fn all_decided_except(covered: &[&str]) -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            block.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    block
}

fn all_decided() -> String {
    all_decided_except(&[])
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
    let dir = keel_sandbox("report");
    write(
        &dir,
        "keel/waves/0002-ok.md",
        &format!(
            "---\ntransforms:\n  tidy: {{chore: \"лад у документах\", files: [README.md]}}\n{}---\n\n## transform: tidy\n\nдокументи в лад\n",
            all_decided()
        ),
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
    // Fixture kept up with the ladder (0011): every floor of chapter
    // 7 stands, the unchecked line is gone -- §7.8's border speaks
    // in its place. The scenario's words hold: the report still
    // names its own limits aloud.
    assert!(
        out.contains("green form is not yet meaning") || out.contains("зелена форма — ще не сенс"),
        "the §7.8 border named aloud:\n{out}"
    );
    assert!(
        out.contains("test tags (§5.5"),
        "test tags among the checked since the tag floor:\n{out}"
    );
    assert!(
        out.contains("links (chapter 3"),
        "links among the checked since the graph floor:\n{out}"
    );
    assert!(
        out.contains("held (§7.6") || out.contains("форми контрактів (§7.6"),
        "contract holding among the checked now:\n{out}"
    );
    assert!(
        !out.contains("not yet checked"),
        "the unchecked line has left the report (0011):\n{out}"
    );

    // Without the broken file -- exit 0; honesty about the unchecked stays.
    fs::remove_file(dir.join("keel/contracts/broken.md")).unwrap();
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "report:\n{out}");
    assert!(
        out.contains("green form is not yet meaning"),
        "green does not hide its own limits (§7.8):\n{out}"
    );
}

/// proves: missing-keel-dir-refuses@01149b -- holds §9.7: a refusal
/// carries its reason and what to do instead.
#[test]
fn missing_keel_dir_refuses() {
    // A bare directory with no keel/ in it -- through the one hand,
    // so it does not outlive the test either (wave 0030).
    let dir = sandbox("nokeel");

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
    let dir = keel_sandbox("nocfg");
    write(
        &dir,
        "keel/waves/0003-w.md",
        &format!(
            "---\ntransforms:\n  t: {{chore: \"tidy\", files: [a]}}\n{}---\n\n## transform: t\n\ntidy work\n",
            all_decided()
        ),
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
    let dir = keel_sandbox("nolang-field");
    write(&dir, "keel.toml", "# no lang here\n");
    write(
        &dir,
        "keel/waves/0003-w.md",
        &format!(
            "---\ntransforms:\n  t: {{chore: \"tidy\", files: [a]}}\n{}---\n\n## transform: t\n\ntidy work\n",
            all_decided()
        ),
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
    let dir = keel_sandbox("lang-en");
    write(&dir, "keel.toml", "lang = \"en\"\n");
    write(
        &dir,
        "keel/waves/0004-w.md",
        &format!(
            "---\ntransforms:\n  t: {{chore: \"tidy\", files: [a]}}\n{}---\n\n## transform: t\n\ntidy work\n",
            all_decided()
        ),
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
    let dir = keel_sandbox("lang-uk");
    write(&dir, "keel.toml", "lang = \"uk\"\n");
    write(
        &dir,
        "keel/waves/0004-w.md",
        &format!(
            "---\ntransforms:\n  t: {{chore: \"tidy\", files: [a]}}\n{}---\n\n## transform: t\n\ntidy work\n",
            all_decided()
        ),
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
        let dir = keel_sandbox(&format!("plural-{n}"));
        write(&dir, "keel.toml", "lang = \"uk\"\n");
        for i in 1..=n {
            write(
                &dir,
                &format!("keel/waves/000{i}-w.md"),
                &format!(
                    "---\ntransforms:\n  t: {{chore: \"tidy\", files: [a]}}\n{}---\n\n## transform: t\n\ntidy work\n",
                    all_decided()
                ),
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
    let dir = keel_sandbox("refs");
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
            "---\nscenarios:\n  s: {{proves: anchor@{good}, covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\n{}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
            all_decided_except(&["functional.correctness"])
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
    // Since the tag floor the fixture carries real git history --
    // where history exists and never held beef00, the strict verdict
    // stands (a no-git directory gets a word instead, held by
    // old_revision_legal_when_historic).
    write(
        &dir,
        "keel/waves/0006-stale.md",
        &format!(
            "---\nscenarios:\n  s: {{proves: anchor@beef00, covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\n{}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir.as_ref() as &std::path::Path)
            .args([
                "-c",
                "user.email=keel@test",
                "-c",
                "user.name=keel-test",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "history that never held beef00"]);
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a stale revision is a finding:\n{out}");
    assert!(
        out.contains("beef00"),
        "names the recorded revision:\n{out}"
    );
    assert!(out.contains(&good), "names the current revision:\n{out}");
    assert!(
        out.contains("§5.6"),
        "says the closed-wave caveat aloud:\n{out}"
    );
    assert!(
        out.contains("checked: 1"),
        "the report counts verified references (Z-8):\n{out}"
    );

    // A withdrawn scenario is outside judgement (§2.12): its stale
    // proves must NOT be a finding (review finding Z-1).
    fs::remove_file(dir.join("keel/waves/0006-stale.md")).unwrap();
    write(
        &dir,
        "keel/waves/0006-gone.md",
        &format!(
            "---\nscenarios:\n  s:\n    proves: anchor@beef00\n    withdrawn: \"знято\"\ntransforms:\n  t: {{chore: \"tidy\", files: [lib/a.ex]}}\n{}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
            all_decided()
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a withdrawn scenario's stale proves is not judged (§2.12):\n{out}"
    );
}

/// proves: missing-contract-named@73cf9e -- holds §7.1: a reference
/// into nowhere is a finding with names, not silence and not a crash.
#[test]
fn missing_contract_named() {
    let dir = keel_sandbox("ghost-ref");
    write(
        &dir,
        "keel/waves/0007-w.md",
        &format!(
            "---\nscenarios:\n  s: {{proves: ghost@abcd12, covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\n{}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a dangling reference is a finding:\n{out}");
    assert!(out.contains("ghost"), "names the missing contract:\n{out}");
    assert!(
        out.contains("0007-w"),
        "names the wave that points at it:\n{out}"
    );
}

/// proves: rev-command-prints@801d36 -- holds §5.1/§5.5: authors copy
/// current revisions from the tool instead of computing by hand.
#[test]
fn rev_command_prints() {
    let dir = keel_sandbox("rev-cmd");
    write(&dir, "keel.toml", "lang = \"uk\"\n");
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: Anchor\nexports: [\"run()\"]\n---\n\nprose\n",
    );
    write(
        &dir,
        "keel/waves/0008-w.md",
        &format!(
            "---\nscenarios:\n  s: {{proves: anchor@abcd, covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\n{}---\n\n## scenario: s\n\nbody of s\n\n## transform: t\n\nwork\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let anchor = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();
    let scenario = keel::rev::text_rev("body of s\n");

    let (out, _err, code) = keel(&["rev", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains(&format!("anchor@{anchor}")),
        "prints the contract revision:\n{out}"
    );
    assert!(
        out.contains(&format!("0008-w/s@{scenario}")),
        "prints the scenario revision as wave/scenario@rev:\n{out}"
    );
    assert!(
        out.contains("наступний крок"),
        "prints the next step in uk:\n{out}"
    );
    assert!(
        out.contains("конфіг: keel.toml (lang = uk)"),
        "rev names its config source like check does (Z-3):\n{out}"
    );

    // A broken document stands next to the revisions as a refusal.
    write(&dir, "keel/contracts/broken.md", "---\nmodule: X\n");
    let (out, _err, code) = keel(&["rev", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a refusal makes the exit non-zero:\n{out}");
    assert!(
        out.contains("не закрита"),
        "the refusal stands in the output:\n{out}"
    );
    assert!(
        out.contains(&format!("anchor@{anchor}")),
        "intact revisions still printed:\n{out}"
    );

    // Without keel.toml the defaults are said aloud here too (Z-3).
    let dir = keel_sandbox("rev-nocfg");
    write(
        &dir,
        "keel/waves/0009-w.md",
        &format!(
            "---\ntransforms:\n  t: {{chore: \"tidy\", files: [a]}}\n{}---\n\n## transform: t\n\ntidy work\n",
            all_decided()
        ),
    );
    let (out, _err, code) = keel(&["rev", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("no keel.toml"),
        "the defaults do not pass themselves off as read, in rev too:\n{out}"
    );
}

/// proves: unknown-cut-refused@be4afe -- holds §3.4: a slug outside
/// the embedded vocabulary is a finding, not a new answer.
#[test]
fn unknown_cut_refused() {
    let dir = keel_sandbox("badcut");
    write(
        &dir,
        "keel/waves/0005-w.md",
        "---\nscenarios:\n  s: {covers: [functional.correctnes]}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\n---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "an alien slug is a finding:\n{out}");
    assert!(
        out.contains("functional.correctnes"),
        "names the alien slug:\n{out}"
    );
    // The scenario says "covers or decisions" and "names the slug and
    // the wave" -- both halves are held (review R-9).
    assert!(
        out.contains("keel/waves/0005-w.md"),
        "names the wave the alien slug sits in:\n{out}"
    );
    let dir = keel_sandbox("baddecision");
    write(
        &dir,
        "keel/waves/0005-w.md",
        "---\nscenarios:\n  s: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\ndecisions:\n  reliability.faultlessnes: \"typo\"\n---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "an alien slug in decisions is a finding too:\n{out}"
    );
    assert!(
        out.contains("reliability.faultlessnes") && out.contains("decisions"),
        "names the alien decisions slug and its holder:\n{out}"
    );
}

/// proves: silence-forbidden@9bd959 -- holds §10.3: every cut gets
/// exactly one answer; withdrawn covers do not count (§2.12).
#[test]
fn silence_forbidden() {
    // All forty answered except two -- both missing cuts are listed
    // in the one finding (a list, not a sample; review R-9).
    let dir = keel_sandbox("silence");
    let mut decisions = String::new();
    for cut in keel::graph::cuts() {
        if *cut != "functional.correctness"
            && *cut != "performance.capacity"
            && *cut != "safety.fail-safe"
        {
            decisions.push_str(&format!("  {cut}: \"n/a\"\n"));
        }
    }
    write(
        &dir,
        "keel/waves/0005-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\ndecisions:\n{decisions}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n"
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "silence is a finding:\n{out}");
    assert!(
        out.contains("performance.capacity") && out.contains("safety.fail-safe"),
        "lists every missing cut:\n{out}"
    );

    // A withdrawn scenario's covers do not close a cut: the promise
    // died (§2.12), so the cut is silent again.
    let dir = keel_sandbox("dead-cover");
    write(
        &dir,
        "keel/waves/0005-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\n  gone:\n    covers: [performance.capacity]\n    withdrawn: \"знято\"\ntransforms:\n  t:\n    implements: [s]\n    files: [lib/a.ex]\ndecisions:\n{decisions}---\n\n## scenario: s\n\nbody\n\n## scenario: gone\n\nold body\n\n## transform: t\n\nwork\n"
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a dead cover is not an answer:\n{out}");
    assert!(
        out.contains("performance.capacity"),
        "the cut held only by a withdrawn cover is listed missing:\n{out}"
    );
}

/// proves: broken-links-named@4b2a4e -- holds §7.1/§7.2: links into
/// nowhere and dependency cycles are findings with names.
#[test]
fn broken_links_named() {
    let dir = keel_sandbox("links");
    write(
        &dir,
        "keel/waves/0005-a.md",
        "---\ndepends_on: [0006-b, 0099-ghost-wave]\nscenarios:\n  s:\n    covers: [functional.correctness]\n    superseded_by: nobody-anywhere\n  selfy:\n    withdrawn: \"folded into s\"\n    superseded_by: selfy\ntransforms:\n  t:\n    implements: [s, ghost-scenario]\n    files: [lib/a.ex]\n---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
    );
    write(
        &dir,
        "keel/waves/0006-b.md",
        "---\ndepends_on: [0005-a]\ntransforms:\n  t: {chore: \"tidy\", files: [lib/b.ex]}\n---\n\n## transform: t\n\nwork\n",
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "broken links are findings:\n{out}");
    assert!(
        out.contains("ghost-scenario"),
        "implements into nowhere named:\n{out}"
    );
    assert!(
        out.contains("0099-ghost-wave"),
        "depends_on into nowhere named:\n{out}"
    );
    assert!(
        out.contains("nobody-anywhere"),
        "a successor unknown to any wave named:\n{out}"
    );
    // Second birth (review R-5): a scenario naming itself as its own
    // successor passes as "found somewhere" today -- §2.12 wants a
    // successor that took over, and nothing takes over from itself.
    assert!(
        out.contains("names itself"),
        "a self-successor named as a finding:\n{out}"
    );
    assert!(
        out.contains("0005-a") && out.contains("0006-b"),
        "the dependency cycle names its waves:\n{out}"
    );
}

/// proves: old-revision-legal-when-historic@dfd598 -- holds §5.6: an
/// old contract revision is legal when its text truly lived in the
/// file's git history; a revision history never held stays a
/// finding; a truncated (shallow) history gets a word instead of a
/// verdict.
#[test]
fn old_revision_legal_when_historic() {
    let git = |dir: &Path, args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.email=keel@test",
                "-c",
                "user.name=keel-test",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };

    // The contract's old text is committed, a wave records it, then
    // the contract moves on -- the old revision is true in history.
    // Since the 0006 narrowing the blessing belongs to closed waves,
    // so the fixture is one: a matching tag, a review next to the
    // wave, the cargo adapter on (fixture adaptation, the scenario's
    // own words unchanged).
    let dir = keel_sandbox("historic");
    git(&dir, &["init", "-q", "-b", "main"]);
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(
        &dir,
        "tests/s_test.rs",
        &format!(
            "/// proves: s@{}\n#[test]\nfn holds_s() {{}}\n",
            keel::rev::text_rev("body\n")
        ),
    );
    write(&dir, "keel/reviews/0007-w.md", "# Рецензія\n\nok\n");
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: A\nexports: [\"one()\"]\n---\n\nold words\n",
    );
    let old_rev = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "old contract"]);
    write(
        &dir,
        "keel/waves/0007-w.md",
        &format!(
            "---\nscenarios:\n  s:\n    proves: anchor@{old_rev}\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: A\nexports: [\"one()\", \"two()\"]\n---\n\nnew words\n",
    );
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "wave and newer contract"]);

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "an old revision true in history is legal (§5.6):\n{out}"
    );
    assert!(
        out.contains("true in the file's history"),
        "the legality said with its word:\n{out}"
    );

    // A revision history never held stays a finding.
    let (out, _err, code) = {
        write(
            &dir,
            "keel/waves/0008-w.md",
            &format!(
                "---\nscenarios:\n  s:\n    proves: anchor@bbbbbb\n    covers: [performance.capacity]\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
                all_decided_except(&["performance.capacity"])
            ),
        );
        keel(&["check", dir.to_str().unwrap()])
    };
    assert_eq!(code, 1, "a fabricated revision is still a finding:\n{out}");
    assert!(
        out.contains("anchor@bbbbbb"),
        "the fabricated reference named:\n{out}"
    );

    // A shallow history gives a word, not a verdict.
    fs::remove_file(dir.join("keel/waves/0008-w.md")).unwrap();
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: A\nexports: [\"one()\", \"two()\", \"three()\"]\n---\n\nnewest words\n",
    );
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "newest contract"]);
    write(
        &dir,
        ".git/shallow",
        "0000000000000000000000000000000000000000\n",
    );

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "truncated history is not the wave's fault:\n{out}");
    assert!(
        out.contains("history is truncated"),
        "the shallow clone named with a word, no verdict:\n{out}"
    );

    // Second birth (review R-2): the scenario says "where there is no
    // history OR it is truncated -- no verdict". A keel directory
    // without git must get the word too, not a strict finding.
    let dir = keel_sandbox("nogit-history");
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: A\nexports: [\"one()\"]\n---\n\nnew words\n",
    );
    write(
        &dir,
        "keel/waves/0007-w.md",
        &format!(
            "---\nscenarios:\n  s:\n    proves: anchor@badc0f\n    covers: [functional.correctness]\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody\n\n## transform: t\n\nwork\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "no git history is not the wave's fault either:\n{out}"
    );
    assert!(
        out.contains("no git history"),
        "the absence of history named with a word, no verdict:\n{out}"
    );
}

/// proves: open-wave-stale-is-red-again@b8292f -- holds the §5.6
/// narrowing (review 0005, R-9): the history blessing belongs to
/// structurally closed waves only -- an open wave with a stale
/// reference is a finding again (§5.1: update deliberately) -- and
/// historic references are named by name, not only counted.
#[test]
fn open_wave_stale_is_red_again() {
    let git = |dir: &Path, args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args([
                "-c",
                "user.email=keel@test",
                "-c",
                "user.name=keel-test",
                "-c",
                "commit.gpgsign=false",
            ])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
    };
    let dir = keel_sandbox("narrowing");
    write(&dir, "keel.toml", "adapter = \"cargo\"\n");
    write(
        &dir,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    fs::create_dir_all(dir.join("keel/reviews")).unwrap();
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: A\nexports: [\"one()\"]\n---\n\nold words\n",
    );
    let old_rev = keel::rev::contract_rev(&dir.join("keel/contracts/anchor.md")).unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "."]);
    git(&dir, &["commit", "-q", "-m", "old contract in history"]);

    // A structurally CLOSED wave holding the old revision: its live
    // scenario has a matching tag and its review lies next to it.
    write(
        &dir,
        "keel/waves/0016-done.md",
        &format!(
            "---\nscenarios:\n  t:\n    proves: anchor@{old_rev}\n    covers: [functional.correctness]\ntransforms:\n  w:\n    implements: [t]\n    files: [src/lib.rs]\n{}---\n\n## scenario: t\n\nbody of t\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let t_rev = keel::rev::text_rev("body of t\n");
    write(
        &dir,
        "tests/t_test.rs",
        &format!("/// proves: t@{t_rev}\n#[test]\nfn holds_t() {{}}\n"),
    );
    write(&dir, "keel/reviews/0016-done.md", "# Рецензія\n\nok\n");
    // An OPEN wave holding the same old revision: its scenario has
    // no tag anywhere.
    write(
        &dir,
        "keel/waves/0017-open.md",
        &format!(
            "---\nscenarios:\n  u:\n    proves: anchor@{old_rev}\n    covers: [performance.capacity]\ntransforms:\n  w:\n    implements: [u]\n    files: [src/lib.rs]\n{}---\n\n## scenario: u\n\nbody of u\n",
            all_decided_except(&["performance.capacity"])
        ),
    );
    // The contract moves on -- both references grow stale.
    write(
        &dir,
        "keel/contracts/anchor.md",
        "---\nmodule: A\nexports: [\"one()\", \"two()\"]\n---\n\nnew words\n",
    );
    git(&dir, &["add", "."]);
    git(
        &dir,
        &["commit", "-q", "-m", "waves and the newer contract"],
    );

    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 1,
        "an open wave's stale reference is a finding again:\n{out}"
    );
    assert!(
        out.contains("0017-open"),
        "the open wave named in the finding:\n{out}"
    );
    assert!(
        out.contains("0016-done") && out.contains(&format!("anchor@{old_rev}")),
        "the closed wave's historic reference named by name (R-9):\n{out}"
    );
    assert!(
        !out.contains("0016-done: записано") && !out.contains("0016-done.md — хвиля"),
        "the closed wave carries no finding:\n{out}"
    );
}

/// proves: double-answer-found@e47a77 -- holds §10.3 "exactly one":
/// a cut closed by two live covers, or closed by a live cover and
/// decided at once, is a finding naming the cut and both holders; a
/// dead cover next to a decision is the lawful §2.12 pair.
#[test]
fn double_answer_found() {
    // Two live covers of one cut.
    let dir = keel_sandbox("doublecover");
    write(
        &dir,
        "keel/waves/0030-w.md",
        &format!(
            "---\nscenarios:\n  p: {{covers: [functional.correctness]}}\n  q: {{covers: [functional.correctness]}}\ntransforms:\n  t:\n    implements: [p, q]\n    files: [src/lib.rs]\n{}---\n\n## scenario: p\n\nbody of p\n\n## scenario: q\n\nbody of q\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "two live covers are a finding (§10.3):\n{out}");
    assert!(
        out.contains("functional.correctness") && out.contains("\"p\"") && out.contains("\"q\""),
        "the cut and both holders named:\n{out}"
    );
    assert!(
        out.contains("2 live covers"),
        "the finding counts the holders (review R-7):\n{out}"
    );

    // A live cover and a decision of the same cut at once.
    let dir = keel_sandbox("coveranddecided");
    write(
        &dir,
        "keel/waves/0031-w.md",
        &format!(
            "---\nscenarios:\n  r: {{covers: [performance.capacity]}}\ntransforms:\n  t:\n    implements: [r]\n    files: [src/lib.rs]\n{}---\n\n## scenario: r\n\nbody of r\n",
            all_decided()
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 1, "a cover next to a decision is a finding:\n{out}");
    assert!(
        out.contains("performance.capacity") && out.contains("\"r\"") && out.contains("decided"),
        "the cut, the scenario and the decision named:\n{out}"
    );

    // The lawful pair: a dead cover and the decision that remains.
    let dir = keel_sandbox("deadpair");
    write(
        &dir,
        "keel/waves/0032-w.md",
        &format!(
            "---\nscenarios:\n  s: {{covers: [functional.correctness]}}\n  gone:\n    covers: [security.integrity]\n    withdrawn: \"folded\"\ntransforms:\n  t:\n    implements: [s]\n    files: [src/lib.rs]\n{}---\n\n## scenario: s\n\nbody of s\n\n## scenario: gone\n\nold body\n\n## transform: t\n\nwork\n",
            all_decided_except(&["functional.correctness"])
        ),
    );
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(
        code, 0,
        "a dead cover next to a decision is the lawful §2.12 pair:\n{out}"
    );
}
