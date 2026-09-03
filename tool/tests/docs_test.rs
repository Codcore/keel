//! Scenario tests of wave 0001-strict-headers, transform read-headers.
//!
//! Every test carries a `proves: <scenario>@<revision>` tag -- the
//! revision per §5.3-§5.4: the scenario section body, repeated spaces
//! and newlines collapsed into one space, sha256, first six hex
//! characters. For now hands compute the revision (bootstrap); rung 2
//! (`keel rev`) is bound to reproduce this recipe.

mod common;

#[allow(unused_imports)]
use common::{Sandbox, keel_sandbox};

use keel::docs;
use std::fs;
use std::path::{Path, PathBuf};

fn write(dir: &Path, rel: &str, text: &str) -> PathBuf {
    let p = dir.join(rel);
    fs::write(&p, text).unwrap();
    p
}

/// proves: broken-header-refuses@240948 -- holds §7.9.
#[test]
fn broken_header_refuses() {
    let dir = keel_sandbox("broken");

    // The header is not closed: no second `---`.
    let p = write(&dir, "keel/waves/0002-x.md", "---\nscenarios:\n");
    let r = docs::read_wave(&p).unwrap_err();
    assert_eq!(r.file, p);
    assert!(r.reason.contains("not closed"), "reason: {}", r.reason);
    assert!(
        !r.instead.is_empty(),
        "a refusal must say what to do instead"
    );

    // No header at all.
    let p = write(&dir, "keel/waves/0003-y.md", "# просто текст, без шапки\n");
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("no header"), "reason: {}", r.reason);
    assert!(!r.instead.is_empty());

    // Broken YAML inside the header.
    let p = write(&dir, "keel/contracts/c.md", "---\nmodule: [unclosed\n---\n");
    let r = docs::read_contract(&p).unwrap_err();
    assert!(r.reason.contains("YAML"), "reason: {}", r.reason);
    assert!(!r.instead.is_empty());
}

/// proves: unknown-field-refuses@4fa15d -- holds §7.9.
#[test]
fn unknown_field_refuses() {
    let dir = keel_sandbox("unknown");

    // A typo in a wave field: scenarois instead of scenarios.
    let p = write(
        &dir,
        "keel/waves/0004-typo.md",
        "---\nscenarois:\n  a: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [a]\n    files: [lib/a.ex]\n---\n",
    );
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("unknown field"), "reason: {}", r.reason);
    assert!(
        r.reason.contains("scenarois"),
        "names the field itself: {}",
        r.reason
    );
    assert!(!r.instead.is_empty());

    // An unknown field inside a scenario.
    let p = write(
        &dir,
        "keel/waves/0005-inner.md",
        "---\nscenarios:\n  a: {covvers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [a]\n    files: [lib/a.ex]\n---\n",
    );
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("covvers"), "reason: {}", r.reason);

    // An unknown contract field.
    let p = write(
        &dir,
        "keel/contracts/typo.md",
        "---\nmodule: X\nexporst:\n  - \"run()\"\n---\n",
    );
    let r = docs::read_contract(&p).unwrap_err();
    assert!(r.reason.contains("exporst"), "reason: {}", r.reason);
}

