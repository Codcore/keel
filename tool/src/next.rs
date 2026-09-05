//! Rung 11, the step hand (contract tool-next; §9.2, §9.10, §8.4):
//! exactly one step, and the package is self-sufficient -- a fresh
//! agent continues from this word out of any death of a session.
//! next advises, it never judges: green tests belong to the hook
//! and to close.

use crate::adapter;
use crate::close::{self, State};
use crate::config;
use crate::docs;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::rev;
use crate::scope;
use crate::tags;
use crate::targs;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// The `keel next` package: one step out of the state. A broken
/// document is the first step of its own -- mend it, by name.
/// The same step in the ANSWER SHAPE a named agent's session hook
/// expects (wave 0025). Claude Code injects a hook's plain stdout
/// into the agent's context, so its shape is the plain step. Cursor
/// takes context only as JSON, and the field is `additional_context`
/// (its own docs) -- so for Cursor the step rides wrapped. The names
/// come from the one home of the agent question (config::AGENTS); an
/// unknown one is refused with the names that are known, because a
/// hook that silently says nothing is worse than a hook that refuses.
///
/// The shape lives here because the home of the step is one -- not in
/// generated, which knows files, not words.
pub fn step_for(root: &Path, agent: &str) -> Result<String, Refusal> {
    if !crate::config::AGENTS.contains(&agent) {
        return Err(unknown_agent(root, agent));
    }
    // A hook that speaks must always speak. When the step cannot be
    // said -- no keel/ here yet, a broken keel.toml, a document that
    // does not read -- the refusal is itself the word the agent
    // needs, so it rides in the agent's own shape and the exit stays
    // green. Measured, not guessed: right after `keel init`, with no
    // wave yet, the step refuses; and in Cursor an exit code of 2
    // means "block the action" (their docs, for compatibility with
    // Claude Code), so a refusing hook must not exit 2. `keel next`
    // without --for keeps its own behaviour, untouched.
    let said = match step(root) {
        Ok(said) => said,
        Err(refusal) => format!("{refusal}"),
    };
    say_for(agent, &said)
}

/// One word in a named agent's answer shape. The hook of a tool whose
/// config court refused needs this before any step can be read, so
/// the shaping is its own hand -- and it judges the agent's name the
/// same way, from the one home (config::AGENTS).
pub fn say_for(agent: &str, said: &str) -> Result<String, Refusal> {
    if !crate::config::AGENTS.contains(&agent) {
        return Err(unknown_agent(Path::new("."), agent));
    }
    if agent != "cursor" {
        return Ok(said.to_string());
    }
    // JSON by hand, and only because the payload is one string: the
    // escaping below is the whole of the JSON string grammar we need,
    // and the scenario parses the result with a real parser.
    let mut escaped = String::with_capacity(said.len() + 16);
    for ch in said.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if (c as u32) < 0x20 => escaped.push_str(&format!("\\u{:04x}", c as u32)),
            c => escaped.push(c),
        }
    }
    Ok(format!("{{\"additional_context\": \"{escaped}\"}}\n"))
}

/// The one word for an agent this release does not know.
fn unknown_agent(root: &Path, agent: &str) -> Refusal {
    // Through i18n like every other word of the tool (review 0025
    // R-6: these two were the only refusals in the file speaking
    // English into a Ukrainian frame). i18n is already initialised by
    // the time a hook or a person can reach here.
    let known = crate::config::AGENTS.join(", ");
    Refusal {
        file: root.to_path_buf(),
        reason: ta(
            "next-unknown-agent",
            targs!("agent" => agent.to_string(), "known" => known.clone()),
        ),
        instead: ta("next-unknown-agent-instead", targs!("known" => known)),
    }
}

