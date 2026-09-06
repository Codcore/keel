//! Tags of tests (§5.5, §7.5; contract tool-tags): the line
//! `proves: <scenario>@<revision>` in a comment before a test is the
//! record whose revision a wave holds. This module only parses --
//! which files to read is the adapter's knowledge, and nothing here
//! runs or writes.

use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::{Path, PathBuf};

pub struct TestTag {
    pub file: PathBuf,
    pub test: String,
    pub scenario: String,
    pub rev: String,
}

/// Reads the named test files and collects the tags. A tag with no
/// test function following it is a refusal by name: a record that
/// holds nothing is worse than none.
pub fn scan(files: &[PathBuf]) -> Result<Vec<TestTag>, Refusal> {
    let mut out = Vec::new();
    for file in files {
        let text = read(file)?;
        out.extend(scan_text(file, &text)?);
    }
    Ok(out)
}

/// The same parse from a ready string -- for texts that do not lie
/// on disk, such as a test file at the fork point read out of git
/// (§7.15).
pub fn scan_text(file: &Path, text: &str) -> Result<Vec<TestTag>, Refusal> {
    let mut out = Vec::new();
    // A byte-order mark is not the first character of a tag: it hid a
    // first-line tag in silence (global review 2026-09-06, bugs R-26).
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    // Rust: the contents of string, raw-string and char literals are
    // blanked before the lines are read -- a `// proves:` line inside
    // an `r#"…"#` fixture was a tag over the next fn (bugs R-15). The
    // comments stay, because the tags live in them; the newlines
    // stay, because the lines are what is read.
    let blanked;
    let text = if file.extension().and_then(|e| e.to_str()) == Some("rs") {
        blanked = blank_rust_literals(text);
        blanked.as_str()
    } else {
        text
    };
    {
        let marks = marks(file);
        let declares = declares(file);
        // By the file's own name, not by its comment marks: `#` opens
        // a comment in ruby AND in elixir, so comparing the marks put
        // every ruby file down the elixir road (caught by ruby's own
        // courts the moment the third tongue landed).
        let elixir = matches!(
            file.extension().and_then(|e| e.to_str()),
            Some("ex") | Some("exs")
        );
        // Python's docstring is elixir's fence, and a test inside
        // a class is named with the class in front -- as ExUnit's
        // `describe` names, only the block's border is INDENTATION
        // and not `end` (wave 0045).
        let python = file.extension().and_then(|e| e.to_str()) == Some("py");
        // node names a test by a STRING, as ExUnit does -- `test('it
        // works', …)`, `it("…")`, or in backticks -- and by its bare
        // name even inside `describe` (wave 0046).
        let javascript = matches!(
            file.extension().and_then(|e| e.to_str()),
            Some("js") | Some("mjs") | Some("cjs") | Some("ts") | Some("mts")
        );
        // rspec names an example by its FULL DESCRIPTION: every
        // `describe`/`context` above it, joined the way rspec joins
        // them, then the example's own string (wave 0047). A spec
        // file is told by its name -- `_spec.rb` -- and read with a
        // stack of groups by `do…end` depth, elixir's mechanism with
        // python's stack.
        let spec = file
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with("_spec.rb"));
        let mut groups: Vec<(SpecGroup, usize)> = Vec::new();
        let mut heredoc_end: Option<String> = None;
        // A STACK of classes, because they nest: pytest names a
        // method `TestOuter::TestInner::test_x` (review 0045 R-10).
        let mut classes: Vec<(String, usize)> = Vec::new();
        // A decorator may run over several lines -- black writes
        // `@pytest.mark.parametrize(` and closes the parenthesis
        // three lines down (review 0045 R-4). Its continuation lines
        // stand between a tag and its `def` as the first line does.
        let mut decorating: i32 = 0;
        let mut pending: Option<(String, String)> = None;
        let mut describing: Option<(String, usize)> = None;
        let mut depth: usize = 0;
        let mut heredoc = false;
        for line in text.lines() {
            let trimmed = line.trim();
            // `@doc """ … """` with an example inside is the most
            // ordinary thing an elixir file contains, and an example
            // is written in the language it documents. A line reading
            // `test "an example" is the shape we use` inside one is
            // prose: it declares nothing, and it must not orphan the
            // tag standing above the heredoc (measured -- a legal
            // file `mix test` runs green was refused). Counted, not
            // flagged, because a heredoc may open and close on one
            // line.
            if spec && let Some(terminator) = &heredoc_end {
                // Inside a heredoc: the terminator closes it, and
                // nothing in between is code (review 0047 R-2).
                if trimmed == terminator {
                    heredoc_end = None;
                }
                continue;
            }
            if elixir || python {
                let fences = trimmed.matches("\"\"\"").count() + trimmed.matches("'''").count();
                let was = heredoc;
                if fences % 2 == 1 {
                    heredoc = !heredoc;
                }
                if was || heredoc {
                    continue;
                }
            }
            if let Some((scenario, rev)) = tag_in(trimmed, marks) {
                if let Some((held, held_rev)) = pending.take() {
                    return Err(dangling(file, &held, &held_rev));
                }
                // The record's shape is §5.2's: 4-6 hex characters --
                // a crooked record refuses as itself, not as a
                // dressed-up staleness (review R-8).
                if !(4..=6).contains(&rev.len()) || !rev.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(Refusal {
                        file: file.to_path_buf(),
                        reason: ta("tags-bad-rev", targs!("scenario" => scenario, "rev" => rev)),
                        instead: t("tags-bad-rev-instead"),
                    });
                }
                pending = Some((scenario, rev));
                continue;
            }
            if python {
                // The decorator's parentheses are counted in CODE: a
                // `(` inside one of its strings (`["f(x", "g(y"]`) is
                // text, and counting it left the tag with "no test
                // function right after it" (bugs R-20).
                if decorating > 0 {
                    decorating += parens(&blank_quoted(trimmed));
                    continue;
                }
                if trimmed.starts_with('@') {
                    decorating = parens(&blank_quoted(trimmed)).max(0);
                    continue;
                }
            }
            let declared = if spec {
                // The line's CODE: a trailing `# …` outside quotes is
                // not code, and a `do` in it opens nothing (review
                // 0047 R-2).
                let code = ruby_code(trimmed);
                // A heredoc's body is text, and its `end`s are words
                // of the text: skip to the terminator.
                if let Some(terminator) = ruby_heredoc(&code) {
                    heredoc_end = Some(terminator);
                }
                // Depth is counted over the code with its strings
                // blanked: `it "syncs end-to-end"` and `expect("the
                // end")` carry the word `end` as text, and counting
                // it closed the group early (bugs R-9).
                let (opens, ends) = ruby_depth(&blank_quoted(&code));
                let mut named: Option<String> = None;
                if let Some(group) = spec_group(&code, &groups) {
                    groups.push((group, depth));
                } else if pending.is_some() {
                    match spec_example(&code) {
                        Some(SpecExample::Named(name)) => {
                            let (scenario, rev) = pending.clone().unwrap();
                            if let Some((bad, what)) = groups.iter().find_map(|(g, _)| match g {
                                SpecGroup::Shared => Some(("tags-spec-shared", String::new())),
                                SpecGroup::Named {
                                    fault: Some(what), ..
                                } => Some(("tags-spec-nonliteral", what.clone())),
                                _ => None,
                            }) {
                                return Err(spec_refusal(file, bad, &scenario, &rev, &what));
                            }
                            named = Some(spec_full_description(&groups, &name));
                        }
                        Some(SpecExample::Dynamic) => {
                            let (scenario, rev) = pending.take().unwrap();
                            return Err(spec_refusal(
                                file,
                                "tags-spec-dynamic",
                                &scenario,
                                &rev,
                                "",
                            ));
                        }
                        Some(SpecExample::NoName) => {
                            let (scenario, rev) = pending.take().unwrap();
                            return Err(spec_refusal(
                                file,
                                "tags-spec-one-liner",
                                &scenario,
                                &rev,
                                "",
                            ));
                        }
                        None => {}
                    }
                }
                // Depth after this line: what it opened, less what it
                // closed -- and every group opened at or above the new
                // depth is closed with it.
                depth = (depth + opens).saturating_sub(ends);
                while groups.last().is_some_and(|(_, opened)| *opened >= depth) {
                    groups.pop();
                }
                named
            } else if elixir {
                // A comment is not code: `# TODO: decide what to do`
                // opened a block and gave the next test a group it
                // did not have (bugs R-10).
                if trimmed.starts_with('#') {
                    None
                } else {
                    // A `describe` opens a group whose name ExUnit puts
                    // in front of every test inside it; `end` closes the
                    // innermost block, and only a describe's own end
                    // clears the group -- so the depth is counted.
                    if let Some(group) = describe_name(trimmed) {
                        describing = Some((group, depth));
                    }
                    // `end` as a WORD: `endpoint = …` starts with the
                    // letters and closes nothing (bugs R-10).
                    let first_word: String = trimmed
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if first_word == "end" {
                        // The depth recorded when the block opened is
                        // the depth OUTSIDE it; its own `do` raised the
                        // count by one, so the `end` that closes it sits
                        // one deeper. Off by that one, the group leaked to
                        // the end of the module and named every test after
                        // it wrongly -- and a scenario was called proven by
                        // a test that had just failed (review 0042 R-1).
                        if let Some((_, opened)) = &describing
                            && depth == *opened + 1
                        {
                            describing = None;
                        }
                        depth = depth.saturating_sub(1);
                    } else if trimmed.ends_with(" do")
                        || trimmed.ends_with(" do:")
                        || trimmed == "do"
                    {
                        depth += 1;
                    }
                    test_name(trimmed).map(|name| match &describing {
                        Some((group, _)) => format!("{group} {name}"),
                        None => name,
                    })
                }
            } else if javascript {
                match js_call(trimmed) {
                    Some(JsCall::Named(name)) => Some(name),
                    // A name node builds at run time, or a subtest
                    // that cannot run without its parent: neither
                    // is a declaration this tag can hold, and the
                    // refusal says WHICH, not "no test function"
                    // (review 0046 R-3).
                    Some(JsCall::Dynamic) if pending.is_some() => {
                        let (scenario, rev) = pending.take().unwrap();
                        return Err(js_refusal(file, "tags-js-dynamic", &scenario, &rev));
                    }
                    Some(JsCall::Subtest) if pending.is_some() => {
                        let (scenario, rev) = pending.take().unwrap();
                        return Err(js_refusal(file, "tags-js-subtest", &scenario, &rev));
                    }
                    _ => None,
                }
            } else if python {
                // A class opens a group at its own indentation and
                // holds it while the lines below sit deeper; a line
                // of CODE back at that indentation or shallower
                // closes it -- and every class opened deeper than it
                // with it. A comment is not code: python lets one sit
                // in column 0 inside a class, and reading it as the
                // block's end lost the method's class (review 0045
                // R-9).
                let indent = line.len() - line.trim_start().len();
                if !trimmed.is_empty() && !trimmed.starts_with('#') {
                    while classes.last().is_some_and(|(_, opened)| indent <= *opened) {
                        classes.pop();
                    }
                }
                if let Some(name) = class_name(trimmed) {
                    classes.push((name, indent));
                    None
                } else {
                    fn_name(trimmed, declares).map(|name| {
                        let mut full: Vec<&str> = classes.iter().map(|(g, _)| g.as_str()).collect();
                        full.push(&name);
                        full.join("::")
                    })
                }
            } else {
                fn_name(trimmed, declares)
            };
            if let Some(name) = declared {
                if let Some((scenario, rev)) = pending.take() {
                    out.push(TestTag {
                        file: file.to_path_buf(),
                        test: name,
                        scenario,
                        rev,
                    });
                }
                continue;
            }
            if pending.is_some() && !stands_between(trimmed, elixir || python) {
                let (scenario, rev) = pending.take().unwrap();
                return Err(dangling(file, &scenario, &rev));
            }
        }
        if let Some((scenario, rev)) = pending {
            return Err(dangling(file, &scenario, &rev));
        }
    }
    Ok(out)
}

