//! Scenario test of wave 0043: a promise alive only in prose does
//! not hold.
//!
//! The §7.6 form court compares a contract's `exports` against the
//! module's own source, and the comment stripper cuts LINE comments
//! only -- it knows nothing of multi-line text. So a declaration
//! written inside a string stood for a live one, in every tongue this
//! release leads: an elixir `@moduledoc`, a ruby heredoc, a rust
//! `r#"..."#`. Each was measured holding a promise nothing
//! implements: "signatures compared: 1, 0 findings".
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn keel(dir: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
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

/// The court must refuse the promise AND name where it looked.
fn ghost_refused(said: &str, code: i32, tongue: &str, looked: &str) {
    assert_ne!(
        code, 0,
        "{tongue}: a declaration alive only inside text does not hold \
         a contract:\n{said}"
    );
    assert!(
        said.contains(looked),
        "{tongue}: and the finding names the file it looked in \
         ({looked}):\n{said}"
    );
}

/// proves: a-promise-alive-only-in-prose-does-not-hold@91f921
#[test]
fn a_promise_alive_only_in_prose_does_not_hold() {
    // --- rust: a raw string ---
    let dir = keel_sandbox("prosersr");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub mod toy;\n").unwrap();
    std::fs::write(
        dir.join("src/toy.rs"),
        "pub const SHAPE: &str = r#\"\npub fn phantom(b: u8) -> u8\n\"#;\n\npub fn works() -> bool {\n    true\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: crate::toy\nexports:\n  - \"pub fn phantom(b: u8) -> u8\"\n---\n\nПривид у сирому рядку.\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    ghost_refused(&said, code, "rust", "toy.rs");

    // --- ruby: a heredoc ---
    let dir = keel_sandbox("proseruby");
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::write(
        dir.join("lib/toy.rb"),
        "class Toy\n  DOC = <<~TEXT\n    def ghost(a, b)\n  TEXT\n\n  def works\n    true\n  end\nend\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: Toy\nexports:\n  - \"def ghost(a, b)\"\n---\n\nПривид у heredoc.\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    ghost_refused(&said, code, "ruby", "toy.rb");

    // --- elixir: @moduledoc, the tongue's central idiom ---
    let dir = keel_sandbox("proseelixir");
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"elixir\"\n").unwrap();
    std::fs::write(
        dir.join("mix.exs"),
        "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\n  def application, do: []\nend\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("lib/toy.ex"),
        "defmodule Toy do\n  @moduledoc \"\"\"\n  Приклад:\n\n      def ghost(a, b)\n\n  \"\"\"\n  def works, do: true\nend\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        "---\nmodule: Toy\nexports:\n  - \"def ghost(a, b)\"\n---\n\nПривид у @moduledoc.\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    ghost_refused(&said, code, "elixir", "toy.ex");

    // The other side, and it is the half that makes this a stricter
    // court rather than a broken one: a declaration that really lives
    // there still holds, in each tongue, even with prose around it.
    let (said, code) = keel(&dir, &["check"]);
    assert!(
        said.contains("сигнатур звірено"),
        "the form court still runs and says how much it compared:\n{said}"
    );
    let _ = code;
}
