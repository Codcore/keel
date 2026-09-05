//! Scenario test of wave 0045: a python contract holds its form.
//!
//! No pytest needed here: the form court reads source, and the layout
//! it reads is python's own -- `src/toy/__init__.py`, `src/toy.py`,
//! `toy/__init__.py`, `toy.py`, and `toy.sub` under any of them.
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

fn sandbox(name: &str, module: &str, export: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(
        dir.join("keel.toml"),
        "lang = \"uk\"\nadapter = \"python\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        format!("---\nmodule: {module}\nexports:\n  - \"{export}\"\n---\n\nТіло контракту.\n"),
    )
    .unwrap();
    dir
}

/// proves: a-python-contract-holds-its-form@d1acbd
#[test]
fn a_python_contract_holds_its_form() {
    // The src layout with a package: `toy.bar` is `src/toy/bar.py`.
    let dir = sandbox("pyhold", "toy.bar", "def works(a: int, b: int) -> int");
    std::fs::create_dir_all(dir.join("src/toy")).unwrap();
    std::fs::write(dir.join("src/toy/__init__.py"), "").unwrap();
    std::fs::write(
        dir.join("src/toy/bar.py"),
        "\"\"\"A module.\n\n    def ghost(a, b)\n\"\"\"\n\n# def commented(x)\ndef works(a: int, b: int) -> int:\n    return a + b\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a signature found in the module holds:\n{said}");
    assert!(
        said.contains("сигнатур звірено: 1"),
        "and was compared:\n{said}"
    );

    // Every layout python keeps a module in, each one found.
    for (name, layout, module) in [
        ("pyflat", "src/toy.py", "toy"),
        ("pyroot", "toy.py", "toy"),
        ("pypkg", "toy/__init__.py", "toy"),
        ("pysrcpkg", "src/toy/__init__.py", "toy"),
    ] {
        let dir = sandbox(name, module, "def works() -> bool");
        let path = dir.join(layout);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "def works() -> bool:\n    return True\n").unwrap();
        let (said, code) = keel(&dir, &["check"]);
        assert_eq!(
            code, 0,
            "{name}: the layout {layout} is python's own:\n{said}"
        );
    }

    // A ghost alive only in a docstring does not hold, in this
    // tongue as in the other three.
    let dir = sandbox("pyghost", "toy", "def ghost(a, b)");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/toy.py"),
        "\"\"\"\n    def ghost(a, b)\n\"\"\"\ndef works():\n    return True\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(code, 0, "a docstring is not source:\n{said}");
    assert!(
        said.contains("src/toy.py"),
        "and the finding names the file it looked in:\n{said}"
    );

    // Diverged: the name is there and the signature is not.
    let dir = sandbox("pydiverged", "toy", "def works(a: int) -> bool");
    std::fs::write(
        dir.join("toy.py"),
        "def works(a: str) -> bool:\n    return True\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(code, 0, "a diverged signature does not hold:\n{said}");
    assert!(
        said.contains("розійш") || said.contains("works"),
        "and is named:\n{said}"
    );

    // Missing: every path that was tried is named.
    let dir = sandbox("pymissing", "toy.gone", "def works()");
    std::fs::create_dir_all(dir.join("src/toy")).unwrap();
    std::fs::write(dir.join("src/toy/__init__.py"), "").unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(code, 0, "a module that is not there does not hold:\n{said}");
    assert!(
        said.contains("src/toy/gone.py"),
        "and the paths it looked along are named:\n{said}"
    );
}

/// Two things review 0045 measured about where the court looks: a
/// name with a leading dot walked OUTSIDE the project (`Path::join`
/// of an absolute piece replaces the root), and when both `toy.py`
/// and `toy/__init__.py` stand, python imports the package -- the
/// court read the file.
///
/// proves: a-python-contract-holds-its-form@d1acbd
#[test]
fn the_court_looks_where_python_does_and_nowhere_else() {
    // R-7: a dotted name that begins with a dot is no module, and
    // certainly not a path on the machine.
    let dir = sandbox("pydot", ".etc.hostname", "def anything()");
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(
        code, 0,
        "a name with an empty segment holds nothing:\n{said}"
    );
    assert!(
        !said.contains("сигнатур звірено: 1"),
        "and nothing outside the project was ever compared:\n{said}"
    );
    let dir = sandbox("pydotdot", "..toy", "def works()");
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(code, 0, "nor does `..toy`:\n{said}");
    assert!(
        said.contains("..toy"),
        "and the finding names what was asked for:\n{said}"
    );

    // R-8: both a file and a package -- the package is what python
    // imports, so the package is what the court reads.
    let dir = sandbox("pyboth", "toy", "def works(a: int) -> bool");
    std::fs::create_dir_all(dir.join("src/toy")).unwrap();
    std::fs::write(
        dir.join("src/toy/__init__.py"),
        "def works(a: int) -> bool:\n    return True\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/toy.py"),
        "def works(a: str) -> bool:\n    return True\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "the package's signature holds, as python would load it:\n{said}"
    );
}
