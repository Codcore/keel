//! Тести сценаріїв хвилі 0001-strict-headers, трансформа
//! check-walks-project. Ганяємо справжній бінарник: сценарії
//! обіцяють поведінку команди — звіт і код виходу, а не функцію.
//!
//! Теги proves — редакція за §5.3–§5.4, рахована руками (bootstrap).

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

/// proves: check-reports-every-file@327c12 — тримає §7.9 і урок №4:
/// зіпсоване назване, сусіди перевірені, неперевірене назване вголос.
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
        "відмова в документі — знахідка, вихід 1; звіт:\n{out}"
    );
    for f in [
        "keel/waves/0002-ok.md",
        "keel/contracts/session-run.md",
        "keel/contracts/broken.md",
    ] {
        assert!(out.contains(f), "звіт мусить назвати {f}:\n{out}");
    }
    assert!(
        out.contains("не закрита"),
        "зіпсований названий з причиною:\n{out}"
    );
    assert!(
        out.contains("ще не перевірено"),
        "неперевірене назване вголос:\n{out}"
    );
    assert!(
        out.contains("редакції"),
        "серед неперевіреного — редакції:\n{out}"
    );

    // Без зіпсованого — вихід 0; чесність про неперевірене лишається.
    fs::remove_file(dir.join("keel/contracts/broken.md")).unwrap();
    let (out, _err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "звіт:\n{out}");
    assert!(
        out.contains("ще не перевірено"),
        "зелене не ховає власних меж:\n{out}"
    );
}

/// proves: missing-keel-dir-refuses@01149b — тримає §9.7: відмова несе
/// причину і що робити натомість.
#[test]
fn missing_keel_dir_refuses() {
    let dir = std::env::temp_dir().join(format!("keel-0001c-{}-nokeel", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();

    let (_out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    assert_eq!(code, 2, "відмова самої команди — вихід 2; stderr:\n{err}");
    assert!(err.contains("keel/"), "називає, чого бракує:\n{err}");
    assert!(err.contains("створи"), "каже, що робити натомість:\n{err}");
}
