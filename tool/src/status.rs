//! Rung 11, the stage eye (contract tool-status; §6.5, §6.8, §9.2):
//! where we stand, said from the state and never from memory. The
//! battery does not run here -- the stage is structural (tags,
//! references, the review), and the report says that aloud.

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
use std::collections::BTreeMap;
use std::path::Path;

/// The `keel status` report and the count of refusal rows: broken
/// documents do not topple the overview -- each stands as a row of
/// the refusal school, and the exit is red only over them.
pub fn report(root: &Path) -> Result<(String, usize), Refusal> {
    let config = config::read(root)?;
    if !config.rust_adapter() {
        return Err(Refusal {
            file: root.join("keel.toml"),
            reason: t("status-needs-adapter"),
            instead: t("status-needs-adapter-instead"),
        });
    }
    let scan = docs::scan(root)?;
    let mut refusals: Vec<Refusal> = scan.refusals;
    let found = tags::scan(&adapter::test_files(root)?)?;

    // The waves whose sections read: a wave whose body refuses is a
    // refusal row, never a silently guessed stage. The legal map
    // feeds the namesake court of wave_state (0006 school).
    let mut legal: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut judged: Vec<&docs::Wave> = Vec::new();
    for wave in &scan.waves {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        match rev::scenario_revs(&path) {
            Ok(revs) => {
                for (name, revision) in revs {
                    legal.entry(name).or_default().push(revision);
                }
                judged.push(wave);
            }
            Err(refusal) => refusals.push(refusal),
        }
    }

    let mut report = t("status-title");
    report.push('\n');

    // Where we ourselves stand (§8.2): a wave branch, a plan branch,
    // a foreign name, or git's silence -- each its own word.
    let branch = scope::current_branch(root);
    let branch_line = match &branch {
        Some(name) if judged.iter().any(|w| w.slug == *name) => {
            ta("status-branch-wave", targs!("branch" => name.clone()))
        }
        Some(name)
            if name
                .strip_prefix("plan/")
                .is_some_and(|rest| judged.iter().any(|w| w.slug == rest)) =>
        {
            ta("status-branch-plan", targs!("branch" => name.clone()))
        }
        // A branch named after a wave whose document refused is said
        // so (review 0012 R-9): "named as no wave" would be a lie
        // with the wave's broken file right there.
        Some(name)
            if refusals.iter().any(|r| {
                r.file
                    .file_stem()
                    .is_some_and(|s| s.to_string_lossy() == name.as_str())
                    && r.file
                        .parent()
                        .and_then(|p| p.file_name())
                        .is_some_and(|d| d == "waves")
            }) =>
        {
            ta("status-branch-broken", targs!("branch" => name.clone()))
        }
        Some(name) => ta("status-branch-other", targs!("branch" => name.clone())),
        None => t("status-branch-none"),
    };
    report.push_str(&branch_line);
    report.push_str("\n\n");

    let mut closed: u64 = 0;
    let mut working: u64 = 0;
    let mut plans: u64 = 0;
    let mut awaiting: Vec<String> = Vec::new();
    for wave in &judged {
        match close::wave_state(root, wave, &found, &legal, None)? {
            State::Closed { refs_unjudged: 0 } => {
                closed += 1;
                report.push_str(&ta(
                    "status-wave-closed",
                    targs!("wave" => wave.slug.clone()),
                ));
                report.push('\n');
            }
            State::Closed { refs_unjudged } => {
                closed += 1;
                report.push_str(&ta(
                    "status-wave-closed-unjudged",
                    targs!("wave" => wave.slug.clone(), "count" => refs_unjudged),
                ));
                report.push('\n');
            }
            State::ClosedLight => {
                // On its own branch a light wave rides -- no merge
                // happened, so its fact is not claimed (review 0012
                // R-6): the state is derived, never guessed.
                if branch.as_deref() == Some(wave.slug.as_str()) {
                    working += 1;
                    report.push_str(&ta(
                        "status-wave-light-own",
                        targs!("wave" => wave.slug.clone()),
                    ));
                } else {
                    closed += 1;
                    report.push_str(&ta(
                        "status-wave-closed-light",
                        targs!("wave" => wave.slug.clone()),
                    ));
                }
                report.push('\n');
            }
            State::Plan => {
                plans += 1;
                report.push_str(&ta("status-wave-plan", targs!("wave" => wave.slug.clone())));
                report.push('\n');
                // Awaiting its start: every dependency structurally
                // closed. A dependency that is missing or unreadable
                // withholds the call -- the graph floor of check names
                // it; here nothing is guessed ready.
                let deps_closed = wave.depends_on.iter().all(|dep| {
                    judged
                        .iter()
                        .find(|w| w.slug == *dep)
                        .is_some_and(|w| close::structural(root, w, &found).unwrap_or(false))
                });
                if deps_closed {
                    awaiting.push(ta("status-awaiting", targs!("wave" => wave.slug.clone())));
                }
            }
            State::Progress(lacks) => {
                working += 1;
                report.push_str(&ta(
                    "status-wave-progress",
                    targs!("wave" => wave.slug.clone()),
                ));
                report.push('\n');
                for lack in lacks {
                    report.push_str(&format!("           {lack}\n"));
                }
            }
        }
    }

    for refusal in &refusals {
        let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
        report.push_str(&format!(
            "  {:<8} {} — {}\n           {}: {}\n",
            t("word-red"),
            shown.display(),
            refusal.reason,
            t("word-instead"),
            refusal.instead
        ));
    }

    report.push('\n');
    for line in &awaiting {
        report.push_str(line);
        report.push('\n');
    }
    report.push_str(&ta(
        "status-counts",
        targs!("closed" => closed, "working" => working, "plans" => plans),
    ));
    report.push('\n');
    report.push_str(&t("status-no-battery"));
    report.push('\n');
    report.push_str(&t("status-next"));
    report.push('\n');
    Ok((report, refusals.len()))
}
