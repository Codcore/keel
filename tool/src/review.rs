//! The reviewer's package (contract tool-review; §9.9): the machine
//! assembles a self-sufficient package for a fresh-context reviewer
//! -- the reviewer is never the author, and "what did we keep
//! silent about" cannot be asked of the one who just kept silent.
//! The package fakes none of the reviewer's per-line human work and
//! passes no verdicts: the courts are check and close.

use crate::docs::{self, TransformKind};
use crate::i18n::{t, ta};
use crate::map;
use crate::refusal::Refusal;
use crate::rev;
use crate::scope;
use crate::targs;
use std::fmt::Write as _;
use std::path::Path;

/// Assembles the §9.9 package for the branch's wave (§8.2); any
/// other branch is a refusal aloud -- which wave the package is for
/// The wave a PLAN branch is named after (§8.2: `plan/<wave>`), or
/// None where the branch is not one.
fn plan_branch_wave(root: &Path, waves: &[docs::Wave]) -> Option<String> {
    let branch = scope::current_branch(root)?;
    let named = branch.strip_prefix("plan/")?;
    waves
        .iter()
        .find(|w| w.slug == named)
        .map(|w| w.slug.clone())
}

/// The package a plan gets: not the whole plan to read, but the
/// places where untruth lives.
///
/// The requirement that shapes this is speed, and it is the operator's
/// own (2026-09-08): a review of the plan that takes fifteen minutes
/// of reading will not be done, and the hole stays exactly where it
/// was. So the tool narrows -- it knows where to look, and the reader
/// answers ten questions instead of forty.
///
/// What it does NOT do: judge whether an answer is true. The machine
/// catches an ABSENT answer already (`graph-silence`); an untrue one
/// is caught by a reader, and this package only says where.
fn plan_package(root: &Path, wave: &docs::Wave) -> String {
    let text = std::fs::read_to_string(root.join("keel/waves").join(format!("{}.md", wave.slug)))
        .unwrap_or_default();
    let mut out = t("review-plan-title");
    out.push('\n');
    writeln!(out, "{}", t("review-plan-why")).unwrap();

    // Cuts closed by a promise, each beside the promise that closes
    // it and the promise's own words: the reader's one question is
    // whether THIS promise proves THIS cut, and they judge the text,
    // never a retelling.
    let mut covered: Vec<(String, String, String)> = Vec::new();
    for (name, scenario) in &wave.scenarios {
        if scenario.withdrawn.is_some() {
            continue;
        }
        for cut in &scenario.covers {
            let body = section(&text, &format!("scenario: {name}")).unwrap_or_default();
            covered.push((cut.clone(), name.clone(), body));
        }
    }
    covered.sort();
    if !covered.is_empty() {
        writeln!(out, "\n{}", t("review-plan-covered-header")).unwrap();
        for (cut, scenario, body) in &covered {
            writeln!(
                out,
                "  {}",
                ta(
                    "review-plan-covered",
                    targs!("cut" => cut.clone(), "scenario" => scenario.clone())
                )
            )
            .unwrap();
            let line = body
                .lines()
                .map(str::trim)
                .find(|line| !line.is_empty())
                .unwrap_or("")
                .to_string();
            writeln!(
                out,
                "{}",
                ta("review-plan-covered-body", targs!("body" => line))
            )
            .unwrap();
        }
    }

    // Cuts decided with a bare formula and no reason after it: "не
    // застосовується" alone is mechanically an answer and empty of
    // one. §10.3 asks for a reason -- "не застосовується, бо…".
    let mut shrugs: Vec<(String, String)> = wave
        .decisions
        .iter()
        .map(|(cut, said)| (cut.clone(), said.clone()))
        .filter(|(_, said)| {
            let said = said.trim();
            !said.contains("бо")
                && !said.contains("because")
                && !said.contains(':')
                && said.chars().count() < 40
        })
        .collect();
    shrugs.sort();
    if !shrugs.is_empty() {
        writeln!(out, "\n{}", t("review-plan-decided-header")).unwrap();
        // FIVE, and the count of the rest. The requirement that shapes
        // this package is speed: a review that hands a person
        // twenty-five lines is the forty-question reading it was meant
        // to replace, and it will not be done. Five is enough to see
        // whether this plan writes reasons at all -- which is the
        // question §10.3 actually asks.
        const SHOWN: usize = 5;
        for (cut, said) in shrugs.iter().take(SHOWN) {
            writeln!(
                out,
                "  {}",
                ta(
                    "review-plan-decided",
                    targs!("cut" => cut.clone(), "said" => said.trim().to_string())
                )
            )
            .unwrap();
        }
        if shrugs.len() > SHOWN {
            writeln!(
                out,
                "  {}",
                ta(
                    "review-plan-decided-more",
                    targs!("count" => (shrugs.len() - SHOWN) as u64)
                )
            )
            .unwrap();
        }
    }

    // More cuts closed than promises made: the rule "exactly one live
    // cover per cut" pushes an author to drag a pair in when there
    // are more promises than qualities they speak about -- measured
    // on a real wave, where three promises of four were one decision
    // in three voices.
    let promises = wave
        .scenarios
        .iter()
        .filter(|(_, s)| s.withdrawn.is_none())
        .count();
    if promises > 0 && covered.len() > promises {
        writeln!(
            out,
            "\n  {}",
            ta(
                "review-plan-crowded",
                targs!("scenarios" => promises as u64, "cuts" => covered.len() as u64)
            )
        )
        .unwrap();
    }

    writeln!(out, "\n{}", t("review-plan-footer")).unwrap();
    out
}

