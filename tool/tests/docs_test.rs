//! Тести сценаріїв хвилі 0001-strict-headers, трансформа read-headers.
//!
//! Кожен тест несе тег `proves: <сценарій>@<редакція>` — редакція за
//! §5.3–§5.4: тіло секції сценарію, повторні пробіли й переноси
//! згорнуті в один пробіл, sha256, перші шість шістнадцяткових
//! знаків. Поки редакцію рахують руки (bootstrap); щабель 2 (`keel
//! rev`) зобовʼязаний відтворити цей рецепт.

use keel::docs;
use std::fs;
use std::path::{Path, PathBuf};

/// Окрема тека на тест: тести не діляться станом і не заважають одне
/// одному (§7.13 ганяє їх кілька разів поспіль).
fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0001-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::create_dir_all(dir.join("keel/contracts")).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) -> PathBuf {
    let p = dir.join(rel);
    fs::write(&p, text).unwrap();
    p
}

/// proves: broken-header-refuses@240948 — тримає §7.9.
#[test]
fn broken_header_refuses() {
    let dir = sandbox("broken");

    // Шапка не закрита: другого `---` нема.
    let p = write(&dir, "keel/waves/0002-x.md", "---\nscenarios:\n");
    let r = docs::read_wave(&p).unwrap_err();
    assert_eq!(r.file, p);
    assert!(r.reason.contains("не закрита"), "причина: {}", r.reason);
    assert!(!r.instead.is_empty(), "відмова мусить казати, що робити натомість");

    // Шапки нема зовсім.
    let p = write(&dir, "keel/waves/0003-y.md", "# просто текст, без шапки\n");
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("шапки нема"), "причина: {}", r.reason);
    assert!(!r.instead.is_empty());

    // Битий YAML усередині шапки.
    let p = write(&dir, "keel/contracts/c.md", "---\nmodule: [unclosed\n---\n");
    let r = docs::read_contract(&p).unwrap_err();
    assert!(r.reason.contains("YAML"), "причина: {}", r.reason);
    assert!(!r.instead.is_empty());
}

/// proves: unknown-field-refuses@4fa15d — тримає §7.9.
#[test]
fn unknown_field_refuses() {
    let dir = sandbox("unknown");

    // Одрук у полі хвилі: scenarois замість scenarios.
    let p = write(
        &dir,
        "keel/waves/0004-typo.md",
        "---\nscenarois:\n  a: {covers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [a]\n    files: [lib/a.ex]\n---\n",
    );
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("невідоме поле"), "причина: {}", r.reason);
    assert!(r.reason.contains("scenarois"), "називає само поле: {}", r.reason);
    assert!(!r.instead.is_empty());

    // Невідоме поле всередині сценарію.
    let p = write(
        &dir,
        "keel/waves/0005-inner.md",
        "---\nscenarios:\n  a: {covvers: [functional.correctness]}\ntransforms:\n  t:\n    implements: [a]\n    files: [lib/a.ex]\n---\n",
    );
    let r = docs::read_wave(&p).unwrap_err();
    assert!(r.reason.contains("covvers"), "причина: {}", r.reason);

    // Невідоме поле контракту.
    let p = write(
        &dir,
        "keel/contracts/typo.md",
        "---\nmodule: X\nexporst:\n  - \"run()\"\n---\n",
    );
    let r = docs::read_contract(&p).unwrap_err();
    assert!(r.reason.contains("exporst"), "причина: {}", r.reason);
}

/// proves: valid-wave-parses@8b543c — тримає §2.3–§2.5, §2.11, §2.12,
/// §4.1, §4.12: повний словник хвилі читається в дані без втрат.
#[test]
fn valid_wave_parses() {
    let dir = sandbox("valid-wave");
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
    assert_eq!((pr.slug.as_str(), pr.rev.as_str()), ("session-run", "7c40de"));
    assert_eq!(alive.covers, vec!["functional.correctness", "safety.fail-safe"]);

    let (_, gone) = &w.scenarios[1];
    assert_eq!(gone.withdrawn.as_deref(), Some("обіцянку зняла хвиля 0009"));
    assert_eq!(gone.superseded_by.as_deref(), Some("alive"));

    let (_, work) = &w.transforms[0];
    match &work.kind {
        docs::TransformKind::Implements(s) => assert_eq!(s, &vec!["alive"]),
        other => panic!("не те: {other:?}"),
    }
    assert_eq!(
        work.files,
        vec![
            docs::ScopeLine::Path("lib/session.ex".into()),
            docs::ScopeLine::OneNewIn("priv/migrations/".into()),
        ]
    );

    let (_, tidy) = &w.transforms[1];
    match &tidy.kind {
        docs::TransformKind::Chore(why) => assert_eq!(why, "оновлення залежности без обіцянки"),
        other => panic!("не те: {other:?}"),
    }

    assert_eq!(
        w.decisions,
        vec![("performance.time-behaviour".to_string(), "не міряємо: разова команда".to_string())]
    );

    // Законна відсутність — не помилка: хвиля всуціль chore без сценаріїв.
    let p = write(
        &dir,
        "keel/waves/0007-chore-only.md",
        "---\ntransforms:\n  tidy:\n    chore: \"форматування\"\n    files: [README.md]\n---\n",
    );
    let w = docs::read_wave(&p).unwrap();
    assert!(w.scenarios.is_empty());
    assert!(w.decisions.is_empty());
}

/// proves: valid-contract-parses@863c4e — тримає §2.7–§2.8: наша
/// обіцянка (module + exports) і чужа (verify) читаються в дані;
/// контракт без жодної — відмова «нічого не обіцяє».
#[test]
fn valid_contract_parses() {
    let dir = sandbox("valid-contract");

    // Наш контракт: module + exports.
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

    // Чужа обіцянка: verify-команда (§2.8), module не обовʼязковий.
    let p = write(
        &dir,
        "keel/contracts/redis-up.md",
        "---\nverify: \"redis-cli ping\"\n---\n\nРедіс живий.\n",
    );
    let c = docs::read_contract(&p).unwrap();
    assert_eq!(c.verify.as_deref(), Some("redis-cli ping"));
    assert!(c.module.is_none());
    assert!(c.exports.is_empty());

    // Ні exports, ні verify — контракт нічого не обіцяє (§2.10).
    let p = write(
        &dir,
        "keel/contracts/empty.md",
        "---\nmodule: X\n---\n\nСлова без перевірки.\n",
    );
    let r = docs::read_contract(&p).unwrap_err();
    assert!(r.reason.contains("не обіцяє"), "причина: {}", r.reason);
    assert!(!r.instead.is_empty());
}
