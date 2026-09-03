//! The mouth of the tool (contract tool-speak; the operator's §8.6
//! decision): everything the thin generated block does not carry must
//! live in the binary itself -- the methodology and the forty cuts
//! included, because a project where keel stands has neither.
//!
//! The texts are EMBEDDED, so there is one source and it cannot drift
//! from the binary by construction: a release that changed a document
//! changed its own mouth.

use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::Path;

/// The checklist a person reads (QUALITY.md) and the methodology of
/// this generation, both as this release was built with them.
const CHECKLIST: &str = include_str!("../../QUALITY.md");
const METHOD: &str = include_str!("../../docs/uk/METHODOLOGY-V2.md");

/// The checklist this release was built with -- handed out so a
/// caller can judge it against the courts' own list without a second
/// copy living anywhere.
pub fn checklist() -> &'static str {
    CHECKLIST
}

/// The forty cuts with the family and the question each asks, taken
/// from the checklist and held against the judge's own list.
///
/// The judge is `graph::cuts()` and stays the one home of the
/// vocabulary: this hand never keeps a second copy, it only finds the
/// question for each slug the judge already holds.
pub fn cuts() -> Vec<(&'static str, &'static str, &'static str)> {
    cuts_from(CHECKLIST).unwrap_or_default()
}

/// The same, from a checklist handed in -- so the court between the
/// judge's list and the document a person reads can be PLAYED, not
/// merely promised. A cut whose question the document no longer
/// carries is a refusal: what is judged and what is read must be one
/// list, or the difference must be said aloud.
pub fn cuts_from(checklist: &str) -> Result<Vec<(&str, &str, &str)>, Refusal> {
    let mut family = String::new();
    let mut found: Vec<(String, &str)> = Vec::new();
    let mut hollow: Vec<String> = Vec::new();
    for line in checklist.lines() {
        if let Some(title) = line.strip_prefix("### ") {
            // "1. Functional suitability" -> the first word, lowered,
            // is the family the slugs are built from.
            family = title
                .split_once(". ")
                .map(|(_, name)| name)
                .unwrap_or(title)
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_lowercase();
            continue;
        }
        let Some(rest) = line.strip_prefix("- **") else {
            continue;
        };
        let Some((name, question)) = rest.split_once("** — ") else {
            continue;
        };
        if family.is_empty() {
            continue;
        }
        let slug = format!("{family}.{}", name.replace(' ', "-"));
        // A question emptied by an accidental edit is drift as much
        // as one renamed: the slug would be served with a dash and
        // nothing after it (review 0027 R-1, its sixth breakage).
        if question.trim().is_empty() {
            hollow.push(slug.clone());
        }
        found.push((slug, question.trim()));
    }

    if !hollow.is_empty() {
        return Err(Refusal {
            file: Path::new("QUALITY.md").to_path_buf(),
            reason: ta(
                "speak-cuts-hollow",
                targs!("cuts" => hollow.join(", "), "count" => hollow.len().to_string()),
            ),
            instead: t("speak-cuts-hollow-instead"),
        });
    }

    let mut paired = Vec::with_capacity(crate::graph::cuts().len());
    let mut lost: Vec<&str> = Vec::new();
    for slug in crate::graph::cuts() {
        match found.iter().find(|(name, _)| name == slug) {
            Some((_, question)) => {
                let (family, _) = slug.split_once('.').unwrap_or((slug, slug));
                paired.push((*slug, family, *question));
            }
            None => lost.push(slug),
        }
    }
    if !lost.is_empty() {
        return Err(Refusal {
            file: Path::new("QUALITY.md").to_path_buf(),
            reason: ta(
                "speak-cuts-drifted",
                targs!("cuts" => lost.join(", "), "count" => lost.len().to_string()),
            ),
            instead: t("speak-cuts-drifted-instead"),
        });
    }

    // Drift is not only a question that went missing. Measured by my
    // own hand before the review: a document whose questions were
    // REORDERED, and one carrying a forty-first question no court
    // judges, both passed -- and both are the same defect. What is
    // judged and what is read must be ONE list: same members, same
    // order.
    let stray: Vec<&str> = found
        .iter()
        .map(|(name, _)| name.as_str())
        .filter(|name| !crate::graph::cuts().contains(name))
        .collect();
    if !stray.is_empty() {
        return Err(Refusal {
            file: Path::new("QUALITY.md").to_path_buf(),
            reason: ta(
                "speak-cuts-stray",
                targs!("cuts" => stray.join(", "), "count" => stray.len().to_string()),
            ),
            instead: t("speak-cuts-stray-instead"),
        });
    }
    let read: Vec<&str> = found.iter().map(|(name, _)| name.as_str()).collect();
    if let Some((at, (judged, was))) = crate::graph::cuts()
        .iter()
        .zip(read.iter())
        .enumerate()
        .find(|(_, (judged, was))| **judged != **was)
    {
        return Err(Refusal {
            file: Path::new("QUALITY.md").to_path_buf(),
            reason: ta(
                "speak-cuts-order",
                targs!("at" => (at + 1).to_string(), "judged" => (*judged).to_string(), "read" => (*was).to_string()),
            ),
            instead: t("speak-cuts-order-instead"),
        });
    }
    Ok(paired)
}

