//! Chapter 5: revisions (contract tool-rev). The short signature of
//! a text, held by whoever leans on it; this rung takes the counting
//! over from the author's hands.

use crate::docs::{self, Refusal, Wave};
use crate::i18n::{t, ta};
use crate::targs;
use sha2::{Digest, Sha256};
use std::path::Path;

/// The recipe (§5.3-§5.4), reproducing the hand recipe of waves
/// 0001-0002 byte for byte: runs of whitespace collapse into one
/// space, edges trimmed, sha256, first six hex characters.
pub fn text_rev(text: &str) -> String {
    let normalized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let digest = Sha256::digest(normalized.as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    hex[..6].to_string()
}

/// A contract is hashed as the whole file, header included (§5.3).
pub fn contract_rev(path: &Path) -> Result<String, Refusal> {
    Ok(text_rev(&read(path)?))
}

/// Scenario revisions of a wave, in declared order: each scenario is
/// hashed as its section body -- from the line after
/// `## scenario: <name>` to the next `## ` heading (§5.3). A scenario
/// declared in the header without a body section refuses by name, and
/// so does a duplicated section: half of §7.7 arrives here naturally.
pub fn scenario_revs(path: &Path) -> Result<Vec<(String, String)>, Refusal> {
    let wave = docs::read_wave(path)?;
    let text = read(path)?;

    let mut sections: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            if let Some(name) = rest.strip_prefix("scenario: ") {
                current = Some((name.trim().to_string(), String::new()));
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(line);
            body.push('\n');
        }
    }
    if let Some(section) = current.take() {
        sections.push(section);
    }

    for (name, _) in &sections {
        if sections.iter().filter(|(n, _)| n == name).count() > 1 {
            return Err(Refusal {
                file: path.to_path_buf(),
                reason: ta("rev-dup-section", targs!("name" => name.clone())),
                instead: ta("rev-dup-section-instead", targs!("name" => name.clone())),
            });
        }
    }

    let mut out = Vec::new();
    for (name, _) in &wave.scenarios {
        match sections.iter().find(|(n, _)| n == name) {
            Some((_, body)) if body.split_whitespace().next().is_none() => {
                return Err(Refusal {
                    file: path.to_path_buf(),
                    reason: ta("rev-empty-section", targs!("name" => name.clone())),
                    instead: ta("rev-empty-section-instead", targs!("name" => name.clone())),
                });
            }
            Some((_, body)) => out.push((name.clone(), text_rev(body))),
            None => {
                return Err(Refusal {
                    file: path.to_path_buf(),
                    reason: ta("rev-missing-section", targs!("name" => name.clone())),
                    instead: ta(
                        "rev-missing-section-instead",
                        targs!("name" => name.clone()),
                    ),
                });
            }
        }
    }
    Ok(out)
}

/// Prefix comparison (§5.2): a record of four to six characters
/// passes when it matches the start of the current revision.
pub fn matches(recorded: &str, actual: &str) -> bool {
    (4..=6).contains(&recorded.len()) && actual.starts_with(recorded)
}

