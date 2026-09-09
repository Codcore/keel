//! The closure court (contract tool-close; §6.5, journal A2): a wave
//! is closed by consequences, not by commit archaeology -- main's
//! messages are never read, squash cannot break the verdict.

use crate::adapter;
use crate::config;
use crate::docs::{self, Wave};
use crate::holding;
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
    /// A wave with nothing to prove whose merge has not happened: the
    /// fact of §6.5 is the wave file standing in main, and until it
    /// does the wave WILL close by merge, not is closed. The flag says
    /// whether a main could be asked at all (wave 0052).
    AwaitingMerge(bool),
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
/// MEASURED, not guessed (review 0031 R-5), and measured AGAIN
/// (global review 2026-09-06, methodology R-15; wave 0053): one full
/// `keel close` on this repository leaves 3.0-3.2 GiB in the target
/// directory of this generation (the battery of 195 tests, three
/// runs sharing one target) -- the first measurement, 1.26 GiB,
/// belonged to a smaller battery and stood in the word for twenty
/// waves. The first version of this constant said 4 GiB from the
/// ceiling and refused with 3.5 GiB free, where the work would have
/// finished with 2.2 GiB to spare; the second said 2 GiB and was
/// under the measured weight; the third said 3 GiB and was under it
/// again -- a closing of wave 0055 left 3.7 GiB in this project's own
/// target (review 0055 R-10), and a guard that undercounts by a
/// quarter lets through exactly the run it stands to stop. The
/// number is the measured weight rounded up to whole gibibytes, and
/// the word carries this number and no other.
const NEEDED_BYTES: u64 = 4 * 1024 * 1024 * 1024;

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

