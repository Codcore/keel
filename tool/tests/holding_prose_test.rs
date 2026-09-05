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
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"elixir\"\n",
    )
    .unwrap();
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

/// The shapes the first cut of this wave got wrong, and the ones it
/// never took (review 0043 R-1, R-2, R-4, R-9, R-10).
///
/// The heavy half of that review was measured on real source, not on
/// examples: `syn`'s `ident.strip_prefix("r#")` opened a raw string
/// that was never open, and `net/protocol.rb`'s `<<-End` was read as
/// waiting for a line saying `E`. Both shapes stand here by name.
///
/// proves: a-promise-alive-only-in-prose-does-not-hold@91f921
#[test]
fn what_the_reader_must_not_mistake_for_text() {
    // --- rust: LIVE code that must keep holding ---
    for (name, source) in [
        // The `syn` shape: `"r#"` is a string, not a raw string.
        (
            "liveraw",
            "pub fn unraw(ident: &str) -> Option<&str> { ident.strip_prefix(\"r#\") }\npub fn phantom(b: u8) -> u8 { b }\n",
        ),
        // The `crossterm` shape: a `\r` before the closing quote.
        (
            "livecr",
            "pub fn shout(e: u8) { println!(\"Error: {e:?}\\r\"); }\npub fn phantom(b: u8) -> u8 { b }\n",
        ),
        // keel's own next.rs shape: a char literal, not a lifetime.
        (
            "livechar",
            "pub fn escape(c: char) -> &'static str {\n    match c {\n        '\\r' => \"\\\\r\",\n        _ => \"\",\n    }\n}\npub fn phantom(b: u8) -> u8 { b }\n",
        ),
        // A lifetime really is a lifetime.
        (
            "livelifetime",
            "pub fn borrow<'a>(s: &'a str) -> &'a str { s }\npub fn phantom(b: u8) -> u8 { b }\n",
        ),
        // `for r in ...` does not open a raw string.
        (
            "livefor",
            "pub fn walk(v: Vec<u8>) { for r in v { let _ = r; } }\npub fn phantom(b: u8) -> u8 { b }\n",
        ),
    ] {
        let dir = rust_sandbox(name, source, "pub fn phantom(b: u8) -> u8");
        let (said, code) = keel(&dir, &["check"]);
        assert_eq!(
            code, 0,
            "{name}: a live declaration still holds -- a court that \
             refuses live code is broken, not strict:\n{said}"
        );
    }

    // --- rust: the ghosts that must NOT hold, in every text shape ---
    for (name, source) in [
        (
            "ghostraw",
            "pub const S: &str = r#\"\npub fn phantom(b: u8) -> u8\n\"#;\n",
        ),
        (
            "ghostraw2",
            "pub const S: &str = r##\"\npub fn phantom(b: u8) -> u8\n\"##;\n",
        ),
        (
            "ghostbyte",
            "pub const S: &[u8] = br\"\npub fn phantom(b: u8) -> u8\n\";\n",
        ),
        (
            "ghostplain",
            "pub const S: &str = \"\npub fn phantom(b: u8) -> u8\n\";\n",
        ),
        ("ghostblock", "/*\npub fn phantom(b: u8) -> u8 { b }\n*/\n"),
        (
            "ghostnested",
            "/* /* nested */\npub fn phantom(b: u8) -> u8 { b }\n*/\n",
        ),
    ] {
        let dir = rust_sandbox(name, source, "pub fn phantom(b: u8) -> u8");
        let (said, code) = keel(&dir, &["check"]);
        assert_ne!(code, 0, "{name}: text is not source:\n{said}");
        assert!(
            said.contains("src/toy.rs"),
            "{name}: and the finding names where it looked:\n{said}"
        );
        assert!(
            !said.contains(&format!("{}/src/toy.rs", dir.display())),
            "{name}: by the name a PERSON would use, not the absolute \
             path -- that is a machine talking to itself:\n{said}"
        );
    }

    // --- ruby: LIVE code, in the shapes the corpus taught ---
    for (name, source) in [
        // `net/protocol.rb`: `<<-End` waits for `End`, not for `E`.
        (
            "rblive1",
            "class Toy\n  X = <<-End\n    text\n  End\n\n  def real(a, b)\n    a + b\n  end\nend\n",
        ),
        // `openssl/ssl.rb`: a lowercase, underscored word.
        (
            "rblive2",
            "class Toy\n  PEM = <<-_end_of_pem_\n    text\n  _end_of_pem_\n\n  def real(a, b)\n    a + b\n  end\nend\n",
        ),
        // `bundler/settings.rb`: `$'` is a global, not a quote.
        (
            "rblive3",
            "class Toy\n  def pick(k)\n    @r[$'] = self[k] if k =~ /^local\\./\n  end\n\n  def real(a, b)\n    a + b\n  end\nend\n",
        ),
        // `rbs/test.rb`: `class <<self` is a singleton class.
        (
            "rblive4",
            "class Toy\n  class <<self\n    def other\n    end\n  end\n\n  def real(a, b)\n    a + b\n  end\nend\nself\n",
        ),
        // `<<~\'WORD\'` and `<<~"WORD"` quote the word; racc's own
        // `PARSER_TEXT = <<\'__end_of_file__\'` is that shape.
        (
            "rblive6",
            "class Toy\n  X = <<~'RUBY'\n    text\n  RUBY\n\n  def real(a, b)\n    a + b\n  end\nend\n",
        ),
        // A shovel is a shovel.
        (
            "rblive5",
            "class Toy\n  def push(list, item)\n    list<<Item\n  end\n\n  def real(a, b)\n    a + b\n  end\nend\n",
        ),
    ] {
        let dir = ruby_sandbox(name, source, "def real(a, b)");
        let (said, code) = keel(&dir, &["check"]);
        assert_eq!(code, 0, "{name}: a live ruby method still holds:\n{said}");
    }

    // --- ruby: ghosts ---
    for (name, source) in [
        (
            "rbghost1",
            "class Toy\n  DOC = <<~TEXT\n    приклад:\n    def phantom(a, b)\n  TEXT\nend\n",
        ),
        (
            "rbghost2",
            "class Toy\n  DOC = <<-lower\n    def phantom(a, b)\n  lower\nend\n",
        ),
        (
            "rbghost3",
            "class Toy\n=begin\n  def phantom(a, b)\n=end\nend\n",
        ),
        // The quoted word, racc's own shape.
        (
            "rbghost4",
            "class Toy\n  DOC = <<~'RUBY'\n    приклад:\n    def phantom(a, b)\n  RUBY\nend\n",
        ),
    ] {
        let dir = ruby_sandbox(name, source, "def phantom(a, b)");
        let (said, code) = keel(&dir, &["check"]);
        assert_ne!(code, 0, "{name}: text is not source in ruby:\n{said}");
    }

    // --- elixir: live, and the charlist fence ---
    let dir = elixir_sandbox(
        "exlive",
        "defmodule Toy do\n  @doc \"\"\"\n  Приклад:\n\n      def ghost(a, b)\n\n  \"\"\"\n  def real(a, b), do: a + b\nend\n",
        "def real(a, b)",
    );
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a live elixir def still holds:\n{said}");

    let charlist = format!(
        "defmodule Toy do\n  @doc {q}\n      def phantom(a, b)\n  {q}\n  def real(a, b), do: a + b\nend\n",
        q = "'''"
    );
    let dir = elixir_sandbox("exghost2", &charlist, "def phantom(a, b)");
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(code, 0, "a charlist fence is text too:\n{said}");
}

