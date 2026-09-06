//! Scenario test of wave 0053: the courts hold their mutants.
//!
//! Born green (§6.3, the named exception): each case here is a
//! mutant that survived the battery or a clause that had no assert
//! (global review 2026-09-06, tests R-9, R-12; bugs R-25), and the
//! commit of this birth records every mutant played against it. One
//! precision from review 0053 (R-6): the byte-for-byte assert of
//! wave 0040 had NOT vanished -- `json_out_test` still kills the
//! json mutant on the base -- what vanished was the constancy of the
//! package between two runs, and that is what the first case holds.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

const BODY: &str = "тіло обіцянки\n\n";

fn decisions_except(covered: &[&str]) -> String {
    let mut d = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !covered.contains(cut) {
            d.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    d
}

/// A rust crate: Cargo.toml, src/lib.rs, tests/ -- and the frame of
/// the methodology with the wave text given.
fn crate_with(name: &str, adapter: &str, wave_text: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        format!("lang = \"uk\"\nadapter = \"{adapter}\"\n"),
    )
    .unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn works() -> bool { true }\n").unwrap();
    std::fs::write(dir.join("keel/waves/0001-a-wave.md"), wave_text).unwrap();
    std::fs::write(
        dir.join("keel/reviews/0001-a-wave.md"),
        "# Рецензія\n\nok\n",
    )
    .unwrap();
    dir
}

/// The plain full wave: scenario `it-works`, transform `work` over
/// src/lib.rs.
fn plain_wave() -> String {
    format!(
        "---\nscenarios:\n  it-works:\n    covers: [functional.correctness]\ntransforms:\n  work:\n    implements:\n      - it-works\n    files:\n      - src/lib.rs\n{}---\n\n## scenario: it-works\n{BODY}## transform: work\nтіло роботи\n",
        decisions_except(&["functional.correctness"])
    )
}

fn settle(dir: &Path) {
    git(dir, &["init", "-q", "-b", "main"]);
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "-m", "base"]);
    git(dir, &["checkout", "-q", "-b", "0001-a-wave"]);
}

fn run(dir: &Path, args: &[&str]) -> (String, String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

fn executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path).unwrap().permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms).unwrap();
    }
}