/// What may stand between a tag and the declaration it holds. Blank
/// lines and doc lines always may. Rust's `#[test]` rides in on the
/// `#`; elixir writes its attributes with `@`, and `@tag :slow` --
/// an everyday ExUnit line -- orphaned the tag on a file mix runs
/// green (review 0042 R-7). Anything else is code, and code between
/// the two means the tag holds nothing.
fn stands_between(trimmed: &str, elixir: bool) -> bool {
    trimmed.is_empty()
        || trimmed.starts_with("///")
        || trimmed.starts_with("//")
        || trimmed.starts_with('#')
        || (elixir && trimmed.starts_with('@'))
}

/// Which marks open a comment in this file -- the file's own name
/// answers, not the project's config (review 0038 R-3). `#` is a
/// comment in ruby and the fence of a raw string in Rust, so a Rust
/// probe that builds a ruby fixture is still a Rust file: reading
/// `#` there turned a line inside `r#"..."#` into a tag and reddened
/// projects that had done nothing.
fn marks(file: &Path) -> &'static [&'static str] {
    match file.extension().and_then(|e| e.to_str()) {
        Some("rb") | Some("py") => &["#"],
        Some("exs") | Some("ex") => ELIXIR_MARKS,
        _ => &["///", "//!", "//"],
    }
}