/// The `keel rev` report: current revisions of every document, in
/// the project language; broken documents stand next to them as
/// refusals, never silence (the command inherits scan's refusals).
pub fn report(root: &Path, config: &crate::config::Config) -> Result<(String, usize), Refusal> {
    let scan = docs::scan(root)?;
    let mut refusals: Vec<Refusal> = scan.refusals;

    let mut lines: Vec<String> = Vec::new();
    for contract in &scan.contracts {
        let path = root
            .join("keel/contracts")
            .join(format!("{}.md", contract.slug));
        match contract_rev(&path) {
            Ok(revision) => lines.push(format!("  {}@{revision}", contract.slug)),
            Err(refusal) => refusals.push(refusal),
        }
    }
    for wave in &scan.waves {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        match scenario_revs(&path) {
            Ok(revs) => {
                for (name, revision) in revs {
                    lines.push(format!("  {}/{name}@{revision}", wave.slug));
                }
            }
            Err(refusal) => refusals.push(refusal),
        }
    }

    let mut report = crate::i18n::t("rev-title");
    report.push('\n');
    let config_line = if !config.present {
        crate::i18n::t("check-config-absent")
    } else if config.lang_set {
        ta(
            "check-config-present",
            targs!("lang" => config.lang.clone()),
        )
    } else {
        crate::i18n::t("check-config-lang-default")
    };
    report.push_str(&config_line);
    report.push('\n');
    report.push('\n');
    for line in &lines {
        report.push_str(line);
        report.push('\n');
    }
    for refusal in &refusals {
        let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
        report.push_str(&format!(
            "  {:<8} {} — {}\n           {}: {}\n",
            crate::i18n::t("word-red"),
            shown.display(),
            refusal.reason,
            crate::i18n::t("word-instead"),
            refusal.instead
        ));
    }
    report.push('\n');
    report.push_str(&crate::i18n::t("rev-next"));
    report.push('\n');
    Ok((report, refusals.len()))
}

/// File reading with the same refusal school as docs.
fn read(path: &Path) -> Result<String, Refusal> {
    std::fs::read_to_string(path).map_err(|e| Refusal {
        file: path.to_path_buf(),
        reason: if e.kind() == std::io::ErrorKind::InvalidData {
            crate::i18n::t("docs-not-utf8")
        } else {
            ta("docs-unreadable", targs!("error" => e.to_string()))
        },
        instead: if e.kind() == std::io::ErrorKind::InvalidData {
            crate::i18n::t("docs-not-utf8-instead")
        } else {
            crate::i18n::t("docs-unreadable-instead")
        },
    })
}

/// The body of one `## <title>` section, verbatim -- the packages of
/// §9.10 hand bodies out through the same court that revisions them.
/// None where the section is absent. Verbatim means the words, not
/// the carriage returns (the 0009 R-3 school; review 0012 R-4).
pub(crate) fn section(text: &str, title: &str) -> Option<String> {
    let text = text.replace("\r\n", "\n");
    for part in text.split("\n## ") {
        if let Some(rest) = part.strip_prefix(title)
            && rest.starts_with('\n')
        {
            return Some(rest.trim_matches('\n').to_string());
        }
    }
    None
}

/// §7.7's other half: the set of names in the header equals the set
/// of section headings in the body, both ways. A header transform
/// with no "## transform:" section and a section declared by no
/// header entry are findings by name -- an orphan does not live in
/// silence. The header-scenario side is held by scenario_revs'
/// refusals, as before. Pairs of (reason, instead); the verdicts
/// are check's to print.
pub fn body_court(path: &Path, wave: &Wave) -> Result<Vec<(String, String)>, Refusal> {
    let text = read(path)?.replace("\r\n", "\n");
    let mut body_scenarios: Vec<String> = Vec::new();
    let mut body_transforms: Vec<String> = Vec::new();
    let mut out = Vec::new();
    for part in text.split("\n## ").skip(1) {
        let heading = part.lines().next().unwrap_or("").trim();
        if let Some(name) = heading.strip_prefix("scenario: ") {
            body_scenarios.push(name.trim().to_string());
        } else if let Some(name) = heading.strip_prefix("transform: ") {
            body_transforms.push(name.trim().to_string());
        } else if heading.starts_with("scenario:") || heading.starts_with("transform:") {
            // The very word without its space is not free prose
            // (review 0011 R-6): a near-miss is named, never silent.
            out.push((
                ta("rev-nearmiss", targs!("heading" => heading.to_string())),
                t("rev-nearmiss-instead"),
            ));
        }
    }
    // A duplicated transform section is not guessed between (review
    // 0011 R-7) -- the same court scenario sections get from
    // scenario_revs' refusals.
    let mut seen: Vec<&String> = Vec::new();
    for name in &body_transforms {
        if seen.contains(&name) {
            out.push((
                ta("rev-dup-transform", targs!("name" => name.clone())),
                t("rev-dup-transform-instead"),
            ));
        } else {
            seen.push(name);
        }
    }
    for (name, _) in &wave.transforms {
        if !body_transforms.iter().any(|b| b == name) {
            out.push((
                ta("rev-transform-no-body", targs!("name" => name.clone())),
                t("rev-transform-no-body-instead"),
            ));
        }
    }
    for name in &body_scenarios {
        if !wave.scenarios.iter().any(|(n, _)| n == name) {
            out.push((
                ta(
                    "rev-orphan-section",
                    targs!("kind" => "scenario".to_string(), "name" => name.clone()),
                ),
                t("rev-orphan-section-instead"),
            ));
        }
    }
    for name in &body_transforms {
        if !wave.transforms.iter().any(|(n, _)| n == name) {
            out.push((
                ta(
                    "rev-orphan-section",
                    targs!("kind" => "transform".to_string(), "name" => name.clone()),
                ),
                t("rev-orphan-section-instead"),
            ));
        }
    }
    Ok(out)
}

