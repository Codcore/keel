//! The closure court (contract tool-close; §6.5, journal A2): a wave
//! is closed by consequences, not by commit archaeology -- main's
//! messages are never read, squash cannot break the verdict.

use crate::adapter;
use crate::config;
use crate::docs::{self, Wave};
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::rev;
use crate::scope;
use crate::tags::{self, TestTag};
use crate::targs;
use std::collections::BTreeMap;
use std::path::Path;

/// How many times the closure battery runs (§7.13, wave 0019): one
/// run hides whatever depends on order, time, or a process that
/// outlived its test. Three is the tool's own discipline -- a
/// constant, not a knob.
pub(crate) const BATTERY_RUNS: usize = 3;

/// The verdicts of the closure battery, one per run, keyed like the
/// adapter's map: (test file stem, function name).
pub(crate) type Battery = BTreeMap<(String, String), Vec<bool>>;

/// The structural stages of a wave -- close's own verdicts, opened
/// pub(crate) so the stage eye (rung 11) asks instead of duplicating.
pub(crate) enum State {
    Closed {
        refs_unjudged: u64,
    },
    ClosedLight,
    /// Called off after it was started (§6): nothing to prove and
    /// nothing to wait for, and the reason travels with it.
    Cancelled(String),
    Plan,
    Progress(Vec<String>),
}

/// The `keel close` command: the battery run three times (§7.13),
/// then one of three states
/// per wave; the second number counts the blockers -- the lacks of
/// the wave the current branch is named after (§8.2). Other waves
/// inform, they do not punish.
/// What the closing court wants free before it starts.
///
/// MEASURED, not guessed (review 0031 R-5): one full `keel close` on
/// this repository leaves 1.26 GiB in the target directory -- the
/// three battery runs (§7.13) share one target, they do not each
/// build their own. The first version of this constant said 4 GiB
/// from the ceiling and refused with 3.5 GiB free, where the work
/// would have finished with 2.2 GiB to spare. Two gigabytes is the
/// measured price plus room for a project larger than this one.
const NEEDED_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/// Free bytes on the filesystem holding this project, or nothing
/// when the question cannot be asked -- a court that cannot see the
/// disk still runs, it just cannot warn.
fn free_bytes(root: &Path) -> Option<u64> {
    let out = std::process::Command::new("df")
        .args(["-B1", "--output=avail"])
        .arg(root)
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .nth(1)?
        .trim()
        .parse()
        .ok()
}

/// Bytes as whole gigabytes, the unit a person reasons in here.
fn gigabytes(bytes: u64) -> u64 {
    bytes / (1024 * 1024 * 1024)
}

/// Gigabytes to one decimal place, rendered as a string: a target of
/// 1.26 GiB reported as "1" would hide the very number this wave
/// exists to name.
fn tenths_of_gigabyte(bytes: u64) -> String {
    let tenths = bytes / (1024 * 1024 * 102);
    format!("{}.{}", tenths / 10, tenths % 10)
}

/// What a directory weighs, walked once. A price nobody can see is
/// the reason two reviewers refused to run this court at all.
fn directory_bytes(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| match entry.metadata() {
            Ok(meta) if meta.is_dir() => directory_bytes(&entry.path()),
            Ok(meta) => meta.len(),
            Err(_) => 0,
        })
        .sum()
}

