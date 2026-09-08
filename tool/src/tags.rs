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
    /// The line of the declaration the tag holds, counted from 1 --
    /// what a runner that selects by `file:line` is handed (wave
    /// 0051: mix excludes a test with letters beyond ASCII under
    /// `--only`, and runs it by its line).
    pub line: usize,
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
        // Rails declares a test by a STRING inside an ordinary ruby
        // file: `test "greets a user" do` in an
        // `ActiveSupport::TestCase`. Measured on a real application
        // before wave 0059: the reader knew `def test_…` alone, so a
        // Rails promise could not be proven at all -- not a corner
        // but the framework's everyday style. The third reading of
        // this tongue rides in the same file as the first, because
        // the DECLARATION is what differs and the file cannot say
        // which of the two it uses.
        let minitest = file.extension().and_then(|e| e.to_str()) == Some("rb") && !spec;
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
        for (at, line) in text.lines().enumerate() {
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
                // The line's CODE: a comment is not code -- a whole
                // line of it (`# TODO: decide what to do` opened a
                // block and gave the next test a group it did not
                // have, bugs R-10) and a trailing one alike (`x = 1 #
                // then do` did the same, review 0051 R-1) -- so the
                // comment is cut off outside quotes by the hand ruby
                // uses, and the words are counted over the code with
                // its strings blanked.
                let code = ruby_code(trimmed);
                if code.is_empty() {
                    None
                } else {
                    // A `describe` opens a group whose name ExUnit puts
                    // in front of every test inside it; `end` closes the
                    // innermost block, and only a describe's own end
                    // clears the group -- so the depth is counted.
                    if let Some(group) = describe_name(&code) {
                        describing = Some((group, depth));
                    }
                    let (opens, ends) = elixir_depth(&blank_quoted(&code));
                    depth += opens;
                    for _ in 0..ends {
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
                    }
                    test_name(&code).map(|name| match &describing {
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
            } else if minitest {
                // `def test_…` first, the Rails string second: a file
                // may hold both, and a method declaration is never a
                // `test "…" do` line.
                let code = ruby_code(trimmed);
                let declared_here = fn_name(trimmed, declares);
                // A name built at RUN TIME is a name nobody can read
                // from the source, and the tag over it holds nothing.
                // The second reading of this tongue refuses that
                // aloud and the contract records it; the third one
                // used to hand the person a wrong trail instead --
                // `-n test_greets_#{…}`, then "the run did not
                // execute the test", then advice to check the tag,
                // which was never the fault (review 0059 R-8).
                if pending.is_some()
                    && declared_here.is_none()
                    && let Some(name) = rails_test_name(&code)
                    && name.contains("#{")
                {
                    let (scenario, rev) = pending.take().unwrap();
                    return Err(js_refusal(file, "tags-rails-dynamic", &scenario, &rev));
                }
                declared_here.or_else(|| rails_test_name(&code))
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
                        line: at + 1,
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

/// The quote marks ruby allows around a string: rubocop's default
/// (Style/StringLiterals) asks for the single ones where there is no
/// interpolation, so a great many Rails projects hold `test 'greets'
/// do` (measured by the author 2026-09-07: with double quotes alone
/// that line read as no test at all).
const QUOTES: &[char] = &['"', '\''];

/// Rails names a test by a STRING and builds the method itself:
/// `test "greets a user" do` becomes `test_greets_a_user`, which is
/// the name minitest reports and the name `-n` selects. This reader
/// returns THAT name, not the string, because a tag must carry the
/// name the runner answers to.
///
/// The rule is ActiveSupport's own: `"test_" + name.gsub(/\s+/, "_")`
/// -- every RUN of whitespace becomes one underscore, and nothing
/// else is touched (a name with non-ASCII letters keeps them, as
/// wave 0051 measured for `-n`).
pub fn rails_test_name(code: &str) -> Option<String> {
    // Both call forms ActiveSupport takes: `test "x" do` and
    // `test("x") do` (review 0059 R-10 measured the second read as no
    // test at all). The parenthesis is stripped here, and the tail
    // after the string is then `) do`, which the reader below
    // already allows.
    let head = code.trim_start();
    let raw = match head.strip_prefix("test(") {
        Some(rest) => quoted_after_marks(
            &format!("test {}", rest.trim_start()),
            "test ",
            QUOTES,
            true,
        )?,
        None => quoted_after_marks(head, "test ", QUOTES, true)?,
    };
    // The escapes ruby resolves BEFORE ActiveSupport ever sees the
    // string: `test "tab\tname"` is one word, a tab and another, so
    // the method is `test_tab_name` and not `test_tabtname` (review
    // 0059 R-10 measured that difference as "the run did not execute
    // the test"). Only the whitespace ones matter here -- they are
    // the ones the gsub below then folds; everything else already
    // stands for itself.
    let name = raw
        .replace("\\t", "\t")
        .replace("\\n", "\n")
        .replace("\\r", "\r")
        .replace("\\\"", "\"")
        .replace("\\'", "'")
        .replace("\\\\", "\\");
    if name.is_empty() {
        return None;
    }
    // `test_` plus the name with every RUN of whitespace as one
    // underscore -- ActiveSupport's own `gsub(/\s+/, "_")`, and
    // whitespace means what ruby's `\s` means: a tab is not a
    // literal `t` (review 0059 R-10 measured `tab\tname` reading as
    // `test_tabtname` while the runner had built `test_tab_name`). A
    // name of nothing but spaces is `test__` there, and so it is
    // here: the reader's business is to say what the runner will
    // answer to, not to judge the name.
    let mut out = String::from("test_");
    let mut space = false;
    for ch in name.chars() {
        if ch.is_whitespace() {
            space = true;
            continue;
        }
        if space {
            out.push('_');
            space = false;
        }
        out.push(ch);
    }
    if space {
        out.push('_');
    }
    Some(out)
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

/// What a line of elixir opens and closes: a block `do` at its end
/// and every anonymous `fn` in it open one level each; every `end`
/// WORD closes one. Elixir closes both with `end`, and a reader that
/// opened on `do` alone lost the group at every multi-line `fn ->
/// … end` (review 0051 R-1) -- while `endpoint = …` starts with the
/// letters and closes nothing (bugs R-10), and `fn x -> x end` on one
/// line opens and closes on that line. A `do:` keyword form has no
/// `end` and opens nothing, even where the line breaks right after
/// it. Counted over the code with its strings blanked: `"the end"`
/// is text.
fn elixir_depth(code: &str) -> (usize, usize) {
    let words: Vec<&str> = code
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
        .collect();
    let ends = keyword_ends(code);
    let mut opens = words.iter().filter(|w| **w == "fn").count();
    if code.ends_with(" do") || code == "do" {
        opens += 1;
    }
    (opens, ends)
}

/// How many times `end` stands in the code as the KEYWORD: a whole
/// word, and not the symbol `:end`, the atom `:end`, the hash key
/// `end:`, the method `.end` of a range, or an `@end` variable --
/// each of those is the letters, not the closer (review 0051 R-2
/// found the symbol next to the percent literal it named).
fn keyword_ends(code: &str) -> usize {
    let chars: Vec<char> = code.chars().collect();
    let is_word = |c: char| c.is_alphanumeric() || c == '_';
    let mut count = 0usize;
    let mut i = 0usize;
    while i + 3 <= chars.len() {
        if chars[i] == 'e' && chars[i + 1] == 'n' && chars[i + 2] == 'd' {
            let before = if i > 0 { Some(chars[i - 1]) } else { None };
            let after = chars.get(i + 3).copied();
            let whole = !before.is_some_and(is_word) && !after.is_some_and(is_word);
            let letters = matches!(before, Some('.') | Some(':') | Some('@') | Some('$'))
                || matches!(after, Some('?') | Some('!'))
                || (after == Some(':') && chars.get(i + 4) != Some(&':'));
            if whole && !letters {
                count += 1;
            }
            i += 3;
            continue;
        }
        i += 1;
    }
    count
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
    let ends = keyword_ends(code);
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
    quoted_after_marks(trimmed, word, &['"'], false)
}

/// The same, with the quote marks this tongue allows. ExUnit writes a
/// name in double quotes only (single ones are a charlist there);
/// ruby takes both, and rubocop's default asks for SINGLE ones on a
/// string with no interpolation -- so `test 'greets a user' do` is
/// what a great many Rails projects actually hold. Measured by the
/// author 2026-09-07: with double quotes alone that line read as no
/// test at all, which is the very fault wave 0059 exists to fix,
/// wearing the other spelling.
fn quoted_after_marks(
    trimmed: &str,
    word: &str,
    marks: &[char],
    keep_escapes: bool,
) -> Option<String> {
    let rest = trimmed.strip_prefix(word)?.trim_start();
    let quote = rest.chars().next().filter(|c| marks.contains(c))?;
    let rest = &rest[quote.len_utf8()..];
    // The closing quote is the first UNESCAPED one: `test "it's
    // \"quoted\"" do` is one name, and splitting at the first quote
    // cut it in the middle -- the tag above it then had "no test
    // right after it" and the whole court refused (final review
    // 2026-09-06, bugs R-17; wave 0055). The name is unescaped as
    // ExUnit reads it, since that is the name a `proves:` tag must
    // carry.
    let mut name = String::new();
    let mut chars = rest.char_indices();
    let mut closed: Option<usize> = None;
    while let Some((at, ch)) = chars.next() {
        match ch {
            '\\' => match chars.next() {
                // The backslash is kept where the CALLER resolves the
                // escapes itself: ruby turns `\t` into a tab before
                // ActiveSupport builds the method name, and a reader
                // that dropped the backslash here could no longer
                // tell that letter from this one (review 0059 R-10).
                Some((_, escaped)) => {
                    if keep_escapes {
                        name.push('\\');
                    }
                    name.push(escaped);
                }
                None => return None,
            },
            ch if ch == quote => {
                closed = Some(at);
                break;
            }
            _ => name.push(ch),
        }
    }
    let tail = &rest[closed? + 1..];
    // `do` may sit on the line or open a block on the next; what must
    // not follow is more of the string.
    if name.is_empty() {
        return None;
    }
    let tail = tail.trim();
    // `)` closes a call written with parentheses -- `test("x") do`,
    // the form ActiveSupport takes beside the bare one (review 0059
    // R-10). ExUnit never writes it, so nothing else changes.
    if tail.is_empty() || tail.starts_with("do") || tail.starts_with(',') || tail.starts_with(')') {
        Some(name)
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
/// words or parentheses counts code and not text. Ruby's other
/// spellings of a string are blanked the same way: `%w[end start]`,
/// `%q(end)`, `%i{…}`, `%r<…>`, `%s|…|` -- the word `end` inside one
/// closed a group (review 0051 R-2). Bracket delimiters nest; a
/// literal that runs past the line is a named border, as is the bare
/// `%(…)`, which is modulo as often as a string.
fn blank_quoted(line: &str) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        let (open, close, nests) = if ch == '"' || ch == '\'' {
            (ch, ch, false)
        } else if ch == '%'
            && let Some(delim) = percent_open(&chars, i)
        {
            out.push('%');
            out.push(chars[i + 1]);
            i += 2;
            match delim {
                '(' => ('(', ')', true),
                '[' => ('[', ']', true),
                '{' => ('{', '}', true),
                '<' => ('<', '>', true),
                other => (other, other, false),
            }
        } else {
            out.push(ch);
            i += 1;
            continue;
        };
        out.push(open);
        i += 1;
        let mut depth = 1usize;
        while i < chars.len() {
            let c = chars[i];
            if c == '\\' {
                out.push(' ');
                i += 1;
                if i < chars.len() {
                    out.push(' ');
                    i += 1;
                }
                continue;
            }
            if nests && c == open {
                depth += 1;
            } else if c == close {
                depth -= 1;
                if depth == 0 {
                    out.push(c);
                    i += 1;
                    break;
                }
            }
            out.push(' ');
            i += 1;
        }
    }
    out
}

/// The delimiter of a percent literal opening at `i` -- `%w[`,
/// `%q(`, `%i{`, `%r<`, `%s|` … -- where `%` starts a token and a
/// literal's letter follows it. `a % b` and `a%w` are modulo, and a
/// delimiter that could be a word is none.
fn percent_open(chars: &[char], i: usize) -> Option<char> {
    if i > 0 && (chars[i - 1].is_alphanumeric() || matches!(chars[i - 1], '_' | ')' | ']')) {
        return None;
    }
    let letter = *chars.get(i + 1)?;
    if !"wWqQiIrsx".contains(letter) {
        return None;
    }
    let delim = *chars.get(i + 2)?;
    if delim.is_alphanumeric() || delim == '_' || delim.is_whitespace() {
        return None;
    }
    Some(delim)
}

/// Rust text with the contents of its literals blanked and everything
/// else -- comments above all, where the tags live -- kept as it is:
/// the ONE reader of rust, the form court's own (`holding`), asked to
/// keep the comments. A second copy of it lived here through the
/// first reading of wave 0051 and did not know the C-string prefixes
/// of Rust 1.77, `c"…"` and `cr#"…"#` -- a tag inside one was read as
/// a tag and the real one behind it was lost in silence (review 0051
/// R-3). The newlines inside a literal stay, so a line still counts
/// from the top.
fn blank_rust_literals(text: &str) -> String {
    crate::holding::read_rust(text, crate::holding::Comments::Keep)
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
    match std::fs::read(path) {
        // The bytes are the trouble, not the path: a test file in
        // latin-1 gave "stream did not contain valid UTF-8" with the
        // advice "check the path and access permissions" -- for a
        // fault it did not have (final review 2026-09-06, bugs R-16;
        // wave 0055). The file opened and was read; what it holds is
        // not text this reader can read, and the word says which
        // file and what to do with it.
        Ok(bytes) => String::from_utf8(bytes).map_err(|e| Refusal {
            file: path.to_path_buf(),
            reason: ta(
                "tags-not-utf8",
                targs!("at" => e.utf8_error().valid_up_to() as u64),
            ),
            instead: t("tags-not-utf8-instead"),
        }),
        Err(e) => Err(Refusal {
            file: path.to_path_buf(),
            reason: ta("docs-unreadable", targs!("error" => e.to_string())),
            instead: t("docs-unreadable-instead"),
        }),
    }
}