pub fn judge(root: &Path) -> Result<(String, usize, Vec<RedCommand>), Refusal> {
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
    if !config.adapter_known() {
        return Err(Refusal {
            file: root.join("keel.toml"),
            reason: t("close-needs-adapter"),
            instead: ta(
                "close-needs-adapter-instead",
                targs!("known" => crate::config::Language::known()),
            ),
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
    // A language that builds nothing costs no disk, and this court
    // stops demanding a crate of it (wave 0038): ruby has no build
    // directory to measure, to warn about or to sweep.
    let target = adapter::build_dir(root);
    let needed = NEEDED_BYTES;
    if let adapter::BuildDir::At(target) = &target
        && adapter::builds_heavily(root)
        && let Some(free) = free_bytes(root).filter(|free| *free < needed)
    {
        return Err(Refusal {
            file: target.clone(),
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
    match &target {
        adapter::BuildDir::At(target) => eprintln!(
            "{}",
            ta(
                // The measured cost belongs to the tongue that pays
                // it: keel's own cargo build wants gigabytes, and a
                // mix project's `_build` measured 148 KiB (review
                // 0042 R-4 -- the warning was out by four orders of
                // magnitude, and its "measured" number was cargo's).
                if adapter::builds_heavily(root) {
                    "close-price"
                } else {
                    "close-price-light"
                },
                targs!(
                    "target" => target.display().to_string(),
                    "needed" => gigabytes(NEEDED_BYTES)
                )
            )
        ),
        adapter::BuildDir::Nothing => eprintln!("{}", t("close-price-nothing-built")),
        // The adapter could not say where, and will say why itself a
        // breath later: no price line is honester than a wrong one.
        adapter::BuildDir::Unknown => {}
    }
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
    for wave in scan.waves.iter().filter(|w| w.cancelled.is_none()) {
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
    // Counted ONCE (review 0043 R-5). A red test that a scenario of
    // this branch's own wave claims is already a lack under that
    // wave, named there with its scenario; adding it here again made
    // `blockers` say two where one test fell. So the tally below
    // counts only the reds nobody's promise was already holding.
    //
    // And "claims" means what `wave_state` means by it (wave 0050):
    // a tag of a LIVE scenario of this wave whose revision is the
    // scenario's current one -- or, where no such tag exists, a tag
    // of that scenario holding no other wave's legal revision, which
    // `wave_state` names as this wave's stale record. By the bare
    // name every tag of every scenario was claimed, withdrawn ones
    // included, so a red test under a crooked revision or under a
    // dead promise was neither a lack nor a red nobody claims: the
    // court printed "red test" and "closed" in one breath (global
    // review 2026-09-06, bugs cut R-2).
    let mut claimed: std::collections::BTreeSet<(String, String)> = Default::default();
    if let Some(slug) = &branch {
        for wave in scan.waves.iter().filter(|wave| &wave.slug == slug) {
            let path = root.join("keel/waves").join(format!("{}.md", wave.slug));
            let revs = rev::scenario_revs(&path)?;
            for (name, scenario) in &wave.scenarios {
                if scenario.withdrawn.is_some() {
                    continue;
                }
                let current = revs
                    .iter()
                    .find(|(n, _)| n == name)
                    .map(|(_, r)| r.as_str())
                    .unwrap_or("");
                let all: Vec<&TestTag> = found.iter().filter(|t| t.scenario == *name).collect();
                let mine: Vec<&TestTag> = all
                    .iter()
                    .copied()
                    .filter(|t| rev::matches(&t.rev, current))
                    .collect();
                let foreign = |t: &&TestTag| {
                    legal
                        .get(name)
                        .is_some_and(|revs| revs.iter().any(|r| rev::matches(&t.rev, r)))
                };
                let named: Vec<&TestTag> = if mine.is_empty() {
                    all.iter().copied().filter(|t| !foreign(t)).collect()
                } else {
                    mine
                };
                for tag in named {
                    claimed.insert((adapter::battery_key(root, &tag.file), tag.test.clone()));
                }
            }
        }
    }
    let red_tests = battery
        .iter()
        .filter(|(key, runs)| runs.iter().any(|green| !green) && !claimed.contains(*key))
        .count();
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
    // Every command of the repository's files that ran and failed,
    // with its words -- verify and ci alike, in the order the court
    // met them (wave 0070).
    let mut red_commands: Vec<RedCommand> = Vec::new();
    let mut verify_blockers = 0usize;
    // Counted apart from the broken ones: a promise whose proof did
    // not run is not a broken promise, and one word for both would
    // say of a distrusted command that it failed (wave 0055).
    let mut verify_unrun = 0usize;
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
            // A proof that did not run is not a proof, and this
            // court says so in its footer too (wave 0055): it used to
            // print "no blockers" right under the row naming the
            // command that did not run (final review 2026-09-06, bugs
            // R-13). The VERDICT of distrust stays check's, as wave
            // 0010 promised -- this court does not duplicate the
            // finding, and its exit is unchanged.
            verify_lines.push(ta(
                "close-verify-untrusted",
                targs!("command" => command.clone(), "contract" => contract.slug.clone()),
            ));
            verify_unrun += 1;
            continue;
        }
        match run_command(root, command) {
            Ok(()) => verify_lines.push(ta(
                "close-verify-passed",
                targs!("command" => command.clone(), "contract" => contract.slug.clone()),
            )),
            Err(words) => {
                red_commands.push(RedCommand {
                    command: command.clone(),
                    words: unmarked(&words),
                });
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

    // The form court of §7.6 (wave 0050), asked here of the same
    // contracts `keel check` asks it of -- the plan window of §6.5
    // and a plan branch stay outside it, in check's own words. It
    // lived in check alone, so a contract promising a unit the code
    // does not hold was red there and "closed", exit 0, here (global
    // review 2026-09-06, methodology cut R-2). Every finding is a
    // blocker by name: a form the code does not hold does not merge.
    let mut form_blockers = 0usize;
    if scope::current_branch(root).is_some_and(|b| b.starts_with("plan/")) {
        report.push_str(&t("check-holding-plan"));
        report.push('\n');
    } else {
        let window = holding::plan_window(root, &scan.waves, &found, &scan.contracts);
        let judged: Vec<docs::Contract> = scan
            .contracts
            .iter()
            .filter(|c| !window.iter().any(|(slug, _)| slug == &c.slug))
            .cloned()
            .collect();
        let findings = holding::court(root, &config, &judged);
        report.push_str(&ta(
            "close-form-judged",
            targs!("count" => findings.len() as u64),
        ));
        report.push('\n');
        for (place, reason, instead) in findings {
            report.push_str(&format!(
                "  {place}: {reason}\n           {}: {instead}\n",
                t("word-instead")
            ));
            form_blockers += 1;
        }
        for (contract, wave) in &window {
            report.push_str("  ");
            report.push_str(&ta(
                "check-holding-window",
                targs!("contract" => contract.clone(), "wave" => wave.clone()),
            ));
            report.push('\n');
        }
    }

    // The project's own ci (wave 0019, the first field's gift)
    // through the same §7.16 gate as verify: a trusted command runs
    // exactly once as the project's own merge gate -- never as a
    // contract's proof; untrusted, none, undecided and absent are
    // each a word, never a run. "Trusted" means "runs".
    let mut ci_blocker = 0usize;
    let mut ci_unrun = false;
    let ci_line = match config.ci.as_deref() {
        None => t("close-ci-absent"),
        Some("") => t("close-ci-undecided"),
        Some("none") => t("close-ci-none"),
        Some(command) if !crate::trust::trusted(&config, command) => {
            // The exit stays check's: wave 0010 promised that
            // distrust is check's verdict and this court does not
            // duplicate the finding, and that promise is alive. What
            // wave 0055 ends is the CONTRADICTION -- the footer said
            // "no blockers" under a row saying a proof did not run
            // (final review 2026-09-06, bugs R-13).
            ci_unrun = true;
            ta(
                "close-ci-untrusted",
                targs!("command" => command.to_string()),
            )
        }
        Some(command) => match run_command(root, command) {
            Ok(()) => ta("close-ci-passed", targs!("command" => command.to_string())),
            Err(words) => {
                ci_blocker = 1;
                // The same words go to the JSON package as a field of
                // their own (wave 0070): a harness that wants the
                // reason must not parse prose to find it -- that is
                // exactly what the package exists to spare it.
                red_commands.push(RedCommand {
                    command: command.to_string(),
                    words: unmarked(&words),
                });
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
    let mut own_awaiting = false;
    let mut own_light = false;
    for wave in &scan.waves {
        let state = wave_state(root, wave, &found, &legal, Some(&battery))?;
        let own = branch.as_deref() == Some(wave.slug.as_str());
        if own && docs::weight(wave) == docs::Weight::Light {
            own_light = true;
        }
        // A wave whose own branch this is does not read as closed
        // while the court is watching its battery fail. Waves closed
        // in earlier generations keep their verdict: their promises
        // were proven at their time, and today's red is not their
        // lack -- but it IS this one's, whichever promise the red
        // test belongs to (wave 0043).
        // A wave that was CANCELLED has nothing to prove, and its
        // reason is the whole point of the line (review 0043 R-6);
        // a wave still in work names its own lacks, and a plan has
        // no tests yet. None of those is "closed", so none of them
        // is what this rule is for.
        if own
            && red_tests > 0
            && !matches!(
                state,
                State::Progress(_) | State::Plan | State::Cancelled(_)
            )
        {
            report.push_str(&ta(
                "close-held-by-red",
                targs!("wave" => wave.slug.clone(), "count" => red_tests as u64),
            ));
            report.push('\n');
            continue;
        }
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
            State::AwaitingMerge(seen) => {
                if own {
                    own_awaiting = true;
                }
                let key = if seen {
                    "close-awaiting-merge"
                } else {
                    "close-awaiting-merge-unseen"
                };
                report.push_str(&ta(key, targs!("wave" => wave.slug.clone())));
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
    // The court watched these fail with its own eyes, three runs
    // each (§7.13), and named them above -- and then closed the wave
    // anyway, because blockers were counted only from the promises
    // of the branch's own wave. A red test nobody claims never became
    // a lack, so a court that SAW red left with 0. Measured in rust,
    // ruby and elixir alike: the hole was in this court, not in an
    // adapter (wave 0043). A court that did not run says so and a
    // person knows they do not know; a court that saw red and left
    // green passes itself off as read.
    if red_tests > 0 {
        report.push_str(&ta(
            "close-red-blockers",
            targs!("count" => red_tests as u64),
        ));
        report.push('\n');
    }
    if verify_blockers > 0 {
        report.push_str(&ta(
            "close-verify-blockers",
            targs!("count" => verify_blockers as u64),
        ));
        report.push('\n');
    }
    if verify_unrun > 0 {
        report.push_str(&ta(
            "close-verify-unproven",
            targs!("count" => verify_unrun as u64),
        ));
        report.push('\n');
    }
    // The project's own gate, when trust does not let it run: the
    // first cut of this wave took the "no blockers" footer away and
    // put nothing in its place, so a person running only this court
    // saw a closed wave and no summary line at all (review 0055 R-6).
    if ci_unrun {
        report.push_str(&t("close-ci-unproven"));
        report.push('\n');
    }
    if form_blockers > 0 {
        report.push_str(&ta(
            "close-form-blockers",
            targs!("count" => form_blockers as u64),
        ));
        report.push('\n');
    }
    if ci_blocker > 0 {
        report.push_str(&t("close-ci-blocker"));
        report.push('\n');
    }
    if blockers > 0 {
        // The blockers are named by the wave's own weight (global
        // review 2026-09-06, methodology R-16: "a full wave" was said
        // over a light one waiting for its report).
        let key = if own_light {
            "close-blockers-light"
        } else {
            "close-blockers"
        };
        report.push_str(&ta(
            key,
            targs!("wave" => branch.unwrap_or_default(), "count" => blockers as u64),
        ));
        report.push('\n');
    } else if own_plan {
        // The honest plan footer stays honest under a red tree
        // (review 0043 R-7): a plan PR merges as a plan (§6.6), and
        // sec. 8.3's own words are that a gate always shut stops
        // being read. The reds are counted above and carry the exit
        // code themselves -- this line says what KIND of PR this is,
        // not that all is well.
        // The honest footer for the plan branch (review R-2): a plan
        // PR merges as a plan (§6.6), and the old words would lie.
        report.push_str(&ta(
            "close-plan-own",
            targs!("wave" => branch.unwrap_or_default()),
        ));
        report.push('\n');
    } else if verify_blockers == 0
        && verify_unrun == 0
        && !ci_unrun
        && form_blockers == 0
        && ci_blocker == 0
        && red_tests == 0
    {
        // The branch's own wave may be named and unblocked at once:
        // a light wave waiting for its merge (review 0052 R-13 -- the
        // old word called such a branch "not named as an unclosed
        // wave").
        if own_awaiting {
            report.push_str(&ta(
                "close-no-blockers-awaiting",
                targs!("wave" => branch.unwrap_or_default()),
            ));
        } else {
            report.push_str(&t("close-no-blockers"));
        }
        report.push('\n');
    }
    // And what the price actually came to (review 0031 R-6: the
    // scenario promised this sentence and the first cut of the work
    // simply did not carry it).
    if let adapter::BuildDir::At(target) = &target {
        report.push_str(&ta(
            "close-price-paid",
            targs!(
                "target" => target.display().to_string(),
                "size" => tenths_of_gigabyte(directory_bytes(target))
            ),
        ));
        report.push('\n');
    }
    // The last ceiling, over everything the commands said together.
    // Per-command windows do not bound a project with twenty red
    // contracts -- review 0070 R-3 measured 1691 lines from twenty --
    // and `--json` puts this whole report into ONE field.
    let report = capped_report(report);
    Ok((
        report,
        blockers + verify_blockers + form_blockers + ci_blocker + red_tests,
        red_commands,
    ))
}

/// One command of the repository's files that ran and failed, with
/// the words it left, so a harness reads the reason from a field
/// instead of parsing a report (wave 0070, issue #45).
///
/// The same WINDOW as the prose, but not the same text: the report's
/// ceiling shares four hundred lines between every red command, and
/// this field does not -- each entry keeps its window whole. On a
/// hundred commands the prose carries six lines each and the field
/// eighty. The ceiling is for a person and for one `report` string;
/// this is for a machine, and a machine wants them all.
#[derive(Debug, Clone)]
pub struct RedCommand {
    pub command: String,
    pub words: String,
}

/// How many lines of a red command's output the report carries from
/// each end. Issue #45 came from a project whose gate is ten steps
/// (setup, rubocop, erb_lint, reek, flay, flog, brakeman, gitleaks,
/// tests, seeds): the failing step speaks in the middle, and the last
/// line came from tailwind. A window from both ends catches the step
/// that broke and the tail that ended it; the middle, where a
/// thousand green lines live, is what gets cut.
///
/// The number is per COMMAND, not per report: this project has three
/// `verify` commands plus its `ci`, and a budget shared between them
/// would let the last one eat what the first needed.
const WINDOW: usize = 40;

/// Runs one trusted command from the repository's files through
/// `sh -c` at the root -- the verify of a contract, the project's
/// ci: success is silence; failure carries the command's OUTPUT, in
/// a window from both ends, stderr then stdout, with a line saying
/// how much was cut -- else the keyed word (0010 review R-5) when
/// there was nothing to carry. A command that does not start fails
/// with the system's words.
///
/// Wave 0070, from issue #45: this held `child.output()` -- the whole
/// of both streams -- and kept ONE line, the last non-empty one. A
/// ten-step gate could not be diagnosed from its own log, and the
/// advice to run the command again is addressed to nobody on a
/// runner, where the environment that produced the failure is gone
/// when the job ends. The output was always in hand; it was thrown
/// away.
///
/// The text carried is the command's own, verbatim within the line:
/// its language, its encoding, its words, its INDENT. What goes is
/// the ANSI sequences and the bare control bytes -- every one except
/// the tab, because in a diagnostic the indent is the meaning and a
/// dropped tab welds columns together (review 0070 R-1). A line over
/// `LINE_CAP` is cut and says so; `visible`, which trimmed and ate
/// tabs, is gone with this wave.
///
/// Nothing is masked, and the verdict says so -- a masker weaker than
/// gitleaks would give a false calm, which is worse than an honest
/// warning.
fn run_command(root: &Path, command: &str) -> Result<(), String> {
    let mut child = std::process::Command::new("sh");
    child.arg("-c").arg(command).current_dir(root);
    // As clean as the battery (wave 0050): the hook's git variables
    // and the inherited cargo target are forgotten here too, since a
    // verify that runs the same crate's tests under a shared cache
    // gets the shifted verdicts the battery refuses, and one that
    // asks git for its repository sees the hook's (global review
    // 2026-09-06, bugs cut R-18).
    scope::forget_the_hook(&mut child);
    child
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR");
    let out = child.output().map_err(|e| e.to_string())?;
    if out.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // A stream that was not valid UTF-8 comes back with U+FFFD in
    // place of the bytes that were not. Under a frame that says
    // "verbatim" that substitution must not be silent (review 0070
    // R-4): the reader is looking at a character the command never
    // printed, and only the tool knows it.
    // Питаємо декодер, а не довжину: обрізана чотирибайтова
    // послідовність дає рівно один U+FFFD на три байти, довжина не
    // міняється, і евристика на len() сліпа саме там, де підміна
    // найтиповіша (рецензія 0070, друге коло).
    let mangled =
        std::str::from_utf8(&out.stderr).is_err() || std::str::from_utf8(&out.stdout).is_err();
    let mut said = String::new();
    for (stream, text) in [("stderr", &stderr), ("stdout", &stdout)] {
        let window = window_of(text);
        if window.is_empty() {
            continue;
        }
        said.push(FRAME_MARK);
        said.push_str(&ta("close-said-stream", targs!("stream" => stream)));
        said.push('\n');
        said.push_str(&window);
    }
    if said.is_empty() {
        return Err(t("close-verify-no-words"));
    }
    if mangled {
        said.push(FRAME_MARK);
        said.push_str("    ");
        said.push_str(&t("close-said-not-utf8"));
        said.push('\n');
    }
    Err(said)
}

/// How many characters of one quoted line the report carries. A
/// window that counts LINES and not their length is no window at all:
/// a single `print('x' * 3_000_000)` walked through the first cut of
/// this whole, and so did a progress bar, whose twenty thousand
/// frames are separated by `\r` and are therefore ONE line to
/// `lines()` (review 0070 R-2, measured: 289 665 bytes).
const LINE_CAP: usize = 400;

/// How many lines the whole report carries from all commands
/// together. Per-command windows do not bound a project with twenty
/// red contracts (review 0070 R-3, measured: 1691 lines), and the
/// JSON package puts the whole report in ONE field.
const REPORT_CAP: usize = 400;

/// The fewest lines any one command keeps when the report is over
/// its ceiling. A budget spent first-come leaves the last commands
/// with nothing -- review 0070 measured fifteen of twenty getting not
/// one word, which is exactly what `WINDOW` three constants above
/// forbids for the same reason.
const FLOOR_PER_COMMAND: usize = 6;

/// A byte no command's output can carry into the report: `quoted`
/// drops every control character except the tab, so this mark cannot
/// be forged from outside. It rides on each quoted line and is
/// stripped on the way out.
///
/// The first cut of this ceiling recognised a quoted line by its four
/// spaces of indent -- and the court's OWN verdicts are indented too.
/// Review 0070 measured the result: the list of a wave's lacks
/// vanished under its own heading, with nothing saying it had been
/// cut. A ceiling that eats the verdict is worse than no ceiling.
const QUOTE_MARK: char = '\u{1}';

/// The same words without the marks: what goes into the JSON package
/// and into any other reader that is not the report. The marks are
/// the report's own bookkeeping, and a package carrying them would
/// break the promise that the field holds the same text as the prose
/// (review 0070, third round: six U+0001 per red command).
fn unmarked(words: &str) -> String {
    words
        .chars()
        .filter(|c| *c != QUOTE_MARK && *c != FRAME_MARK)
        .collect()
}

/// The mark on the court's OWN lines inside a block -- the frame that
/// says whose stream this is, how much is shown, how much was cut,
/// whether the bytes were UTF-8. They belong to the block, so the
/// ceiling must see them to know where a block ends -- and they are
/// the court speaking, so the ceiling must never eat them.
///
/// The first cut of the ceiling marked them like the quote itself,
/// and every block lost its own "N shown, M cut" line: the window
/// stopped saying it was a window in exactly the report where the
/// ceiling made it one.
const FRAME_MARK: char = '\u{2}';

/// The report with its quoted lines bounded, and bounded FAIRLY: each
/// command's block keeps at least `FLOOR_PER_COMMAND` quoted lines,
/// and the rest of the ceiling is shared evenly.
///
/// Two things are never cut. The court's verdicts outside a block
/// carry no mark at all. The court's frame INSIDE a block -- whose
/// stream, how much shown, how much cut, whether the bytes were UTF-8
/// -- carries `FRAME_MARK`, so the ceiling can see where the block
/// ends without eating the only line that says the block was cut.
///
/// `REPORT_CAP` is therefore a target, not a hard bound: with more
/// red commands than `REPORT_CAP / FLOOR_PER_COMMAND`, the floor
/// wins and the report grows -- 101 red commands keep 606 lines, not
/// 400. That is deliberate: a command with no words at all is the
/// defect this wave exists to remove, and a ceiling that restores it
/// for the last commands would restore it in the worst place.
fn capped_report(report: String) -> String {
    let marked = |l: &str| l.starts_with(QUOTE_MARK) || l.starts_with(FRAME_MARK);
    let quoted = report.lines().filter(|l| l.starts_with(QUOTE_MARK)).count();
    if quoted <= REPORT_CAP {
        return unmarked(&report);
    }
    // How many blocks there are: a block is a run of quoted lines,
    // and one command may have two (stderr and stdout).
    let mut blocks = 0usize;
    let mut inside = false;
    for line in report.lines() {
        let here = marked(line);
        if here && !inside {
            blocks += 1;
        }
        inside = here;
    }
    let allowance = (REPORT_CAP / blocks.max(1)).max(FLOOR_PER_COMMAND);

    // Each block is walked twice: once to know its length, once to
    // print its head and tail. Cheap -- the report is already in
    // memory and already bounded per command.
    let lines: Vec<&str> = report.lines().collect();
    let mut out = String::with_capacity(report.len());
    let mut i = 0usize;
    while i < lines.len() {
        if !marked(lines[i]) {
            out.push_str(lines[i]);
            out.push('\n');
            i += 1;
            continue;
        }
        let start = i;
        while i < lines.len() && marked(lines[i]) {
            i += 1;
        }
        let block = &lines[start..i];
        // The court's own frame lines never count against the budget
        // and are never dropped: it is the QUOTE they bound that the
        // ceiling is for.
        let quotes: Vec<&&str> = block.iter().filter(|l| l.starts_with(QUOTE_MARK)).collect();
        if quotes.len() <= allowance {
            for line in block {
                out.push_str(
                    line.trim_start_matches(QUOTE_MARK)
                        .trim_start_matches(FRAME_MARK),
                );
                out.push('\n');
            }
            continue;
        }
        let head = allowance / 2;
        let tail = allowance - head;
        let cut = quotes.len() - head - tail;
        let mut seen = 0usize;
        let mut said = false;
        for line in block {
            if line.starts_with(FRAME_MARK) {
                out.push_str(line.trim_start_matches(FRAME_MARK));
                out.push('\n');
                continue;
            }
            seen += 1;
            if seen > head && seen <= head + cut {
                if !said {
                    out.push_str("    ");
                    out.push_str(&ta("close-said-report-cut", targs!("count" => cut as u64)));
                    out.push('\n');
                    said = true;
                }
                continue;
            }
            out.push_str(line.trim_start_matches(QUOTE_MARK));
            out.push('\n');
        }
    }
    out
}

/// One line of a command's output as the report may carry it: the
/// colours and the bare control bytes go, because a verdict quoting
/// an escape sequence says nothing -- and NOTHING ELSE goes.
///
/// This is deliberately not `visible`, which also trims and drops
/// tabs. Review 0070 R-1 measured what that costs under a frame that
/// says "verbatim": rubocop's caret stopped pointing at the offence
/// (`^^^^` moved four columns left), rustc's moved two, and
/// `col1\tcol2` became `col1col2`. In diagnostics the indent IS the
/// meaning, and a frame that promises the command's own words must
/// keep them.
///
/// A line longer than `LINE_CAP` is cut, and the cut says so in the
/// line itself -- silence there would be the same lie one size up.
fn quoted(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            if chars.peek() == Some(&'[') {
                chars.next();
                for c in chars.by_ref() {
                    if ('@'..='~').contains(&c) {
                        break;
                    }
                }
            } else {
                chars.next();
            }
            continue;
        }
        // The tab stays: it is the only control character that
        // carries meaning in a diagnostic, and dropping it welds
        // columns together.
        if c.is_control() && c != '\t' {
            continue;
        }
        out.push(c);
    }
    if out.chars().count() > LINE_CAP {
        let kept: String = out.chars().take(LINE_CAP).collect();
        let cut = out.chars().count() - LINE_CAP;
        return format!(
            "{kept}{}",
            ta("close-said-long", targs!("count" => cut as u64))
        );
    }
    out
}

/// The lines of one stream as the report carries them: `WINDOW` from
/// each end, with a line naming how many are shown and how many were
/// cut. Empty when the stream said nothing that survives `quoted` --
/// a stream of pure colour is a stream of no words, and the caller
/// skips it rather than printing a heading over nothing.
///
/// Only the lines that get printed are read through `quoted`: the
/// first cut mapped every line of a million-line output and paid 5.6x
/// the instructions and 2.3x the memory for six kilobytes of print
/// (review 0070 R-8, measured).
fn window_of(text: &str) -> String {
    let total = text.lines().count();
    if total == 0 || text.lines().all(|l| quoted(l).trim().is_empty()) {
        return String::new();
    }
    fn put(out: &mut String, line: &str) {
        out.push(QUOTE_MARK);
        out.push_str("    ");
        out.push_str(&quoted(line));
        out.push('\n');
    }
    let mut out = String::new();
    if total <= WINDOW * 2 + 1 {
        for line in text.lines() {
            put(&mut out, line);
        }
        out.push(FRAME_MARK);
        out.push_str("    ");
        out.push_str(&ta("close-said-shown", targs!("count" => total as u64)));
        out.push('\n');
        return out;
    }
    for line in text.lines().take(WINDOW) {
        put(&mut out, line);
    }
    out.push(FRAME_MARK);
    out.push_str("    ");
    out.push_str(&ta(
        "close-said-cut",
        targs!("shown" => (WINDOW * 2) as u64, "count" => (total - WINDOW * 2) as u64),
    ));
    out.push('\n');
    for line in text.lines().skip(total - WINDOW) {
        put(&mut out, line);
    }
    out
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
        // A cancelled wave is NOT closed: its promises were never
        // kept, so a wave depending on it is not ready either
        // (review 0037 R-19). It is simply outside judgement, which
        // `judge` says in its own words.
        State::Closed { .. } | State::ClosedLight
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
    // The weight is not asked here any more (review 0037 R-22): §9.9
    // now wants a reviewer for every wave, so the second human look
    // no longer hangs on this question, and §6.8's own line says the
    // weight decides the number of pull requests and nothing else.
    // What this asks is its own question: is there a promise to
    // prove at all?
    wave.scenarios.is_empty()
}

/// The review report of a wave as HISTORY carries it (§9.9; wave
/// 0055).
///
/// `None` where no commit carries the file. The final review of
/// 2026-09-06 (bugs R-7) measured the closing court reading the
/// working tree: a report written and never committed made a wave
/// "closed", and the merge that followed carried no record at all --
/// the very thing §9.9 puts into history. Where there is no history
/// to ask -- no git, or a repository whose HEAD is not born yet --
/// the file on disk is all there is, and it is read.
pub(crate) fn report_text(root: &Path, slug: &str) -> Option<String> {
    let relative = format!("keel/reviews/{slug}.md");
    // One git call on the road that answers (review 0055 R-13: this
    // court asks it once per wave, and asking twice doubled the
    // processes a `keel status` over fifty-five waves spends).
    //
    // `HEAD:<path>` is read from the top of the WORK TREE, not from
    // the directory git was pointed at: a keel project living in a
    // subdirectory of a bigger repository had its report in history
    // and this court could not see it (review 0055 R-3). `HEAD:./…`
    // is git's own spelling for "relative to here".
    let shown = scope::git_at(root)
        .arg("show")
        .arg(format!("HEAD:./{relative}"))
        .output();
    match shown {
        Ok(out) if out.status.success() => {
            return Some(String::from_utf8_lossy(&out.stdout).into_owned());
        }
        // git ran and did not find it: absent from history -- unless
        // there is no history to ask, and then the file on disk is
        // all there is.
        Ok(_) => {
            let born = scope::git_at(root)
                .args(["rev-parse", "--verify", "--quiet", "HEAD"])
                .output()
                .is_ok_and(|out| out.status.success());
            if born {
                return None;
            }
        }
        // git did not run at all.
        Err(_) => {}
    }
    std::fs::read_to_string(root.join(&relative)).ok()
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
        // An empty file is no review here either -- one word about
        // one state in every court (review 0037 R-2 for the full
        // wave; global review 2026-09-06, methodology R-13; wave 0053),
        // and a file no commit carries is not one at all (wave 0055).
        match report_text(root, &wave.slug) {
            None => return Ok(State::Progress(vec![t("close-lack-review")])),
            Some(text) if text.split_whitespace().next().is_none() => {
                return Ok(State::Progress(vec![t("close-lack-review-empty")]));
            }
            Some(_) => {}
        }
        // "Closed by the fact of merge" only where the fact
        // stands: the wave file in main. The first reading
        // called a chore wave closed the moment its report lay
        // beside it, on a branch main had never seen (global
        // review 2026-09-06, methodology R-5).
        // On the wave's OWN branch the fact is its work in the
        // trunk, not its file: "its file and its work arrive in
        // main by one PR" (§6.5; review 0052 R-6).
        let own = scope::current_branch(root).as_deref() == Some(wave.slug.as_str());
        let fact = if own {
            scope::work_in_trunk(root)
        } else {
            scope::stands_in_main(root, &format!("keel/waves/{}.md", wave.slug))
        };
        return Ok(match fact {
            Some(true) => State::ClosedLight,
            Some(false) => State::AwaitingMerge(true),
            None => State::AwaitingMerge(false),
        });
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
                // The same key the adapter wrote, by the same hand
                // (review 0045 R-1): a stem alone let two files of one
                // name share an entry.
                let stem = adapter::battery_key(root, &tag.file);
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

    // The §9.9 gate held by mechanics: every wave carries its review
    // (the operator's decision of 2026-09-04). An EMPTY file is not
    // one -- review 0037 R-2 measured `: > file` passing the gate,
    // with the verdict then claiming "the review report is beside
    // it", which is more than the machine ever looked at.
    match report_text(root, &wave.slug) {
        None => lacks.push(t("close-lack-review")),
        Some(text) if text.split_whitespace().next().is_none() => {
            lacks.push(t("close-lack-review-empty"))
        }
        Some(_) => {}
    }

    if lacks.is_empty() {
        Ok(State::Closed { refs_unjudged })
    } else {
        Ok(State::Progress(lacks))
    }
}
