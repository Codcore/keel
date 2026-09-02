//! The first floor of checks: "documents read" (bootstrap rung 1).
//! Not the whole methodology check yet -- and the report says so
//! itself: green about the unchecked is forbidden (lesson 4 of the
//! notes triage).

use crate::adapter;
use crate::config::Config;
use crate::docs;
use crate::graph;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::rev;
use crate::scope;
use crate::tags;
use crate::targs;
use std::fmt::Write as _;
use std::path::Path;
use std::process::Command;

pub struct Outcome {
    pub report: String,
    pub findings: usize,
}

/// Walks the documents under the root and reports on every file:
/// intact ones checked, broken ones named with a reason; at the end
/// -- what this floor has checked, what it has not yet, and the next
/// step. The report language is lang from the config; the report
/// also says where the language came from.
pub fn run(root: &Path, config: &Config) -> Result<Outcome, Refusal> {
    let scan = docs::scan(root)?;

    let mut rows: Vec<(String, Option<String>)> = Vec::new();
    for wave in &scan.waves {
        rows.push((format!("keel/waves/{}.md", wave.slug), None));
    }
    for contract in &scan.contracts {
        rows.push((format!("keel/contracts/{}.md", contract.slug), None));
    }
    for refusal in &scan.refusals {
        let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
        rows.push((
            shown.display().to_string(),
            Some(format!(
                "{}\n           {}: {}",
                refusal.reason,
                t("word-instead"),
                refusal.instead
            )),
        ));
    }
    // The second floor (§7.1/§7.3): every contract reference in a
    // wave header is followed to its file and its revision compared.
    // A mismatch is not yet a verdict: an old revision that truly
    // lived in the file's git history is legal (§5.6) -- and a
    // truncated history gets a word, not a judgement.
    let shallow = is_shallow(root);
    let has_history = has_git(root);
    let mut ref_rows: std::collections::BTreeSet<(String, String)> = Default::default();
    let mut refs_checked: u64 = 0;
    let mut refs_historic: u64 = 0;
    let mut refs_unjudged: u64 = 0;
    let mut refs_no_history: u64 = 0;
    for wave in &scan.waves {
        let wave_path = format!("keel/waves/{}.md", wave.slug);
        // A withdrawn scenario is outside judgement (§2.12): its
        // proves is not followed -- a guard that lies gets deleted.
        let mut refs: Vec<&docs::ContractRef> = wave
            .scenarios
            .iter()
            .filter(|(_, sc)| sc.withdrawn.is_none())
            .filter_map(|(_, sc)| sc.proves.as_ref())
            .collect();
        for (_, transform) in &wave.transforms {
            refs.extend(transform.contracts.iter());
        }
        for reference in refs {
            let contract_path = root
                .join("keel/contracts")
                .join(format!("{}.md", reference.slug));
            let verdict = if !contract_path.is_file() {
                Some((
                    ta(
                        "check-ref-missing",
                        targs!("wave" => wave.slug.clone(), "contract" => reference.slug.clone(), "recorded" => reference.rev.clone()),
                    ),
                    ta(
                        "check-ref-missing-instead",
                        targs!("contract" => reference.slug.clone()),
                    ),
                ))
            } else {
                match rev::contract_rev(&contract_path) {
                    Ok(actual) if rev::matches(&reference.rev, &actual) => {
                        refs_checked += 1;
                        None
                    }
                    Ok(actual) => {
                        let relative = format!("keel/contracts/{}.md", reference.slug);
                        if !has_history {
                            // "Where there is no history -- no
                            // verdict" (§5.6; review R-2): the
                            // absence of git is not the wave's fault.
                            refs_no_history += 1;
                            None
                        } else if shallow {
                            refs_unjudged += 1;
                            None
                        } else if revision_in_history(root, &relative, &reference.rev) {
                            refs_historic += 1;
                            None
                        } else {
                            Some((
                                ta(
                                    "check-ref-stale",
                                    targs!("wave" => wave.slug.clone(), "contract" => reference.slug.clone(), "recorded" => reference.rev.clone(), "actual" => actual),
                                ),
                                t("check-ref-stale-instead"),
                            ))
                        }
                    }
                    Err(refusal) => Some((refusal.reason, refusal.instead)),
                }
            };
            if let Some((reason, instead)) = verdict {
                ref_rows.insert((
                    wave_path.clone(),
                    format!("{reason}\n           {}: {instead}", t("word-instead")),
                ));
            }
        }
    }
    for (path, text) in ref_rows {
        rows.push((path, Some(text)));
    }

    // The graph floor (chapter 3): in-wave and cross-wave links.
    for wave in &scan.waves {
        let wave_path = format!("keel/waves/{}.md", wave.slug);
        for (reason, instead) in graph::wave_findings(wave) {
            rows.push((
                wave_path.clone(),
                Some(format!(
                    "{reason}\n           {}: {instead}",
                    t("word-instead")
                )),
            ));
        }
    }
    for (wave_slug, reason, instead) in graph::cross_findings(&scan.waves) {
        rows.push((
            format!("keel/waves/{wave_slug}.md"),
            Some(format!(
                "{reason}\n           {}: {instead}",
                t("word-instead")
            )),
        ));
    }

    // The scope floor (chapter 4): the branch judged against the
    // declared files -- or an honest line that no judging happened.
    // Not compared is not a finding: the deviation is named, green is
    // not painted over the unverified.
    let scope_status = match scope::current_branch(root) {
        None => t("check-scope-skipped-no-git"),
        Some(branch) => match scope::branch_wave(root, &scan.waves) {
            None => ta("check-scope-skipped-not-wave", targs!("branch" => branch)),
            Some(slug) => {
                let wave_path = format!("keel/waves/{slug}.md");
                let wave = scan.waves.iter().find(|w| w.slug == slug).unwrap();
                let compared = scope::compare_base(root)
                    .and_then(|base| scope::findings(root, wave).map(|list| (base, list)));
                match compared {
                    Ok(((sha, from_main), list)) => {
                        for (reason, instead) in list {
                            rows.push((
                                wave_path.clone(),
                                Some(format!(
                                    "{reason}\n           {}: {instead}",
                                    t("word-instead")
                                )),
                            ));
                        }
                        let short = sha.get(..7).unwrap_or(&sha).to_string();
                        let base_text = if from_main {
                            ta("check-scope-base-main", targs!("sha" => short))
                        } else {
                            ta("check-scope-base-first", targs!("sha" => short))
                        };
                        ta(
                            "check-scope-compared",
                            targs!("branch" => slug, "base" => base_text),
                        )
                    }
                    Err(refusal) => {
                        rows.push((
                            wave_path,
                            Some(format!(
                                "{}\n           {}: {}",
                                refusal.reason,
                                t("word-instead"),
                                refusal.instead
                            )),
                        ));
                        t("check-scope-skipped-refused")
                    }
                }
            }
        },
    };
    // The tag floor (§5.5, §7.5): proves tags in the test files the
    // adapter names, judged against the scenarios' current revisions.
    // A withdrawn scenario's tag is not judged (§2.12). Only the
    // cargo adapter is served on this rung -- anything else is a
    // skip said aloud, never a silent green.
    let mut tags_checked: u64 = 0;
    let tags_status = match config.adapter.as_deref() {
        None => t("check-tags-skipped-no-adapter"),
        Some("cargo") => match tag_rows(root, &scan.waves, &mut tags_checked) {
            Ok(tag_findings) => {
                for (path, text) in tag_findings {
                    rows.push((path, Some(text)));
                }
                ta("check-tags-count", targs!("count" => tags_checked))
            }
            Err(refusal) => {
                let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
                rows.push((
                    shown.display().to_string(),
                    Some(format!(
                        "{}\n           {}: {}",
                        refusal.reason,
                        t("word-instead"),
                        refusal.instead
                    )),
                ));
                t("check-tags-skipped-refused")
            }
        },
        Some(other) => ta(
            "check-tags-skipped-adapter",
            targs!("name" => other.to_string()),
        ),
    };
    rows.sort();

    let mut report = t("check-title");
    report.push('\n');
    let config_line = if !config.present {
        t("check-config-absent")
    } else if config.lang_set {
        ta(
            "check-config-present",
            targs!("lang" => config.lang.clone()),
        )
    } else {
        t("check-config-lang-default")
    };
    writeln!(report, "{config_line}\n").unwrap();

    for (path, verdict) in &rows {
        match verdict {
            None => {
                writeln!(
                    report,
                    "  {:<8} {path} — {}",
                    t("word-green"),
                    t("check-header-reads")
                )
                .unwrap();
            }
            Some(text) => {
                writeln!(report, "  {:<8} {path} — {text}", t("word-red")).unwrap();
            }
        }
    }
    if rows.is_empty() {
        writeln!(report, "  {}", t("check-no-documents")).unwrap();
    }

    let findings = rows.iter().filter(|(_, v)| v.is_some()).count();
    let documents = scan.waves.len() + scan.contracts.len();
    writeln!(
        report,
        "\n{}",
        ta("check-refs-count", targs!("count" => refs_checked))
    )
    .unwrap();
    if refs_historic > 0 {
        writeln!(
            report,
            "{}",
            ta("check-refs-historic", targs!("count" => refs_historic))
        )
        .unwrap();
    }
    if refs_unjudged > 0 {
        writeln!(report, "{}", t("check-refs-shallow")).unwrap();
    }
    if refs_no_history > 0 {
        writeln!(report, "{}", t("check-refs-no-history")).unwrap();
    }
    writeln!(
        report,
        "{}\n{}\n{}\n{}\n{}",
        tags_status,
        scope_status,
        t("check-checked"),
        t("check-unchecked"),
        ta(
            "check-summary",
            targs!("docs" => documents as u64, "refusals" => findings as u64)
        )
    )
    .unwrap();
    let next = if findings > 0 {
        t("check-next-fix")
    } else if documents == 0 {
        t("check-next-first-wave")
    } else {
        t("check-next-rung")
    };
    writeln!(report, "{next}").unwrap();

    Ok(Outcome { report, findings })
}