pub fn judge(root: &Path) -> Result<(String, usize), Refusal> {
    // Research never merges (§4.13). This is the court that says
    // whether a branch may go in, so this is where the ban lives --
    // and it is said before anything is built, since nothing here
    // can end in a merge anyway.
    if let Some(name) = crate::scope::spike_branch(root) {
        return Err(Refusal {
            file: root.to_path_buf(),
            reason: ta("close-spike", targs!("branch" => format!("spike/{name}"))),
            instead: t("close-spike-instead"),
        });
    }
    let config = config::read(root)?;
    if !config.rust_adapter() {
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
    // The price, said before it is paid (wave 0031). This court
    // builds the judged project into ITS OWN target directory on
    // purpose -- an inherited shared cache shifts verdicts (§6.7,
    // the heal of 0005 per review 0008 R-8) -- so the fix is not to
    // stop paying, it is to say what it costs. Measured when this
    // wave was planned: tool/target stood at 3.3 GB, and the
    // reviewers of waves 0028 and 0029 BOTH skipped running this
    // court because the disk was too tight.
    // The directory this court actually builds into -- review 0031
    // R-4 found both the price and the refusal naming "tool/target"
    // for every project, while the adapter builds into the crate
    // root's own target: on a project whose Cargo.toml is at the
    // root, the refusal named a directory that does not exist and
    // the instead swept nothing.
    let target = adapter::crate_root(root)?.join("target");
    let needed = NEEDED_BYTES;
    if let Some(free) = free_bytes(root).filter(|free| *free < needed) {
        return Err(Refusal {
            file: target,
            reason: ta(
                "close-no-room",
                targs!("free" => gigabytes(free), "needed" => gigabytes(needed)),
            ),
            instead: t("close-no-room-instead"),
        });
    }
    // Said BEFORE the work, and said where a person can see it now:
    // review 0031 R-1 measured the whole report, price line included,
    // appearing 101 seconds in -- after the target was already built.
    // A warning that arrives with the bill is not a warning.
    eprintln!(
        "{}",
        ta(
            "close-price",
            targs!(
                "target" => target.display().to_string(),
                "needed" => gigabytes(NEEDED_BYTES)
            )
        )
    );
    let found = tags::scan(&adapter::test_files(root)?)?;
    // The battery runs several times before green is believed
    // (§7.13): the adapter keeps its word -- one battery, one cargo
    // run -- and the court folds the runs.
    let mut battery: Battery = BTreeMap::new();
    for _ in 0..BATTERY_RUNS {
        for (key, green) in adapter::run_all(root)? {
            battery.entry(key).or_default().push(green);
        }
    }
    let branch = scope::branch_wave(root, &scan.waves);
    // A scenario namesake may live in several waves: every wave's own
    // revision is legal for the slug, and a tag holding a foreign
    // wave's revision is not this wave's lack (review R-3).
    let mut legal: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for wave in &scan.waves {
        let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
        for (name, revision) in rev::scenario_revs(&path)? {
            legal.entry(name).or_default().push(revision);
        }
    }

    let mut report = t("close-title");
    report.push('\n');
    report.push_str(&ta(
        "close-battery",
        targs!("count" => battery.len() as u64, "runs" => BATTERY_RUNS as u64),
    ));
    report.push('\n');
    // What the court just watched fail, by name (bug audit B6): it
    // ran the battery three times, saw red, and said only that the
    // wave is not closed -- so a person had to run the whole battery
    // again to learn what this court had already seen.
    let mut fell: Vec<String> = battery
        .iter()
        .filter(|(_, runs)| runs.iter().any(|green| !green))
        .map(|((file, test), runs)| {
            let every = runs.iter().all(|green| !green);
            ta(
                if every {
                    "close-test-red"
                } else {
                    "close-test-flaky"
                },
                targs!("file" => file.clone(), "test" => test.clone()),
            )
        })
        .collect();
    fell.sort();
    for line in &fell {
        report.push_str(line);
        report.push('\n');
    }

    // The verify of live contracts (§7.6, §2.8), under the §7.16
    // trust court: only a matching fingerprint runs; a failing
    // command is a blocker -- a broken foreign promise does not
    // merge; distrust is check's verdict, said here by name only.
    // The project's ci follows through the same gate (wave 0019).
    let mut verify_count: u64 = 0;
    let mut verify_blockers = 0usize;
    let mut verify_lines: Vec<String> = Vec::new();
    for contract in &scan.contracts {
        if contract.withdrawn.is_some() {
            continue;
        }
        let Some(command) = &contract.verify else {
            continue;
        };
        verify_count += 1;
        if !crate::trust::trusted(&config, command) {
            verify_lines.push(ta(
                "close-verify-untrusted",
                targs!("command" => command.clone(), "contract" => contract.slug.clone()),
            ));
            continue;
        }
        match run_command(root, command) {
            Ok(()) => verify_lines.push(ta(
                "close-verify-passed",
                targs!("command" => command.clone(), "contract" => contract.slug.clone()),
            )),
            Err(words) => {
                verify_lines.push(ta(
                    "close-verify-failed",
                    targs!("command" => command.clone(), "contract" => contract.slug.clone(), "words" => words),
                ));
                verify_blockers += 1;
            }
        }
    }
    report.push_str(&ta("close-verify-count", targs!("count" => verify_count)));
    report.push('\n');
    for line in &verify_lines {
        report.push_str("  ");
        report.push_str(line);
        report.push('\n');
    }

    // The project's own ci (wave 0019, the first field's gift)
    // through the same §7.16 gate as verify: a trusted command runs
    // exactly once as the project's own merge gate -- never as a
    // contract's proof; untrusted, none, undecided and absent are
    // each a word, never a run. "Trusted" means "runs".
    let mut ci_blocker = 0usize;
    let ci_line = match config.ci.as_deref() {
        None => t("close-ci-absent"),
        Some("") => t("close-ci-undecided"),
        Some("none") => t("close-ci-none"),
        Some(command) if !crate::trust::trusted(&config, command) => ta(
            "close-ci-untrusted",
            targs!("command" => command.to_string()),
        ),
        Some(command) => match run_command(root, command) {
            Ok(()) => ta("close-ci-passed", targs!("command" => command.to_string())),
            Err(words) => {
                ci_blocker = 1;
                ta(
                    "close-ci-failed",
                    targs!("command" => command.to_string(), "words" => words),
                )
            }
        },
    };
    report.push_str("  ");
    report.push_str(&ci_line);
    report.push('\n');
    report.push('\n');

    let mut blockers = 0usize;
    let mut own_plan = false;
    for wave in &scan.waves {
        let state = wave_state(root, wave, &found, &legal, Some(&battery))?;
        let own = branch.as_deref() == Some(wave.slug.as_str());
        match state {
            State::Closed { refs_unjudged: 0 } => {
                report.push_str(&ta("close-closed", targs!("wave" => wave.slug.clone())));
                report.push('\n');
            }
            State::Closed { refs_unjudged } => {
                // Green is not painted over the unjudged (review R-4):
                // where history cannot testify, the line says so.
                report.push_str(&ta(
                    "close-closed-unjudged",
                    targs!("wave" => wave.slug.clone(), "count" => refs_unjudged),
                ));
                report.push('\n');
            }
            State::ClosedLight => {
                report.push_str(&ta(
                    "close-closed-light",
                    targs!("wave" => wave.slug.clone()),
                ));
                report.push('\n');
            }
            State::Cancelled(why) => {
                report.push_str(&ta(
                    "close-cancelled",
                    targs!("wave" => wave.slug.clone(), "why" => why),
                ));
                report.push('\n');
            }
            State::Plan => {
                report.push_str(&ta("close-plan", targs!("wave" => wave.slug.clone())));
                report.push('\n');
                if own {
                    own_plan = true;
                }
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
    if verify_blockers > 0 {
        report.push_str(&ta(
            "close-verify-blockers",
            targs!("count" => verify_blockers as u64),
        ));
        report.push('\n');
    }
    if ci_blocker > 0 {
        report.push_str(&t("close-ci-blocker"));
        report.push('\n');
    }
    if blockers > 0 {
        report.push_str(&ta(
            "close-blockers",
            targs!("wave" => branch.unwrap_or_default(), "count" => blockers as u64),
        ));
        report.push('\n');
    } else if own_plan {
        // The honest footer for the plan branch (review R-2): a plan
        // PR merges as a plan (§6.6), and the old words would lie.
        report.push_str(&ta(
            "close-plan-own",
            targs!("wave" => branch.unwrap_or_default()),
        ));
        report.push('\n');
    } else if verify_blockers == 0 && ci_blocker == 0 {
        report.push_str(&t("close-no-blockers"));
        report.push('\n');
    }
    // And what the price actually came to (review 0031 R-6: the
    // scenario promised this sentence and the first cut of the work
    // simply did not carry it).
    report.push_str(&ta(
        "close-price-paid",
        targs!(
            "target" => target.display().to_string(),
            "size" => tenths_of_gigabyte(directory_bytes(&target))
        ),
    ));
    report.push('\n');
    Ok((report, blockers + verify_blockers + ci_blocker))
}

/// Runs one trusted command from the repository's files through
/// `sh -c` at the root -- the verify of a contract, the project's
/// ci: success is silence; failure carries the command's last
/// non-empty line of stderr, else stdout, else the keyed word (0010
/// review R-5) -- no raw English inside a localized verdict. A
/// command that does not start fails with the system's words.
fn run_command(root: &Path, command: &str) -> Result<(), String> {
    let out = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(root)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    Err(stderr
        .lines()
        .rev()
        .map(visible)
        .find(|l| !l.is_empty())
        .or_else(|| stdout.lines().rev().map(visible).find(|l| !l.is_empty()))
        .unwrap_or_else(|| t("close-verify-no-words")))
}

/// The visible text of a line: the colours a command paints are not
/// words. `cargo fmt --check` ends its diff with a bare reset
/// sequence, and a verdict quoting that escape says nothing at all
/// -- found in the field on slugline, wave 0019; the same school as
/// 0010 review R-5, where a verdict must carry words, not noise.
fn visible(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                // A CSI sequence: the opener is not its final byte
                // (the first cut of this fix broke right here), so
                // it is eaten before the hunt for one in @..~.
                chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            } else {
                // Two-character escapes such as ESC ( B.
                chars.next();
            }
            continue;
        }
        if c.is_control() {
            // Bare control bytes are not words either: rustfmt
            // paints a shift-in after every reset.
            continue;
        }
        out.push(c);
    }
    out.trim().to_string()
}

/// Structural closure -- without running the tests: every live
/// scenario carries a matching tag, the references converge, a full
/// wave has its review file. The §5.6 floor in check leans on this:
/// the history blessing belongs to the structurally closed.
pub fn structural(root: &Path, wave: &Wave, tags: &[TestTag]) -> Result<bool, Refusal> {
    // Without the whole project's waves in hand, structural judges a
    // namesake tag conservatively: a foreign revision does not count,
    // so the blessing is withheld, never wrongly granted.
    let wave_path = root.join("keel/waves").join(format!("{}.md", wave.slug));
    let mut legal: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, revision) in rev::scenario_revs(&wave_path)? {
        legal.entry(name).or_default().push(revision);
    }
    Ok(matches!(
        wave_state(root, wave, tags, &legal, None)?,
        State::Closed { .. } | State::ClosedLight | State::Cancelled(_)
    ))
}