pub fn step(root: &Path) -> Result<String, Refusal> {
    let config = config::read(root)?;
    if !config.rust_adapter() {
        return Err(Refusal {
            file: root.join("keel.toml"),
            reason: t("next-needs-adapter"),
            instead: t("next-needs-adapter-instead"),
        });
    }
    let scan = docs::scan(root)?;
    let mut report = t("next-title");
    report.push_str("\n\n");

    if let Some(first) = scan.refusals.first() {
        report.push_str(&fix_step(root, first, scan.refusals.len()));
        return Ok(report);
    }

    let branch = scope::current_branch(root);
    if let Some(name) = &branch {
        if let Some(wave) = scan.waves.iter().find(|w| w.slug == *name) {
            // Called off (§6.3-a): the hand of the loop does not
            // drive work on a wave nobody is doing. Review 0037 R-7
            // measured it handing out "write the test" for a
            // scenario of a cancelled wave.
            if let Some(why) = &wave.cancelled {
                report.push_str(&ta(
                    "next-cancelled",
                    targs!("wave" => wave.slug.clone(), "why" => why.clone()),
                ));
                report.push('\n');
                return Ok(report);
            }
            report.push_str(&wave_step(root, wave, &scan.waves)?);
            return Ok(report);
        }
        if let Some(rest) = name.strip_prefix("plan/")
            && scan.waves.iter().any(|w| w.slug == rest)
        {
            report.push_str(&ta("next-plan-branch", targs!("wave" => rest.to_string())));
            report.push('\n');
            return Ok(report);
        }
    }

    // Off a wave branch: the readiness overview -- what starts, what
    // continues, or the honest word that planning is the step.
    let found = tags::scan(&adapter::test_files(root)?)?;
    let mut legal: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for wave in scan.waves.iter().filter(|w| w.cancelled.is_none()) {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        match rev::scenario_revs(&path) {
            Ok(revs) => {
                for (name, revision) in revs {
                    legal.entry(name).or_default().push(revision);
                }
            }
            Err(refusal) => {
                report.push_str(&fix_step(root, &refusal, 1));
                return Ok(report);
            }
        }
    }
    let mut lines: Vec<String> = Vec::new();
    for wave in scan.waves.iter().filter(|w| w.cancelled.is_none()) {
        match close::wave_state(root, wave, &found, &legal, None)? {
            State::Plan => {
                let deps_closed = wave.depends_on.iter().all(|dep| {
                    // A cancelled dependency is not a closed one: its
                    // promises were never kept (review 0037 R-19).
                    scan.waves
                        .iter()
                        .find(|w| w.slug == *dep)
                        .is_some_and(|w| close::structural(root, w, &found).unwrap_or(false))
                });
                if deps_closed {
                    lines.push(ta("next-ready", targs!("wave" => wave.slug.clone())));
                }
            }
            State::Progress(_) => {
                lines.push(ta("next-working", targs!("wave" => wave.slug.clone())));
            }
            _ => {}
        }
    }
    if lines.is_empty() {
        report.push_str(&t("next-all-closed"));
        report.push('\n');
    } else {
        for line in &lines {
            report.push_str(line);
            report.push('\n');
        }
    }
    Ok(report)
}

