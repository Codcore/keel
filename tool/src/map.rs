//! The quality map (contract tool-map; §10.7): every cut mapped to
//! what closes it or how it is decided. The map draws from the §10.3
//! records and the tags -- it runs no tests (the full court is keel
//! close) and does not fake the reviewer's per-row confirmation.

use crate::adapter;
use crate::config;
use crate::docs;
use crate::graph;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::rev;
use crate::scope;
use crate::tags::{self, TestTag};
use crate::targs;
use std::path::Path;

/// Draws the map: the branch's wave where the branch is named as one
/// (§8.2) -- the reviewer package item (§9.9) -- and the project map
/// anywhere else; which view and why is said aloud first.
pub fn draw(root: &Path) -> Result<String, Refusal> {
    let config = config::read(root)?;
    let scan = docs::scan(root)?;
    if let Some(refusal) = scan.refusals.into_iter().next() {
        // A map over documents it cannot read would decorate shadows;
        // check names every broken file -- fix them first.
        return Err(refusal);
    }
    let found: Option<Vec<TestTag>> = if config.adapter_known() {
        Some(tags::scan(&adapter::test_files(root)?)?)
    } else {
        None
    };
    // The honest unread word (review 0017 R-3): a named yet unknown
    // adapter is not painted absent.
    let unread = if config.adapter.is_some() {
        ta(
            "map-proof-unknown",
            targs!("known" => crate::config::Language::known()),
        )
    } else {
        t("map-proof-unread")
    };

    let mut report = t("map-title");
    report.push('\n');

    if let Some(slug) = scope::branch_wave(root, &scan.waves) {
        let wave = scan.waves.iter().find(|w| w.slug == slug).unwrap();
        report.push_str(&ta("map-view-wave", targs!("wave" => slug.clone())));
        report.push_str("\n\n");
        let wave_path = root.join("keel/waves").join(format!("{}.md", slug));
        let revs = rev::scenario_revs(&wave_path)?;
        for cut in graph::cuts() {
            let answer = wave_answer(wave, cut, &revs, found.as_deref(), &unread);
            report.push_str(&format!("  {cut} — {answer}\n"));
        }
    } else {
        let branch = scope::current_branch(root).unwrap_or_else(|| "?".to_string());
        report.push_str(&ta("map-view-project", targs!("branch" => branch)));
        report.push_str("\n\n");
        // Every wave's scenario revisions read once, not once per
        // cut (review R-2): the truth is the same, the reading is
        // one per wave.
        let mut per_wave: Vec<(&docs::Wave, Vec<(String, String)>)> = Vec::new();
        for wave in &scan.waves {
            let wave_path = root.join("keel/waves").join(format!("{}.md", wave.slug));
            per_wave.push((wave, rev::scenario_revs(&wave_path)?));
        }
        for cut in graph::cuts() {
            // Answers gathered wave by wave, in name order: the
            // youngest wave's word speaks, the older are counted.
            let mut answers: Vec<String> = Vec::new();
            for (wave, revs) in &per_wave {
                if let Some(answer) = wave_answer_if_any(wave, cut, revs, found.as_deref(), &unread)
                {
                    answers.push(format!("{answer} ({})", wave.slug));
                }
            }
            let line = match answers.len() {
                0 => t("map-unanswered"),
                n => {
                    let youngest = answers.last().unwrap().clone();
                    if n > 1 {
                        format!(
                            "{youngest}; {}",
                            ta("map-older", targs!("count" => (n - 1) as u64))
                        )
                    } else {
                        youngest
                    }
                }
            };
            report.push_str(&format!("  {cut} — {line}\n"));
        }
    }
    Ok(report)
}

/// The wave's answer for one cut -- or the honest word that there is
/// none (the silence court is keel check).
fn wave_answer(
    wave: &docs::Wave,
    cut: &str,
    revs: &[(String, String)],
    found: Option<&[TestTag]>,
    unread: &str,
) -> String {
    wave_answer_if_any(wave, cut, revs, found, unread).unwrap_or_else(|| t("map-unanswered"))
}

fn wave_answer_if_any(
    wave: &docs::Wave,
    cut: &str,
    revs: &[(String, String)],
    found: Option<&[TestTag]>,
    unread: &str,
) -> Option<String> {
    // A live cover speaks first; a dead cover does not count (§2.12),
    // so the decision -- the answer that remains -- speaks instead.
    let live = wave
        .scenarios
        .iter()
        .find(|(_, sc)| sc.withdrawn.is_none() && sc.covers.iter().any(|c| c == cut));
    if let Some((name, _)) = live {
        let proof = match found {
            None => unread.to_string(),
            Some(tags) => {
                let current = revs
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, r)| r.as_str())
                    .unwrap_or("");
                let proven = tags
                    .iter()
                    .any(|t| t.scenario == *name && rev::matches(&t.rev, current));
                if proven {
                    t("map-proof-proven")
                } else {
                    t("map-proof-unproven")
                }
            }
        };
        return Some(ta(
            "map-covered",
            targs!("scenario" => name.clone(), "proof" => proof),
        ));
    }
    wave.decisions
        .iter()
        .find(|(c, _)| c == cut)
        .map(|(_, reason)| {
            // Word for word, with whitespace runs collapsed to one space
            // (§5.4's school; review R-3): a multiline reason must not
            // break the one-row-per-cut shape.
            let flat = reason.split_whitespace().collect::<Vec<_>>().join(" ");
            ta("map-decided", targs!("reason" => flat))
        })
}
