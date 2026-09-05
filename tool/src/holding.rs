//! Contracts' form court (contract tool-holding; §7.6, §2.7, §2.9):
//! promised signatures compared as collapsed text against the
//! module's source. Green form is not yet meaning (§7.8) -- that
//! gap is the reviewer's; and where there is nothing to compare
//! with, the report says so aloud instead of green. The module runs
//! nothing, builds nothing, writes nothing (§7.10).

use crate::adapter;
use crate::config::{Config, Language};
use crate::docs::{Contract, Wave};
use crate::i18n::{t, ta};
use crate::tags::TestTag;
use crate::targs;
use std::path::Path;

enum Comparability {
    /// The module's source, and the file it was read from: a finding
    /// that says a promise is not in the module must say WHICH file
    /// it looked in, or a person has to guess the layout (wave 0043,
    /// the same lesson review 0038 R-16 taught for a module that is
    /// not there at all).
    Source(String, String),
    NoAdapter,
    /// Not a module name of this crate at all: it carries a slash or
    /// a `..`, so it points somewhere outside. Review 0035 R-2:
    /// `Path::join` with an absolute component REPLACES the path, so
    /// `module: /home/.../real.rs` had this court read a file outside
    /// the project and call the contract held.
    Outside,
    /// The module was looked for and is not there. Wave 0035: this
    /// used to be a note beside a green verdict, and a single-segment
    /// name was silently read as `src/lib.rs` whatever the field
    /// said -- so `module: /etc/passwd` reported a signature checked
    /// and no finding at all. Renaming a module disarmed §7.6 with
    /// every signature under it, which §7.15 never forgives a test.
    Missing(String),
    NoFile,
}

/// The §7.6 form court over live contracts with module + exports: a
/// promised signature found in the module's collapsed source is
/// silence; the unit's name present with a different text is
/// "diverged" carrying the promise; an absent name is "vanished".
/// The incomparable is skipped here -- `survey` says it aloud.
pub fn court(
    root: &Path,
    config: &Config,
    contracts: &[Contract],
) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for contract in contracts {
        let Some((module, place)) = judged(contract) else {
            continue;
        };
        let (source, read) = match comparability(root, config, module) {
            Comparability::Source(source, read) => (source, read),
            // Named and not there: a finding, not a margin note
            // (wave 0035).
            Comparability::Missing(looked) => {
                out.push((
                    place.clone(),
                    ta(
                        "holding-module-missing",
                        targs!("contract" => contract.slug.clone(), "module" => module.to_string(), "looked" => looked),
                    ),
                    t("holding-module-missing-instead"),
                ));
                continue;
            }
            // A name that leaves the crate is never compared, and
            // never silence (wave 0035).
            Comparability::Outside => {
                out.push((
                    place.clone(),
                    ta(
                        "holding-module-outside",
                        targs!("contract" => contract.slug.clone(), "module" => module.to_string()),
                    ),
                    t("holding-module-outside-instead"),
                ));
                continue;
            }
            _ => continue,
        };
        // Comments are not code (0010 review R-3): a promise that
        // survives only in a comment has vanished.
        let bare = strip_comments(&source, config.language());
        let flat_source = collapse(&bare);
        for signature in &contract.exports {
            if found_bounded(&flat_source, &collapse(signature)) {
                continue;
            }
            let name = unit_name(signature).unwrap_or_else(|| signature.clone());
            if found_bounded(&bare, &name) {
                out.push((
                    place.clone(),
                    ta(
                        "holding-diverged",
                        targs!("contract" => contract.slug.clone(), "signature" => signature.clone(), "name" => name),
                    ),
                    t("holding-diverged-instead"),
                ));
            } else {
                out.push((
                    place.clone(),
                    ta(
                        "holding-vanished",
                        targs!("contract" => contract.slug.clone(), "name" => name, "file" => read.clone()),
                    ),
                    t("holding-vanished-instead"),
                ));
            }
        }
    }
    out
}