/// Elixir writes `#` as ruby does. Which is why the two are told
/// apart by the file's own extension and never by these marks --
/// their DECLARATIONS differ (`def name` there, `test "name" do`
/// here) and the marks do not.
const ELIXIR_MARKS: &[&str] = &["#"];

/// The keyword a test declaration opens with in this file, for the
/// same reason and by the same answer. Elixir is not in this list:
/// its tests are not named by an identifier at all (see `test_name`).
fn declares(file: &Path) -> &'static [&'static str] {
    match file.extension().and_then(|e| e.to_str()) {
        Some("rb") => &["def "],
        // `async def test_…` is an everyday pytest form under
        // pytest-asyncio and anyio (review 0045 R-5).
        Some("py") => &["def ", "async def "],
        _ => &["fn "],
    }
}

/// An ExUnit test is named by a STRING, not an identifier: `test "it
/// works" do`. Rust and Ruby both take the word after a keyword, and
/// this is neither -- the third form the reader had to learn (wave
/// 0042). The name is the string itself, exactly as ExUnit reports it
/// and exactly as `mix test --only` selects it.
pub fn test_name(trimmed: &str) -> Option<String> {
    quoted_after(trimmed, "test ")
}

/// Opening parentheses on this line less closing ones -- a decorator
/// is finished when its count is back to nothing.
fn parens(trimmed: &str) -> i32 {
    trimmed.matches('(').count() as i32 - trimmed.matches(')').count() as i32
}