/// The tag floor's findings: stale tags and orphan tags, judged
/// against every wave's scenario revisions; matching tags counted.
/// A scenario slug may live in several waves -- any matching
/// revision passes, and the stale message shows the first.
fn tag_rows(
    root: &Path,
    waves: &[docs::Wave],
    checked: &mut u64,
) -> Result<Vec<(String, String)>, Refusal> {
    let files = adapter::test_files(root)?;
    let found = tags::scan(&files)?;

    let mut revs: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    let mut withdrawn: std::collections::BTreeSet<String> = Default::default();
    for wave in waves {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        for (name, revision) in rev::scenario_revs(&path)? {
            let entry = wave.scenarios.iter().find(|(n, _)| *n == name);
            if entry.is_some_and(|(_, sc)| sc.withdrawn.is_some()) {
                withdrawn.insert(name.clone());
            }
            revs.entry(name).or_default().push(revision);
        }
    }

    let mut out = Vec::new();
    for tag in &found {
        if withdrawn.contains(&tag.scenario) {
            continue;
        }
        let shown = tag.file.strip_prefix(root).unwrap_or(&tag.file);
        match revs.get(&tag.scenario) {
            None => out.push((
                shown.display().to_string(),
                format!(
                    "{}\n           {}: {}",
                    ta(
                        "tags-orphan",
                        targs!("test" => tag.test.clone(), "scenario" => tag.scenario.clone()),
                    ),
                    t("word-instead"),
                    t("tags-orphan-instead")
                ),
            )),
            Some(current) if current.iter().any(|c| rev::matches(&tag.rev, c)) => {
                *checked += 1;
            }
            Some(current) => out.push((
                shown.display().to_string(),
                format!(
                    "{}\n           {}: {}",
                    ta(
                        "tags-stale",
                        targs!("test" => tag.test.clone(), "scenario" => tag.scenario.clone(), "recorded" => tag.rev.clone(), "actual" => current[0].clone()),
                    ),
                    t("word-instead"),
                    t("tags-stale-instead")
                ),
            )),
        }
    }
    Ok(out)
}

