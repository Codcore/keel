//! The one hand that splices a named section into the text of
//! keel.toml (contract tool-confedit): trust records fingerprints
//! with it, the generated integrations record digests. The config
//! belongs to the person -- comments, order, foreign sections and
//! line endings all survive; a line is set into it, never over it.

/// Sets each `key = "value"` into `[section]`: an existing key is
/// rewritten in place, a new one is added at the end of the
/// section, and a section that does not exist is born at the end of
/// the file. Keys are compared with their whitespace collapsed (the
/// 0010 school), so twins of one command fold into a single line
/// instead of multiplying. The hand returns text -- reading, the
/// strict re-parse before landing (0010 review R-1) and the words of
/// a refusal belong to the writer.
pub fn upsert(text: &str, section: &str, entries: &[(String, String)]) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let header = format!("[{section}]");
    let at = lines.iter().position(|l| is_header(l, section));

    let (start, mut end) = match at {
        Some(at) => {
            let end = lines[at + 1..]
                .iter()
                .position(|l| l.trim_start().starts_with('['))
                .map_or(lines.len(), |offset| at + 1 + offset);
            (at + 1, end)
        }
        None => {
            if lines.last().is_some_and(|l| !l.trim().is_empty()) {
                lines.push(String::new());
            }
            lines.push(header);
            (lines.len(), lines.len())
        }
    };

    let mut fresh: Vec<String> = Vec::new();
    for (key, value) in entries {
        let flat = collapse(key);
        let stands: Vec<usize> = lines[start..end]
            .iter()
            .enumerate()
            .filter(|(_, l)| line_key(l).is_some_and(|k| collapse(&k) == flat))
            .map(|(offset, _)| start + offset)
            .collect();
        match stands.split_first() {
            Some((&first, twins)) => {
                lines[first] = toml_line(key, value);
                for &twin in twins.iter().rev() {
                    lines.remove(twin);
                    end -= 1;
                }
            }
            None => fresh.push(toml_line(key, value)),
        }
    }

    // New lines go after the section's last real line, never after
    // the blank tail that separates it from what follows.
    let mut at = end;
    while at > start && lines[at - 1].trim().is_empty() {
        at -= 1;
    }
    for line in fresh.into_iter().rev() {
        lines.insert(at, line);
    }

    let mut out = lines.join(eol);
    // The predicate trust carried since 0010 (review 0022 R-4:
    // the refactor must not change a byte of its behaviour).
    if text.ends_with('\n') || !text.contains('\n') {
        out.push_str(eol);
    }
    out
}

/// The section header as TOML reads it (the 0010 R-1 school):
/// spaces inside the brackets and a trailing comment are the same
/// header, not a reason to append a second section.
fn is_header(line: &str, section: &str) -> bool {
    let trimmed = line.trim_start();
    let Some(rest) = trimmed.strip_prefix('[') else {
        return false;
    };
    let Some(end) = rest.find(']') else {
        return false;
    };
    if rest[..end].trim() != section {
        return false;
    }
    let after = rest[end + 1..].trim();
    after.is_empty() || after.starts_with('#')
}

/// The key of a section line as written -- basic (with the TOML
/// escapes decoded), literal, or bare -- or None for comments and
/// blanks.
fn line_key(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    if let Some(rest) = trimmed.strip_prefix('"') {
        let mut key = String::new();
        let mut chars = rest.chars();
        while let Some(c) = chars.next() {
            match c {
                '\\' => match chars.next()? {
                    'u' => key.push(unescaped(&mut chars, 4)?),
                    'U' => key.push(unescaped(&mut chars, 8)?),
                    't' => key.push('\t'),
                    'n' => key.push('\n'),
                    'r' => key.push('\r'),
                    'b' => key.push('\u{0008}'),
                    'f' => key.push('\u{000C}'),
                    other => key.push(other),
                },
                '"' => return Some(key),
                _ => key.push(c),
            }
        }
        None
    } else if let Some(rest) = trimmed.strip_prefix('\'') {
        rest.split('\'').next().map(str::to_string)
    } else {
        trimmed.split('=').next().map(|key| key.trim().to_string())
    }
}