/// The one step of a wave branch, first by declaration order: an
/// untagged scenario births its test; a drifted tag updates it; then
/// the first transform whose declared files are not all touched;
/// then the review; then the PR.
fn wave_step(root: &Path, wave: &docs::Wave, waves: &[docs::Wave]) -> Result<String, Refusal> {
    let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
    let revs = match rev::scenario_revs(&path) {
        Ok(revs) => revs,
        Err(refusal) => return Ok(fix_step(root, &refusal, 1)),
    };
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    let found = tags::scan(&adapter::test_files(root)?)?;
    // The legal revisions of every wave feed the namesake court
    // (review 0012 R-1, the close school): a tag holding another
    // wave's legal revision is that wave's proof, never this drift.
    let mut legal: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for other in waves {
        let other_path = root.join("keel/waves").join(format!("{}.md", other.slug));
        match rev::scenario_revs(&other_path) {
            Ok(revs) => {
                for (name, revision) in revs {
                    legal.entry(name).or_default().push(revision);
                }
            }
            Err(refusal) => return Ok(fix_step(root, &refusal, 1)),
        }
    }
    let mut out = String::new();

    for (name, sc) in &wave.scenarios {
        if sc.withdrawn.is_some() {
            continue;
        }
        let current = revs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, r)| r.clone())
            .unwrap_or_default();
        let mine: Vec<_> = found.iter().filter(|t| t.scenario == *name).collect();
        if !mine.is_empty() && mine.iter().any(|t| rev::matches(&t.rev, &current)) {
            continue;
        }
        // No proof of our own. A namesake tag that legally proves
        // another wave is not ours to rewrite (review 0012 R-1,
        // §5.6): only a record matching no wave at all is this
        // wave's own drift to mend; otherwise the step is the birth
        // of this wave's own test.
        let foreign = |t: &&crate::tags::TestTag| {
            legal
                .get(name)
                .is_some_and(|revs| revs.iter().any(|r| rev::matches(&t.rev, r)))
        };
        if let Some(crooked) = mine.iter().find(|t| !foreign(t)) {
            out.push_str(&ta(
                "next-step-stale",
                targs!("scenario" => name.clone(), "recorded" => crooked.rev.clone(), "actual" => current.clone()),
            ));
            out.push('\n');
            return Ok(out);
        }
        out.push_str(&ta("next-step-red", targs!("scenario" => name.clone())));
        out.push('\n');
        out.push_str(&ta(
            "next-body-label",
            targs!("scenario" => name.clone(), "rev" => current.clone()),
        ));
        out.push('\n');
        let body = rev::section(&text, &format!("scenario: {name}")).unwrap_or_default();
        for line in body.lines() {
            out.push_str(&format!("    {line}\n"));
        }
        out.push_str(&ta(
            "next-tag-line",
            targs!("scenario" => name.clone(), "rev" => current.clone()),
        ));
        out.push('\n');
        let tests = adapter::crate_root(root)?.join("tests");
        let shown = tests.strip_prefix(root).unwrap_or(&tests);
        out.push_str(&ta(
            "next-tests-dir",
            targs!("dir" => format!("{}/", shown.display())),
        ));
        out.push('\n');
        return Ok(out);
    }

    let (changed, added) = branch_files(root)?;
    for (name, transform) in &wave.transforms {
        // Every `one new in` line promises exactly one file (§4.1;
        // review 0012 R-8): any other count leaves the transform the
        // step -- "assembled" is not said over what scope reddens.
        let mut dirs: BTreeMap<&str, usize> = BTreeMap::new();
        for line in &transform.files {
            if let docs::ScopeLine::OneNewIn(d) = line {
                *dirs.entry(d.as_str()).or_insert(0) += 1;
            }
        }
        let untouched = transform.files.iter().any(|line| match line {
            docs::ScopeLine::Path(p) => !changed.contains(p),
            docs::ScopeLine::OneNewIn(d) => {
                added.iter().filter(|f| f.starts_with(d.as_str())).count() != dirs[d.as_str()]
            }
        });
        if !untouched {
            continue;
        }
        match &transform.kind {
            docs::TransformKind::Implements(_) => {
                out.push_str(&ta("next-step-transform", targs!("name" => name.clone())));
            }
            docs::TransformKind::Chore(reason) => {
                out.push_str(&ta(
                    "next-step-chore",
                    targs!("name" => name.clone(), "reason" => reason.clone()),
                ));
            }
        }
        out.push('\n');
        out.push_str(&t("next-files-label"));
        out.push('\n');
        for line in &transform.files {
            let shown = match line {
                docs::ScopeLine::Path(p) => p.clone(),
                docs::ScopeLine::OneNewIn(d) => format!("one new in {d}"),
            };
            out.push_str(&format!("    {shown}\n"));
        }
        match rev::section(&text, &format!("transform: {name}")) {
            Some(body) => {
                out.push_str(&ta("next-section-label", targs!("name" => name.clone())));
                out.push('\n');
                for line in body.lines() {
                    out.push_str(&format!("    {line}\n"));
                }
            }
            None => {
                // A silently incomplete package is no package
                // (review 0012 R-3): the missing section is a word.
                out.push_str(&ta("next-section-missing", targs!("name" => name.clone())));
                out.push('\n');
            }
        }
        for reference in &transform.contracts {
            let contract_path = root
                .join("keel/contracts")
                .join(format!("{}.md", reference.slug));
            out.push_str(&ta(
                "next-contract-label",
                targs!("contract" => reference.slug.clone(), "rev" => reference.rev.clone()),
            ));
            out.push('\n');
            match std::fs::read_to_string(&contract_path) {
                Ok(body) => {
                    for line in body.lines() {
                        out.push_str(&format!("    {line}\n"));
                    }
                }
                Err(_) => {
                    out.push_str(&format!(
                        "    {}\n",
                        ta(
                            "next-contract-missing",
                            targs!("contract" => reference.slug.clone())
                        )
                    ));
                }
            }
        }
        if let docs::TransformKind::Implements(scenarios) = &transform.kind {
            let mut runs: Vec<String> = Vec::new();
            for scenario in scenarios {
                for tag in found.iter().filter(|t| t.scenario == *scenario) {
                    let stem = tag
                        .file
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    runs.push(format!(
                        "    cargo test --test {stem} {} -- --exact\n",
                        tag.test
                    ));
                }
            }
            if runs.is_empty() {
                // A bare label over nothing paints the unchecked
                // (review 0012 R-7): the empty run list is a word.
                out.push_str(&t("next-run-none"));
                out.push('\n');
            } else {
                out.push_str(&t("next-run-label"));
                out.push('\n');
                for run in &runs {
                    out.push_str(run);
                }
            }
        }
        return Ok(out);
    }

    // §9.9 asks the report of EVERY wave (the operator's decision of
    // 2026-09-04). It used to ask it of full waves only, and review
    // 0036 measured what that meant once the weight was counted by
    // §6.8 as written: a wave with one transform and a promise would
    // ride one PR with nobody reading it. Weight still decides how
    // many pull requests (§6.8, §8.1) and nothing else.
    let light = docs::weight(wave) == docs::Weight::Light;
    if !root
        .join("keel/reviews")
        .join(format!("{}.md", wave.slug))
        .is_file()
    {
        out.push_str(&ta("next-step-review", targs!("wave" => wave.slug.clone())));
        out.push('\n');
        return Ok(out);
    }

    // The PR words go by weight (§6.8; the debt named by the 0015
    // dogfood): a light wave hears its own -- no painted word about
    // a review it never needed.
    if light {
        out.push_str(&t("next-step-pr-light"));
    } else {
        out.push_str(&t("next-step-pr"));
    }
    out.push('\n');
    Ok(out)
}