/// proves: the-courts-hold-their-mutants@a1f95f
#[test]
fn the_courts_hold_their_mutants() {
    let rev = keel::rev::text_rev(BODY);

    // --- every reading command is steady, and its --json package
    // carries the very prose (0040: "without --json the output stays
    // byte for byte the same" -- the assert that vanished, tests R-9) ---
    let dir = crate_with("mutjson", "rust", &plain_wave());
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!("/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(toy::works());\n}}\n"),
    )
    .unwrap();
    settle(&dir);
    for command in ["check", "status", "next", "map", "cuts", "version"] {
        let (plain, plain_err, plain_code) = run(&dir, &[command]);
        let (again, again_err, again_code) = run(&dir, &[command]);
        assert_eq!(plain, again, "{command} is steady");
        assert_eq!(plain_err, again_err, "{command} is steady on stderr");
        assert_eq!(plain_code, again_code, "{command} is steady in its code");
        let (json, _, json_code) = run(&dir, &[command, "--json"]);
        assert_eq!(
            json_code, plain_code,
            "{command} --json leaves with the same code"
        );
        let package: serde_json::Value = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("{command} --json is one JSON object: {e}\n{json}"));
        assert_eq!(
            package["report"].as_str().unwrap_or_default(),
            plain.as_str(),
            "{command}: the package's report is the plain prose, byte for byte"
        );
    }

    // --- the launcher over an unknown target and over a server that
    // does not answer: a refusal aloud, nothing installed, no step
    // onto the network (0048; tests R-12) ---
    let dir = keel_sandbox("mutlauncher");
    let repo = dir.join("source");
    std::fs::create_dir_all(repo.join("tool")).unwrap();
    std::fs::write(
        repo.join("tool/Cargo.toml"),
        "[package]\nname = \"keel\"\nversion = \"2.0.0\"\n",
    )
    .unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "the release"]);
    git(&repo, &["tag", "v2.0.0"]);
    let stub = dir.join("stub");
    std::fs::create_dir_all(&stub).unwrap();
    let real_curl = ["/usr/bin/curl", "/bin/curl", "/usr/local/bin/curl"]
        .into_iter()
        .find(|c| Path::new(c).is_file())
        .unwrap_or("/usr/bin/curl");
    std::fs::write(
        stub.join("curl"),
        format!(
            "#!/bin/sh\nfor a in \"$@\"; do case \"$a\" in http://*|https://*) echo \"$a\" >> '{}'; exit 22;; esac; done\nexec {real_curl} \"$@\"\n",
            dir.join("curl-http.log").display()
        ),
    )
    .unwrap();
    executable(&stub.join("curl"));
    let releases = dir.join("releases");
    std::fs::create_dir_all(&releases).unwrap();
    let installer = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("install.sh");
    let bare_path = std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .filter(|d| !d.contains("cargo") && !d.contains("rustup"))
        .collect::<Vec<_>>()
        .join(":");
    let install = |shims: &Path, releases: &str| {
        let out = Command::new("sh")
            .arg(&installer)
            .current_dir(&*dir)
            .env("PATH", format!("{}:{bare_path}", shims.display()))
            .env("KEEL_REPO", &repo)
            .env("KEEL_HOME", dir.join("home"))
            .env("KEEL_BIN", dir.join("bin"))
            .env("KEEL_REF", "2.0.0")
            .env("KEEL_RELEASES", releases)
            .output()
            .unwrap();
        (
            format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            ),
            out.status.code().unwrap_or(-1),
        )
    };
    let nothing_installed = |said: &str| {
        let versions = dir.join("home").join("versions");
        let homes = std::fs::read_dir(&versions)
            .map(|d| d.flatten().count())
            .unwrap_or(0);
        assert_eq!(homes, 0, "nothing was installed under versions/:\n{said}");
        assert!(
            !dir.join("curl-http.log").exists(),
            "and the installer never stepped onto the network:\n{said}"
        );
    };
    // An unknown target: a shim `uname` answers Plan9/mips.
    let odd = dir.join("odd");
    std::fs::create_dir_all(&odd).unwrap();
    std::fs::write(
        odd.join("uname"),
        "#!/bin/sh\ncase \"$1\" in -s) echo Plan9;; -m) echo mips;; *) echo Plan9;; esac\n",
    )
    .unwrap();
    executable(&odd.join("uname"));
    std::fs::copy(stub.join("curl"), odd.join("curl")).unwrap();
    executable(&odd.join("curl"));
    let (said, code) = install(&odd, &format!("file://{}", releases.display()));
    assert_ne!(
        code, 0,
        "an unknown target with no cargo to build is a refusal:\n{said}"
    );
    assert!(
        said.contains("no release is built for Plan9/mips"),
        "and the refusal names the machine it found no release for:\n{said}"
    );
    nothing_installed(&said);
    // A server that does not answer: the address names a place that
    // is not there.
    let (said, code) = install(&stub, "file:///nonexistent-keel-releases");
    assert_ne!(
        code, 0,
        "a server that does not answer, with no cargo to build, is a refusal:\n{said}"
    );
    assert!(
        said.contains("no release v2.0.0") && said.contains("file:///nonexistent-keel-releases"),
        "and the refusal names the tag and the address:\n{said}"
    );
    nothing_installed(&said);

    // --- a run that lost a verdict is not green (§7.13; bugs R-25):
    // the tagged test exists in the first run only, and the closing
    // court says "green in 1 of 3 runs" ---
    let dir = crate_with("mutlost", "rust", &plain_wave());
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\nbuild = \"build.rs\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("build.rs"),
        "fn main() {\n    println!(\"cargo:rerun-if-changed=counter\");\n    println!(\"cargo:rustc-check-cfg=cfg(first_run)\");\n    let n: u32 = std::fs::read_to_string(\"counter\").ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);\n    if n == 0 {\n        println!(\"cargo:rustc-cfg=first_run\");\n    }\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-works@{rev}\n#[cfg(first_run)]\n#[test]\nfn it_works() {{\n    let counter = concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/counter\");\n    let n: u32 = std::fs::read_to_string(counter).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);\n    std::fs::write(counter, (n + 1).to_string()).unwrap();\n    assert!(toy::works());\n}}\n\n#[test]\nfn always() {{\n    assert!(toy::works());\n}}\n"
        ),
    )
    .unwrap();
    std::fs::write(dir.join(".gitignore"), "target\ncounter\n").unwrap();
    settle(&dir);
    let (said, _, code) = run(&dir, &["close"]);
    assert!(
        said.contains("зелений у 1 з 3 бігів") && said.contains("§7.13"),
        "a verdict lost in two of three runs is not green:\n{said}"
    );
    assert_ne!(code, 0, "and the wave does not close:\n{said}");

    // --- the battery's child does not hear KEEL_BRANCH (bugs R-25):
    // the court names its branch to itself, never to the tests ---
    let dir = crate_with("mutbranch", "rust", &plain_wave());
    std::fs::write(
        dir.join("tests/w_test.rs"),
        format!(
            "/// proves: it-works@{rev}\n#[test]\nfn it_works() {{\n    assert!(std::env::var(\"KEEL_BRANCH\").is_err(), \"the battery heard the court's branch\");\n    assert!(toy::works());\n}}\n"
        ),
    )
    .unwrap();
    settle(&dir);
    // The word arrives as CI hands it over -- in the environment --
    // and the court must not pass it on to the tests it runs.
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["close", dir.to_str().unwrap()])
        .env("KEEL_BRANCH", "0001-a-wave")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        said.contains("0001-a-wave: закрита"),
        "the tests run without the court's own word about the branch:\n{said}"
    );
    assert_eq!(out.status.code(), Some(0), "and the wave closes:\n{said}");
}
