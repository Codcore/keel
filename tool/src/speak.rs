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
/// The same checklist in the other tongue of this release (wave
/// 0028, the operator's decision: nothing Ukrainian where the
/// settings say English, and nothing English where they say
/// Ukrainian).
const CHECKLIST_UK: &str = include_str!("../../docs/uk/QUALITY.md");
const METHOD: &str = include_str!("../../docs/uk/METHODOLOGY-V2.md");
/// The methodology in the other tongue of this release (wave 0029:
/// the second half of the operator's decision -- nothing Ukrainian
/// where the settings say English). The Ukrainian text stays the
/// source of truth, and the English one says so in its own opening.
const METHOD_EN: &str = include_str!("../../docs/en/METHODOLOGY-V2.md");

/// The checklist this release was built with -- handed out so a
/// caller can judge it against the courts' own list without a second
/// copy living anywhere.
pub fn checklist() -> &'static str {
    CHECKLIST
}

/// The checklist in the language a project speaks.
pub fn checklist_for(lang: &str) -> &'static str {
    match lang {
        "uk" => CHECKLIST_UK,
        _ => CHECKLIST,
    }
}

/// Every checklist this release carries, by language -- so a court
/// can judge them all rather than the one that happens to be served.
pub fn checklists() -> Vec<(&'static str, &'static str)> {
    vec![("en", CHECKLIST), ("uk", CHECKLIST_UK)]
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
    // Which document this is, so a refusal can name it. A checklist
    // handed in from elsewhere (a probe, a doctored copy) is judged
    // as the tongue it matches, and named as the tongue it is
    // (review 0028 R-6: every refusal used to name QUALITY.md, the
    // English file, whichever document it had just judged).
    let named = if checklist == CHECKLIST_UK {
        "docs/uk/QUALITY.md"
    } else {
        "QUALITY.md"
    };
    let mut family = String::new();
    let mut found: Vec<(String, &str)> = Vec::new();
    let mut hollow: Vec<String> = Vec::new();
    // The nine headings a person reads, in the order they stand.
    // Judged too (review 0028 R-3): a document whose items carry full
    // slugs used to skip the family branch entirely, so all nine
    // headings could be deleted from the Ukrainian checklist and both
    // `keel cuts` and `keel check` stayed green while the document a
    // person reads had become wrong.
    let mut headings: Vec<String> = Vec::new();
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
            headings.push(title.to_string());
            continue;
        }
        let Some(rest) = line.strip_prefix("- **") else {
            continue;
        };
        let Some((name, question)) = rest.split_once("** — ") else {
            continue;
        };
        // A name that carries a dot IS the slug. That is how a
        // TRANSLATED checklist keeps the machine names a person types
        // into decisions: while translating its headings and its
        // questions around them (wave 0028).
        let slug = if name.contains('.') {
            name.trim().to_string()
        } else {
            if family.is_empty() {
                continue;
            }
            format!("{family}.{}", name.replace(' ', "-"))
        };
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
            file: Path::new(named).to_path_buf(),
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
            file: Path::new(named).to_path_buf(),
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
            file: Path::new(named).to_path_buf(),
            reason: ta(
                "speak-cuts-stray",
                targs!("cuts" => stray.join(", "), "count" => stray.len().to_string()),
            ),
            instead: t("speak-cuts-stray-instead"),
        });
    }
    // Nine headings, one per family, in the order the courts hold the
    // families: what a person reads must be grouped the way the
    // vocabulary is grouped, in every tongue.
    let families: Vec<&str> = {
        let mut seen: Vec<&str> = Vec::new();
        for slug in crate::graph::cuts() {
            let (family, _) = slug.split_once('.').unwrap_or((slug, slug));
            if !seen.contains(&family) {
                seen.push(family);
            }
        }
        seen
    };
    if headings.len() != families.len() {
        return Err(Refusal {
            file: Path::new(named).to_path_buf(),
            reason: ta(
                "speak-cuts-headings",
                targs!("read" => headings.len().to_string(), "judged" => families.len().to_string()),
            ),
            instead: t("speak-cuts-headings-instead"),
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
            file: Path::new(named).to_path_buf(),
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
pub fn cuts_report(lang: &str) -> Result<String, Refusal> {
    // The court lives in cuts_from, not at the call site: a second
    // buyer of this contract used to get an empty list and a title
    // saying "forty" above it (review 0027 R-5).
    let paired = cuts_from(checklist_for(lang))?;
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

/// Every methodology this release carries, by language -- so a court
/// can judge their skeletons against each other.
pub fn methods() -> Vec<(&'static str, &'static str)> {
    vec![("en", METHOD_EN), ("uk", METHOD)]
}

/// The methodology in the language a project speaks.
pub fn method_for(lang: &str) -> &'static str {
    // Named, not defaulted: a language added to LANGUAGES and to
    // methods() but forgotten here would have read English in
    // silence (review 0029 R-10). The frame refuses an unknown
    // language before this hand is ever reached; this arm is the
    // second lock, and it names which tongue it fell back to.
    match lang {
        "uk" => METHOD,
        "en" => METHOD_EN,
        _ => METHOD_EN,
    }
}

/// The skeletons of two methodologies held against each other: the
/// same chapters in the same order, the same paragraph numbers, none
/// of them empty.
pub fn methods_agree() -> Result<(), Refusal> {
    translation_is_current()?;
    methods_agree_from(&methods())
}

/// Whether the translation was made from the Ukrainian text as it now
/// stands (constitution, rule 4). Review 0029 R-2: nothing tied the
/// two documents together, so the original could be rewritten and the
/// translation left behind -- the skeletons would still agree, the
/// gate would stay green, and nobody would learn of it for a year.
/// The methodology already owns the mechanism for exactly this, and
/// it is the mechanism used here: the English text records the
/// revision it was translated from, and a stale record is a finding.
pub fn translation_is_current() -> Result<(), Refusal> {
    let recorded = METHOD_EN
        .lines()
        .find_map(|line| {
            line.split_once("`translated_from: ")
                .and_then(|(_, rest)| rest.split_once('`'))
                .map(|(revision, _)| revision.trim().to_string())
        })
        .ok_or_else(|| Refusal {
            file: Path::new(named_method("en")).to_path_buf(),
            reason: t("speak-method-unrecorded"),
            instead: t("speak-method-unrecorded-instead"),
        })?;
    let standing = crate::rev::text_rev(METHOD);
    if crate::rev::matches(&recorded, &standing) {
        return Ok(());
    }
    Err(Refusal {
        file: Path::new(named_method("en")).to_path_buf(),
        reason: ta(
            "speak-method-stale",
            targs!("recorded" => recorded, "standing" => standing),
        ),
        instead: t("speak-method-stale-instead"),
    })
}

/// The same, over documents handed in -- so the court can be PLAYED
/// and not merely promised, which is the norm this contract already
/// sets for the checklist and did not keep for the methodology
/// (review 0029 R-3: `methods_agree()` took no argument, so a probe
/// could not feed it a broken text, and a mutant that always
/// answered Ok passed all eighty-seven tests).
pub fn methods_agree_from(texts: &[(&str, &'static str)]) -> Result<(), Refusal> {
    // The shape of one text: its chapters, each with the numbers of
    // its pieces.
    type Shape<'a> = Vec<(&'a str, Vec<String>)>;
    let mut carried: Option<(&str, Shape<'_>)> = None;
    for (lang, text) in texts {
        let chapters = chapters_of(text);
        // A chapter with an EMPTY BODY is a translation stopped at
        // its heading. Not "a chapter without pieces": the three
        // appendices are prose and carry none by design, and the two
        // texts must simply agree on that -- which the shape
        // comparison below does. Before this, the Constitution's
        // eight rules and all three appendices sat outside the court
        // entirely, and every rule could be deleted while the gate
        // stayed green (review 0029 R-4).
        if let Some(title) = empty_chapter(text) {
            return Err(Refusal {
                file: Path::new(named_method(lang)).to_path_buf(),
                reason: ta(
                    "speak-method-empty-chapter",
                    targs!("chapter" => title, "lang" => (*lang).to_string()),
                ),
                instead: t("speak-method-skeleton-instead"),
            });
        }
        if let Some(number) = hollow_paragraph(text) {
            return Err(Refusal {
                file: Path::new(named_method(lang)).to_path_buf(),
                reason: ta(
                    "speak-method-hollow",
                    targs!("number" => number, "lang" => (*lang).to_string()),
                ),
                instead: t("speak-method-hollow-instead"),
            });
        }
        let shape: Shape<'_> = chapters
            .iter()
            .map(|(title, pieces)| {
                (
                    *title,
                    pieces.iter().map(|(number, _)| number.clone()).collect(),
                )
            })
            .collect();
        match &carried {
            None => carried = Some((lang, shape)),
            Some((first_lang, first_shape)) => {
                if first_shape.len() != shape.len() {
                    return Err(Refusal {
                        file: Path::new(named_method(lang)).to_path_buf(),
                        reason: ta(
                            "speak-method-chapters",
                            targs!(
                                "lang" => (*lang).to_string(),
                                "read" => shape.len().to_string(),
                                "other" => (*first_lang).to_string(),
                                "judged" => first_shape.len().to_string()
                            ),
                        ),
                        instead: t("speak-method-skeleton-instead"),
                    });
                }
                // Chapter by chapter, piece by piece: a number that
                // moved to another chapter is drift even when the
                // whole list still matches.
                for (at, ((_, first_numbers), (_, numbers))) in
                    first_shape.iter().zip(shape.iter()).enumerate()
                {
                    if first_numbers != numbers {
                        let disagreement = first_numbers
                            .iter()
                            .zip(numbers.iter())
                            .find(|(a, b)| a != b)
                            .map_or_else(
                                || {
                                    ta(
                                        "speak-method-count",
                                        targs!(
                                            "at" => (at + 1).to_string(),
                                            "other" => (*first_lang).to_string(),
                                            "judged" => first_numbers.len().to_string(),
                                            "lang" => (*lang).to_string(),
                                            "read" => numbers.len().to_string()
                                        ),
                                    )
                                },
                                |(a, b)| {
                                    // Each side named with ITS OWN
                                    // tongue: the pair used to run in
                                    // the opposite order to the
                                    // sentence around it (R-7).
                                    ta(
                                        "speak-method-numbers",
                                        targs!(
                                            "other" => (*first_lang).to_string(),
                                            "first" => a.clone(),
                                            "lang" => (*lang).to_string(),
                                            "second" => b.clone()
                                        ),
                                    )
                                },
                            );
                        return Err(Refusal {
                            file: Path::new(named_method(lang)).to_path_buf(),
                            reason: disagreement,
                            instead: t("speak-method-skeleton-instead"),
                        });
                    }
                }
            }
        }
    }
    Ok(())
}

/// The title of the first chapter with nothing under its heading.
fn empty_chapter(text: &'static str) -> Option<String> {
    let mut standing: Option<(String, bool)> = None;
    for line in text.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            if let Some((title, empty)) = standing.take()
                && empty
            {
                return Some(title);
            }
            standing = Some((title.to_string(), true));
            continue;
        }
        if let Some((_, empty)) = standing.as_mut()
            && !line.trim().is_empty()
            && line.trim() != "---"
        {
            *empty = false;
        }
    }
    standing.and_then(|(title, empty)| empty.then_some(title))
}

/// The number of the first paragraph whose body is empty, if any.
fn hollow_paragraph(text: &str) -> Option<String> {
    let mut standing: Option<(String, bool)> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("**§")
            && let Some((number, opening)) = rest.split_once(".**")
        {
            if let Some((number, empty)) = standing.take()
                && empty
            {
                return Some(number);
            }
            standing = Some((number.to_string(), opening.trim().is_empty()));
            continue;
        }
        if let Some((_, empty)) = standing.as_mut()
            && !line.trim().is_empty()
            && !line.starts_with("## ")
        {
            *empty = false;
        }
    }
    standing.and_then(|(number, empty)| empty.then_some(number))
}

/// The file a methodology lives in, for a refusal to name.
fn named_method(lang: &str) -> &'static str {
    match lang {
        "uk" => "docs/uk/METHODOLOGY-V2.md",
        _ => "docs/en/METHODOLOGY-V2.md",
    }
}

/// The methodology: its contents, or one paragraph of it.
pub fn method(lang: &str, asked: Option<&str>) -> Result<String, Refusal> {
    let text = method_for(lang);
    let chapters = chapters_of(text);
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
    if let Some(said) = whole_chapter(text, wanted) {
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
        .filter(|number| number.contains('.') && number.split('.').next() == Some(head))
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
        file: Path::new(named_method(lang)).to_path_buf(),
        reason: ta("speak-method-unknown", targs!("asked" => asked.to_string())),
        instead,
    })
}