/// The contract promises that blanking keeps every line where it
/// was. Nothing read that promise back, so nothing held it (review
/// 0043 R-9, mutation H22): the reader is asked here directly.
#[test]
fn blanking_keeps_every_line_where_it_was() {
    for (tongue, source) in [
        (
            "rust",
            "pub fn a() {}\npub const S: &str = r#\"\nghost\nmore\n\"#;\npub fn b() {}\n",
        ),
        (
            "ruby",
            "class Toy\n  DOC = <<~TEXT\n    ghost\n    more\n  TEXT\n  def b\n  end\nend\n",
        ),
        (
            "elixir",
            "defmodule Toy do\n  @doc \"\"\"\n  ghost\n  more\n  \"\"\"\n  def b, do: 1\nend\n",
        ),
    ] {
        let bare = keel::holding::strip_for_test(source, tongue);
        assert_eq!(
            bare.lines().count(),
            source.lines().count(),
            "{tongue}: a blanked text leaves its lines behind, so a \
             line number still counts from the top:\n{bare}"
        );
    }
}

fn contract(dir: &Path, module: &str, export: &str) {
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        format!("---\nmodule: {module}\nexports:\n  - \"{export}\"\n---\n\nТіло контракту.\n"),
    )
    .unwrap();
}

fn rust_sandbox(name: &str, source: &str, export: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub mod toy;\n").unwrap();
    std::fs::write(dir.join("src/toy.rs"), source).unwrap();
    contract(&dir, "crate::toy", export);
    dir
}

fn ruby_sandbox(name: &str, source: &str, export: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"ruby\"\n").unwrap();
    std::fs::write(dir.join("lib/toy.rb"), source).unwrap();
    contract(&dir, "Toy", export);
    dir
}

fn elixir_sandbox(name: &str, source: &str, export: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"elixir\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("mix.exs"),
        "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\", elixir: \"~> 1.14\"]\n  def application, do: []\nend\n",
    )
    .unwrap();
    std::fs::write(dir.join("lib/toy.ex"), source).unwrap();
    contract(&dir, "Toy", export);
    dir
}