/// A wave with nothing to prove: it carries no scenario at all, so
/// there is no test to wait for and merging closes it (§2.11).
///
/// This used to be called `light` and cite §6.8, and it counted by a
/// rule of its own -- so the tool said one weight here and another in
/// `status`, both citing the same paragraph (review 0036 R-1). Weight
/// is §6.8's question and lives in `docs::weight`; this is a
/// different question, and the two are asked apart now. The weight
/// still decides the ceremony: a FULL wave is never closed by merge
/// alone, even with nothing to prove, because §6.8 buys a second
/// human look for exactly that case -- a chore that grows a contract
/// (review R-2 measured `close` calling one closed and green).
pub(crate) fn nothing_to_prove(wave: &docs::Wave) -> bool {
    wave.scenarios.is_empty() && docs::weight(wave) == docs::Weight::Light
}

pub(crate) fn wave_state(
    root: &Path,
    wave: &docs::Wave,
    found: &[TestTag],
    legal: &BTreeMap<String, Vec<String>>,
    battery: Option<&Battery>,
) -> Result<State, Refusal> {
    if let Some(why) = &wave.cancelled {
        return Ok(State::Cancelled(why.clone()));
    }
    // A wave with no promises has no test to wait for, but §9.9 asks
    // a person to read it all the same (the operator's decision of
    // 2026-09-04): merging is its closure only once the report lies
    // beside it.
    if nothing_to_prove(wave) {
        let report = root.join("keel/reviews").join(format!("{}.md", wave.slug));
        if report.is_file() {
            return Ok(State::ClosedLight);
        }
        return Ok(State::Progress(vec![t("close-lack-review")]));
    }

    let wave_path = root.join("keel/waves").join(format!("{}.md", wave.slug));
    let revs = rev::scenario_revs(&wave_path)?;
    let live: Vec<&String> = wave
        .scenarios
        .iter()
        .filter(|(_, sc)| sc.withdrawn.is_none())
        .map(|(n, _)| n)
        .collect();

    // A plan on main without a single tag of its own is not red
    // (§6.5). A namesake's tag holding another wave's legal revision
    // is that wave's proof, not this one's start (review 0012 R-2,
    // the 0011 R-9 school): it neither starts the plan nor hides it
    // from the awaiting list. A crooked record still counts as a
    // start -- it is this wave's own staleness to answer for.
    let started = |name: &&String| {
        let current = revs
            .iter()
            .find(|(n, _)| n == *name)
            .map(|(_, r)| r.as_str())
            .unwrap_or("");
        found.iter().filter(|t| t.scenario == **name).any(|t| {
            rev::matches(&t.rev, current)
                || !legal
                    .get(*name)
                    .is_some_and(|revs| revs.iter().any(|r| rev::matches(&t.rev, r)))
        })
    };
    let any_tag = live.iter().any(started);
    if !any_tag && !live.is_empty() {
        return Ok(State::Plan);
    }

    let mut lacks: Vec<String> = Vec::new();
    for name in &live {
        let current = revs
            .iter()
            .find(|(n, _)| n == *name)
            .map(|(_, r)| r.clone())
            .unwrap_or_default();
        // A namesake's tag holding another wave's legal revision is
        // that wave's proof, not this wave's lack (review R-3); a
        // record matching no wave at all is a crooked one and stands
        // as staleness here.
        let all: Vec<&TestTag> = found.iter().filter(|t| t.scenario == **name).collect();
        let mine: Vec<&TestTag> = all
            .iter()
            .copied()
            .filter(|t| rev::matches(&t.rev, &current))
            .collect();
        if mine.is_empty() {
            let foreign = |t: &&TestTag| {
                legal
                    .get(*name)
                    .is_some_and(|revs| revs.iter().any(|r| rev::matches(&t.rev, r)))
            };
            if let Some(crooked) = all.iter().find(|t| !foreign(t)) {
                lacks.push(ta(
                    "close-lack-stale",
                    targs!("scenario" => (*name).clone(), "recorded" => crooked.rev.clone(), "actual" => current.clone()),
                ));
            } else {
                lacks.push(ta(
                    "close-lack-untagged",
                    targs!("scenario" => (*name).clone()),
                ));
            }
            continue;
        }
        for tag in mine {
            if let Some(battery) = battery {
                let stem = tag
                    .file
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                // Green only when green in every run (§7.13): green
                // in some runs is a lack with its count, never a
                // blessing by the one green run; red in all stays red.
                match battery.get(&(stem, tag.test.clone())) {
                    Some(runs) if runs.len() == BATTERY_RUNS && runs.iter().all(|g| *g) => {}
                    Some(runs) if runs.iter().any(|g| *g) => lacks.push(ta(
                        "close-lack-flaky",
                        targs!("scenario" => (*name).clone(), "test" => tag.test.clone(), "green" => runs.iter().filter(|g| **g).count() as u64, "runs" => BATTERY_RUNS as u64),
                    )),
                    Some(_) => lacks.push(ta(
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

    // The references of the wave converge (§6.4): the current text
    // or a revision true in history; where history cannot testify
    // (shallow, no git) the reference is counted unjudged, and the
    // closed line says so instead of claiming convergence (R-4).
    let mut refs_unjudged: u64 = 0;
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
        if !crate::check::history_testifies(root) {
            refs_unjudged += 1;
            continue;
        }
        if crate::check::revision_in_history(root, &relative, &reference.rev) {
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
        Ok(State::Closed { refs_unjudged })
    } else {
        Ok(State::Progress(lacks))
    }
}
