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
    // Limits gathered while judging, said in the verdict's own
    // margin rather than swallowed (§4.10, wave 0031).
    let mut extra_limits: Vec<String> = Vec::new();
    let mut cancelled_rows: Vec<String> = Vec::new();
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
        // A wave called off is outside judgement whole (§6.3-a):
        // review 0037 R-1 measured this court and the §7.7 one below
        // still judging it, so the report said "not judged" and gave
        // a finding two lines apart -- the very self-contradiction
        // this wave came to end elsewhere.
        if wave.cancelled.is_some() {
            continue;
        }
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
        // Called off: outside judgement, and said so once, below.
        if wave.cancelled.is_some() {
            cancelled_rows.push(ta(
                "check-wave-cancelled",
                targs!("wave" => wave.slug.clone(), "why" => wave.cancelled.clone().unwrap_or_default()),
            ));
            continue;
        }
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
    // §4.12: a document does not vanish. Judged on every branch
    // named after anything but research -- on `spike/*` the sentence
    // of §4.13 is "the documents are not judged", and review 0036 R-5
    // measured this court making that sentence a lie in its own
    // report.
    let researching = scope::spike_branch(root).is_some();
    if !researching {
        // The limit said aloud instead of painted green (§4.10): a
        // truncated history, no history at all, or a trunk this clone
        // cannot name gives no base to compare against -- and review
        // 0036 R-6 and R-8 measured both halves of the silence, one
        // of them inventing findings about a file deleted years ago.
        match compare_state(root, shallow, has_history) {
            Compared::Yes => {
                for (file, reason, instead) in vanished_documents(root, &scan) {
                    rows.push((
                        file,
                        Some(format!(
                            "{reason}\n           {}: {instead}",
                            t("word-instead")
                        )),
                    ));
                }
            }
            Compared::No(why) => extra_limits.push(why),
            Compared::Silent => {}
        }
    }

    // A language this release does not know is a FINDING with the
    // list of the ones it does (wave 0038). Not a refusal: a project
    // that named a language keel cannot lead yet still gets its
    // documents, links, scope and revisions judged. Not silence
    // either: before this wave an unknown name simply meant "not
    // Rust", so a typo skipped the language-shaped courts without
    // ever saying which -- and §4.10 calls that worse than red.
    if let Some(named) = config.adapter.as_deref()
        && config.language().is_none()
    {
        rows.push((
            "keel.toml".to_string(),
            Some(format!(
                "{}\n           {}: {}",
                ta(
                    "config-unknown-adapter",
                    targs!("named" => named.to_string(), "known" => crate::config::Language::known()),
                ),
                t("word-instead"),
                ta(
                    "config-unknown-adapter-instead",
                    targs!("known" => crate::config::Language::known()),
                ),
            )),
        ));
    }

    let generated: Vec<String> = config.generated.iter().map(|(k, _)| k.clone()).collect();
    let scope_status = match scope::current_branch(root) {
        None => t("check-scope-skipped-no-git"),
        // A plan branch is judged too, and by §4.9: it carries the
        // plan and nothing else. The conformance audit (ВАЖКА-4)
        // measured the paragraph held by nothing -- `plan/<wave>` is
        // not the name of a wave, so the whole floor was skipped.
        // Research is outside the methodology and says so (§4.13).
        // The norm promised the ban held BY MACHINE and the word
        // `spike` was nowhere in the code: the branch was judged
        // like any other stranger's, which is to say not at all.
        Some(branch) if branch.starts_with("spike/") => {
            ta("check-scope-spike", targs!("branch" => branch))
        }
        Some(branch) if branch.starts_with("plan/") => {
            let planned = scope::plan_branch(root).unwrap_or_default();
            let known = scan.waves.iter().any(|w| w.slug == planned);
            // Compared, or said aloud that it was not: review 0036
            // R-7 measured a shallow clone whose base IS the head
            // printing "judged by §4.9" over a comparison that never
            // happened -- the §4.10 lie word for word.
            match compare_state(root, shallow, has_history) {
                Compared::No(why) => {
                    extra_limits.push(why);
                    ta(
                        "check-scope-plan-unjudged",
                        targs!("branch" => branch, "wave" => planned),
                    )
                }
                Compared::Silent => ta(
                    "check-scope-plan-unjudged",
                    targs!("branch" => branch, "wave" => planned),
                ),
                Compared::Yes => match scope::plan_findings(root, &generated) {
                    Ok(list) => {
                        for (file, reason, instead) in list {
                            rows.push((
                                file,
                                Some(format!(
                                    "{reason}\n           {}: {instead}",
                                    t("word-instead")
                                )),
                            ));
                        }
                        let key = if known {
                            "check-scope-plan"
                        } else {
                            "check-scope-plan-nowave"
                        };
                        ta(key, targs!("branch" => branch, "wave" => planned))
                    }
                    // A branch git cannot answer about is a line in
                    // the report, not the end of it -- the wave
                    // branch beside it has said so since 0012, and
                    // review 0036 R-13 measured this one throwing the
                    // whole verdict away instead.
                    Err(refusal) => {
                        rows.push((
                            refusal.file.display().to_string(),
                            Some(format!(
                                "{}\n           {}: {}",
                                refusal.reason,
                                t("word-instead"),
                                refusal.instead
                            )),
                        ));
                        t("check-scope-skipped-refused")
                    }
                },
            }
        }
        Some(branch) => match scope::branch_wave(root, &scan.waves) {
            None => ta("check-scope-skipped-not-wave", targs!("branch" => branch)),
            Some(slug)
                if scan
                    .waves
                    .iter()
                    .any(|w| w.slug == slug && w.cancelled.is_some()) =>
            {
                ta(
                    "check-scope-cancelled",
                    targs!("branch" => branch, "wave" => slug),
                )
            }
            Some(slug) => {
                let wave_path = format!("keel/waves/{slug}.md");
                let wave = scan.waves.iter().find(|w| w.slug == slug).unwrap();
                let compared = scope::compare_base(root)
                    .and_then(|base| scope::findings(root, wave).map(|list| (base, list)));
                match compared {
                    Ok(((sha, from_main), list)) => {
                        // §6.8/§8.1: a FULL wave rides two branches
                        // and two PRs. When its own file was born in
                        // this very diff, the plan PR never happened
                        // -- and the second human look the paragraph
                        // asks for went with it.
                        if docs::weight(wave) == docs::Weight::Full
                            && git_out(
                                root,
                                &[
                                    "diff",
                                    "--name-only",
                                    "--no-renames",
                                    "--diff-filter=A",
                                    &sha,
                                    "HEAD",
                                    "--",
                                    &wave_path,
                                ],
                            )
                            .is_some_and(|out| !out.trim().is_empty())
                            // Unless the plan branch exists and
                            // already carries the file: §8.1's two
                            // PRs are then under way, the plan one
                            // simply not merged yet. Review 0036 R-10
                            // measured the lawful sequence accused,
                            // with an instead telling the author to
                            // do what they had already done.
                            && !planned_elsewhere(root, &slug, &wave_path)
                        {
                            rows.push((
                                wave_path.clone(),
                                Some(format!(
                                    "{}\n           {}: {}",
                                    ta(
                                        "scope-full-one-branch",
                                        targs!("wave" => slug.clone(), "weight" => t("word-weight-full")),
                                    ),
                                    t("word-instead"),
                                    ta("scope-full-one-branch-instead", targs!("wave" => slug.clone())),
                                )),
                            ));
                        }
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
                        // §7.5, judged BY THE BRANCH: a wave that
                        // has work commits and no tag at all was red
                        // nowhere (conformance audit ВАЖКА-7) -- only
                        // `keel close` saw it, and only afterwards.
                        // The named exception of §6.3 said aloud:
                        // a green birth is lawful when its commit
                        // records the mutant, and the machine does
                        // NOT check the mutant is real -- so the
                        // verdict names it and hands it to the
                        // reviewer instead of swallowing it.
                        extra_limits.extend(mutant_births(root, &sha));
                        for (scenario, instead) in untested_scenarios(
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
    let mut limits = verdict_limits(root, refs_unjudged);
    limits.extend(extra_limits);
    limits.extend(cancelled_rows);
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

/// Whether the wave's own plan branch exists and already carries its
/// file (§8.1): then the plan PR is under way and the file did not
/// come into being beside the work.
fn planned_elsewhere(root: &Path, slug: &str, wave_path: &str) -> bool {
    for branch in [format!("plan/{slug}"), format!("origin/plan/{slug}")] {
        if git_out(root, &["cat-file", "-e", &format!("{branch}:{wave_path}")]).is_some() {
            return true;
        }
    }
    false
}

/// Whether a comparison against the fork point is possible at all,
/// and if not, in what words. Review 0036 R-6, R-7 and R-8 measured
/// three silences behind one green: a truncated clone whose base is
/// HEAD itself, a project with no git, and a trunk not called `main`
/// -- where the base falls back to the root commit and every ancient
/// deletion is dragged up as a finding nobody can act on.
enum Compared {
    Yes,
    /// Nothing to compare and nothing worth saying: there is no
    /// repository here at all.
    Silent,
    No(String),
}

fn compare_state(root: &Path, shallow: bool, has_history: bool) -> Compared {
    // A directory with no git at all is not a clone with problems --
    // it is not a clone, and asking it about fork points would be
    // the noise review 0031 R-8 already took out once.
    if !has_history {
        return Compared::Silent;
    }
    if shallow {
        return Compared::No(t("limit-shallow-diff"));
    }
    let Ok((base, from_main)) = scope::compare_base(root) else {
        return Compared::No(t("limit-no-base"));
    };
    if !from_main {
        return Compared::No(t("limit-no-trunk"));
    }
    // A base that IS the head is the trunk itself, once a truncated
    // history has been ruled out above: there are no commits of our
    // own to compare, and saying so on every main-branch run would
    // be noise, not honesty (the verdict-limits probe of wave 0031
    // measured exactly that).
    let _ = base;
    Compared::Yes
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

/// The green births of §6.3's named exception on this branch: every
/// `red:` commit whose message carries a mutant line. Said aloud as a
/// limit of the verdict, because that is exactly what it is -- the
/// machine took the author's word.
fn mutant_births(root: &Path, base: &str) -> Vec<String> {
    let Some(log) = git_line(
        root,
        &["log", "--format=%s%x1f%b%x1e", &format!("{base}..HEAD")],
    ) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for entry in log.split('\u{1e}') {
        let Some((subject, body)) = entry.split_once('\u{1f}') else {
            continue;
        };
        let subject = subject.trim();
        // The same prefix the gate reads, space and all (review
        // 0037 R-8): `red:` without it matched `red:x` too.
        let Some(rest) = subject.strip_prefix("red: ") else {
            continue;
        };
        let Some(scenario) = rest.split_whitespace().next() else {
            continue;
        };
        if let Some((broke, named)) = crate::gate::mutant_line(body) {
            out.push(ta(
                "check-red-mutant",
                targs!(
                    "scenario" => scenario.to_string(),
                    "broke" => broke,
                    "named" => named
                ),
            ));
        }
    }
    out
}

/// §7.5 judged by the BRANCH: a wave whose branch carries work
/// commits must have a tag for every live promise. The conformance
/// audit (ВАЖКА-7) measured a branch with work and NOT ONE test
/// getting zero findings -- §7.5 was held only by `keel close`, and
/// only after the fact.
///
/// A wave approved and not started stays silent: the paragraph says
/// so in as many words, and no work commit means nothing was
/// promised proof yet. Where the tags were not read at all (no rust
/// adapter), nothing is judged rather than everything accused.
fn untested_scenarios(
    root: &Path,
    wave: &docs::Wave,
    base: &str,
    found: Option<&Vec<tags::TestTag>>,
) -> Vec<(String, String)> {
    let Some(found) = found else {
        return Vec::new();
    };
    if is_shallow(root) {
        return Vec::new();
    }
    let Some(subjects) = git_line(root, &["log", "--format=%s", &format!("{base}..HEAD")]) else {
        return Vec::new();
    };
    // A work commit is the §8.4 grammar: the transform's own slug,
    // then a colon. `red:` births and anything else are not work.
    let worked = subjects.lines().any(|line| {
        let head = line.trim().split(':').next().unwrap_or("").trim();
        wave.transforms.iter().any(|(slug, _)| slug == head)
    });
    if !worked {
        return Vec::new();
    }
    let mut out = Vec::new();
    for (name, scenario) in &wave.scenarios {
        if scenario.withdrawn.is_some() || found.iter().any(|tag| &tag.scenario == name) {
            continue;
        }
        out.push((
            ta("check-untested", targs!("scenario" => name.clone())),
            ta("check-untested-instead", targs!("scenario" => name.clone())),
        ));
    }
    out
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
    // A wave called off is outside judgement whole (§6.3-a) -- its
    // half-written body is not read, and its refusal is not this
    // court's row (review 0037 R-1).
    for wave in waves.iter().filter(|w| w.cancelled.is_none()) {
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

/// §4.12: a document does not vanish. A wave or contract file gone
/// against the base is a finding by slug -- unless a living document
/// claims the inheritance with `renamed_from`. Two claimants, or a
/// claimant in the other directory, are findings of their own: the
/// old name cannot lead to both, and a wave is not a contract.
///
/// The conformance audit (ВАЖКА-3) measured the paragraph held by
/// nothing: the file could simply be deleted, every promise in it
/// with it, while §2.12 says a promise dies by `withdrawn` and by
/// nothing else. `renamed_from` was parsed and read by no one.
fn vanished_documents(root: &Path, scan: &docs::Scan) -> Vec<(String, String, String)> {
    let Ok((base, _)) = scope::compare_base(root) else {
        return Vec::new();
    };
    let Some(listing) = git_out(
        root,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            "--diff-filter=D",
            &base,
            "HEAD",
            "--",
            "keel/waves",
            "keel/contracts",
        ],
    ) else {
        return Vec::new();
    };
    // Who claims what, among the documents alive at HEAD.
    let mut heirs: std::collections::BTreeMap<&str, Vec<(&str, &str)>> = Default::default();
    for wave in &scan.waves {
        if let Some(from) = &wave.renamed_from {
            heirs
                .entry(from.as_str())
                .or_default()
                .push((wave.slug.as_str(), "keel/waves"));
        }
    }
    for contract in &scan.contracts {
        if let Some(from) = &contract.renamed_from {
            heirs
                .entry(from.as_str())
                .or_default()
                .push((contract.slug.as_str(), "keel/contracts"));
        }
    }

    let mut out = Vec::new();
    let mut said: std::collections::BTreeSet<&str> = Default::default();
    for path in listing.lines().map(str::trim).filter(|l| !l.is_empty()) {
        let Some((home, file)) = path.rsplit_once('/') else {
            continue;
        };
        let Some(slug) = file.strip_suffix(".md") else {
            continue;
        };
        let claimed = heirs.get(slug).map(Vec::as_slice).unwrap_or(&[]);
        match claimed {
            [] => out.push((
                path.to_string(),
                ta("scope-vanished", targs!("slug" => slug.to_string())),
                ta("scope-vanished-instead", targs!("slug" => slug.to_string())),
            )),
            [(heir, where_)] => {
                if *where_ != home {
                    out.push((
                        format!("{where_}/{heir}.md"),
                        ta(
                            "scope-moved-across",
                            targs!("slug" => slug.to_string(), "heir" => heir.to_string()),
                        ),
                        t("scope-moved-across-instead"),
                    ));
                }
            }
            many => {
                if said.insert(slug) {
                    let names: Vec<String> =
                        many.iter().map(|(heir, _)| (*heir).to_string()).collect();
                    out.push((
                        path.to_string(),
                        ta(
                            "scope-two-heirs",
                            targs!("slug" => slug.to_string(), "heirs" => names.join(", ")),
                        ),
                        t("scope-two-heirs-instead"),
                    ));
                }
            }
        }
    }
    out
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
