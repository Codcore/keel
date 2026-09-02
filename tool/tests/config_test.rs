//! Scenario tests of wave 0002-config-and-language, transform
//! read-config. proves tags -- revisions per §5.3-§5.4, computed by
//! hand (bootstrap; the rev rung rides wave 0003).

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

/// proves: config-reads-strictly@1a978b -- holds contract
/// tool-config: the vocabulary reads whole; the unknown and the
/// malformed refuse, never skip.
#[test]
fn config_reads_strictly() {
    // The full vocabulary reads into data.
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

    // An unknown field refuses, naming it.
    let dir = sandbox("typo");
    write(&dir, "keel.toml", "lang = \"uk\"\nlangg = \"en\"\n");
    let r = config::read(&dir).unwrap_err();
    assert!(r.reason.contains("langg"), "names the field: {}", r.reason);
    assert!(!r.instead.is_empty());

    // A wrong type refuses, naming the field and the line (Z-4a).
    let dir = sandbox("badtype");
    write(&dir, "keel.toml", "lang = 42\n");
    let r = config::read(&dir).unwrap_err();
    assert!(r.reason.contains("lang"), "names the field: {}", r.reason);
    assert!(r.reason.contains("line"), "names the line: {}", r.reason);
    assert!(!r.instead.is_empty());

    // Broken TOML refuses, with a position (Z-4a).
    let dir = sandbox("broken");
    write(&dir, "keel.toml", "lang = \"uk\n");
    let r = config::read(&dir).unwrap_err();
    assert!(r.reason.contains("line"), "names the line: {}", r.reason);

    // lang outside the embedded set refuses, naming the available.
    let dir = sandbox("nolang");
    write(&dir, "keel.toml", "lang = \"pl\"\n");
    let r = config::read(&dir).unwrap_err();
    assert!(r.reason.contains("pl"), "names the value: {}", r.reason);
    assert!(
        r.reason.contains("en") && r.reason.contains("uk"),
        "names available languages: {}",
        r.reason
    );

    // An absent keel.toml is not an error: honest defaults.
    let dir = sandbox("absent");
    let c = config::read(&dir).unwrap();
    assert!(!c.present);
    assert_eq!(c.lang, "en");
}