/// What a javascript line declares, for the tag above it.
pub enum JsCall {
    /// `test('name', …)` in any of the forms below: a test node will
    /// report under this name.
    Named(String),
    /// A test whose name is a template with `${…}` in it -- node
    /// builds the name at run time, and no reader of the source can
    /// know it.
    Dynamic,
    /// `t.test('…')`: a subtest of the enclosing `test`. node runs it
    /// only through its parent -- `--test-name-pattern` over the
    /// subtest's own name runs nothing (measured, review 0046 R-3) --
    /// so a tag over it can hold nothing on its own.
    Subtest,
}

/// node names a test by a STRING: `test('it works', …)`, `it("…")`,
/// the same in backticks -- and in the forms review 0046 R-3 played
/// from node's own documentation: `test.only(`, `test.skip(`,
/// `test.todo(`, their `it.` cousins, `await test(`, `const p =
/// test(`, and any width of space before the parenthesis. A `.skip`
/// or `.todo` is read as the declaration it is: node reports it
/// `# SKIP`/`# TODO`, the court says "did not run", and that is the
/// truth -- not "the tag holds nothing". `describe(` is a group, not
/// a test. Escapes read: `\'` is a quote, `\\` a backslash, `\n` a
/// line break (which node then prints in a form no reader can tell
/// from a literal backslash-n -- the contract names that border).
pub fn js_call(trimmed: &str) -> Option<JsCall> {
    let mut rest = trimmed.trim_start();
    if let Some(after) = rest.strip_prefix("await ") {
        rest = after.trim_start();
    } else if let Some(after) = ["const ", "let ", "var "]
        .iter()
        .find_map(|word| rest.strip_prefix(word))
    {
        // `const p = test(` -- a binding, then the call.
        let after = after
            .trim_start()
            .trim_start_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '$')
            .trim_start();
        rest = after.strip_prefix('=')?.trim_start();
        if let Some(after) = rest.strip_prefix("await ") {
            rest = after.trim_start();
        }
    }
    // The callee: `test` or `it`, bare or with `.only`/`.skip`/
    // `.todo`; `t.test(` and `context.test(` are subtests.
    let callee = match ["test", "it"]
        .iter()
        .find_map(|word| rest.strip_prefix(word))
    {
        Some(after) => after,
        None => {
            let after_object =
                rest.trim_start_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '$');
            return match after_object.strip_prefix(".test") {
                Some(after) if after_object.len() < rest.len() => after
                    .trim_start()
                    .starts_with('(')
                    .then_some(JsCall::Subtest),
                _ => None,
            };
        }
    };
    let callee = match callee.strip_prefix('.') {
        Some(method) => ["only", "skip", "todo"]
            .iter()
            .find_map(|word| method.strip_prefix(word))?,
        None => callee,
    };
    let rest = callee.trim_start().strip_prefix('(')?.trim_start();
    let quote = rest.chars().next()?;
    if !matches!(quote, '\'' | '"' | '`') {
        return None;
    }
    let mut name = String::new();
    let mut chars = rest[1..].chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(next) = chars.next() {
                    name.push(match next {
                        'n' => '\n',
                        't' => '\t',
                        'r' => '\r',
                        other => other,
                    });
                }
            }
            '$' if quote == '`' && chars.peek() == Some(&'{') => return Some(JsCall::Dynamic),
            c if c == quote => return (!name.is_empty()).then_some(JsCall::Named(name)),
            c => name.push(c),
        }
    }
    None
}

/// The name a javascript line declares a test under, if it does.
pub fn js_test_name(trimmed: &str) -> Option<String> {
    match js_call(trimmed)? {
        JsCall::Named(name) => Some(name),
        _ => None,
    }
}

/// A group in a spec file, as rspec names it in a full description.
enum SpecGroup {
    /// `describe "#works"`, `context "when called"`, `RSpec.describe
    /// Toy`, `describe Toy, "with an arg"` -- with the description rspec
    /// composes for it, whether its LAST argument is a module (the
    /// join rule below asks), the module it names if any (for a
    /// `described_class` below it), and the argument the reader could
    /// not read, if one was there.
    Named {
        text: String,
        last_is_module: bool,
        module: Option<String>,
        fault: Option<String>,
    },
    /// `shared_examples`/`shared_context`: examples inside have as
    /// many names as the places that include them.
    Shared,
}

/// An example line in a spec file.
enum SpecExample {
    /// `it "name" do`, `specify 'name' do`, `example "name", :meta do`
    Named(String),
    /// `it "works #{value}"`: a name rspec builds at run time.
    Dynamic,
    /// `it { … }` or `it do … end` -- rspec names it by its location,
    /// and no tag can.
    NoName,
}