/// A broken document as the step itself: the first refusal in full,
/// the rest as a count -- keel check names them all.
fn fix_step(root: &Path, refusal: &Refusal, total: usize) -> String {
    let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
    let mut out = ta(
        "next-step-fix",
        targs!("file" => shown.display().to_string(), "reason" => refusal.reason.clone(), "instead" => refusal.instead.clone()),
    );
    out.push('\n');
    if total > 1 {
        out.push_str(&ta(
            "next-step-fix-more",
            targs!("count" => (total - 1) as u64),
        ));
        out.push('\n');
    }
    out
}

/// What the branch changed and what it added, against the merge-base
/// (the scope school: renames as departure plus arrival, quotePath
/// off so names compare as themselves).
fn branch_files(root: &Path) -> Result<(BTreeSet<String>, BTreeSet<String>), Refusal> {
    let (base, _) = scope::compare_base(root)?;
    let changed = git_names(
        root,
        &["diff", "--name-only", "--no-renames", &base, "HEAD"],
    )?;
    let added = git_names(
        root,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            "--diff-filter=A",
            &base,
            "HEAD",
        ],
    )?;
    Ok((changed, added))
}

fn git_names(root: &Path, args: &[&str]) -> Result<BTreeSet<String>, Refusal> {
    let refuse = |error: String| Refusal {
        file: root.to_path_buf(),
        reason: ta("scope-git-failed", targs!("error" => error)),
        instead: t("scope-git-failed-instead"),
    };
    let out = crate::scope::git_at(root)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .output()
        .map_err(|e| refuse(e.to_string()))?;
    if !out.status.success() {
        return Err(refuse(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect())
}