/// A chapter served whole, when its name is asked for. The match is
/// a case-insensitive prefix, so "Додаток Б" and "конституція" both
/// find their chapter.
fn whole_chapter(text: &'static str, wanted: &str) -> Option<String> {
    if wanted.chars().all(|c| c.is_ascii_digit() || c == '.') {
        return None;
    }
    let lowered = wanted.to_lowercase();
    let mut carried: Option<String> = None;
    let mut taking = false;
    for line in text.lines() {
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

/// The methodology split into chapters, each with its pieces.
///
/// A chapter's pieces are its `**§N.M.**` paragraphs. A chapter that
/// has none of those -- the Constitution, written as eight numbered
/// rules -- is split by those rules instead, so that "Конституція —
/// 0" stops reading as "there is nothing here". A numbered line
/// INSIDE a §-paragraph is a list item and never a piece: taking it
/// for one served §6.3 at 9% of itself and made the contents count
/// pieces that do not exist (review 0029 R-5).
fn chapters_of(text: &'static str) -> Vec<(&'static str, Vec<(String, String)>)> {
    let mut chapters: Vec<(&'static str, Vec<(String, String)>)> = Vec::new();
    let mut bodies: Vec<Vec<&str>> = Vec::new();
    for line in text.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            chapters.push((title, Vec::new()));
            bodies.push(Vec::new());
            continue;
        }
        if let Some(body) = bodies.last_mut() {
            body.push(line);
        }
    }
    for (at, body) in bodies.iter().enumerate() {
        let paragraphs = if body.iter().any(|line| line.starts_with("**§")) {
            pieces(body, |line| {
                line.strip_prefix("**§")
                    .and_then(|rest| rest.split_once(".**"))
                    .map(|(number, opening)| {
                        (number.to_string(), format!("**§{number}.**{opening}"))
                    })
            })
        } else {
            pieces(body, |line| {
                numbered_rule(line)
                    .map(|(number, opening)| (number.clone(), format!("{number}. {opening}")))
            })
        };
        chapters[at].1 = paragraphs;
    }
    chapters
}

/// The pieces of one chapter's body, by whatever opens a piece.
fn pieces(
    body: &[&str],
    opens: impl Fn(&str) -> Option<(String, String)>,
) -> Vec<(String, String)> {
    let mut found: Vec<(String, String)> = Vec::new();
    let mut carried: Option<(String, String)> = None;
    for line in body {
        if let Some((number, opening)) = opens(line) {
            if let Some(piece) = carried.take() {
                found.push(piece);
            }
            carried = Some((number, opening));
            continue;
        }
        if let Some((_, text)) = carried.as_mut() {
            text.push('\n');
            text.push_str(line);
        }
    }
    if let Some(piece) = carried.take() {
        found.push(piece);
    }
    found
}
