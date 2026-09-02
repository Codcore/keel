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
use std::process::Command;

/// Assembles the §9.9 package for the branch's wave (§8.2); any
/// other branch is a refusal aloud -- which wave the package is for
/// is not guessed.
pub fn package(root: &Path) -> Result<String, Refusal> {
    let scan = docs::scan(root)?;
    if let Some(refusal) = scan.refusals.into_iter().next() {
        // A package over unread documents would guess; check names
        // every broken file -- fix them first.
        return Err(refusal);
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
    let rel = format!("keel/waves/{slug}.md");
    let wave_path = root.join(&rel);
    let text = std::fs::read_to_string(&wave_path).map_err(|e| Refusal {
        file: wave_path.clone(),
        reason: format!("the wave file cannot be read: {e}"),
        instead: "check the path and file permissions".to_string(),
    })?;
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

    // Transforms verbatim -- the caveat paragraphs (§2.10) live in
    // these bodies and ride whole, so none is dropped by parsing
    // prose the mechanics cannot read.
    writeln!(out, "\n{}", t("review-transforms-header")).unwrap();
    for (name, _) in &wave.transforms {
        if let Some(body) = section(&text, &format!("transform: {name}")) {
            writeln!(out, "## transform: {name}\n{body}").unwrap();
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
        match drift_anchor(root, &rel) {
            None => writeln!(out, "{}", t("review-drift-unverified")).unwrap(),
            Some(anchor) => {
                let short = anchor.get(..7).unwrap_or(&anchor).to_string();
                writeln!(out, "{}", ta("review-drift-header", targs!("sha" => short))).unwrap();
                match old_wave_files(root, &anchor, &rel, &slug) {
                    None => writeln!(out, "  {}", t("review-drift-unreadable")).unwrap(),
                    Some(old_files) => {
                        let mut drifted = 0;
                        for line in wave_files(wave) {
                            if !old_files.contains(&line) {
                                writeln!(
                                    out,
                                    "  {}",
                                    ta("review-drift-line", targs!("file" => line.clone()))
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
            match git_out(root, &["diff", base_sha]) {
                Some(diff) if !diff.trim().is_empty() => out.push_str(&diff),
                Some(_) => writeln!(out, "  {}", t("review-diff-empty")).unwrap(),
                None => writeln!(out, "  {}", t("review-diff-unverified")).unwrap(),
            }
        }
    }

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

/// The first commit that added the wave file -- the drift anchor of
/// this generation, named aloud in the package.
fn drift_anchor(root: &Path, rel: &str) -> Option<String> {
    let log = git_out(root, &["log", "--diff-filter=A", "--format=%H", "--", rel])?;
    log.lines()
        .last()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
}

/// The wave's declared files as of the anchor commit, read through
/// a temporary copy so the strict parser judges the old text too.
fn old_wave_files(root: &Path, anchor: &str, rel: &str, slug: &str) -> Option<Vec<String>> {
    let old_text = git_show(root, anchor, rel)?;
    let dir = std::env::temp_dir().join(format!("keel-review-anchor-{}", std::process::id()));
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join(format!("{slug}.md"));
    std::fs::write(&path, &old_text).ok()?;
    let wave = docs::read_wave(&path).ok();
    let _ = std::fs::remove_dir_all(&dir);
    wave.map(|w| wave_files(&w))
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
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).into_owned())
}