/// One argument of `describe`/`it`, as rspec would read it.
enum SpecArg {
    /// A string, or a symbol -- text as rspec prints it.
    Text(String),
    /// A constant that names a module or a class: `Toy`, `Toy::Inner`.
    Module(String),
    /// `described_class`: the nearest module a group above named.
    DescribedClass,
    /// Metadata (`:slow`, `focus: true`) after the description.
    Meta,
    /// A string with `#{…}` in it.
    Dynamic,
    /// Anything the reader cannot know the text of: a variable, a
    /// method, a constant whose value is not its name (`Toy::VERSION`).
    Unknown(String),
}

/// The line's code: a trailing `# comment` outside quotes is not
/// code. `#{…}` inside a double-quoted string is not a comment either.
fn ruby_code(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut quote: Option<char> = None;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match quote {
            Some(q) => {
                out.push(ch);
                if ch == '\\' {
                    if let Some(next) = chars.next() {
                        out.push(next);
                    }
                } else if ch == q {
                    quote = None;
                }
            }
            None => match ch {
                '"' | '\'' => {
                    quote = Some(ch);
                    out.push(ch);
                }
                '#' => break,
                _ => out.push(ch),
            },
        }
    }
    out.trim_end().to_string()
}

/// The terminator of a heredoc this line opens (`<<~TXT`, `<<-TXT`,
/// `<<TXT`, quoted or not), if it opens one.
fn ruby_heredoc(code: &str) -> Option<String> {
    let at = code.find("<<")?;
    let rest = code[at + 2..].trim_start_matches(['~', '-']);
    let rest = rest.trim_start_matches(['\'', '"']);
    let word: String = rest
        .chars()
        .take_while(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || *c == '_')
        .collect();
    (!word.is_empty() && word.chars().next().is_some_and(|c| c.is_ascii_uppercase()))
        .then_some(word)
}

/// What a line of ruby opens and closes: a block `do` (or `do |x|`) at
/// its end, a `def`/`if`/`unless`/`case`/`begin`/`while`/`until`/
/// `class`/`module`/`for` as its first word (or right after `=`), and
/// every `end` word in it. Ruby closes all of these with `end`, and a
/// reader that opened on `do` alone lost a group at every `def` in a
/// spec (review 0047 R-2). An endless `def x = …` opens nothing.
fn ruby_depth(code: &str) -> (usize, usize) {
    let words: Vec<&str> = code
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
        .collect();
    let ends = words.iter().filter(|w| **w == "end").count();
    let mut opens = 0usize;
    let block = code.ends_with(" do") || code == "do" || code.contains(" do |");
    if block {
        opens += 1;
    }
    const OPENERS: [&str; 10] = [
        "def", "if", "unless", "case", "begin", "while", "until", "class", "module", "for",
    ];
    let first = words.first().copied().unwrap_or("");
    let after_assign = code
        .split_once('=')
        .map(|(_, rest)| rest.trim_start())
        .and_then(|rest| rest.split_whitespace().next())
        .filter(|word| OPENERS.contains(word) && !block);
    if OPENERS.contains(&first) {
        let endless = first == "def" && {
            let after = code["def".len()..].trim_start();
            let after = match after.find('(') {
                Some(open) => after[open..]
                    .trim_start_matches(|c| c != ')')
                    .trim_start_matches(')'),
                None => after.trim_start_matches(|c: char| {
                    c.is_alphanumeric() || c == '_' || c == '.' || c == '?' || c == '!'
                }),
            };
            after.trim_start().starts_with('=') && !after.trim_start().starts_with("==")
        };
        // `while … do` and `for … in … do` open once, not twice.
        let loop_with_do = block && matches!(first, "while" | "until" | "for");
        if !(endless || loop_with_do) {
            opens += 1;
        }
    } else if after_assign.is_some() && !code.starts_with("expect") {
        opens += 1;
    }
    (opens, ends)
}

