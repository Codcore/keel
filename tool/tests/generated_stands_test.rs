//! A guard of a class, not a promise (patch of review 0035 R-9): no
//! file this repository generates may drift from what the release
//! writes.
//!
//! The finding: wave 0035 needed a line in `.github/workflows/keel.yml`
//! and I typed it into the file instead of into the generator. From
//! then on `keel update` refused the file as edited by hand, its
//! digest in [generated] stood stale, and any project that ran the
//! tool got a workflow WITHOUT that line -- the fix lived in this
//! repository alone. §9.7 is what refused, and it was right; the
//! mistake was mine, and nothing measured it.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

#[test]
fn no_generated_file_is_edited_by_hand() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let config = std::fs::read_to_string(repo.join("keel.toml")).unwrap();

    // The names this project records as generated, read from its own
    // config rather than listed here.
    let mut names: Vec<String> = Vec::new();
    let mut inside = false;
    for line in config.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            inside = line == "[generated]";
            continue;
        }
        if inside && let Some((key, _)) = line.split_once('=') {
            names.push(key.trim().trim_matches('"').to_string());
        }
    }
    assert!(
        names.len() >= 5,
        "this project generates several files: {names:?}"
    );

    // A copy holding the config and those files, and nothing else:
    // `keel update` then says of each whether it still stands as the
    // release writes it.
    let dir = keel_sandbox("generated");
    std::fs::write(dir.join("keel.toml"), &config).unwrap();
    // The generated CI now reads the project's LAYOUT as well as its
    // config -- where the crate is, and which toolchain the project
    // pins (wave 0044). A copy carrying only keel.toml would be a
    // world thinner than the one it judges, and the probe would
    // report drift that is only its own fixture's (the lesson of
    // review 0041). So the markers travel with it.
    for marker in ["Cargo.toml", "tool/Cargo.toml", "rust-toolchain.toml"] {
        let from = repo.join(marker);
        if !from.is_file() {
            continue;
        }
        let to = dir.join(marker);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(&from, &to).unwrap();
    }
    for name in &names {
        let from = repo.join(name);
        assert!(from.is_file(), "{name} is recorded as generated and exists");
        let to = dir.join(name);
        std::fs::create_dir_all(to.parent().unwrap()).unwrap();
        std::fs::copy(&from, &to).unwrap();
    }

    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["update", dir.to_str().unwrap()])
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );

    for name in &names {
        let row = said
            .lines()
            .find(|line| line.contains(name.as_str()))
            .unwrap_or_else(|| panic!("keel update says something about {name}:\n{said}"));
        assert!(
            row.contains("уже стоїть") || row.contains("already stands"),
            "{name} still stands as this release writes it -- a hand \
             that edits a generated file leaves the fix in this \
             repository alone (review 0035 R-9):\n{row}"
        );
    }
}
