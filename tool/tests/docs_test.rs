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