/// The `keel rev --write` hand (NEW-CONCEPT): the drifted records
/// of OPEN waves -- proves and transform contracts alike -- are
/// rewritten onto the current contract revisions, by name with old
/// and new; the closed are left to history's court (§5.6). Header
/// surgery only: full slug@revision tokens in the header slice,
/// never a section body -- and the rewritten header re-reads
/// through the strict parser before landing by plan::write_new's
/// dot-temp and rename, or nothing lands. A record pointing at a
/// missing contract is not rewritten -- check names it (§7.1).
/// Returns the report, how many records were rewritten, and how many
/// findings stopped it. The third number is the one the exit code is
/// made of: the bug audit (B5) measured this hand printing a red
/// line, then "nothing drifts", and exiting zero -- so CI saw success
/// where the words said failure.
pub fn write(root: &Path) -> Result<(String, usize, usize), Refusal> {
    let config = crate::config::read(root)?;
    if !config.adapter_known() {
        return Err(Refusal {
            file: root.join("keel.toml"),
            reason: t("rev-write-needs-adapter"),
            instead: t("rev-write-needs-adapter-instead"),
        });
    }
    let scan = docs::scan(root)?;
    if let Some(refusal) = scan.refusals.into_iter().next() {
        // Surgery over a broken set of documents would cut blind.
        return Err(refusal);
    }
    let found = crate::tags::scan(&crate::adapter::test_files(root)?)?;

    let mut report = t("rev-write-title");
    report.push('\n');
    let mut rewritten: usize = 0;
    let mut findings: usize = 0;
    let mut kept: usize = 0;
    for wave in &scan.waves {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        let mut refs: Vec<&docs::ContractRef> = wave
            .scenarios
            .iter()
            .filter(|(_, sc)| sc.withdrawn.is_none())
            .filter_map(|(_, sc)| sc.proves.as_ref())
            .collect();
        for (_, transform) in &wave.transforms {
            refs.extend(transform.contracts.iter());
        }
        let mut stale: Vec<(String, String, String)> = Vec::new();
        for reference in refs {
            let contract_path = root
                .join("keel/contracts")
                .join(format!("{}.md", reference.slug));
            if !contract_path.is_file() {
                continue;
            }
            let current = contract_rev(&contract_path)?;
            if !matches(&reference.rev, &current)
                && !stale
                    .iter()
                    .any(|(s, r, _)| s == &reference.slug && r == &reference.rev)
            {
                stale.push((reference.slug.clone(), reference.rev.clone(), current));
            }
        }
        if stale.is_empty() {
            continue;
        }
        if crate::close::structural(root, wave, &found)? {
            kept += 1;
            report.push_str(&ta("rev-write-kept", targs!("wave" => wave.slug.clone())));
            report.push('\n');
            continue;
        }
        // The surgery and its landing, caught per wave: a refusal
        // here becomes a red row and stops the pass -- the report of
        // what already landed is never eaten with it (review 0016
        // R-3).
        let surgery = (|| -> Result<Vec<(String, String, String, usize)>, Refusal> {
            let text = read(&path)?;
            let Some(split_at) = header_span(&text) else {
                return Ok(Vec::new());
            };
            let (head, body) = text.split_at(split_at);
            let mut new_head = head.to_string();
            let mut landed: Vec<(String, String, String, usize)> = Vec::new();
            for (slug, recorded, current) in &stale {
                let (replaced, count) = replace_token(&new_head, slug, recorded, current);
                new_head = replaced;
                landed.push((slug.clone(), recorded.clone(), current.clone(), count));
            }
            let new_text = format!("{new_head}{body}");
            docs::read_wave_text(&wave.slug, &new_text, &path)?;
            crate::plan::write_new(&path, &new_text)?;
            Ok(landed)
        })();
        match surgery {
            Ok(landed) => {
                for (slug, recorded, current, count) in landed {
                    rewritten += count;
                    report.push_str(&ta(
                        "rev-write-rewritten",
                        targs!("wave" => wave.slug.clone(), "contract" => slug, "old" => recorded, "new" => current),
                    ));
                    report.push('\n');
                }
            }
            Err(refusal) => {
                findings += 1;
                let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
                report.push_str(&format!(
                    "  {:<8} {} — {}\n           {}: {}\n",
                    t("word-red"),
                    shown.display(),
                    refusal.reason,
                    t("word-instead"),
                    refusal.instead
                ));
                break;
            }
        }
    }
    // The none-word speaks of the open waves: the closed keep their
    // legally drifted records (§5.6) and their leaving lines above.
    // "Nothing drifts" is not said over a red line: the report and
    // the exit code tell the same story or neither is worth reading.
    if findings > 0 {
        report.push_str(&ta("rev-write-stopped", targs!("count" => findings as u64)));
    } else if kept > 0 && rewritten == 0 {
        // Something DID drift; §5.6 is why it stays. Saying "nothing
        // drifted" two lines under the record left standing is the
        // self-contradiction the bug audit (B5) measured.
        report.push_str(&ta("rev-write-only-kept", targs!("count" => kept as u64)));
    } else if rewritten == 0 {
        report.push_str(&t("rev-write-none"));
    } else {
        report.push_str(&ta("rev-write-count", targs!("count" => rewritten as u64)));
    }
    report.push('\n');
    Ok((report, rewritten, findings))
}