/// The group a spec line opens, if it opens one. Its description is
/// rspec's: the arguments joined by rspec's rule (a module followed by
/// `#…`, `::…` or `.…` -- no space; anything else -- a space);
/// symbols after the first argument and `key: value` pairs are
/// metadata, not description. `described_class` is the nearest module
/// a group above named.
fn spec_group(code: &str, groups: &[(SpecGroup, usize)]) -> Option<SpecGroup> {
    let rest = code.strip_prefix("RSpec.").unwrap_or(code);
    for word in ["shared_examples_for", "shared_examples", "shared_context"] {
        if rest.starts_with(word) && rest[word.len()..].starts_with([' ', '(']) {
            return Some(SpecGroup::Shared);
        }
    }
    let after = [
        "describe",
        "context",
        "xdescribe",
        "xcontext",
        "fdescribe",
        "fcontext",
        "feature",
    ]
    .iter()
    .find_map(|word| {
        rest.strip_prefix(word)
            .filter(|after| after.starts_with([' ', '(']))
    })?;
    let args = after.trim_start_matches(['(', ' ']);
    let mut text = String::new();
    let mut last_is_module = false;
    let mut module: Option<String> = None;
    let mut fault: Option<String> = None;
    for (at, arg) in spec_args(args).into_iter().enumerate() {
        let (part, is_module) = match spec_arg(&arg, at) {
            SpecArg::Text(t) => (t, false),
            SpecArg::Module(name) => {
                module = Some(name.clone());
                (name, true)
            }
            SpecArg::DescribedClass => match groups.iter().rev().find_map(|(g, _)| match g {
                SpecGroup::Named {
                    module: Some(m), ..
                } => Some(m.clone()),
                _ => None,
            }) {
                Some(name) => (name, true),
                None => {
                    fault.get_or_insert(arg.clone());
                    continue;
                }
            },
            SpecArg::Meta => continue,
            SpecArg::Dynamic => {
                fault.get_or_insert(arg.clone());
                continue;
            }
            SpecArg::Unknown(what) => {
                fault.get_or_insert(what);
                continue;
            }
        };
        if text.is_empty() {
            text = part;
        } else {
            let method_like = part.starts_with(['#', '.']) || part.starts_with("::");
            if !(last_is_module && method_like) {
                text.push(' ');
            }
            text.push_str(&part);
        }
        last_is_module = is_module;
    }
    if text.is_empty() && fault.is_none() {
        return None;
    }
    Some(SpecGroup::Named {
        text,
        last_is_module,
        module,
        fault,
    })
}

/// The example a spec line declares, if it declares one.
fn spec_example(code: &str) -> Option<SpecExample> {
    let after = [
        "it", "specify", "example", "xit", "xspecify", "xexample", "fit", "fspecify", "fexample",
        "scenario",
    ]
    .iter()
    .find_map(|word| {
        code.strip_prefix(word)
            .filter(|after| after.starts_with([' ', '(', '{']))
    })?;
    let args = after.trim_start();
    if args.starts_with('{') || args == "do" || args.starts_with("do ") {
        return Some(SpecExample::NoName);
    }
    let args = args.trim_start_matches(['(', ' ']);
    let first = spec_args(args).into_iter().next()?;
    Some(match spec_arg(&first, 0) {
        SpecArg::Text(name) => SpecExample::Named(name),
        SpecArg::Dynamic => SpecExample::Dynamic,
        _ => SpecExample::NoName,
    })
}

/// One argument, read as rspec would: the first is always part of the
/// description; a symbol or a `key: value` after it is metadata.
fn spec_arg(arg: &str, at: usize) -> SpecArg {
    if let Some(text) = spec_string(arg) {
        return if text.contains("#{") {
            SpecArg::Dynamic
        } else {
            SpecArg::Text(text)
        };
    }
    if let Some(symbol) = arg.strip_prefix(':') {
        if at > 0 {
            return SpecArg::Meta;
        }
        let symbol = symbol.trim_matches(['"', '\'']);
        return SpecArg::Text(symbol.to_string());
    }
    if at > 0 && arg.contains(": ") {
        return SpecArg::Meta;
    }
    if arg == "described_class" {
        return SpecArg::DescribedClass;
    }
    let path = arg.trim_end_matches(')');
    let is_constant = path.split("::").all(|seg| {
        !seg.is_empty()
            && seg.chars().next().is_some_and(|c| c.is_ascii_uppercase())
            && seg.chars().all(|c| c.is_alphanumeric() || c == '_')
    });
    if is_constant {
        // `Toy::VERSION` is a value, not a module: rspec prints what
        // it holds, which no reader of the source knows.
        let last = path.rsplit("::").next().unwrap_or(path);
        let all_caps = last
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
            && last.len() > 1;
        return if all_caps {
            SpecArg::Unknown(arg.to_string())
        } else {
            SpecArg::Module(path.to_string())
        };
    }
    SpecArg::Unknown(arg.to_string())
}

/// The arguments of a call, up to the block (` do` or `{`) and split
/// on commas outside quotes; a closing parenthesis is not an
/// argument.
fn spec_args(args: &str) -> Vec<String> {
    let chars: Vec<char> = args.chars().collect();
    let mut body = String::new();
    let mut quote: Option<char> = None;
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if let Some(q) = quote {
            body.push(ch);
            if ch == '\\' && i + 1 < chars.len() {
                body.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if ch == q {
                quote = None;
            }
            i += 1;
            continue;
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                body.push(ch);
            }
            '{' => break,
            ')' => {}
            'd' if chars.get(i + 1) == Some(&'o')
                && (i == 0 || chars[i - 1].is_whitespace() || chars[i - 1] == ')')
                && chars
                    .get(i + 2)
                    .is_none_or(|c| c.is_whitespace() || *c == '|') =>
            {
                break;
            }
            _ => body.push(ch),
        }
        i += 1;
    }
    let mut out: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    let mut chars = body.chars();
    while let Some(ch) = chars.next() {
        match quote {
            Some(q) => {
                current.push(ch);
                if ch == '\\' {
                    if let Some(next) = chars.next() {
                        current.push(next);
                    }
                } else if ch == q {
                    quote = None;
                }
            }
            None => match ch {
                '"' | '\'' => {
                    quote = Some(ch);
                    current.push(ch);
                }
                ',' => {
                    out.push(current.trim().to_string());
                    current.clear();
                }
                _ => current.push(ch),
            },
        }
    }
    if !current.trim().is_empty() {
        out.push(current.trim().to_string());
    }
    out
}