/// is not guessed.
pub fn package(root: &Path) -> Result<String, Refusal> {
    let scan = docs::scan(root)?;
    if let Some(refusal) = scan.refusals.into_iter().next() {
        // A package over unread documents would guess; check names
        // every broken file -- fix them first.
        return Err(refusal);
    }
    // The PLAN branch gets a package of its own (wave 0064). Until
    // now this was a refusal -- "the branch is not named as a wave" --
    // so the only fresh eye a wave ever got arrived at closing, when
    // all the work was already done under the plan it should have
    // judged. The forty answers are written HERE; the approval of
    // §6.6 stands between, held by nothing but a person's reading.
    //
    // The suit is one and the same -- a fresh eye over promises -- so
    // it stays one command, and the FORM of the package follows the
    // branch, exactly as `keel next` already gives different steps on
    // different branches (the operator's decision of 2026-09-08).
    if let Some(slug) = plan_branch_wave(root, &scan.waves) {
        let wave = scan.waves.iter().find(|w| w.slug == slug).unwrap();
        return Ok(plan_package(root, wave));
    }
    let Some(slug) = scope::branch_wave(root, &scan.waves) else {
        let branch = scope::current_branch(root).unwrap_or_else(|| "?".to_string());
        return Err(Refusal {
            file: root.to_path_buf(),
            reason: ta("review-not-wave", targs!("branch" => branch)),
            instead: t("review-not-wave-instead"),
        });
    };
    let wave = scan.waves.iter().find(|w| w.slug == slug).unwrap();
    // A wave called off is outside judgement, and §6.3-a says every
    // court says so aloud -- this one assembled a package in silence
    // (global review 2026-09-06, methodology R-12; wave 0053).
    if let Some(why) = &wave.cancelled {
        let mut out = t("review-title");
        out.push('\n');
        writeln!(
            out,
            "{}",
            ta(
                "review-cancelled",
                targs!("wave" => slug.clone(), "why" => why.clone())
            )
        )
        .unwrap();
        return Ok(out);
    }
    let rel = format!("keel/waves/{slug}.md");
    let wave_path = root.join(&rel);
    let text = std::fs::read_to_string(&wave_path).map_err(|e| Refusal {
        file: wave_path.clone(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    // CRLF normalized for section parsing (review 0009 R-3): the
    // package must not lose the Why and the caveats to Windows line
    // endings -- verbatim means the words, not the carriage returns.
    let text = text.replace("\r\n", "\n");
    let revs = rev::scenario_revs(&wave_path)?;

    let mut out = t("review-title");
    out.push('\n');
    writeln!(out, "{}", ta("review-wave", targs!("wave" => slug.clone()))).unwrap();

    // The Why, verbatim.
    writeln!(out, "\n{}", t("review-why-header")).unwrap();
    match section(&text, "Why") {
        Some(why) => writeln!(out, "{why}").unwrap(),
        None => writeln!(out, "{}", t("review-why-missing")).unwrap(),
    }

    // Scenarios with their revisions (§5.3), bodies included: the
    // package is self-sufficient.
    writeln!(out, "\n{}", t("review-scenarios-header")).unwrap();
    for (name, scenario) in &wave.scenarios {
        let revision = revs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, r)| r.as_str())
            .unwrap_or("?");
        let fate = if scenario.withdrawn.is_some() {
            t("review-scenario-withdrawn")
        } else {
            String::new()
        };
        writeln!(out, "  {name}@{revision}{fate}").unwrap();
        if let Some(body) = section(&text, &format!("scenario: {name}")) {
            writeln!(out, "{body}").unwrap();
        }
    }
    if wave.scenarios.is_empty() {
        writeln!(out, "  {}", t("review-scenarios-none")).unwrap();
    }

    // Transforms verbatim -- the caveat paragraphs (§2.10) live in
    // these bodies and ride whole, so none is dropped by parsing
    // prose the mechanics cannot read.
    writeln!(out, "\n{}", t("review-transforms-header")).unwrap();
    for (name, _) in &wave.transforms {
        match section(&text, &format!("transform: {name}")) {
            Some(body) => writeln!(out, "## transform: {name}\n{body}").unwrap(),
            // Named with a word, never dropped in silence (review
            // 0009 R-7): §7.7's header-vs-body floor is a rung ahead.
            None => writeln!(
                out,
                "## transform: {name}\n{}",
                t("review-transform-no-body")
            )
            .unwrap(),
        }
    }

    // Chore reasons (§2.11).
    writeln!(out, "\n{}", t("review-chores-header")).unwrap();
    let mut chores = 0;
    for (name, transform) in &wave.transforms {
        if let TransformKind::Chore(reason) = &transform.kind {
            writeln!(out, "  {name} — \"{reason}\"").unwrap();
            chores += 1;
        }
    }
    if chores == 0 {
        writeln!(out, "  {}", t("review-chores-none")).unwrap();
    }

    // List 1: drift (§4.6) -- files added to scope after the
    // anchor. This generation's anchor is the first commit of the
    // wave file, named aloud; truncated history cannot prove the
    // anchor is the true first commit, so it says a word instead.
    writeln!(out).unwrap();
    if crate::check::is_shallow(root) {
        writeln!(out, "{}", t("review-drift-unverified")).unwrap();
    } else {
        let full = docs::weight(wave) == docs::Weight::Full;
        match drift_anchor(root, &rel, wave.renamed_from.as_deref(), full) {
            None => writeln!(out, "{}", t("review-drift-unverified")).unwrap(),
            Some((anchor, anchored_rel, anchored_slug, kind)) => {
                let short = anchor.get(..7).unwrap_or(&anchor).to_string();
                let key = match kind {
                    Anchor::Fork => "review-drift-header-fork",
                    Anchor::First => "review-drift-header",
                };
                writeln!(out, "{}", ta(key, targs!("sha" => short))).unwrap();
                match old_wave_files(root, &anchor, &anchored_rel, &anchored_slug) {
                    None => writeln!(out, "  {}", t("review-drift-unreadable")).unwrap(),
                    Some(old_files) => {
                        let now = wave_files(wave);
                        let mut drifted = 0;
                        for line in &now {
                            if !old_files.contains(line) {
                                writeln!(
                                    out,
                                    "  {}",
                                    ta("review-drift-line", targs!("file" => line.clone()))
                                )
                                .unwrap();
                                drifted += 1;
                            }
                        }
                        // Quiet narrowing is drift too (review 0009
                        // R-5): a file removed from scope after the
                        // anchor gets its own word.
                        for line in &old_files {
                            if !now.contains(line) {
                                writeln!(
                                    out,
                                    "  {}",
                                    ta("review-drift-removed-line", targs!("file" => line.clone()))
                                )
                                .unwrap();
                                drifted += 1;
                            }
                        }
                        if drifted == 0 {
                            writeln!(out, "  {}", t("review-drift-empty")).unwrap();
                        }
                    }
                }
            }
        }
    }

    // List 2: the quality map (§10.7) -- the wave view, drawn by
    // tool-map.
    writeln!(out, "\n{}", t("review-map-header")).unwrap();
    out.push_str(&map::draw(root)?);

    // List 3: contract-change impact (§5.7) -- every contract whose
    // text differs from the fork point, with the old and new
    // revisions and every holder of a reference to it, by name.
    writeln!(out, "\n{}", t("review-impact-header")).unwrap();
    let base = scope::compare_base(root);
    match &base {
        Err(_) => writeln!(out, "  {}", t("review-impact-unverified")).unwrap(),
        Ok((base_sha, _)) => {
            let mut changed = 0;
            for contract in &scan.contracts {
                let contract_rel = format!("keel/contracts/{}.md", contract.slug);
                let Some(old_text) = git_show(root, base_sha, &contract_rel) else {
                    continue; // born on this branch: no old revision to impact
                };
                let old_rev = rev::text_rev(&old_text);
                let new_text =
                    std::fs::read_to_string(root.join(&contract_rel)).unwrap_or_default();
                let new_rev = rev::text_rev(&new_text);
                if old_rev == new_rev {
                    continue;
                }
                changed += 1;
                writeln!(
                    out,
                    "  {}",
                    ta(
                        "review-impact-contract",
                        targs!("slug" => contract.slug.clone(), "old" => old_rev.clone(), "new" => new_rev.clone())
                    )
                )
                .unwrap();
                for (place, held) in holders(&scan.waves, &contract.slug) {
                    let word = if rev::matches(&held, &new_rev) {
                        t("review-impact-current")
                    } else {
                        t("review-impact-stale")
                    };
                    writeln!(out, "    {place} @{held} — {word}").unwrap();
                }
            }
            if changed == 0 {
                writeln!(out, "  {}", t("review-impact-none")).unwrap();
            }
        }
    }

    // The full branch diff against the fork point.
    match &base {
        Err(_) => writeln!(out, "\n{}", t("review-diff-unverified")).unwrap(),
        Ok((base_sha, _)) => {
            let short = base_sha.get(..7).unwrap_or(base_sha).to_string();
            writeln!(
                out,
                "\n{}",
                ta("review-diff-header", targs!("base" => short))
            )
            .unwrap();
            // base..HEAD, never the working tree (review 0009 R-4):
            // an uncommitted edit is not the branch's.
            match git_out(root, &["diff", base_sha, "HEAD"]) {
                Some(diff) if !diff.trim().is_empty() => out.push_str(&diff),
                Some(_) => writeln!(out, "  {}", t("review-diff-empty")).unwrap(),
                None => writeln!(out, "  {}", t("review-diff-unverified")).unwrap(),
            }
        }
    }

    // The protocol rides with the data, so the package is
    // self-sufficient for a fresh context, not only in facts.
    writeln!(out, "\n{}", t("review-protocol-header")).unwrap();
    writeln!(out, "{}", t("review-protocol-rows")).unwrap();
    writeln!(out, "{}", t("review-protocol-questions")).unwrap();
    writeln!(
        out,
        "{}",
        ta("review-protocol-report", targs!("wave" => slug.clone()))
    )
    .unwrap();

    // The briefing goes LAST, after the material: a reviewer reads
    // it with the wave already in mind, and remembers the end. Until
    // this wave it was not here at all -- it lived in a chat, written
    // by hand for each reviewer, so each got a different one (wave
    // 0032).
    writeln!(out, "\n{}", t("briefing-header")).unwrap();
    for part in [
        "briefing-forbidden",
        "briefing-hygiene",
        "briefing-work",
        "briefing-questions",
    ] {
        writeln!(out, "\n{}", t(part)).unwrap();
    }
    writeln!(
        out,
        "\n{}",
        ta("briefing-report", targs!("wave" => slug.clone()))
    )
    .unwrap();

    Ok(out)
}

/// The body of one `## <title>` section of the wave file.
fn section(text: &str, title: &str) -> Option<String> {
    for part in text.split("\n## ") {
        if let Some(rest) = part.strip_prefix(title)
            && rest.starts_with('\n')
        {
            return Some(rest.trim_matches('\n').to_string());
        }
    }
    None
}

/// Every scope path a wave declares, across its transforms.
fn wave_files(wave: &docs::Wave) -> Vec<String> {
    let mut files = Vec::new();
    for (_, transform) in &wave.transforms {
        for line in &transform.files {
            let text = match line {
                docs::ScopeLine::Path(p) => p.clone(),
                docs::ScopeLine::OneNewIn(d) => format!("one new in {d}"),
            };
            if !files.contains(&text) {
                files.push(text);
            }
        }
    }
    files
}

/// Which commit the anchor is.
enum Anchor {
    /// The fork point with main: the wave file as the plan PR merged
    /// it (§4.6, the norm as written -- wave 0052).
    Fork,
    /// The first commit that added the wave file: a light wave has no
    /// plan PR, and a full wave whose plan is not merged yet has no
    /// fork point carrying its file.
    First,
}

/// The drift anchor (§4.6): for a FULL wave the wave file at the
/// fork point with main -- the plan PR merged, whatever number of
/// commits the plan branch took (the first reading took the first
/// commit of the file, and a plan branch of two commits had its
/// second file "added after the anchor" once merged; global review
/// 2026-09-06, methodology R-7). For a light wave, and for a full
/// one whose plan is not in main yet, the first commit of the file.
/// A renamed wave (renamed_from) keeps the true anchor of its old
/// name, so growth at the rename is not blessed as planned (review
/// 0009 R-5); the anchor is returned with the path and slug it was
/// found under. Border: once the branch is merged, the fork point
/// is the head -- the package is assembled on the branch, before the
/// PR (§9.9).
fn drift_anchor(
    root: &Path,
    rel: &str,
    renamed_from: Option<&str>,
    full: bool,
) -> Option<(String, String, String, Anchor)> {
    if let Some(old) = renamed_from {
        let old_rel = format!("keel/waves/{old}.md");
        if let Some(sha) = first_add(root, &old_rel) {
            return Some((sha, old_rel, old.to_string(), Anchor::First));
        }
    }
    let slug = rel
        .strip_prefix("keel/waves/")
        .and_then(|s| s.strip_suffix(".md"))
        .unwrap_or("wave")
        .to_string();
    if full
        && let Ok((base, from_main)) = scope::compare_base(root)
        && from_main
        && git_show(root, &base, rel).is_some()
    {
        return Some((base, rel.to_string(), slug, Anchor::Fork));
    }
    first_add(root, rel).map(|sha| (sha, rel.to_string(), slug, Anchor::First))
}

fn first_add(root: &Path, rel: &str) -> Option<String> {
    let log = git_out(root, &["log", "--diff-filter=A", "--format=%H", "--", rel])?;
    log.lines()
        .last()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
}

/// The wave's declared files as of the anchor commit, parsed from
/// the text git hands over -- the same strict court, no disk
/// touched (review 0009 R-9).
fn old_wave_files(root: &Path, anchor: &str, rel: &str, slug: &str) -> Option<Vec<String>> {
    let old_text = git_show(root, anchor, rel)?;
    let shown = root.join(rel);
    docs::read_wave_text(slug, &old_text, &shown)
        .ok()
        .map(|w| wave_files(&w))
}

/// Everyone holding a reference to the contract, by name: scenario
/// proves and transform contracts across every wave (§5.7).
fn holders(waves: &[docs::Wave], slug: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for wave in waves {
        for (name, scenario) in &wave.scenarios {
            if let Some(reference) = &scenario.proves
                && reference.slug == slug
            {
                out.push((
                    format!("{}/{} proves {}", wave.slug, name, slug),
                    reference.rev.clone(),
                ));
            }
        }
        for (name, transform) in &wave.transforms {
            for reference in &transform.contracts {
                if reference.slug == slug {
                    out.push((
                        format!("{} transform {} contracts {}", wave.slug, name, slug),
                        reference.rev.clone(),
                    ));
                }
            }
        }
    }
    out
}

fn git_show(root: &Path, commit: &str, rel: &str) -> Option<String> {
    git_out(root, &["show", &format!("{commit}:{rel}")])
}

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