/// The cuts as a report: nine families, forty questions, each under
/// the slug the courts judge by.
pub fn cuts_report() -> Result<String, Refusal> {
    // The court lives in cuts_from, not at the call site: a second
    // buyer of this contract used to get an empty list and a title
    // saying "forty" above it (review 0027 R-5).
    let paired = cuts_from(CHECKLIST)?;
    let mut report = t("speak-cuts-title");
    report.push('\n');
    let mut standing = "";
    for (slug, family, question) in paired {
        if family != standing {
            // The nine families are NAMED, not merely implied by the
            // prefix of a slug (review 0027 R-14).
            report.push('\n');
            report.push_str(&format!("  [{family}]\n"));
            standing = family;
        }
        report.push_str(&format!("  {slug} — {question}\n"));
    }
    report.push('\n');
    report.push_str(&ta(
        "speak-cuts-source",
        targs!("version" => env!("CARGO_PKG_VERSION").to_string()),
    ));
    report.push('\n');
    Ok(report)
}

/// The methodology: its contents, or one paragraph of it.
pub fn method(asked: Option<&str>) -> Result<String, Refusal> {
    let chapters = chapters();
    let Some(asked) = asked else {
        let mut report = t("speak-method-title");
        report.push('\n');
        for (name, paragraphs) in &chapters {
            report.push_str(&format!("  {name} — {}\n", paragraphs.len()));
        }
        report.push('\n');
        report.push_str(&ta(
            "speak-method-source",
            targs!("version" => env!("CARGO_PKG_VERSION").to_string()),
        ));
        report.push('\n');
        return Ok(report);
    };
    let wanted = asked.trim_start_matches('§').trim();
    // A chapter asked for by name is served whole -- which is the
    // only way to reach the Constitution's eight rules and the three
    // appendices, a sixth of the methodology that no paragraph number
    // can reach (review 0027 R-6).
    if let Some(said) = whole_chapter(wanted) {
        return Ok(said);
    }
    for (name, paragraphs) in &chapters {
        if let Some((_, text)) = paragraphs.iter().find(|(number, _)| number == wanted) {
            // Ten paragraphs -- the last of every chapter -- used to
            // carry the document's own "---" rule with them (review
            // 0027 R-11). A piece served is the piece, not its
            // neighbour's fence.
            let text = text
                .trim_end()
                .trim_end_matches("---")
                .trim_end()
                .to_string();
            return Ok(format!("{name}\n\n{text}\n"));
        }
    }
    // The bounds of the chapter this number would live in, so the
    // word is useful and not merely correct.
    let head = wanted.split('.').next().unwrap_or("");
    let neighbours: Vec<&str> = chapters
        .iter()
        .flat_map(|(_, paragraphs)| paragraphs.iter())
        .map(|(number, _)| number.as_str())
        .filter(|number| number.split('.').next() == Some(head))
        .collect();
    // Two states, two words. A chapter that exists gets its bounds;
    // a number belonging to no chapter gets the list of chapters,
    // because "that chapter holds no paragraph of that chapter" is a
    // broken sentence, not help (review 0027 R-7).
    let instead = match (neighbours.first(), neighbours.last()) {
        (Some(first), Some(last)) => ta(
            "speak-method-unknown-instead",
            targs!("bounds" => format!("§{first} … §{last}")),
        ),
        _ => ta(
            "speak-method-nowhere-instead",
            targs!("chapters" => chapters
                .iter()
                .map(|(name, _)| (*name).to_string())
                .collect::<Vec<_>>()
                .join(" | ")),
        ),
    };
    Err(Refusal {
        file: Path::new("METHODOLOGY-V2.md").to_path_buf(),
        reason: ta("speak-method-unknown", targs!("asked" => asked.to_string())),
        instead,
    })
}