/// A string argument's text, escapes read; None for anything that is
/// not a string.
fn spec_string(arg: &str) -> Option<String> {
    let quote = arg.chars().next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    let mut out = String::new();
    let mut chars = arg[1..].chars();
    while let Some(ch) = chars.next() {
        match ch {
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            c if c == quote => return Some(out),
            c => out.push(c),
        }
    }
    None
}

/// rspec's own join (`Metadata#description_separator`, measured): a
/// group after a group whose LAST argument is a module, when its own
/// description starts with `#`, `::` or `.`, joins without a space
/// (`Toy#works`, `Toy::VERSION`); every other join, the example's own
/// name included, takes a space -- `RSpec.describe "Toy"` then
/// `describe "#works"` is `Toy #works`, and `it "#works"` under `Toy`
/// is `Toy #works` (review 0047 R-1: the first cut dropped the space
/// before every `#…`, and the gate said "did not run" over ordinary
/// files).
fn spec_full_description(groups: &[(SpecGroup, usize)], name: &str) -> String {
    let mut out = String::new();
    let mut last_is_module = false;
    for (group, _) in groups {
        if let SpecGroup::Named {
            text,
            last_is_module: module_last,
            ..
        } = group
        {
            if text.is_empty() {
                continue;
            }
            if !out.is_empty()
                && !(last_is_module && (text.starts_with(['#', '.']) || text.starts_with("::")))
            {
                out.push(' ');
            }
            out.push_str(text);
            last_is_module = *module_last;
        }
    }
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(name);
    out
}

fn spec_refusal(file: &Path, key: &str, scenario: &str, rev: &str, what: &str) -> Refusal {
    Refusal {
        file: file.to_path_buf(),
        reason: ta(
            key,
            targs!("scenario" => scenario.to_string(), "rev" => rev.to_string(), "what" => what.to_string()),
        ),
        instead: t(&format!("{key}-instead")),
    }
}

fn js_refusal(file: &Path, key: &str, scenario: &str, rev: &str) -> Refusal {
    Refusal {
        file: file.to_path_buf(),
        reason: ta(
            key,
            targs!("scenario" => scenario.to_string(), "rev" => rev.to_string()),
        ),
        instead: t(&format!("{key}-instead")),
    }
}

/// The name a `class Something:` line opens, for python -- the
/// word between `class` and the parenthesis or colon, which is what
/// pytest puts in front of every method inside it.
pub fn class_name(trimmed: &str) -> Option<String> {
    let rest = trimmed.strip_prefix("class ")?;
    let name: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    (!name.is_empty()).then_some(name)
}

/// The name a `describe "..." do` block opens. ExUnit prefixes every
/// test inside it, so `mix test --only 'test:test <bare name>'`
/// matches NOTHING there -- measured, and it is why the reader tracks
/// the block rather than naming a border (wave 0042).
pub fn describe_name(trimmed: &str) -> Option<String> {
    quoted_after(trimmed, "describe ")
}

fn quoted_after(trimmed: &str, word: &str) -> Option<String> {
    let rest = trimmed.strip_prefix(word)?.trim_start();
    let rest = rest.strip_prefix('"')?;
    let (name, tail) = rest.split_once('"')?;
    // `do` may sit on the line or open a block on the next; what must
    // not follow is more of the string.
    if name.is_empty() {
        return None;
    }
    let tail = tail.trim();
    if tail.is_empty() || tail.starts_with("do") || tail.starts_with(',') {
        Some(name.to_string())
    } else {
        None
    }
}

/// `proves: <scenario>@<rev>` inside a comment line; words after the
/// record are the author's -- only the record is read.
fn tag_in(trimmed: &str, marks: &[&str]) -> Option<(String, String)> {
    // `#[test]` is not a comment in Rust, and it never becomes one
    // here: what follows must read `proves: `, and an attribute does
    // not.
    let comment = marks.iter().find_map(|mark| trimmed.strip_prefix(mark))?;
    let rest = comment.trim_start().strip_prefix("proves: ")?;
    let token = rest.split_whitespace().next()?;
    let (scenario, rev) = token.split_once('@')?;
    if scenario.is_empty() || rev.is_empty() {
        return None;
    }
    Some((scenario.to_string(), rev.to_string()))
}

