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
fn plan_package(root: &Path, wave: &docs::Wave, silent: &[String]) -> Result<String, Refusal> {
    // ONE road to the text, the same the work package walks: read it
    // through `wave_text`, which normalises CRLF. Review 0064 R-3
    // measured what a second road costs -- on a file with `\r\n` this
    // package printed "тіло обіцянки: " and nothing, while the work
    // package printed the line; a defect fixed once (review 0009 R-3)
    // came back on the copy.
    let text = wave_text(root, &wave.slug)?;
    let mut out = t("review-plan-title");
    out.push('\n');
    writeln!(out, "{}", t("review-plan-why")).unwrap();

    // Cuts with no answer at all: the machine already reddens over
    // them (`graph-silence`), and a package that does not repeat it
    // sends a reader to judge a plan the tool has already refused
    // (review 0064 R-5).
    if !silent.is_empty() {
        writeln!(
            out,
            "\n  {}",
            ta("review-plan-silent", targs!("cuts" => silent.join(", ")))
        )
        .unwrap();
    }

    // Cuts closed by a promise, GROUPED BY PROMISE: the body is
    // printed once, and the cuts it claims to prove stand beside it.
    // Review 0064 R-1 measured the ungrouped shape at 75 lines on a
    // plan of six promises -- three times the twenty-five this wave
    // itself called unacceptable, and the body repeated once per cut.
    let mut by_promise: Vec<(String, Vec<String>)> = Vec::new();
    for (name, scenario) in &wave.scenarios {
        if scenario.withdrawn.is_some() {
            continue;
        }
        if scenario.covers.is_empty() {
            continue;
        }
        by_promise.push((name.clone(), scenario.covers.clone()));
    }
    by_promise.sort();
    if !by_promise.is_empty() {
        writeln!(out, "\n{}", t("review-plan-covered-header")).unwrap();
        for (name, cuts) in by_promise.iter().take(SHOWN) {
            writeln!(
                out,
                "  {}",
                ta(
                    "review-plan-covered",
                    targs!("scenario" => name.clone(), "cuts" => cuts.join(", "))
                )
            )
            .unwrap();
            writeln!(
                out,
                "{}",
                ta(
                    "review-plan-covered-body",
                    targs!("body" => promise_line(&text, name))
                )
            )
            .unwrap();
        }
        if by_promise.len() > SHOWN {
            writeln!(
                out,
                "  {}",
                ta(
                    "review-plan-more",
                    targs!("count" => (by_promise.len() - SHOWN) as u64)
                )
            )
            .unwrap();
        }
    }

    // Cuts decided by the FORMULA ALONE. Measured across all 64 waves
    // of this tree before this cut of the code: 1222 answers explain
    // after a colon, 194 carry "бо", 831 are the bare formula -- and
    // nothing lies between. So the question needs no length threshold
    // and no word list, both of which review 0064 R-7 measured
    // catching honest short reasons and missing long empty ones.
    let mut shrugs: Vec<(String, String)> = wave
        .decisions
        .iter()
        .filter(|(_, said)| is_bare_formula(said))
        .map(|(cut, said)| (cut.clone(), said.trim().to_string()))
        .collect();
    shrugs.sort();
    if !shrugs.is_empty() {
        writeln!(out, "\n{}", t("review-plan-decided-header")).unwrap();
        for (cut, said) in shrugs.iter().take(SHOWN) {
            writeln!(
                out,
                "  {}",
                ta(
                    "review-plan-decided",
                    targs!("cut" => cut.clone(), "said" => said.clone())
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

    // More cuts claimed than promises made. Review 0064 R-8 measured
    // the first cut of this rule upside down: it fired on one promise
    // covering two cuts (lawful, and the ordinary shape) and stayed
    // silent on four promises covering one cut each -- which is the
    // crowding it was written for. The question is whether one
    // DECISION speaks with several voices, so it is asked of promises
    // per cut, not cuts per promise.
    let mut voices: std::collections::BTreeMap<&str, usize> = Default::default();
    for (_, scenario) in wave.scenarios.iter().filter(|(_, s)| s.withdrawn.is_none()) {
        for cut in &scenario.covers {
            *voices.entry(cut.as_str()).or_default() += 1;
        }
    }
    let promises = wave
        .scenarios
        .iter()
        .filter(|(_, s)| s.withdrawn.is_none() && !s.covers.is_empty())
        .count();
    let cuts_claimed = voices.len();
    if promises > cuts_claimed && cuts_claimed > 0 {
        writeln!(
            out,
            "\n  {}",
            ta(
                "review-plan-crowded",
                targs!("scenarios" => promises as u64, "cuts" => cuts_claimed as u64)
            )
        )
        .unwrap();
    }

    // A package with no question in it is the case this tool is most
    // likely to meet on a release wave -- no promise, every answer
    // explained -- and review 0065 F-4 measured it: four lines, and
    // not one of them says that nothing was found. Silence that looks
    // like a broken command is worse than a short answer.
    if silent.is_empty() && by_promise.is_empty() && shrugs.is_empty() {
        writeln!(out, "\n  {}", t("review-plan-nothing")).unwrap();
    }
    writeln!(out, "\n{}", t("review-plan-footer")).unwrap();
    Ok(out)
}

/// How many rows of either list a package shows. The requirement that
/// shapes this package is SPEED -- a review nobody runs is the hole it
/// was meant to close -- and it binds every list in it, not one
/// (review 0064 R-1).
const SHOWN: usize = 5;

/// The promise in ONE line a person can judge: its first sentence,
/// whole, not the first line cut wherever the file happened to wrap
/// (review 0064 R-2 measured "…з повним планом, який `keel check`").
/// Where the section says nothing, the package says THAT, rather than
/// printing an empty label.
fn promise_line(text: &str, name: &str) -> String {
    let body = section(text, &format!("scenario: {name}")).unwrap_or_default();
    let flat = body.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.is_empty() {
        return t("review-plan-body-empty");
    }
    match flat.find(". ") {
        Some(at) if at < 240 => flat[..=at].to_string(),
        _ => {
            let mut cut = flat.chars().take(240).collect::<String>();
            if flat.chars().count() > 240 {
                cut.push('…');
            }
            cut
        }
    }
}

/// An answer that is the FORMULA and nothing else. Measured over all
/// 64 waves of this tree: no answer in it stands between "formula
/// alone" and "formula plus an explanation", so the question is asked
/// exactly, and neither a length nor a word list is needed.
fn is_bare_formula(said: &str) -> bool {
    let said = said.trim().trim_end_matches(['.', '—', '-', ':']).trim();
    matches!(
        said,
        "не застосовується" | "not applicable" | "does not apply" | "н/д" | "n/a"
    )
}

/// The wave's file, read the one way BOTH packages read it.
///
/// CRLF normalized for section parsing (review 0009 R-3): the package
/// must not lose the Why and the caveats to Windows line endings --
/// verbatim means the words, not the carriage returns. Wave 0064 gave
/// the plan package a second `read_to_string` of its own and lost
/// exactly that, on exactly those files (review 0064 R-3), so there
/// is one hand now and no second road to forget.
fn wave_text(root: &Path, slug: &str) -> Result<String, Refusal> {
    let path = root.join("keel/waves").join(format!("{slug}.md"));
    let text = std::fs::read_to_string(&path).map_err(|e| Refusal {
        file: path.clone(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    Ok(text.replace("\r\n", "\n"))
}

/// The package a fresh reader gets (§9.9), in the form the branch
/// asks for: the work package on a wave's branch, the plan package on
/// `plan/<wave>`. Which wave it is about is read from the branch and
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
    if let Some(branch) = scope::current_branch(root)
        && let Some(named) = branch.strip_prefix("plan/")
    {
        // A plan branch whose slug names no wave is NOT "some other
        // branch": the old advice -- "stand on the wave's branch" --
        // sends a person where they already are. Review 0064 R-9.
        let Some(wave) = scan.waves.iter().find(|w| w.slug == named) else {
            return Err(Refusal {
                file: root.to_path_buf(),
                reason: ta("review-plan-unknown", targs!("branch" => branch.clone())),
                instead: t("review-plan-unknown-instead"),
            });
        };
        // A wave called off is outside judgement, and §6.3-a says
        // EVERY court says so aloud. The work package learned this in
        // wave 0053 (global review R-12); the plan package was born
        // with the same silence four lines away from that lesson
        // (review 0064 R-4).
        if let Some(why) = &wave.cancelled {
            let mut out = t("review-plan-title");
            out.push('\n');
            writeln!(
                out,
                "{}",
                ta(
                    "review-cancelled",
                    targs!("wave" => wave.slug.clone(), "why" => why.clone())
                )
            )
            .unwrap();
            return Ok(out);
        }
        // Cuts with no answer at all, named by the same court that
        // reddens over them -- so the package cannot send a reader to
        // judge a plan the tool has already refused (review 0064 R-5).
        let silent: Vec<String> = crate::graph::silent_cuts(wave);
        return plan_package(root, wave, &silent);
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
    let text = wave_text(root, &slug)?;
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
