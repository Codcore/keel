//! The closure court (contract tool-close; §6.5, journal A2): a wave
//! is closed by consequences, not by commit archaeology -- main's
//! messages are never read, squash cannot break the verdict.

use crate::adapter;
use crate::config;
use crate::docs;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::rev;
use crate::scope;
use crate::tags::{self, TestTag};
use crate::targs;
use std::collections::BTreeMap;
use std::path::Path;

enum State {
    Closed,
    ClosedLight,
    Plan,
    Progress(Vec<String>),
}

/// The `keel close` command: one battery, then one of three states
/// per wave; the second number counts the blockers -- the lacks of
/// the wave the current branch is named after (§8.2). Other waves
/// inform, they do not punish.
pub fn judge(root: &Path) -> Result<(String, usize), Refusal> {
    let config = config::read(root)?;
    if config.adapter.as_deref() != Some("cargo") {
        return Err(Refusal {
            file: root.join("keel.toml"),
            reason: t("close-needs-adapter"),
            instead: t("close-needs-adapter-instead"),
        });
    }
    let scan = docs::scan(root)?;
    if let Some(refusal) = scan.refusals.into_iter().next() {
        // A court over documents it cannot read would judge shadows;
        // check names every broken file -- fix them first.
        return Err(refusal);
    }
    let found = tags::scan(&adapter::test_files(root)?)?;
    let battery = adapter::run_all(root)?;
    let branch = scope::branch_wave(root, &scan.waves);

    let mut report = t("close-title");
    report.push('\n');
    report.push_str(&ta(
        "close-battery",
        targs!("count" => battery.len() as u64),
    ));
    report.push_str("\n\n");

    let mut blockers = 0usize;
    for wave in &scan.waves {
        let state = wave_state(root, wave, &found, Some(&battery))?;
        let own = branch.as_deref() == Some(wave.slug.as_str());
        match state {
            State::Closed => {
                report.push_str(&ta("close-closed", targs!("wave" => wave.slug.clone())));
                report.push('\n');
            }
            State::ClosedLight => {
                report.push_str(&ta(
                    "close-closed-light",
                    targs!("wave" => wave.slug.clone()),
                ));
                report.push('\n');
            }
            State::Plan => {
                report.push_str(&ta("close-plan", targs!("wave" => wave.slug.clone())));
                report.push('\n');
            }
            State::Progress(lacks) => {
                report.push_str(&ta("close-progress", targs!("wave" => wave.slug.clone())));
                report.push('\n');
                for lack in &lacks {
                    report.push_str(&format!("           {lack}\n"));
                }
                if own {
                    blockers += lacks.len();
                }
            }
        }
    }

    report.push('\n');
    if blockers > 0 {
        report.push_str(&ta(
            "close-blockers",
            targs!("wave" => branch.unwrap_or_default(), "count" => blockers as u64),
        ));
    } else {
        report.push_str(&t("close-no-blockers"));
    }
    report.push('\n');
    Ok((report, blockers))
}

/// Structural closure -- without running the tests: every live
/// scenario carries a matching tag, the references converge, a full
/// wave has its review file. The §5.6 floor in check leans on this:
/// the history blessing belongs to the structurally closed.
pub fn structural(root: &Path, wave: &docs::Wave, tags: &[TestTag]) -> Result<bool, Refusal> {
    Ok(matches!(
        wave_state(root, wave, tags, None)?,
        State::Closed | State::ClosedLight
    ))
}

fn wave_state(
    root: &Path,
    wave: &docs::Wave,
    found: &[TestTag],
    battery: Option<&BTreeMap<(String, String), bool>>,
) -> Result<State, Refusal> {
    // A light wave -- chores only -- is closed by the fact of merge
    // (§6.5) and needs no report.
    let light = wave
        .transforms
        .iter()
        .all(|(_, tr)| matches!(tr.kind, docs::TransformKind::Chore(_)));
    if light {
        return Ok(State::ClosedLight);
    }

    let wave_path = root.join("keel/waves").join(format!("{}.md", wave.slug));
    let revs = rev::scenario_revs(&wave_path)?;
    let live: Vec<&String> = wave
        .scenarios
        .iter()
        .filter(|(_, sc)| sc.withdrawn.is_none())
        .map(|(n, _)| n)
        .collect();

    // A plan on main without a single tag is not red (§6.5).
    let any_tag = live
        .iter()
        .any(|name| found.iter().any(|t| t.scenario == **name));
    if !any_tag && !live.is_empty() {
        return Ok(State::Plan);
    }

    let mut lacks: Vec<String> = Vec::new();
    for name in &live {
        let mine: Vec<&TestTag> = found.iter().filter(|t| t.scenario == **name).collect();
        if mine.is_empty() {
            lacks.push(ta(
                "close-lack-untagged",
                targs!("scenario" => (*name).clone()),
            ));
            continue;
        }
        let current = revs
            .iter()
            .find(|(n, _)| n == *name)
            .map(|(_, r)| r.clone())
            .unwrap_or_default();
        for tag in mine {
            if !rev::matches(&tag.rev, &current) {
                lacks.push(ta(
                    "close-lack-stale",
                    targs!("scenario" => (*name).clone(), "recorded" => tag.rev.clone(), "actual" => current.clone()),
                ));
                continue;
            }
            if let Some(battery) = battery {
                let stem = tag
                    .file
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                match battery.get(&(stem, tag.test.clone())) {
                    Some(true) => {}
                    Some(false) => lacks.push(ta(
                        "close-lack-red",
                        targs!("scenario" => (*name).clone(), "test" => tag.test.clone()),
                    )),
                    None => lacks.push(ta(
                        "close-lack-notrun",
                        targs!("scenario" => (*name).clone(), "test" => tag.test.clone()),
                    )),
                }
            }
        }
    }

    // The references of the wave converge (§6.4): the current text,
    // a revision true in history, or -- where history cannot testify
    // (shallow, no git) -- not disproven, and check already says so
    // aloud in its own report.
    let mut refs: Vec<&docs::ContractRef> = wave
        .scenarios
        .iter()
        .filter(|(_, sc)| sc.withdrawn.is_none())
        .filter_map(|(_, sc)| sc.proves.as_ref())
        .collect();
    for (_, tr) in &wave.transforms {
        refs.extend(tr.contracts.iter());
    }
    for reference in refs {
        let path = root
            .join("keel/contracts")
            .join(format!("{}.md", reference.slug));
        if !path.is_file() {
            lacks.push(ta(
                "close-lack-ref",
                targs!("contract" => reference.slug.clone(), "recorded" => reference.rev.clone()),
            ));
            continue;
        }
        let actual = rev::contract_rev(&path)?;
        if rev::matches(&reference.rev, &actual) {
            continue;
        }
        let relative = format!("keel/contracts/{}.md", reference.slug);
        if !crate::check::history_testifies(root)
            || crate::check::revision_in_history(root, &relative, &reference.rev)
        {
            continue;
        }
        lacks.push(ta(
            "close-lack-ref",
            targs!("contract" => reference.slug.clone(), "recorded" => reference.rev.clone()),
        ));
    }

    // The §9.9 gate held by mechanics: a full wave carries its review.
    if !root
        .join("keel/reviews")
        .join(format!("{}.md", wave.slug))
        .is_file()
    {
        lacks.push(t("close-lack-review"));
    }

    if lacks.is_empty() {
        Ok(State::Closed)
    } else {
        Ok(State::Progress(lacks))
    }
}