/// One full-token replacement pass (review 0016 R-1): the needle
/// `slug@old` counts only where the character before is no slug
/// character (so `rev@x` never strikes inside `tool-rev@x`) and the
/// character after is no hex digit (so a four-character record never
/// eats the start of a six-character one). Returns the new text and
/// how many records were replaced.
fn replace_token(text: &str, slug: &str, old: &str, new: &str) -> (String, usize) {
    let needle = format!("{slug}@{old}");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    let mut count = 0usize;
    let slug_char = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
    let hex_char = |c: char| c.is_ascii_hexdigit() && !c.is_ascii_uppercase();
    while let Some(at) = rest.find(&needle) {
        let prev = rest[..at]
            .chars()
            .next_back()
            .or_else(|| out.chars().next_back());
        let next = rest[at + needle.len()..].chars().next();
        if prev.is_none_or(|c| !slug_char(c)) && next.is_none_or(|c| !hex_char(c)) {
            out.push_str(&rest[..at]);
            out.push_str(slug);
            out.push('@');
            out.push_str(new);
            count += 1;
        } else {
            out.push_str(&rest[..at + needle.len()]);
        }
        rest = &rest[at + needle.len()..];
    }
    out.push_str(rest);
    (out, count)
}

/// The byte length of the header: through the second `---` line
/// inclusive. None only for a shape the strict scan already
/// refused -- defensive, never a guess.
fn header_span(text: &str) -> Option<usize> {
    let mut pos = 0usize;
    let mut dashes = 0usize;
    for raw in text.split_inclusive('\n') {
        pos += raw.len();
        if raw.trim_end_matches(['\n', '\r']) == "---" {
            dashes += 1;
            if dashes == 2 {
                return Some(pos);
            }
        }
    }
    None
}