/// The court's honest margins: how many signatures were compared,
/// and one line per contract whose form no one compared -- with the
/// reason, never painted green (§7.6).
pub(crate) fn survey(root: &Path, config: &Config, contracts: &[Contract]) -> (u64, Vec<String>) {
    let mut checked: u64 = 0;
    let mut uncompared: Vec<String> = Vec::new();
    for contract in contracts {
        let Some((module, _)) = judged(contract) else {
            continue;
        };
        let why = match comparability(root, config, module) {
            Comparability::Source(..) => {
                checked += contract.exports.len() as u64;
                continue;
            }
            Comparability::NoAdapter => {
                // A named yet unknown adapter is not painted absent
                // (review 0017 R-3): the words tell which it is.
                if config.adapter.is_some() {
                    ta(
                        "holding-why-unknown-adapter",
                        targs!("known" => Language::known()),
                    )
                } else {
                    t("holding-why-no-adapter")
                }
            }
            Comparability::NoFile => t("holding-why-no-file"),
            // Said by the court itself as findings, so they are not
            // repeated here as uncompared margins.
            Comparability::Missing(_) | Comparability::Outside => continue,
        };
        uncompared.push(ta(
            "check-holding-uncompared",
            targs!("contract" => contract.slug.clone(), "why" => why),
        ));
    }
    (checked, uncompared)
}

/// The approved-not-started window (§6.5; 0010 review R-1b healed
/// per §6.7): a contract held only by waves with no tag on any live
/// scenario was grown ahead of the code by a lawful plan -- its form
/// is not judged, and the skip is said aloud by name; any tag of a
/// holding wave brings the court back. Pairs of (contract, wave).
pub(crate) fn plan_window(
    root: &Path,
    waves: &[Wave],
    tags: &[TestTag],
    contracts: &[Contract],
) -> Vec<(String, String)> {
    use std::collections::BTreeMap;
    let mut holders: BTreeMap<&str, Vec<&Wave>> = BTreeMap::new();
    for wave in waves.iter().filter(|w| w.cancelled.is_none()) {
        let mut slugs: Vec<&str> = Vec::new();
        for (_, scenario) in &wave.scenarios {
            if scenario.withdrawn.is_none()
                && let Some(reference) = &scenario.proves
            {
                slugs.push(&reference.slug);
            }
        }
        for (_, transform) in &wave.transforms {
            for reference in &transform.contracts {
                slugs.push(&reference.slug);
            }
        }
        for slug in slugs {
            holders.entry(slug).or_default().push(wave);
        }
    }
    // A plan is a wave with at least one live scenario and none of
    // them tagged BY ITS OWN revision (review 0011 R-1/R-9): a wave
    // with every scenario withdrawn is not a plan -- the promised
    // first tag can never arrive, so the court stays; and a
    // namesake tag from a foreign wave, holding a foreign revision,
    // does not start this one.
    let is_plan = |wave: &Wave| {
        let live: Vec<&String> = wave
            .scenarios
            .iter()
            .filter(|(_, sc)| sc.withdrawn.is_none())
            .map(|(name, _)| name)
            .collect();
        if live.is_empty() {
            return false;
        }
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        let Ok(revs) = crate::rev::scenario_revs(&path) else {
            return false;
        };
        !live.iter().any(|name| {
            let own = revs
                .iter()
                .find(|(n, _)| &n == name)
                .map(|(_, r)| r.as_str())
                .unwrap_or("");
            tags.iter()
                .any(|t| t.scenario == **name && crate::rev::matches(&t.rev, own))
        })
    };
    let mut out = Vec::new();
    for contract in contracts {
        if judged(contract).is_none() {
            continue;
        }
        let Some(held) = holders.get(contract.slug.as_str()) else {
            continue;
        };
        if !held.is_empty() && held.iter().all(|w| is_plan(w)) {
            out.push((contract.slug.clone(), held[0].slug.clone()));
        }
    }
    out
}

/// A live contract with module + exports -- the only kind this
/// court judges (§2.12; a verify-only contract's proof runs in
/// close).
fn judged(contract: &Contract) -> Option<(&str, String)> {
    if contract.withdrawn.is_some() || contract.exports.is_empty() {
        return None;
    }
    let module = contract.module.as_deref()?;
    Some((module, format!("keel/contracts/{}.md", contract.slug)))
}