/// proves: valid-wave-parses@8b543c -- holds §2.3-§2.5, §2.11,
/// §2.12, §4.1, §4.12: the full wave vocabulary reads into data with
/// no loss.
#[test]
fn valid_wave_parses() {
    let dir = keel_sandbox("valid-wave");
    let p = write(
        &dir,
        "keel/waves/0006-full.md",
        concat!(
            "---\n",
            "depends_on: [0005-earlier]\n",
            "renamed_from: 0006-old-name\n",
            "scenarios:\n",
            "  alive:\n",
            "    proves: session-run@7c40de\n",
            "    covers: [functional.correctness, safety.fail-safe]\n",
            "  gone:\n",
            "    covers: [performance.capacity]\n",
            "    withdrawn: \"обіцянку зняла хвиля 0009\"\n",
            "    superseded_by: alive\n",
            "transforms:\n",
            "  work:\n",
            "    implements: [alive]\n",
            "    contracts: [session-run@7c40de]\n",
            "    files:\n",
            "      - lib/session.ex\n",
            "      - one new in priv/migrations/\n",
            "  tidy:\n",
            "    chore: \"оновлення залежности без обіцянки\"\n",
            "    files: [mix.lock]\n",
            "decisions:\n",
            "  performance.time-behaviour: \"не міряємо: разова команда\"\n",
            "---\n",
            "\n## Why\n\nтіло тут не читається щаблем 1\n",
        ),
    );
    let w = docs::read_wave(&p).unwrap();
    assert_eq!(w.slug, "0006-full");
    assert_eq!(w.depends_on, vec!["0005-earlier"]);
    assert_eq!(w.renamed_from.as_deref(), Some("0006-old-name"));

    let (name, alive) = &w.scenarios[0];
    assert_eq!(name, "alive");
    let pr = alive.proves.as_ref().unwrap();
    assert_eq!(
        (pr.slug.as_str(), pr.rev.as_str()),
        ("session-run", "7c40de")
    );
    assert_eq!(
        alive.covers,
        vec!["functional.correctness", "safety.fail-safe"]
    );

    let (_, gone) = &w.scenarios[1];
    assert_eq!(gone.withdrawn.as_deref(), Some("обіцянку зняла хвиля 0009"));
    assert_eq!(gone.superseded_by.as_deref(), Some("alive"));

    let (_, work) = &w.transforms[0];
    match &work.kind {
        docs::TransformKind::Implements(s) => assert_eq!(s, &vec!["alive"]),
        other => panic!("not it: {other:?}"),
    }
    assert_eq!(
        work.files,
        vec![
            docs::ScopeLine::Path("lib/session.ex".into()),
            docs::ScopeLine::OneNewIn("priv/migrations/".into()),
        ]
    );

    assert_eq!(
        work.contracts,
        vec![docs::ContractRef {
            slug: "session-run".into(),
            rev: "7c40de".into()
        }]
    );

    let (_, tidy) = &w.transforms[1];
    match &tidy.kind {
        docs::TransformKind::Chore(why) => assert_eq!(why, "оновлення залежности без обіцянки"),
        other => panic!("not it: {other:?}"),
    }

    assert_eq!(
        w.decisions,
        vec![(
            "performance.time-behaviour".to_string(),
            "не міряємо: разова команда".to_string()
        )]
    );

    // Legal absence is not an error: an all-chore wave without scenarios.
    let p = write(
        &dir,
        "keel/waves/0007-chore-only.md",
        "---\ntransforms:\n  tidy:\n    chore: \"форматування\"\n    files: [README.md]\n---\n",
    );
    let w = docs::read_wave(&p).unwrap();
    assert!(w.scenarios.is_empty());
    assert!(w.decisions.is_empty());
}

/// proves: valid-contract-parses@863c4e -- holds §2.7-§2.8: our
/// promise (module + exports) and a foreign one (verify) read into
/// data; a contract with neither refuses as "promises nothing".
#[test]
fn valid_contract_parses() {
    let dir = keel_sandbox("valid-contract");

    // Our contract: module + exports.
    let p = write(
        &dir,
        "keel/contracts/session-run.md",
        concat!(
            "---\n",
            "module: KeelAgent.Session\n",
            "exports:\n",
            "  - \"run(Context.t(), [Tool.t()]) :: Outcome.t()\"\n",
            "  - \"halt(pid()) :: :ok\"\n",
            "---\n\nОдна розмова з однією моделлю.\n",
        ),
    );
    let c = docs::read_contract(&p).unwrap();
    assert_eq!(c.slug, "session-run");
    assert_eq!(c.module.as_deref(), Some("KeelAgent.Session"));
    assert_eq!(c.exports.len(), 2);
    assert!(c.exports[0].starts_with("run("));
    assert!(c.verify.is_none());

    // A foreign promise: a verify command (§2.8), module optional.
    let p = write(
        &dir,
        "keel/contracts/redis-up.md",
        "---\nverify: \"redis-cli ping\"\n---\n\nРедіс живий.\n",
    );
    let c = docs::read_contract(&p).unwrap();
    assert_eq!(c.verify.as_deref(), Some("redis-cli ping"));
    assert!(c.module.is_none());
    assert!(c.exports.is_empty());

    // Lifecycle marks read fine (§2.12, §4.12).
    let p = write(
        &dir,
        "keel/contracts/old-run.md",
        "---\nverify: \"true\"\nwithdrawn: \"замінений новим\"\nsuperseded_by: session-run\nrenamed_from: legacy-run\n---\n",
    );
    let c = docs::read_contract(&p).unwrap();
    assert_eq!(c.withdrawn.as_deref(), Some("замінений новим"));
    assert_eq!(c.superseded_by.as_deref(), Some("session-run"));
    assert_eq!(c.renamed_from.as_deref(), Some("legacy-run"));

    // Neither exports nor verify -- the contract promises nothing (§2.10).
    let p = write(
        &dir,
        "keel/contracts/empty.md",
        "---\nmodule: X\n---\n\nСлова без перевірки.\n",
    );
    let r = docs::read_contract(&p).unwrap_err();
    assert!(
        r.reason.contains("promises nothing"),
        "reason: {}",
        r.reason
    );
    assert!(!r.instead.is_empty());
}

