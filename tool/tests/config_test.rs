//! Тести сценаріїв хвилі 0002-config-and-language, трансформа
//! read-config. Теги proves — редакція за §5.3–§5.4, рахована руками
//! (bootstrap; щабель rev їде хвилею 0003).

use keel::config;
use std::fs;
use std::path::{Path, PathBuf};

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0002-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(dir: &Path, rel: &str, text: &str) -> PathBuf {
    let p = dir.join(rel);
    fs::write(&p, text).unwrap();
    p
}

/// proves: config-reads-strictly@1a978b — тримає контракт tool-config:
/// словник читається весь, невідоме і криве — відмова, не пропуск.
#[test]
fn config_reads_strictly() {
    // Повний словник читається в дані.
    let dir = sandbox("full");
    write(
        &dir,
        "keel.toml",
        concat!(
            "version = \"2.0.3\"\n",
            "adapter = \"elixir\"\n",
            "ci = \"mix ci\"\n",
            "lang = \"uk\"\n",
            "[trust]\n",
            "\"mix ci\" = \"a3f1c07c40de\"\n",
            "[generated]\n",
            "\"skills/keel-plan.md\" = \"0b32af\"\n",
        ),
    );
    let c = config::read(&dir).unwrap();
    assert!(c.present);
    assert_eq!(c.version.as_deref(), Some("2.0.3"));
    assert_eq!(c.adapter.as_deref(), Some("elixir"));
    assert_eq!(c.ci.as_deref(), Some("mix ci"));
    assert_eq!(c.lang, "uk");
    assert_eq!(
        c.trust,
        vec![("mix ci".to_string(), "a3f1c07c40de".to_string())]
    );
    assert_eq!(
        c.generated,
        vec![("skills/keel-plan.md".to_string(), "0b32af".to_string())]
    );

    // Невідоме поле — відмова, що його називає.
    let dir = sandbox("typo");
    write(&dir, "keel.toml", "lang = \"uk\"\nlangg = \"en\"\n");
    let r = config::read(&dir).unwrap_err();
    assert!(r.reason.contains("langg"), "names the field: {}", r.reason);
    assert!(!r.instead.is_empty());

    // Кривий тип — відмова.
    let dir = sandbox("badtype");
    write(&dir, "keel.toml", "lang = 42\n");
    let r = config::read(&dir).unwrap_err();
    assert!(!r.reason.is_empty() && !r.instead.is_empty());

    // Битий TOML — відмова.
    let dir = sandbox("broken");
    write(&dir, "keel.toml", "lang = \"uk\n");
    let r = config::read(&dir).unwrap_err();
    assert!(!r.reason.is_empty());

    // lang поза вшитими мовами — відмова, що називає наявні.
    let dir = sandbox("nolang");
    write(&dir, "keel.toml", "lang = \"pl\"\n");
    let r = config::read(&dir).unwrap_err();
    assert!(r.reason.contains("pl"), "names the value: {}", r.reason);
    assert!(
        r.reason.contains("en") && r.reason.contains("uk"),
        "names available languages: {}",
        r.reason
    );

    // Відсутній keel.toml — не помилка: чесні типові значення.
    let dir = sandbox("absent");
    let c = config::read(&dir).unwrap();
    assert!(!c.present);
    assert_eq!(c.lang, "en");
}