/// Where the module's source lives -- or why it cannot be compared:
/// the cargo adapter names the crate, the segments after it name the
/// directories and the file (the bare crate itself is src/lib.rs),
/// and `src/a/mod.rs` is looked for wherever `src/a.rs` is.
fn comparability(root: &Path, config: &Config, module: &str) -> Comparability {
    if !config.adapter_known() {
        return Comparability::NoAdapter;
    }
    // A name that reaches outside the crate is not a module of it,
    // and the court says so instead of looking (review 0035 R-2).
    if module.starts_with('/')
        || module
            .split("::")
            .any(|part| part == ".." || part.contains('/'))
    {
        return Comparability::Outside;
    }
    // Ruby keeps a constant's source where ruby keeps it (wave
    // 0038): `Toy::Bar` in `lib/toy/bar.rb`. The court asks the
    // adapter where to look instead of knowing one language's
    // layout by heart.
    if matches!(
        config.language(),
        Some(Language::Ruby) | Some(Language::Elixir)
    ) {
        let looked = match config.language() {
            Some(Language::Elixir) => crate::elixir::module_paths(root, module),
            _ => crate::ruby::module_paths(root, module),
        };
        for path in &looked {
            if let Ok(source) = std::fs::read_to_string(path) {
                return Comparability::Source(source, shown_path(root, path));
            }
        }
        // Every path that was tried, not the first of them (review
        // 0038 R-16): the scenario promises "the path it looked
        // along", and three were walked.
        let shown = looked
            .iter()
            .filter_map(|path| path.strip_prefix(root).ok())
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ");
        return Comparability::Missing(if shown.is_empty() {
            module.to_string()
        } else {
            shown
        });
    }
    let segments: Vec<&str> = module.split("::").collect();
    let Ok(crate_dir) = adapter::crate_root(root) else {
        return Comparability::NoFile;
    };
    // The name is looked for as written. A bare crate name is the
    // crate's own root -- but only when it IS the crate's name;
    // anything else is a module nobody can find.
    let path = if segments.len() == 1 {
        // Review 0035 R-16: this used to be wrapped in a `join`
        // and a `to_str` whose result was thrown away -- ceremony
        // that read as a check and was none.
        let named_the_crate = std::fs::read_to_string(crate_dir.join("Cargo.toml"))
            .map(|text| {
                text.lines().any(|line| {
                    line.split_once('=').is_some_and(|(key, value)| {
                        key.trim() == "name"
                            && value.trim().trim_matches('"').replace('-', "_")
                                == module.replace('-', "_")
                    })
                })
            })
            .unwrap_or(false);
        if named_the_crate {
            let lib = crate_dir.join("src/lib.rs");
            if !lib.is_file() {
                return Comparability::Missing("src/lib.rs".to_string());
            }
            lib
        } else {
            let flat = crate_dir.join("src").join(format!("{module}.rs"));
            let as_dir = crate_dir.join("src").join(module).join("mod.rs");
            if flat.is_file() {
                flat
            } else if as_dir.is_file() {
                as_dir
            } else {
                return Comparability::Missing(format!("src/{module}.rs"));
            }
        }
    } else {
        // Every segment after the crate is a directory, the last one
        // a file -- and `src/a/mod.rs` is as lawful as `src/a.rs`
        // (review 0035 R-5 measured the second layout falsely red).
        // Deeper paths are looked for too: waving them through was
        // the same silent disarming this wave exists to end (R-2).
        let inner: Vec<&str> = segments[1..].to_vec();
        let flat = crate_dir
            .join("src")
            .join(format!("{}.rs", inner.join("/")));
        let as_dir = crate_dir.join("src").join(inner.join("/")).join("mod.rs");
        if flat.is_file() {
            flat
        } else if as_dir.is_file() {
            as_dir
        } else {
            return Comparability::Missing(format!("src/{}.rs", inner.join("/")));
        }
    };
    match std::fs::read_to_string(&path) {
        Ok(source) => Comparability::Source(source, shown_path(root, &path)),
        Err(_) => Comparability::NoFile,
    }
}

/// A match is a match only on token boundaries (0010 review
/// R-3/R-6): `pub fn run` is not satisfied by `run_all`, and the
/// verdict words tell divergence from disappearance apart.
fn found_bounded(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let ident = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    let bytes = haystack.as_bytes();
    let mut from = 0;
    while let Some(pos) = haystack[from..].find(needle) {
        let at = from + pos;
        let end = at + needle.len();
        let before_ok = at == 0 || !ident(bytes[at - 1]);
        let after_ok = end >= bytes.len() || !ident(bytes[end]);
        if before_ok && after_ok {
            return true;
        }
        from = at + 1;
    }
    false
}

