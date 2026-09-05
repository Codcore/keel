//! Scenario test of wave 0046: a javascript contract holds its form.
//!
//! No node needed: the form court reads source, in the layouts node
//! keeps a module in -- `src/toy.ts`, `src/toy.js`, `src/toy/index.*`,
//! `toy.*` -- with a fourth comment reader that knows template
//! literals and `'…'` strings.
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
        "lang = \"uk\"\nadapter = \"javascript\"\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("keel/contracts/toy.md"),
        format!("---\nmodule: {module}\nexports:\n  - \"{export}\"\n---\n\nТіло контракту.\n"),
    )
    .unwrap();
    dir
}

/// proves: a-javascript-contract-holds-its-form@e5002f
#[test]
fn a_javascript_contract_holds_its_form() {
    // TypeScript in the src layout, with the ghosts a JS file can
    // hold: a template literal across lines, a block comment, a `'…'`
    // string.
    let dir = sandbox(
        "jshold",
        "toy.bar",
        "export function works(a: number, b: number): number",
    );
    std::fs::create_dir_all(dir.join("src/toy")).unwrap();
    std::fs::write(
        dir.join("src/toy/bar.ts"),
        "const doc = `\n  export function ghost(a: number): number\n`;\n/* export function commented(): void */\nconst s = 'export function quoted(): void';\nexport function works(a: number, b: number): number {\n  return a + b;\n}\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(code, 0, "a signature found in the module holds:\n{said}");
    assert!(
        said.contains("сигнатур звірено: 1"),
        "and was compared:\n{said}"
    );

    // Every layout node keeps a module in, each one found.
    for (name, layout, module) in [
        ("jsts", "src/toy.ts", "toy"),
        ("jsjs", "src/toy.js", "toy"),
        ("jsindex", "src/toy/index.js", "toy"),
        ("jsroot", "toy.js", "toy"),
        ("jsmjs", "src/toy.mjs", "toy"),
    ] {
        let dir = sandbox(name, module, "export function works()");
        let path = dir.join(layout);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "export function works() {\n  return true;\n}\n").unwrap();
        let (said, code) = keel(&dir, &["check"]);
        assert_eq!(
            code, 0,
            "{name}: the layout {layout} is node's own:\n{said}"
        );
    }

    // Typed BEFORE plain (review 0046 R-8, mutation M03): where both
    // `src/toy.ts` and `src/toy.js` exist, the typed one is the
    // module compared, as the contract promises.
    let dir = sandbox("jstsfirst", "toy", "export function works()");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/toy.ts"),
        "export function works() {\n  return true;\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("src/toy.js"),
        "export function other() {\n  return true;\n}\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "with both beside each other, the typed file is the one read:\n{said}"
    );

    // Ghosts do not hold, in each shape of text.
    for (name, source) in [
        (
            "jsghost5",
            "const s = \"export function ghost(a)\";\nexport function works() {}\n",
        ),
        (
            "jsghost1",
            "const doc = `\n  export function ghost(a)\n`;\nexport function works() {}\n",
        ),
        (
            "jsghost2",
            "/*\n  export function ghost(a)\n*/\nexport function works() {}\n",
        ),
        (
            "jsghost3",
            "// export function ghost(a)\nexport function works() {}\n",
        ),
        (
            "jsghost4",
            "const s = 'export function ghost(a)';\nexport function works() {}\n",
        ),
    ] {
        let dir = sandbox(name, "toy", "export function ghost(a)");
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::write(dir.join("src/toy.js"), source).unwrap();
        let (said, code) = keel(&dir, &["check"]);
        assert_ne!(code, 0, "{name}: text is not source:\n{said}");
        assert!(
            said.contains("src/toy.js"),
            "{name}: and the finding names where it looked:\n{said}"
        );
    }

    // A live declaration beside an apostrophe in a comment or a
    // string still holds -- a `'` is not a lifetime here, and it does
    // not swallow the file.
    let dir = sandbox("jsapos", "toy", "export function works()");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/toy.js"),
        "// don't worry\nconst greeting = \"it's fine\";\nconst re = /don't/;\nexport function works() {\n  return 'yes';\n}\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "an apostrophe does not eat the declaration after it:\n{said}"
    );

    // An apostrophe in a REGEX LITERAL on the same line as the
    // declaration: the old reader dropped the rest of the line with
    // the string it thought had opened (review 0046 R-7). Now the
    // quote is put back as code and the declaration after it is read.
    let dir = sandbox("jsregexline", "toy", "export function works()");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/toy.js"),
        "const re = /don't/; export function works() {\n  return re.test('x');\n}\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_eq!(
        code, 0,
        "a quote that never closes on its line is code, and so is the \
         rest of the line:\n{said}"
    );

    // Diverged -- named as diverged (review 0046 R-9): the unit is
    // there under its name, and its signature is not the promised
    // one; that is not "no such unit".
    let dir = sandbox(
        "jsdiverged",
        "toy",
        "export function works(a: number, b: number): number",
    );
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(
        dir.join("src/toy.ts"),
        "export function works(a: number): number {\n  return a;\n}\n",
    )
    .unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(code, 0, "a diverged signature does not hold:\n{said}");
    assert!(
        said.contains("не сходиться") && said.contains("\"works\""),
        "and is named as diverged, by the unit's name:\n{said}"
    );
    assert!(
        !said.contains("такої одиниці"),
        "not as a unit that is not there:\n{said}"
    );

    // Missing: every path that was tried is named.
    let dir = sandbox("jsmissing", "toy.gone", "export function works()");
    std::fs::create_dir_all(dir.join("src/toy")).unwrap();
    let (said, code) = keel(&dir, &["check"]);
    assert_ne!(code, 0, "a module that is not there does not hold:\n{said}");
    assert!(
        said.contains("src/toy/gone.ts") && said.contains("src/toy/gone.js"),
        "and the paths it looked along are named:\n{said}"
    );
}
