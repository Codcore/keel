//! Scenario test of wave 0042: an elixir contract holds its form.
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

/// An elixir project whose contract names `module`, with the source
/// put wherever the case wants it.
fn project(name: &str, module: &str, exports: &str, at: Option<(&str, &str)>) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"elixir\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join("test")).unwrap();
    std::fs::write(
        dir.join("mix.exs"),
        "defmodule Toy.MixProject do\n  use Mix.Project\n  def project, do: [app: :toy, version: \"0.1.0\"]\nend\n",
    )
    .unwrap();
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

/// proves: an-elixir-contract-holds-its-form@fd1111 -- the §7.6 court
/// asks the adapter where a module's source lives, and elixir keeps
/// `Toy.Bar` in `lib/toy/bar.ex`. The comment mark is `#`, the same
/// hand ruby needs -- one job, not two alike.
#[test]
fn an_elixir_contract_holds_its_form() {
    // A promise that is there is silence.
    let dir = project(
        "exheld",
        "Toy.Bar",
        "  - \"def works(a, b)\"\n",
        Some((
            "lib/toy/bar.ex",
            "defmodule Toy.Bar do\n  def works(a, b), do: a + b\nend\n",
        )),
    );
    let (said, code) = check(&dir);
    assert_eq!(code, 0, "a held elixir contract is silence:\n{said}");
    assert!(
        said.contains("сигнатур звірено: 1"),
        "and it really was compared:\n{said}"
    );

    // A promise alive only in a comment has vanished -- the contract's
    // own text says so, and `#` is what opens one here.
    let dir = project(
        "excomment",
        "Toy.Bar",
        "  - \"def works(a, b)\"\n",
        Some((
            "lib/toy/bar.ex",
            "defmodule Toy.Bar do\n  # def works(a, b), do: a + b\nend\n",
        )),
    );
    let (said, code) = check(&dir);
    assert_ne!(code, 0, "a commented promise has vanished:\n{said}");
    assert!(said.contains("works"), "and the court names it:\n{said}");

    // A `//` inside a string is a string, not a comment.
    let dir = project(
        "exurl",
        "Toy.Bar",
        "  - \"def works(a, b)\"\n",
        Some((
            "lib/toy/bar.ex",
            "defmodule Toy.Bar do\n  @url \"http://example.com\"\n  def works(a, b), do: a + b\nend\n",
        )),
    );
    let (said, code) = check(&dir);
    assert_eq!(code, 0, "a URL in a string is not a finding:\n{said}");

    // A module nobody can find is a finding naming where it looked.
    let dir = project("exnowhere", "Toy.Gone", "  - \"def works()\"\n", None);
    let (said, code) = check(&dir);
    assert_ne!(code, 0, "a module nobody can find is a finding:\n{said}");
    assert!(
        said.contains("lib/toy/gone.ex"),
        "naming the path it looked along:\n{said}"
    );

    // And the layout is elixir's own: dots make directories, and an
    // acronym stays one word.
    let dir = project(
        "exacronym",
        "Toy.HTTPServer",
        "  - \"def works()\"\n",
        Some((
            "lib/toy/http_server.ex",
            "defmodule Toy.HTTPServer do\n  def works(), do: true\nend\n",
        )),
    );
    let (said, code) = check(&dir);
    assert_eq!(
        code, 0,
        "Toy.HTTPServer lives in lib/toy/http_server.ex:\n{said}"
    );
}