/// A chapter served whole, when its name is asked for. The match is
/// a case-insensitive prefix, so "Додаток Б" and "конституція" both
/// find their chapter.
fn whole_chapter(wanted: &str) -> Option<String> {
    if wanted.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let lowered = wanted.to_lowercase();
    let mut carried: Option<String> = None;
    let mut taking = false;
    for line in METHOD.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            if taking {
                break;
            }
            taking = title.to_lowercase().starts_with(&lowered);
            if taking {
                carried = Some(format!("## {title}\n"));
            }
            continue;
        }
        if taking && let Some(text) = carried.as_mut() {
            text.push_str(line);
            text.push('\n');
        }
    }
    carried.map(|text| format!("{}\n", text.trim_end().trim_end_matches("---").trim_end()))
}

/// A rule written as "N. **Text**" -- the shape the Constitution
/// uses instead of §-paragraphs.
fn numbered_rule(line: &str) -> Option<(String, String)> {
    let (number, rest) = line.split_once(". ")?;
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    rest.starts_with("**")
        .then(|| (number.to_string(), rest.to_string()))
}

/// The methodology split into chapters, each with its paragraphs.
fn chapters() -> Vec<(&'static str, Vec<(String, String)>)> {
    let mut chapters: Vec<(&'static str, Vec<(String, String)>)> = Vec::new();
    let mut carried: Option<(String, String)> = None;
    for line in METHOD.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            if let (Some((number, text)), Some(last)) = (carried.take(), chapters.last_mut()) {
                last.1.push((number, text));
            }
            chapters.push((title, Vec::new()));
            continue;
        }
        // A chapter written as numbered RULES rather than
        // §-paragraphs (the Constitution) has pieces too: counting
        // only § made it read "0" in the contents -- true by the
        // letter and puzzling to a reader, which is the same defect
        // as a silent one.
        if let Some((number, opening)) = numbered_rule(line) {
            if let (Some((number, text)), Some(last)) = (carried.take(), chapters.last_mut()) {
                last.1.push((number, text));
            }
            carried = Some((number.clone(), format!("{number}. {opening}")));
            continue;
        }
        if let Some(rest) = line.strip_prefix("**§")
            && let Some((number, opening)) = rest.split_once(".**")
        {
            if let (Some((number, text)), Some(last)) = (carried.take(), chapters.last_mut()) {
                last.1.push((number, text));
            }
            carried = Some((number.to_string(), format!("**§{number}.**{opening}")));
            continue;
        }
        if let Some((_, text)) = carried.as_mut() {
            text.push('\n');
            text.push_str(line);
        }
    }
    if let (Some((number, text)), Some(last)) = (carried.take(), chapters.last_mut()) {
        last.1.push((number, text));
    }
    chapters
}
