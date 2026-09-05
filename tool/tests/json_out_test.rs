//! Scenario test of wave 0040: every reading command answers in JSON.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::{Sandbox, sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let mut command = Command::new("git");
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
    let out = command
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
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

/// A project with one wave whose header is broken on purpose, so the
/// document court has something real to be red about.
fn project(name: &str) -> Sandbox {
    let dir = sandbox(name);
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        "---\nscenarios:\n  a-promise:\n    covers: [no.such.cut]\ntransforms: {}\n---\n\n## Why\n\nтіло\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base", "--no-verify"]);
    dir
}

/// A project with a readable wave and a crate, so every count the
/// package claims has a value to be wrong about.
fn whole_project(name: &str) -> Sandbox {
    let dir = sandbox(name);
    fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    for made in ["keel/waves", "keel/contracts", "keel/reviews", "src"] {
        fs::create_dir_all(dir.join(made)).unwrap();
    }
    fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    let mut decisions = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        decisions.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
    }
    fs::write(
        dir.join("keel/waves/0001-a-wave.md"),
        format!(
            "---\nscenarios: {{}}\ntransforms:\n  work:\n    chore: \"рама\"\n    files:\n      - src/lib.rs\n{decisions}---\n\n## Why\n\nтіло\n\n## transform: work\n\nтіло роботи\n"
        ),
    )
    .unwrap();
    // On the wave's own branch, so the closing court has a wave to
    // judge and a blocker to count -- the missing review report.
    git(&dir, &["init", "-q", "-b", "0001-a-wave"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base", "--no-verify"]);
    dir
}

/// The verdict a person reads, for the same call. Every number the
/// package claims is measured against THIS -- review 0040 R-4: the
/// probe used to ask `is_u64()` and `is_array()`, so eleven mutations
/// that made every number in the package a lie passed the whole
/// battery. A type is not a value.
fn prose_of(command: &str, dir: &Path) -> String {
    let (out, err, _) = keel(&[command, dir.to_str().unwrap()]);
    format!("{out}{err}")
}

/// How many rows of the verdict are red.
fn red_rows(prose: &str) -> usize {
    prose
        .lines()
        .filter(|line| line.trim_start().starts_with("червоне"))
        .count()
}

/// proves: every-reading-command-answers-in-json@42b771 -- every harness
/// keel itself writes had to parse prose to learn how many findings
/// there were, and the prose comes in two languages. The concept
/// promised the other road twice and the code never had it.
#[test]
fn every_reading_command_answers_in_json() {
    let dir = project("jsonout");

    // check: the findings a harness needs, as fields -- which file,
    // and why -- with the summary's own numbers beside them.
    let (out, _, code) = keel(&["check", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(code, 1, "the sandbox really is red:\n{out}");
    assert_eq!(
        package["structured"], true,
        "and the package says so:\n{out}"
    );
    let findings = package["findings"].as_array().expect("findings is a list");
    assert!(!findings.is_empty(), "the findings are listed:\n{out}");
    assert!(
        findings
            .iter()
            .all(|one| one["file"].is_string() && one["reason"].is_string()),
        "each with the file that is wrong and why:\n{out}"
    );
    assert_eq!(
        package["summary"]["findings"].as_u64().map(|n| n as usize),
        Some(findings.len()),
        "and the summary counts exactly those:\n{out}"
    );
    assert!(
        package["summary"]["documents"].is_u64() && package["limits"].is_array(),
        "with the documents walked and what was not judged:\n{out}"
    );
    assert!(
        findings.iter().any(|one| one["file"]
            .as_str()
            .is_some_and(|file| file.contains("0001-a-wave"))),
        "the broken wave is named by its path, so a harness need not \
         split a sentence in either tongue:\n{out}"
    );

    // Every number is measured against the verdict a person reads --
    // not merely asked whether it is a number (review 0040 R-4).
    let prose = prose_of("check", &dir);
    assert_eq!(
        findings.len(),
        red_rows(&prose),
        "the package lists exactly the red rows the prose shows:\n{prose}"
    );
    let documents = package["summary"]["documents"].as_u64().unwrap();
    // The number the prose names, read as a number: `contains` on a
    // rendered fragment let "0 документ" match "0 документів" and any
    // count match its own prefix (review 0040 R-4's lesson, applied
    // to my own first cut of this assert).
    let counted: u64 = prose
        .split("підсумок: ")
        .nth(1)
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(|word| word.parse().ok())
        .expect("the verdict counts its documents");
    assert_eq!(
        documents, counted,
        "summary.documents is the number the prose names:\n{prose}"
    );
    let limits = package["limits"].as_array().unwrap();
    assert_eq!(
        limits.len(),
        package["summary"]["limits"].as_u64().unwrap() as usize,
        "the limits listed are the limits counted:\n{out}"
    );
    assert!(
        limits
            .iter()
            .all(|row| row.as_str().is_some_and(|row| prose.contains(row))),
        "and each of them is a line the person is shown too:\n{prose}"
    );
    // The instead is its own field, not prose glued behind a newline
    // and a word that changes with the tongue (review 0040 R-5).
    assert!(
        findings
            .iter()
            .all(|one| !one["reason"].as_str().unwrap_or_default().contains('\n')),
        "no finding's reason carries a rendered row:\n{out}"
    );
    assert!(
        findings
            .iter()
            .any(|one| one["instead"].as_str().is_some_and(|w| !w.is_empty())),
        "and what to do instead stands apart:\n{out}"
    );

    // close and status carry their own number rather than a count a
    // harness would have to read out of prose.
    let (out, _, _) = keel(&["close", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    if let Some(blockers) = package["blockers"].as_u64() {
        let prose = prose_of("close", &dir);
        assert_eq!(
            blockers > 0,
            prose.contains("блокер") && !prose.contains("блокерів нема"),
            "close's blockers are the ones the prose names:\n{prose}"
        );
    } else {
        assert!(
            package["refusal"].is_object(),
            "or close refuses as a package:\n{out}"
        );
    }
    let (out, _, _) = keel(&["status", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["refusal"].is_object(),
        "status over a project with no crate refuses, as a package:\n{out}"
    );
    // And over a project it CAN read, the count is the count: zero,
    // and zero exactly -- not merely "a number" (review 0040 R-4).
    let whole = whole_project("statusok");
    let (out, _, code) = keel(&["status", "--json", whole.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(code, 0, "a whole project's stages are read:\n{out}");
    assert_eq!(
        package["refusals"].as_u64(),
        Some(0),
        "and nothing refused, said as zero rather than as some number:\n{out}"
    );

    // The counts that must NOT be zero, measured where they are not:
    // the broken fixture above has no readable document at all, so a
    // mutation pinning `documents` to 0 passed under it, and `close`
    // refused there so `blockers` was never even present. A number is
    // only judged where it has a value to be wrong about (review 0040
    // R-4, and my own first cut of these asserts).
    let (out, _, _) = keel(&["check", "--json", whole.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    let prose = prose_of("check", &whole);
    let counted: u64 = prose
        .split("підсумок: ")
        .nth(1)
        .and_then(|tail| tail.split_whitespace().next())
        .and_then(|word| word.parse().ok())
        .expect("the verdict counts its documents");
    assert!(counted > 0, "the whole project has documents:\n{prose}");
    assert_eq!(
        package["summary"]["documents"].as_u64(),
        Some(counted),
        "and the package says that very number:\n{out}"
    );

    let (out, _, code) = keel(&["close", "--json", whole.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    let blockers = package["blockers"]
        .as_u64()
        .unwrap_or_else(|| panic!("close ran and counted its blockers:\n{out}"));
    assert_eq!(
        blockers > 0,
        code == 1,
        "and the blockers it counted are the ones it left with:\n{out}"
    );
    let prose = prose_of("close", &whole);
    assert_eq!(
        blockers == 0,
        prose.contains("блокерів нема"),
        "and the ones the prose names:\n{prose}"
    );

    // version answers what a harness pinning a release actually asks.
    let (out, _, _) = keel(&["version", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        package["running"].as_str(),
        Some(env!("CARGO_PKG_VERSION")),
        "version names the binary that really ran:\n{out}"
    );
    // A pinned project, read by the lamp's own unpinned eye: the pin
    // is a value, and the package says which one.
    let pinned = sandbox("apin");
    fs::write(
        pinned.join("keel.toml"),
        "lang = \"uk\"\nversion = \"9.9.9-a-pin\"\n",
    )
    .unwrap();
    let (out, _, _) = keel(&["version", "--json", pinned.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        package["pin"].as_str(),
        Some("9.9.9-a-pin"),
        "and the pin that project really carries:\n{out}"
    );
    let unpinned = sandbox("nopin");
    fs::write(unpinned.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let (out, _, _) = keel(&["version", "--json", unpinned.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["pin"].is_null(),
        "null where there is none, rather than an empty string \
         pretending to be one (NEW-CONCEPT: a number nobody gave is \
         empty, not zero):\n{out}"
    );
    assert_eq!(
        package["lang"].as_str(),
        Some("uk"),
        "and the tongue is the project's own, never blank:\n{out}"
    );

    // cuts hands over the list itself, not a page to scrape.
    let (out, _, _) = keel(&["cuts", "--json", dir.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    let named: Vec<&str> = package["cuts"]
        .as_array()
        .expect("cuts is a list")
        .iter()
        .map(|cut| cut.as_str().unwrap_or_default())
        .collect();
    assert_eq!(named.len(), 40, "cuts gives forty:\n{out}");
    assert_eq!(
        named,
        keel::graph::cuts().to_vec(),
        "and they are the forty the courts judge by, in that order -- \
         not forty of anything:\n{out}"
    );
    let prose = prose_of("cuts", &dir);
    assert!(
        named.iter().all(|cut| prose.contains(cut)),
        "each of them shown to a person too:\n{prose}"
    );
    for command in ["check", "close", "status", "map", "review", "cuts"] {
        let (out, _, _) = keel(&[command, "--json", dir.to_str().unwrap()]);
        let package: serde_json::Value = serde_json::from_str(&out).unwrap();
        assert_eq!(
            package["lang"].as_str(),
            Some("uk"),
            "{command} answers in the tongue the project named:\n{out}"
        );
    }

    // `structured` means what it says: true where the package really
    // carries fields of the command's own, false where it carries
    // prose and the envelope alone. Naming an absence is the point of
    // the field, so a mutation that pinned it to `true` had to be a
    // finding and was not.
    const ENVELOPE_FIELDS: [&str; 8] = [
        "command",
        "exit",
        "keel",
        "lang",
        "ok",
        "report",
        "root",
        "structured",
    ];
    for command in [
        "check", "close", "status", "map", "review", "cuts", "version",
    ] {
        let (out, _, _) = keel(&[command, "--json", whole.to_str().unwrap()]);
        let package: serde_json::Value = serde_json::from_str(&out).unwrap();
        let object = package.as_object().unwrap();
        let own = object
            .keys()
            .filter(|key| !ENVELOPE_FIELDS.contains(&key.as_str()) && *key != "refusal")
            .count();
        assert_eq!(
            package["structured"].as_bool(),
            Some(own > 0),
            "{command} says whether it carries fields of its own, and \
             it carries {own}:\n{out}"
        );
    }

    // And the promise that keeps every existing script working. The
    // honest form of it, after review 0040 R-1: this probe cannot
    // compare two releases, and comparing the binary with ITSELF --
    // which is what stood here -- can only ever measure determinism.
    // What it CAN hold is that the prose a person reads is exactly
    // what the package carries, so the machine road can never quietly
    // become the reason the prose moved.
    for command in [
        "check", "close", "status", "map", "review", "version", "cuts", "rev",
    ] {
        let (plain, _, plain_code) = keel(&[command, dir.to_str().unwrap()]);
        let (out, err, json_code) = keel(&[command, "--json", dir.to_str().unwrap()]);
        assert!(
            err.is_empty(),
            "{command} --json puts nothing beside the package -- a \
             harness reads the whole of stdout, and stderr is not \
             part of it:\n{err}"
        );
        assert_eq!(
            plain_code, json_code,
            "both roads of {command} leave with the same code"
        );
        let package: serde_json::Value = serde_json::from_str(&out).unwrap();
        // A refusal never reached stdout on the prose road, and the
        // package carries it in `report` instead -- that is the one
        // place the two roads differ, and it is the contract's own
        // word.
        if plain_code != 2 {
            assert_eq!(
                package["report"].as_str().unwrap_or_default(),
                plain,
                "and {command}'s package carries that very prose, to the byte"
            );
        } else {
            assert!(
                package["refusal"].is_object(),
                "{command} refused, so the package carries the refusal:\n{out}"
            );
        }
    }

    // The one shape of the prose that this wave was caught changing:
    // a refusal about the project itself has no file name of its own,
    // and the row is rendered without one (review 0040 R-1). The
    // package fills it in; the prose does not.
    let bare = sandbox("nocrate");
    fs::write(
        bare.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"rust\"\n",
    )
    .unwrap();
    fs::create_dir_all(bare.join("keel/waves")).unwrap();
    fs::create_dir_all(bare.join("keel/contracts")).unwrap();
    git(&bare, &["init", "-q", "-b", "main"]);
    let prose = prose_of("check", &bare);
    assert!(
        prose.contains("червоне   — Cargo.toml"),
        "the row keeps the shape every existing script sees:\n{prose}"
    );
    let (out, _, _) = keel(&["check", "--json", bare.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(
        package["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|one| one["file"] == "."),
        "while the field names the project itself rather than nothing:\n{out}"
    );
}