/// What a person reads as CODE in this file, and nothing else.
///
/// Comments were never code (review 0010 R-3), and neither is text
/// (wave 0043). The first cut of that second rule read the two as
/// separate passes over separate marks -- and review 0043 measured
/// what that costs on real source: `ident.strip_prefix("r#")` in
/// `syn` opened a raw string that was never open, `"…\r"` did the
/// same, and 17 of 3419 crates in the local registry lost a live
/// declaration. In ruby, `<<-End` was read as waiting for a line
/// saying `E`, so `net/protocol.rb` hid 35 of its 36 methods.
///
/// So this is ONE reader per tongue family, and it reads the file
/// the way the language does: a string is a string wherever it
/// starts, an escape is an escape, and a mark inside either is
/// neither. Newlines are kept through everything so a line number
/// still counts from the top.
fn strip_comments(source: &str, tongue: Option<Language>) -> String {
    match tongue {
        Some(Language::Ruby) => strip_ruby(source, false),
        Some(Language::Elixir) => strip_ruby(source, true),
        _ => strip_rust(source),
    }
}

/// The reader, opened for a corpus sweep: review 0043 measured this
/// hand against ruby's own library and the local cargo registry, and
/// a rule of this size is not judged by hand-written examples alone.
#[doc(hidden)]
pub fn strip_for_test(source: &str, tongue: &str) -> String {
    strip_comments(
        source,
        match tongue {
            "ruby" => Some(Language::Ruby),
            "elixir" => Some(Language::Elixir),
            _ => Some(Language::Rust),
        },
    )
}

