//! Scenario test of wave 0038: a ruby contract holds its form.

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

fn check(dir: &Path) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", dir.to_str().unwrap()])
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
}

/// A ruby project whose contract names `module`, with the source put
/// wherever the case wants it.
fn project(name: &str, module: &str, exports: &str, at: Option<(&str, &str)>) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    if let Some((path, body)) = at {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, body).unwrap();
    }
    std::fs::write(
        dir.join("keel/contracts/probe.md"),
        format!("---\nmodule: {module}\nexports:\n{exports}---\n\nтіло контракту\n"),
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    dir
}

/// proves: a-ruby-contract-holds-its-form@7dfb2e -- the form court of
/// §7.6 knew one language's layout: `src/<name>.rs`. For every other
/// project it said "nobody compared the form", honestly and
/// uselessly. Ruby keeps a constant's source where ruby keeps it --
/// `Toy::Bar` in `lib/toy/bar.rb` -- and the court reads it there.
#[test]
fn a_ruby_contract_holds_its_form() {
    // The promise is kept: silence.
    let dir = project(
        "rbheld",
        "Toy::Bar",
        "  - \"def self.works\"\n",
        Some((
            "lib/toy/bar.rb",
            "module Toy\n  module Bar\n    def self.works\n      true\n    end\n  end\nend\n",
        )),
    );
    let (said, code) = check(&dir);
    assert_eq!(code, 0, "a kept ruby promise is silence:\n{said}");
    assert!(
        said.contains("сигнатур звірено: 1"),
        "and it is counted as compared, not skipped (§7.6):\n{said}"
    );

    // The bare constant lives in lib/<name>.rb.
    let dir = project(
        "rbbare",
        "Toy",
        "  - \"def self.works\"\n",
        Some((
            "lib/toy.rb",
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )),
    );
    let (said, _) = check(&dir);
    assert!(
        said.contains("сигнатур звірено: 1"),
        "a bare constant is lib/<name>.rb:\n{said}"
    );

    // A promise that is gone is a finding by name -- the same court,
    // the same words, another tongue.
    let dir = project(
        "rbvanished",
        "Toy",
        "  - \"def self.works\"\n  - \"def self.gone\"\n",
        Some((
            "lib/toy.rb",
            "module Toy\n  def self.works\n    true\n  end\nend\n",
        )),
    );
    let (said, code) = check(&dir);
    assert_eq!(code, 1, "a vanished ruby unit is a finding:\n{said}");
    assert!(said.contains("gone"), "and it is named:\n{said}");

    // A module that is not there says where it was looked for.
    let dir = project(
        "rbmissing",
        "Toy::Nowhere",
        "  - \"def self.works\"\n",
        None,
    );
    let (said, code) = check(&dir);
    assert_eq!(code, 1, "a missing ruby module is a finding:\n{said}");
    assert!(
        said.contains("lib/toy/nowhere.rb"),
        "and the finding says where it looked, in ruby's own \
         layout:\n{said}"
    );
}