/// proves: duplicate-name-refuses@cb90af -- holds §7.9: YAML would
/// silently keep the last duplicate, and half the plan would vanish
/// without a trace.
#[test]
fn duplicate_name_refuses() {
    let dir = keel_sandbox("dup");

    // Two scenarios under one name.
    let p = write(
        &dir,
        "keel/waves/0008-dup.md",
        concat!(
            "---\n",
            "scenarios:\n",
            "  same: {covers: [functional.correctness]}\n",
            "  same: {covers: [performance.capacity]}\n",
            "transforms:\n",
            "  t:\n",
            "    implements: [same]\n",
            "    files: [lib/a.ex]\n",
            "---\n",
        ),
    );
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("twice"), "reason: {}", r.reason);
    assert!(
        r.reason.contains("same"),
        "names the duplicated name: {}",
        r.reason
    );

    // Two transforms under one name.
    let p = write(
        &dir,
        "keel/waves/0009-dup-t.md",
        concat!(
            "---\n",
            "transforms:\n",
            "  work: {chore: \"а\", files: [a]}\n",
            "  work: {chore: \"б\", files: [b]}\n",
            "---\n",
        ),
    );
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("work"), "reason: {}", r.reason);

    // A duplicated field inside an entry -- the same disease.
    let p = write(
        &dir,
        "keel/contracts/dup-field.md",
        "---\nverify: \"a\"\nverify: \"b\"\n---\n",
    );
    let r = docs::read_contract(&p).unwrap_err();
    assert!(r.reason.contains("verify"), "reason: {}", r.reason);
}

/// proves: dir-among-docs-refuses@eef1bd -- holds §7.9 and lesson 4:
/// nothing vanishes from the report silently.
#[test]
fn dir_among_docs_refuses() {
    let dir = keel_sandbox("dirs");
    fs::create_dir_all(dir.join("keel/waves/drafts")).unwrap();
    write(
        &dir,
        "keel/waves/drafts/0099-hidden.md",
        "---\ntransforms:\n  t: {chore: \"схована робота\", files: [a]}\n---\n",
    );
    let scan = docs::scan(&dir).unwrap();
    assert!(
        scan.refusals.iter().any(|r| r.reason.contains("directory")),
        "a directory must refuse, not stay silent: {:?}",
        scan.refusals
    );
    assert!(
        scan.waves.is_empty(),
        "a wave from a subdirectory is not read silently"
    );

    // A symlink to a directory -- the same disease.
    #[cfg(unix)]
    {
        let dir = keel_sandbox("dir-link");
        fs::create_dir_all(dir.join("elsewhere")).unwrap();
        std::os::unix::fs::symlink(dir.join("elsewhere"), dir.join("keel/waves/link")).unwrap();
        let scan = docs::scan(&dir).unwrap();
        assert!(
            scan.refusals.iter().any(|r| r.reason.contains("directory")),
            "a symlink to a directory refuses: {:?}",
            scan.refusals
        );
    }
}

/// proves: bare-scenario-refuses@0d40d4 -- holds §3.3: a scenario
/// with no footing at all is an error.
#[test]
fn bare_scenario_refuses() {
    let dir = keel_sandbox("bare");
    let p = write(
        &dir,
        "keel/waves/0010-bare.md",
        "---\nscenarios:\n  floating: {}\ntransforms:\n  t:\n    implements: [floating]\n    files: [lib/a.ex]\n---\n",
    );
    let r = docs::read_wave(&p).unwrap_err();
    assert!(
        r.reason.contains("floating"),
        "names the scenario: {}",
        r.reason
    );
    assert!(
        r.reason.contains("leans on nothing"),
        "reason: {}",
        r.reason
    );

    // A withdrawn scenario without footing is legal: it is outside judgement (§6.3).
    let p = write(
        &dir,
        "keel/waves/0011-gone.md",
        "---\nscenarios:\n  gone: {withdrawn: \"знято хвилею 0012\"}\ntransforms:\n  t: {chore: \"прибирання\", files: [a]}\n---\n",
    );
    assert!(
        docs::read_wave(&p).is_ok(),
        "withdrawn without footing is not an error"
    );
}