fn is_word(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

/// Every newline in `chars[from..to]`, and nothing else: what a
/// blanked stretch leaves behind.
fn only_newlines(out: &mut String, chars: &[char], from: usize, to: usize) {
    for ch in &chars[from..to.min(chars.len())] {
        if *ch == '\n' {
            out.push('\n');
        }
    }
}

/// A raw string opening at `i`: `r"`, `r#"`, `br##"`, `cr"` … The
/// prefix must start a token, or `expr"…"` and `for r in …` would
/// each open one that is not there (review 0043 R-1).
fn raw_open(chars: &[char], i: usize) -> Option<(usize, usize)> {
    if i > 0 && is_word(chars[i - 1]) {
        return None;
    }
    let mut j = i;
    if matches!(chars.get(j), Some('b') | Some('c')) {
        j += 1;
    }
    if chars.get(j) != Some(&'r') {
        return None;
    }
    j += 1;
    let mut hashes = 0usize;
    while chars.get(j) == Some(&'#') {
        hashes += 1;
        j += 1;
    }
    (chars.get(j) == Some(&'"')).then_some((hashes, j + 1))
}

/// An ordinary string opening at `i`, byte and C prefixes included.
fn quote_open(chars: &[char], i: usize) -> Option<usize> {
    if chars[i] == '"' {
        return Some(i + 1);
    }
    if matches!(chars[i], 'b' | 'c')
        && chars.get(i + 1) == Some(&'"')
        && !(i > 0 && is_word(chars[i - 1]))
    {
        return Some(i + 2);
    }
    None
}

/// A char literal at `i`, told from a lifetime: `'a` in `&'a str` is
/// not a literal, and `'\r'` is. The difference is whether a closing
/// quote stands where one must.
fn char_literal(chars: &[char], i: usize) -> Option<usize> {
    match chars.get(i + 1) {
        Some('\\') => {
            let mut j = i + 2;
            match chars.get(j) {
                Some('u') => {
                    while j < chars.len() && chars[j] != '\'' && chars[j] != '\n' {
                        j += 1;
                    }
                }
                Some('x') => j += 3,
                Some(_) => j += 1,
                None => return None,
            }
            (chars.get(j) == Some(&'\'')).then_some(j + 1)
        }
        Some(_) => (chars.get(i + 2) == Some(&'\'')).then_some(i + 3),
        None => None,
    }
}

/// Rust, read as rust reads it: nested block comments, raw strings
/// with their hash count, ordinary strings with their escapes, and
/// char literals told from lifetimes.
fn strip_rust(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(source.len());
    let mut i = 0usize;
    while i < n {
        let ch = chars[i];
        if ch == '/' && chars.get(i + 1) == Some(&'/') {
            let from = i;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            only_newlines(&mut out, &chars, from, i);
            continue;
        }
        if ch == '/' && chars.get(i + 1) == Some(&'*') {
            // Rust's block comments NEST, and a `/*` inside a string
            // is not one -- which is why this reader is one pass and
            // not two.
            let from = i;
            let mut depth = 1usize;
            i += 2;
            while i < n && depth > 0 {
                if chars[i] == '/' && chars.get(i + 1) == Some(&'*') {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            only_newlines(&mut out, &chars, from, i);
            out.push(' ');
            continue;
        }
        if let Some((hashes, after)) = raw_open(&chars, i) {
            let from = i;
            i = after;
            while i < n {
                if chars[i] == '"' {
                    let mut j = i + 1;
                    let mut seen = 0usize;
                    while seen < hashes && chars.get(j) == Some(&'#') {
                        seen += 1;
                        j += 1;
                    }
                    if seen == hashes {
                        i = j;
                        break;
                    }
                }
                i += 1;
            }
            only_newlines(&mut out, &chars, from, i);
            continue;
        }
        if let Some(after) = quote_open(&chars, i) {
            let from = i;
            i = after;
            while i < n {
                if chars[i] == '\\' {
                    i += 2;
                    continue;
                }
                if chars[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            only_newlines(&mut out, &chars, from, i);
            continue;
        }
        if ch == '\''
            && let Some(after) = char_literal(&chars, i)
        {
            i = after;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    out
}

/// The terminator a ruby heredoc opened here waits for -- the WHOLE
/// word, upper or lower, quoted or bare. Cutting it at the first
/// non-uppercase letter made `<<-End` wait for a line saying `E`,
/// and 26 files of ruby's own library lost a live method that way
/// (review 0043 R-2).
///
/// `a << b` is a shovel: a heredoc's word begins where the `<<` ends,
/// with no space between. And `class <<self` is ruby's singleton
/// class, not a heredoc waiting for a line saying `self` -- measured
/// on `rbs/test.rb`, which really does have such a line below.
fn heredoc_word(line: &str) -> Option<String> {
    let mut from = 0usize;
    while let Some(at) = line[from..].find("<<") {
        let at = from + at;
        from = at + 2;
        let rest = &line[at + 2..];
        let rest = rest
            .strip_prefix('~')
            .or_else(|| rest.strip_prefix('-'))
            .unwrap_or(rest);
        let rest = match rest.chars().next() {
            Some(quote @ ('\'' | '"')) => match rest.strip_prefix(quote) {
                Some(inner) => inner,
                None => continue,
            },
            _ => rest,
        };
        let word: String = rest
            .chars()
            .take_while(|c| is_word(*c) && !c.is_ascii_digit() || *c == '_' || c.is_ascii_digit())
            .collect();
        if word == "self" || line[..at].trim_end().ends_with("class") {
            continue;
        }
        match word.chars().next() {
            Some(first) if first.is_alphabetic() || first == '_' => return Some(word),
            _ => continue,
        }
    }
    None
}

/// Ruby and Elixir: comments first, then the longer texts.
///
/// The two passes are in this order because a `<<~TEXT` written
/// inside a comment must not open anything, and because the comment
/// reader below is line-scoped on purpose -- see its own words.
fn strip_ruby(source: &str, elixir: bool) -> String {
    blank_text_lines(&strip_ruby_comments(source), elixir)
}

/// `#` opens a comment in ruby unless it stands inside a string --
/// and `#{...}` interpolation is inside one by definition. Quotes
/// are read rather than guessed at, which is why a URL keeps its
/// fragment.
///
/// **A quote is read within its own line, and that is a decision.**
/// Ruby writes `$'` for the post-match, `?'` for a character, `/…/`
/// for a pattern and `%w[]` for a list -- each of them an unbalanced
/// quote that no reader short of ruby's own lexer tells from a
/// string. Carrying the quote state across lines to catch a
/// multi-line string cost far more than it bought: measured over
/// ruby's own library (1488 files), it hid 513 live `def`s across 87
/// files, `bundler/settings.rb` alone losing 39 to one `repos[$']`.
/// A quote that does not close on its line is therefore not a
/// string, and the line is read as code.
///
/// The direction is chosen, not stumbled into: this court may let a
/// ghost through and say so (a promise living inside a multi-line
/// plain string is a named border, below), but it must not refuse a
/// promise that is ALIVE. A court that refuses live code is not a
/// stricter court; it is a broken one (review 0043 R-2).
fn strip_ruby_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    for line in source.lines() {
        let mut quote: Option<char> = None;
        let mut escaped = false;
        let mut cut = line.len();
        for (at, ch) in line.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' | '\'' => match quote {
                    None => quote = Some(ch),
                    Some(open) if open == ch => quote = None,
                    _ => {}
                },
                '#' if quote.is_none() => {
                    cut = at;
                    break;
                }
                _ => {}
            }
        }
        out.push_str(&line[..cut]);
        out.push('\n');
    }
    out
}

/// The longer texts, blanked line by line: ruby's heredocs and
/// `=begin` block, elixir's `"""` and `'''` fences.
///
/// A heredoc is opened ONLY when its word really stands alone on a
/// line below. That one rule is what keeps `list << Item`, a shovel
/// inside a string, and any heredoc shape this reader does not know
/// from swallowing the rest of the file -- which is exactly what the
/// first cut of this wave did to `net/protocol.rb` (35 of its 36
/// methods) and `openssl/ssl.rb` (all 30).
///
/// Blanked as empty LINES, so every line number still counts from
/// the top of the file.
fn blank_text_lines(source: &str, elixir: bool) -> String {
    let lines: Vec<&str> = source.lines().collect();
    let mut kept: Vec<&str> = Vec::with_capacity(lines.len());
    let mut fence: Option<&str> = None;
    let mut heredoc: Option<String> = None;
    let mut block = false;
    for (at, line) in lines.iter().enumerate() {
        if let Some(word) = &heredoc {
            if line.trim() == word.as_str() {
                heredoc = None;
            }
            kept.push("");
            continue;
        }
        if let Some(mark) = fence {
            if line.contains(mark) {
                fence = None;
            }
            kept.push("");
            continue;
        }
        if elixir {
            // A fence may open and close on one line (`@doc """x"""`),
            // so the marks are counted: an odd number turns the state
            // over, an even one leaves it -- and the line is text
            // either way.
            let mut opened = false;
            for mark in ["\"\"\"", "'''"] {
                let marks = line.matches(mark).count();
                if marks > 0 {
                    if marks % 2 == 1 {
                        fence = Some(mark);
                    }
                    opened = true;
                    break;
                }
            }
            if opened {
                kept.push("");
                continue;
            }
        } else {
            // `=begin`/`=end` is ruby's block comment, and it counts
            // only at the very start of a line.
            if block {
                if line.starts_with("=end") {
                    block = false;
                }
                kept.push("");
                continue;
            }
            if line.starts_with("=begin") {
                block = true;
                kept.push("");
                continue;
            }
            if let Some(word) = heredoc_word(line)
                && lines[at + 1..].iter().any(|below| below.trim() == word)
            {
                heredoc = Some(word);
                kept.push("");
                continue;
            }
        }
        kept.push(line);
    }
    let mut out = kept.join("\n");
    out.push('\n');
    out
}

/// A path as a person would name it: relative to the project root,
/// because an absolute path in a verdict is a machine talking to
/// itself.
fn shown_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .display()
        .to_string()
}

fn collapse(text: &str) -> String {
    // rustfmt's wrapping is formatting, not form (§2.9 compares what
    // the language writes): the trailing comma before a closing
    // brace or parenthesis and the spaces a line break leaves around
    // parentheses are normalized away, on both sides the same way --
    // proven by this court biting its own contract when fmt wrapped
    // the very signature promised here.
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("( ", "(")
        .replace(", )", ")")
        .replace(" )", ")")
        .replace(", }", " }")
        // Ruby writes no types, so the space after a comma is
        // formatting there exactly as rustfmt's wrapping is here --
        // and `def f(a, b)` against `def f(a,b)` was a finding
        // (review 0038, the fourth commitment). Both sides pass
        // through this hand, so the comparison stays honest.
        .replace(", ", ",")
}

/// The unit a signature promises: the word after the language's
/// keyword, so "diverged" and "vanished" are told apart honestly.
fn unit_name(signature: &str) -> Option<String> {
    const KEYWORDS: [&str; 8] = [
        "fn", "enum", "struct", "trait", "const", "static", "type", "mod",
    ];
    let mut tokens = signature.split_whitespace().peekable();
    while let Some(token) = tokens.next() {
        if KEYWORDS.contains(&token) {
            let name: String = tokens
                .next()?
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            return (!name.is_empty()).then_some(name);
        }
    }
    None
}
