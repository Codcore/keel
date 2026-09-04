//! The first floor of checks: "documents read" (bootstrap rung 1).
//! Not the whole methodology check yet -- and the report says so
//! itself: green about the unchecked is forbidden (lesson 4 of the
//! notes triage).

use crate::adapter;
use crate::config::Config;
use crate::docs;
use crate::graph;
use crate::holding;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::rev;
use crate::scope;
use crate::tags;
use crate::targs;
use crate::trust;
use std::fmt::Write as _;
use std::path::Path;

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

    // A row is a document unless it is a court of the binary itself
    // (wave 0027): those two say different things when green, and
    // "the header reads" over a vocabulary would be a small lie.
    // Each court says its own green word: "the header reads" over a
    // vocabulary would be a small lie, and so would the checklist's
    // word over the methodology's row (wave 0029).
    let mut courts: Vec<(String, String)> = Vec::new();
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
    // The sameness of the vocabulary (wave 0027): the forty cuts the
    // courts judge plan completeness by, and the forty questions the
    // checklist offers a person, must be one list. Judged HERE, at
    // the gate every project already runs -- review 0027 R-2 measured
    // that a drift only reddened `keel cuts`, a command nothing names
    // and nobody has to type, so "the drift is red" was true only if
    // asked. The court is about this BINARY, not about the project,
    // and its row says so.
    // Every tongue this release carries, not only the one it serves:
    // a translation drifts as easily as an original (wave 0028).
    for (lang, document) in crate::speak::checklists() {
        // A person reads a language, not a code (review 0028 R-10).
        let row = ta(
            "check-cuts-row",
            targs!("lang" => t(&format!("word-lang-{lang}"))),
        );
        courts.push((row.clone(), t("check-court-holds")));
        rows.push((
            row,
            crate::speak::cuts_from(document).err().map(|refusal| {
                format!(
                    "{}\n           {}: {}",
                    refusal.reason,
                    t("word-instead"),
                    refusal.instead
                )
            }),
        ));
    }

    // The skeletons of the methodologies (wave 0029), a row PER
    // TONGUE as the wave promised three times and delivered once
    // (review 0029 R-8): each row says whether its own text stands
    // whole, and the pair says whether the two agree.
    for (lang, text) in crate::speak::methods() {
        let row = ta(
            "check-method-row",
            targs!("lang" => t(&format!("word-lang-{lang}"))),
        );
        courts.push((row.clone(), t("check-method-holds")));
        rows.push((
            row,
            crate::speak::methods_agree_from(&[(lang, text)])
                .and_then(|()| crate::speak::methods_agree())
                .err()
                .map(|refusal| {
                    format!(
                        "{}\n           {}: {}",
                        refusal.reason,
                        t("word-instead"),
                        refusal.instead
                    )
                }),
        ));
    }

    // The second floor (§7.1/§7.3): every contract reference in a
    // wave header is followed to its file and its revision compared.
    // A mismatch is not yet a verdict: an old revision that truly
    // lived in the file's git history is legal (§5.6) -- and a
    // truncated history gets a word, not a judgement.
    let shallow = is_shallow(root);
    let has_history = has_git(root);
    // Tags are read once and serve three floors: the tag floor, the
    // §7.15 delta, and the §5.6 narrowing through structural closure.
    let found_tags: Option<Result<Vec<tags::TestTag>, Refusal>> = config
        .rust_adapter()
        .then(|| adapter::test_files(root).and_then(|files| tags::scan(&files)));
    let mut ref_rows: std::collections::BTreeSet<(String, String)> = Default::default();
    let mut refs_checked: u64 = 0;
    let mut refs_historic: u64 = 0;
    let mut refs_unjudged: u64 = 0;
    let mut refs_no_history: u64 = 0;
    let mut historic_items: Vec<String> = Vec::new();
    for wave in &scan.waves {
        // The history blessing belongs to the structurally closed
        // wave only (§5.6 narrowed; review 0005, R-9): an open wave
        // updates its references deliberately (§5.1).
        let closed = match &found_tags {
            Some(Ok(found)) => crate::close::structural(root, wave, found).unwrap_or(false),
            _ => false,
        };
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
                        } else if closed && revision_in_history(root, &relative, &reference.rev) {
                            refs_historic += 1;
                            historic_items.push(ta(
                                "check-refs-historic-item",
                                targs!("wave" => wave.slug.clone(), "contract" => reference.slug.clone(), "recorded" => reference.rev.clone()),
                            ));
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

    // The graph floor (chapter 3): in-wave and cross-wave links --
    // and §7.7's other half beside it: the header and the body agree
    // both ways, an orphan section does not live in silence.
    let tags_judged = matches!(&found_tags, Some(Ok(_)));
    for wave in &scan.waves {
        let wave_path = format!("keel/waves/{}.md", wave.slug);
        // The scenario side of §7.7 runs adapter-free too (review
        // 0011 R-2): where the tag floor did not read the file's
        // scenario revisions, their refusals surface here -- "both
        // ways" stays true for every project, not only cargo.
        if !tags_judged && let Err(refusal) = rev::scenario_revs(&root.join(&wave_path)) {
            push_refusal_row(&mut rows, root, &refusal);
        }
        match rev::body_court(&root.join(&wave_path), wave) {
            Ok(findings) => {
                for (reason, instead) in findings {
                    rows.push((
                        wave_path.clone(),
                        Some(format!(
                            "{reason}\n           {}: {instead}",
                            t("word-instead")
                        )),
                    ));
                }
            }
            Err(refusal) => push_refusal_row(&mut rows, root, &refusal),
        }
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
    let live_contracts: Vec<String> = scan
        .contracts
        .iter()
        .filter(|c| c.withdrawn.is_none())
        .map(|c| c.slug.clone())
        .collect();
    for (wave_slug, reason, instead) in graph::cross_findings(&scan.waves, &live_contracts) {
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
                        // The red birth, judged by the BRANCH (§7.12).
                        // Two audits found this independently: the
                        // paragraph names two holders -- the hook and
                        // a branch check -- and only the hook existed.
                        // A hook lives in .git/hooks, which does not
                        // travel with git, so every fresh clone
                        // worked without the guard: the audit
                        // committed work with no test and no red
                        // commit and got zero findings, and then
                        // `keel close` called the wave closed.
                        for (scenario, instead) in unborn_scenarios(
                            root,
                            wave,
                            &sha,
                            found_tags.as_ref().and_then(|r| r.as_ref().ok()),
                        ) {
                            rows.push((
                                wave_path.clone(),
                                Some(format!(
                                    "{scenario}\n           {}: {instead}",
                                    t("word-instead")
                                )),
                            ));
                        }
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
    let known = config.rust_adapter();
    let judged = match (&config.adapter, &found_tags) {
        (None, _) => Err(t("check-tags-skipped-no-adapter")),
        (Some(_), Some(Ok(found))) if known => {
            match tag_rows(root, &scan.waves, found, &mut tags_checked) {
                Ok(tag_findings) => Ok((found, tag_findings)),
                Err(refusal) => {
                    push_refusal_row(&mut rows, root, &refusal);
                    Err(t("check-tags-skipped-refused"))
                }
            }
        }
        (Some(_), Some(Err(refusal))) if known => {
            push_refusal_row(&mut rows, root, refusal);
            Err(t("check-tags-skipped-refused"))
        }
        (Some(other), _) => Err(ta(
            "check-tags-skipped-adapter",
            targs!("name" => other.to_string()),
        )),
    };
    let tags_status = match judged {
        Ok((found, tag_findings)) => {
            for (path, text) in tag_findings {
                rows.push((path, Some(text)));
            }
            // The §7.15 delta: a tag present at the fork point and
            // gone at HEAD, its scenario alive, is a finding where
            // the disarming happened.
            for (path, text) in vanished_rows(root, &scan.waves, found) {
                rows.push((path, Some(text)));
            }
            ta("check-tags-count", targs!("count" => tags_checked))
        }
        Err(status) => status,
    };

    // The trust floor (§7.16, §2.8): commands from repository files
    // -- verify of live contracts, the project's ci -- judged against
    // the recorded fingerprints. Nothing is run; the court stands
    // before any runner exists. Over rubble it does not judge at all
    // (review R-4): a broken document may hide the very command a
    // record answers to, so a skipped court is said aloud instead of
    // an invented door.
    let trust_status = if !scan.refusals.is_empty() {
        t("check-trust-skipped-broken")
    } else {
        let commands_judged = trust::live_commands(config, &scan.contracts).len() as u64;
        for (place, reason, instead) in trust::court(config, &scan.contracts) {
            rows.push((
                place,
                Some(format!(
                    "{reason}\n           {}: {instead}",
                    t("word-instead")
                )),
            ));
        }
        let mut status = ta("check-trust-count", targs!("count" => commands_judged));
        if config.ci.as_deref() == Some("none") {
            status.push_str(&t("check-trust-ci-none"));
        }
        // The asymmetry named (review R-6): an unwritten ci is not
        // "undecided" -- there is no command in the files -- but the
        // silence is not painted green either.
        if config.present && config.ci.is_none() {
            status.push_str(&t("check-trust-ci-absent"));
        }
        status
    };

    // The form floor (§7.6, §2.9): promised signatures against the
    // module's source, as collapsed text; the incomparable is a
    // word with its reason, never green. On a plan branch the court
    // does not run at all (§8.3; 0010 review R-1): the plan grows
    // exports ahead of the code by design (§4.9), and a gate that is
    // always shut stops being read.
    let plan_branch = scope::current_branch(root).is_some_and(|b| b.starts_with("plan/"));
    let holding_status = if plan_branch {
        t("check-holding-plan")
    } else {
        // The approved-not-started window (§6.5; 0010 review R-1b):
        // a contract grown ahead of the code by a lawful plan is not
        // judged for form while no holding wave has started -- and
        // only when the tags were actually read: distrust of unread
        // tags never widens the window.
        let window = match &found_tags {
            Some(Ok(found)) => holding::plan_window(root, &scan.waves, found, &scan.contracts),
            _ => Vec::new(),
        };
        let judged_contracts: Vec<docs::Contract> = scan
            .contracts
            .iter()
            .filter(|c| !window.iter().any(|(slug, _)| slug == &c.slug))
            .cloned()
            .collect();
        for (place, reason, instead) in holding::court(root, config, &judged_contracts) {
            rows.push((
                place,
                Some(format!(
                    "{reason}\n           {}: {instead}",
                    t("word-instead")
                )),
            ));
        }
        let (signatures_checked, uncompared) = holding::survey(root, config, &judged_contracts);
        let mut status = ta("check-holding-count", targs!("count" => signatures_checked));
        for line in uncompared {
            status.push('\n');
            status.push_str(&line);
        }
        for (contract, wave) in &window {
            status.push('\n');
            status.push_str(&ta(
                "check-holding-window",
                targs!("contract" => contract.clone(), "wave" => wave.clone()),
            ));
        }
        status
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
    writeln!(report, "{config_line}").unwrap();
    // The old spelling is a synonym said aloud, never a silent
    // acceptance (wave 0017): the canonical name is the language's.
    if config.adapter_synonym() {
        writeln!(report, "{}", t("check-adapter-synonym")).unwrap();
    }
    writeln!(report).unwrap();

    for (path, verdict) in &rows {
        match verdict {
            None => {
                let word = courts
                    .iter()
                    .find(|(row, _)| row == path)
                    .map_or_else(|| t("check-header-reads"), |(_, word)| word.clone());
                writeln!(report, "  {:<8} {path} — {word}", t("word-green")).unwrap();
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
        for item in &historic_items {
            writeln!(report, "  {item}").unwrap();
        }
    }
    if refs_unjudged > 0 {
        writeln!(report, "{}", t("check-refs-shallow")).unwrap();
    }
    if refs_no_history > 0 {
        writeln!(report, "{}", t("check-refs-no-history")).unwrap();
    }
    let limits = verdict_limits(root, refs_unjudged);
    for limit in &limits {
        writeln!(report, "{limit}").unwrap();
    }
    writeln!(
        report,
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        tags_status,
        trust_status,
        holding_status,
        scope_status,
        t("check-checked"),
        t("check-borders"),
        ta(
            "check-summary",
            targs!(
                "docs" => documents as u64,
                "refusals" => findings as u64,
                "limits" => limits.len() as u64
            )
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

/// What this verdict could NOT judge, in its own words.
///
/// Wave 0031, measured before the work: a full clone gave a 208-line
/// verdict carrying 141 old revisions verified against file history;
/// a shallow clone gave 67 lines and none of them -- and both ended
/// with the same "0 findings". The line everyone reads said the same
/// thing about two very different amounts of judging.
///
/// Nothing here asks the network. A limit that depends on reaching a
/// server is a limit that changes with the weather, so the question
/// asked is the honest one: what does THIS clone know?
fn verdict_limits(root: &Path, refs_unjudged: u64) -> Vec<String> {
    let mut limits = Vec::new();

    // No git, no questions to ask. Review 0031 R-8: a directory with
    // no repository at all was told its clone "knows no origin/main",
    // which is true of a shoebox as well.
    if !has_git(root) {
        return limits;
    }
    // A repository with no commits yet has no base and no branch to
    // speak of either (R-8).
    if git_line(root, &["rev-parse", "--verify", "--quiet", "HEAD"]).is_none() {
        return limits;
    }

    if is_shallow(root) {
        limits.push(ta("limit-shallow", targs!("skipped" => refs_unjudged)));
    }

    // The base the scope court compares a wave branch against, under
    // the trunk's own name and the remote's own name -- review 0031
    // R-8 found both hard-coded, so a project on `master` was told to
    // push its trunk and a project whose remote is `upstream` was
    // told its work does not exist.
    let remote = remote_name(root);
    let trunk = trunk_name(root, remote.as_deref());
    let base = remote
        .as_ref()
        .map(|remote| format!("{remote}/{trunk}"))
        .filter(|base| git_line(root, &["rev-parse", "--verify", "--quiet", base]).is_some());
    match &base {
        Some(base) => {
            let behind = git_line(root, &["rev-list", "--count", &format!("{trunk}..{base}")])
                .and_then(|n| n.parse::<u64>().ok())
                .unwrap_or(0);
            if behind > 0 {
                limits.push(ta(
                    "limit-base-stale",
                    targs!("behind" => behind, "trunk" => trunk.clone(), "base" => base.clone()),
                ));
            }
        }
        None => limits.push(ta(
            "limit-base-local-only",
            targs!("trunk" => trunk.clone()),
        )),
    }

    // Whether the branch being judged has reached origin, as far as
    // this clone knows -- a wave closed only on this disk is not
    // closed.
    // Through scope::current_branch, which knows that the top of the
    // git tree may not be the root of the project: review 0031 R-7
    // caught this asking git directly and naming the PARENT
    // repository's branch for a project living in a subdirectory.
    if let (Some(remote), Some(branch)) = (&remote, scope::current_branch(root))
        && branch != trunk
    {
        let there = format!("{remote}/{branch}");
        match git_line(root, &["rev-parse", "--verify", "--quiet", &there]) {
            None => limits.push(ta(
                "limit-unpushed",
                targs!("branch" => branch.clone(), "remote" => remote.clone()),
            )),
            Some(head) => {
                if git_line(root, &["rev-parse", "HEAD"]).as_deref() != Some(head.as_str()) {
                    limits.push(ta(
                        "limit-ahead",
                        targs!("branch" => branch.clone(), "remote" => remote.clone()),
                    ));
                }
            }
        }
    }

    limits
}

/// Scenarios this branch works on that never earned their red.
///
/// A scenario counts as worked on when a test on this branch carries
/// its tag; it counts as born when some commit between the base and
/// HEAD is named `red: <scenario>`. Waves closed before this court
/// existed are not judged: their branches are long merged, and
/// demanding a red commit from history that is no longer reachable
/// would redden the verdict on its own past. That is why the court
/// asks the BRANCH, not the whole repository.
fn unborn_scenarios(
    root: &Path,
    wave: &docs::Wave,
    base: &str,
    found: Option<&Vec<tags::TestTag>>,
) -> Vec<(String, String)> {
    // A history cut short cannot answer this: the base may be
    // unreachable, or be HEAD itself, and then no commit is looked at
    // at all. Review 0034 R-3 measured both halves of that -- a
    // silent green on a depth-1 clone and a FALSE RED on a depth-2
    // one, where the red birth lies below the graft. The wave named
    // this limit in the scenario's body and did not build it; the
    // court says it now, as every other limit does.
    if is_shallow(root) {
        return vec![(t("check-red-unjudged"), t("check-red-unjudged-instead"))];
    }
    let Some(names) = git_line(root, &["log", "--format=%s", &format!("{base}..HEAD")]) else {
        return Vec::new();
    };
    // The first word after `red:`, exactly as the hook reads it:
    // review 0034 R-8 measured the two disagreeing on
    // `red: a-promise -- the first cut`, where the hook accepted the
    // birth and this court accused the scenario anyway.
    let born: Vec<&str> = names
        .lines()
        .filter_map(|line| line.trim().strip_prefix("red:"))
        .filter_map(|rest| rest.split_whitespace().next())
        .collect();
    let mut unborn = Vec::new();
    for (name, scenario) in &wave.scenarios {
        if scenario.withdrawn.is_some() {
            continue;
        }
        let worked = found.is_some_and(|tags| tags.iter().any(|tag| &tag.scenario == name));
        if !worked || born.iter().any(|word| word == name) {
            continue;
        }
        unborn.push((
            ta("check-red-unborn", targs!("scenario" => name.clone())),
            ta(
                "check-red-unborn-instead",
                targs!("scenario" => name.clone()),
            ),
        ));
    }
    unborn
}

/// The remote this clone actually has: `origin` when it is there,
/// otherwise the only one, and nothing at all when there are none or
/// several (review 0031 R-8: `origin` was assumed and a project
/// pushed to `upstream` was told its work did not exist).
fn remote_name(root: &Path) -> Option<String> {
    let all = git_line(root, &["remote"])?;
    let mut names = all.lines().map(str::trim).filter(|name| !name.is_empty());
    let first = names.next()?;
    if all.lines().any(|name| name.trim() == "origin") {
        return Some("origin".to_string());
    }
    names.next().is_none().then(|| first.to_string())
}

/// What this repository calls its trunk: what the remote's HEAD
/// points at, else `main` if it exists, else `master` (R-8).
fn trunk_name(root: &Path, remote: Option<&str>) -> String {
    let head = remote.and_then(|remote| {
        git_line(
            root,
            &[
                "symbolic-ref",
                "--short",
                &format!("refs/remotes/{remote}/HEAD"),
            ],
        )
    });
    if let Some(name) = head.as_deref().and_then(|head| head.rsplit_once('/')) {
        return name.1.to_string();
    }
    for name in ["main", "master"] {
        if git_line(root, &["rev-parse", "--verify", "--quiet", name]).is_some() {
            return name.to_string();
        }
    }
    "main".to_string()
}

/// One line of git output, or nothing -- a question this clone
/// cannot answer is not an error, it is a limit.
///
/// Through `scope::git_at`, the one hand: the battery caught this
/// wave calling git raw, and it was right twice over. That hand is
/// also deaf to what a git hook leaves in the environment, and
/// without it a limit would change depending on whose repository
/// fired the tool (review 0021 R-3).
fn git_line(root: &Path, args: &[&str]) -> Option<String> {
    let out = scope::git_at(root).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let line = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!line.is_empty()).then_some(line)
}

/// A refusal rendered as a report row, the school of every floor.
fn push_refusal_row(rows: &mut Vec<(String, Option<String>)>, root: &Path, refusal: &Refusal) {
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

/// Names no wave holds alive any more: declared somewhere, withdrawn
/// everywhere they are declared. A namesake still living in another
/// wave keeps the name out of this set -- forgiving a tag by the bare
/// name disarmed the courts over the living promise (review 0035
/// R-4), which is the very silence §7.15 exists to end.
fn fully_withdrawn(waves: &[docs::Wave]) -> std::collections::BTreeSet<&str> {
    let mut declared: std::collections::BTreeSet<&str> = Default::default();
    let mut alive: std::collections::BTreeSet<&str> = Default::default();
    for wave in waves {
        for (name, scenario) in &wave.scenarios {
            declared.insert(name.as_str());
            if scenario.withdrawn.is_none() {
                alive.insert(name.as_str());
            }
        }
    }
    declared.difference(&alive).copied().collect()
}

/// The tag floor's findings: stale tags and orphan tags, judged
/// against every wave's scenario revisions; matching tags counted.
/// A scenario slug may live in several waves -- any matching
/// revision passes, and the stale message shows the first.
fn tag_rows(
    root: &Path,
    waves: &[docs::Wave],
    found: &[tags::TestTag],
    checked: &mut u64,
) -> Result<Vec<(String, String)>, Refusal> {
    // Live revisions and withdrawn ones are kept apart. Review 0035
    // R-4: one set keyed by the bare NAME meant a scenario withdrawn
    // in one wave silenced its namesake living in another -- every
    // tag of the living one went unjudged, stale or orphan alike,
    // and the verdict said nothing. A name is withdrawn only where
    // no wave still holds it alive.
    let mut revs: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for wave in waves {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        for (name, revision) in rev::scenario_revs(&path)? {
            let entry = wave.scenarios.iter().find(|(n, _)| *n == name);
            if entry.is_some_and(|(_, sc)| sc.withdrawn.is_some()) {
                continue;
            }
            revs.entry(name).or_default().push(revision);
        }
    }
    let gone = fully_withdrawn(waves);

    let mut out = Vec::new();
    for tag in found {
        if gone.contains(tag.scenario.as_str()) {
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

/// The §7.15 delta: scenarios whose tag lived at the fork point and
/// is gone at HEAD while the scenario is alive. Old files that no
/// longer parse cannot testify and are skipped -- archaeology may
/// stay silent, HEAD's own strictness already stands; the skip can
/// hide a vanished tag, never paint a false finding.
fn vanished_rows(
    root: &Path,
    waves: &[docs::Wave],
    found: &[tags::TestTag],
) -> Vec<(String, String)> {
    let Ok((base, _)) = scope::compare_base(root) else {
        return Vec::new();
    };
    let Ok(crate_dir) = adapter::crate_root(root) else {
        return Vec::new();
    };
    let tests_rel = crate_dir
        .strip_prefix(root)
        .map(|p| p.join("tests"))
        .unwrap_or_else(|_| std::path::PathBuf::from("tests"));
    let listing = git_out(
        root,
        &[
            "ls-tree",
            "-r",
            "--name-only",
            &base,
            "--",
            &tests_rel.display().to_string(),
        ],
    )
    .unwrap_or_default();

    let head_scenarios: std::collections::BTreeSet<&str> =
        found.iter().map(|t| t.scenario.as_str()).collect();
    let withdrawn = fully_withdrawn(waves);
    let declared: std::collections::BTreeSet<&str> = waves
        .iter()
        .flat_map(|w| w.scenarios.iter())
        .map(|(n, _)| n.as_str())
        .collect();

    let mut out = Vec::new();
    let mut named: std::collections::BTreeSet<String> = Default::default();
    for rel in listing.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let Some(text) = git_out(root, &["show", &format!("{base}:{rel}")]) else {
            continue;
        };
        let Ok(base_tags) = tags::scan_text(Path::new(rel), &text) else {
            continue;
        };
        for tag in base_tags {
            if head_scenarios.contains(tag.scenario.as_str())
                || withdrawn.contains(tag.scenario.as_str())
                || !named.insert(tag.scenario.clone())
            {
                continue;
            }
            // The words say what truly happened: a scenario erased
            // together with its wave is a destroyed promise, not a
            // living one (review R-6).
            let (reason, instead) = if declared.contains(tag.scenario.as_str()) {
                (
                    ta("tags-vanished", targs!("scenario" => tag.scenario.clone())),
                    t("tags-vanished-instead"),
                )
            } else {
                (
                    ta(
                        "tags-vanished-gone",
                        targs!("scenario" => tag.scenario.clone()),
                    ),
                    t("tags-vanished-gone-instead"),
                )
            };
            out.push((
                rel.to_string(),
                format!("{reason}\n           {}: {instead}", t("word-instead")),
            ));
        }
    }
    out
}

/// One quiet git call for the delta floor: success gives stdout,
/// anything else gives nothing -- the callers say their own words.
fn git_out(root: &Path, args: &[&str]) -> Option<String> {
    let out = crate::scope::git_at(root)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Whether history can testify here at all: git serves the root and
/// the clone is not shallow. The closure court leans on this too.
pub(crate) fn history_testifies(root: &Path) -> bool {
    has_git(root) && !is_shallow(root)
}

/// Whether the recorded revision truly lived in the git history of
/// the contract file (§5.6). Any git trouble reads as "not found":
/// the strict verdict stands where history cannot testify.
pub(crate) fn revision_in_history(root: &Path, relative: &str, recorded: &str) -> bool {
    let log = crate::scope::git_at(root)
        .args(["log", "--format=%H", "--", relative])
        .output();
    let Ok(log) = log else { return false };
    if !log.status.success() {
        return false;
    }
    for sha in String::from_utf8_lossy(&log.stdout).lines() {
        let show = crate::scope::git_at(root)
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
    let out = crate::scope::git_at(root)
        .args(["rev-parse", "--git-dir"])
        .output();
    matches!(out, Ok(o) if o.status.success())
}

/// A shallow clone's history is truncated -- old revisions cannot be
/// verified there, and the absence of history is not the wave's
/// fault.
pub(crate) fn is_shallow(root: &Path) -> bool {
    let out = crate::scope::git_at(root)
        .args(["rev-parse", "--is-shallow-repository"])
        .output();
    matches!(out, Ok(o) if o.status.success()
        && String::from_utf8_lossy(&o.stdout).trim() == "true")
}