fn fn_name(trimmed: &str, declares: &[&str]) -> Option<String> {
    // `fn` in Rust, `def` in Ruby -- whichever this file writes.
    let after = declares
        .iter()
        .find_map(|word| trimmed.strip_prefix(word))
        .or_else(|| {
            declares
                .contains(&"fn ")
                .then(|| trimmed.find(" fn ").map(|i| &trimmed[i + " fn ".len()..]))?
        })?;
    // Letters beyond ASCII are letters: `def test_ünïcode` is what
    // pytest and minitest name the test, and cutting it to `test_`
    // selected nothing (bugs R-19).
    let name: String = after
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() { None } else { Some(name) }
}

/// The line with the CONTENTS of its quoted strings blanked -- the
/// quotes stay, the escapes are honoured -- so a reader that counts
/// words or parentheses counts code and not text.
fn blank_quoted(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut quote: Option<char> = None;
    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        match quote {
            Some(q) => {
                if ch == '\\' {
                    out.push(' ');
                    if chars.next().is_some() {
                        out.push(' ');
                    }
                } else if ch == q {
                    quote = None;
                    out.push(ch);
                } else {
                    out.push(' ');
                }
            }
            None => {
                if ch == '"' || ch == '\'' {
                    quote = Some(ch);
                }
                out.push(ch);
            }
        }
    }
    out
}

/// Rust text with the contents of its literals blanked and everything
/// else -- comments above all, where the tags live -- kept as it is.
/// Strings with their escapes, raw strings with their hashes, byte
/// strings, char literals; a lifetime (`'a`) is not a literal. The
/// newlines inside a literal stay, so a line still counts from the
/// top.
fn blank_rust_literals(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;
    let blank = |out: &mut String, c: char| out.push(if c == '\n' { '\n' } else { ' ' });
    while i < n {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        // Comments are kept whole: a quote inside one opens nothing.
        if c == '/' && next == Some('/') {
            while i < n && chars[i] != '\n' {
                out.push(chars[i]);
                i += 1;
            }
            continue;
        }
        if c == '/' && next == Some('*') {
            let mut depth = 0usize;
            loop {
                if i >= n {
                    break;
                }
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    out.push('/');
                    out.push('*');
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    out.push('*');
                    out.push('/');
                    i += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    out.push(chars[i]);
                    i += 1;
                }
            }
            continue;
        }
        let word_before = i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_');
        // Raw strings: r"…", r#"…"#, br"…", br#"…"#.
        if !word_before && (c == 'r' || (c == 'b' && next == Some('r'))) {
            let mut j = i + if c == 'b' { 2 } else { 1 };
            let mut hashes = 0usize;
            while chars.get(j) == Some(&'#') {
                hashes += 1;
                j += 1;
            }
            if chars.get(j) == Some(&'"') {
                out.extend(chars[i..=j].iter());
                i = j + 1;
                loop {
                    if i >= n {
                        break;
                    }
                    if chars[i] == '"' && (0..hashes).all(|h| chars.get(i + 1 + h) == Some(&'#')) {
                        out.push('"');
                        for _ in 0..hashes {
                            out.push('#');
                        }
                        i += 1 + hashes;
                        break;
                    }
                    blank(&mut out, chars[i]);
                    i += 1;
                }
                continue;
            }
        }
        // Strings: "…" and b"…", escapes honoured.
        if c == '"' || (c == 'b' && next == Some('"') && !word_before) {
            if c == 'b' {
                out.push('b');
                i += 1;
            }
            out.push('"');
            i += 1;
            while i < n {
                if chars[i] == '\\' {
                    blank(&mut out, chars[i]);
                    if i + 1 < n {
                        blank(&mut out, chars[i + 1]);
                    }
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    out.push('"');
                    i += 1;
                    break;
                }
                blank(&mut out, chars[i]);
                i += 1;
            }
            continue;
        }
        // Char literals ('a', '\n', '\'') against lifetimes ('a).
        if c == '\'' {
            let literal_end = if next == Some('\\') {
                (i + 2..n.min(i + 12)).find(|&k| chars[k] == '\'')
            } else if chars.get(i + 2) == Some(&'\'') && next.is_some_and(|ch| ch != '\'') {
                Some(i + 2)
            } else {
                None
            };
            if let Some(end) = literal_end {
                out.push('\'');
                for &ch in &chars[i + 1..end] {
                    blank(&mut out, ch);
                }
                out.push('\'');
                i = end + 1;
                continue;
            }
        }
        out.push(c);
        i += 1;
    }
    out
}

fn dangling(file: &Path, scenario: &str, rev: &str) -> Refusal {
    Refusal {
        file: file.to_path_buf(),
        reason: ta(
            "tags-dangling",
            targs!("scenario" => scenario.to_string(), "rev" => rev.to_string()),
        ),
        instead: t("tags-dangling-instead"),
    }
}

fn read(path: &Path) -> Result<String, Refusal> {
    std::fs::read_to_string(path).map_err(|e| Refusal {
        file: path.to_path_buf(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })
}