/// Whether the recorded revision truly lived in the git history of
/// the contract file (§5.6). Any git trouble reads as "not found":
/// the strict verdict stands where history cannot testify.
fn revision_in_history(root: &Path, relative: &str, recorded: &str) -> bool {
    let log = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["log", "--format=%H", "--", relative])
        .output();
    let Ok(log) = log else { return false };
    if !log.status.success() {
        return false;
    }
    for sha in String::from_utf8_lossy(&log.stdout).lines() {
        let show = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(["show", &format!("{}:{relative}", sha.trim())])
            .output();
        let Ok(show) = show else { continue };
        if !show.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&show.stdout);
        if rev::matches(recorded, &rev::text_rev(&text)) {
            return true;
        }
    }
    false
}

/// Whether git serves this root at all -- without it there is no
/// history to testify, and no verdict is passed on old revisions
/// (§5.6).
fn has_git(root: &Path) -> bool {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-dir"])
        .output();
    matches!(out, Ok(o) if o.status.success())
}

/// A shallow clone's history is truncated -- old revisions cannot be
/// verified there, and the absence of history is not the wave's
/// fault.
fn is_shallow(root: &Path) -> bool {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--is-shallow-repository"])
        .output();
    matches!(out, Ok(o) if o.status.success()
        && String::from_utf8_lossy(&o.stdout).trim() == "true")
}