/// One \uXXXX / \UXXXXXXXX escape decoded.
fn unescaped(chars: &mut std::str::Chars, width: usize) -> Option<char> {
    let hex: String = (0..width).map(|_| chars.next()).collect::<Option<_>>()?;
    char::from_u32(u32::from_str_radix(&hex, 16).ok()?)
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn toml_line(key: &str, value: &str) -> String {
    let mut escaped = String::new();
    for c in key.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            c if c.is_control() => escaped.push_str(&format!("\\u{:04X}", c as u32)),
            c => escaped.push(c),
        }
    }
    format!("\"{escaped}\" = \"{value}\"")
}

/// Sets `key = value` among the file's top-level keys, leaving
/// everything else byte for byte: a person's comments, the order of
/// the lines, and any section below.
///
/// A commented key (`# lang = "uk"`) is the vocabulary showing what
/// could be set, so setting it replaces that very line -- the answer
/// lands where the reader already looked for it. The bug audit
/// measured `keel setup` rebuilding the whole file instead and
/// taking a person's own comments with it.
pub fn upsert_root(text: &str, entries: &[(String, String)]) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    // The root block ends where the first section begins.
    let end = lines
        .iter()
        .position(|line| line.trim_start().starts_with('['))
        .unwrap_or(lines.len());

    for (key, value) in entries {
        let row = format!("{key} = {value}");
        // The key is matched by its NAME, not by the spelling of the
        // spaces around it: `lang="uk"` and `ci  =  "x"` are valid
        // TOML, and matching a raw prefix missed them and appended a
        // duplicate key -- which is not TOML at all, so one setup
        // bricked a healthy config (review 0034 R-1). The
        // neighbouring hand has had this school since wave 0010; this
        // one did not inherit it.
        let names = |line: &str| -> Option<String> {
            let bare = line.trim_start().trim_start_matches('#').trim_start();
            let (name, _) = bare.split_once('=')?;
            Some(name.trim().to_string())
        };
        let at = lines[..end]
            .iter()
            .position(|line| names(line).as_deref() == Some(key.as_str()));
        match at {
            Some(at) => lines[at] = row,
            None => lines.insert(end, row),
        }
    }
    let mut out = lines.join(eol);
    if text.ends_with('\n') {
        out.push_str(eol);
    }
    out
}

/// Removes from `[section]` every key that is not in `keep`,
/// leaving the rest of the file untouched.
///
/// Review 0034 R-4: `keel setup` computed which trust records were
/// still live and filtered a list -- but the text it wrote into was
/// the person's own config with every record already in it, and
/// `upsert` can only add or rewrite. The filter was inert, and the
/// wizard left behind trust for commands nobody runs, which turns
/// `keel check` red and sends the person to edit by hand: exactly
/// the defect R-10 of review 0032 had already fixed once.
pub fn retain(text: &str, section: &str, keep: &[String]) -> String {
    let eol = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let Some(at) = lines.iter().position(|line| is_header(line, section)) else {
        return text.to_string();
    };
    let end = lines[at + 1..]
        .iter()
        .position(|line| line.trim_start().starts_with('['))
        .map(|offset| at + 1 + offset)
        .unwrap_or(lines.len());

    let kept: Vec<String> = keep.iter().map(|word| collapse(word)).collect();
    let mut out: Vec<String> = Vec::new();
    for (number, line) in lines.drain(..).enumerate() {
        let inside = number > at && number < end;
        let name = line
            .trim_start()
            .split_once('=')
            .map(|(name, _)| collapse(name.trim().trim_matches('"')));
        let drop = inside
            && !line.trim().is_empty()
            && !line.trim_start().starts_with('#')
            && name.is_some_and(|name| !kept.contains(&name));
        if !drop {
            out.push(line);
        }
    }
    let mut text_out = out.join(eol);
    if text.ends_with('\n') {
        text_out.push_str(eol);
    }
    text_out
}
