//! The judgement of a commit (contract tool-gate; journal A3, §7.12,
//! §8.4): "seen red" becomes a fact the machine checks, not a word
//! the author remembers. The judgement runs where the branch is
//! named as a wave; everywhere else it passes with a word -- the
//! same honesty school as the scope floor.

use crate::adapter::{self, Outcome};
use crate::config;
use crate::docs;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::scope;
use crate::tags;
use crate::targs;
use std::path::Path;

/// Judges the commit message from the file (as the commit-msg hook
/// hands it over) and returns the report plus the exit code, the
/// mode already applied: strict blocks, soft says the same words
/// with code 0, manual says the judgement is off.
pub fn run(root: &Path, message_file: &Path) -> Result<(String, i32), Refusal> {
    let config = config::read(root)?;
    let message = std::fs::read_to_string(message_file).map_err(|e| Refusal {
        file: message_file.to_path_buf(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    let subject = message.lines().next().unwrap_or("").trim().to_string();
    let mutant = mutant_line(&message);

    let mode_line = if config.mode_set {
        ta("gate-mode", targs!("mode" => config.mode.clone()))
    } else {
        t("gate-mode-default")
    };

    if config.mode == "manual" {
        let report = format!("{mode_line}\n{}\n", t("gate-manual"));
        return Ok((report, 0));
    }

    let scan = docs::scan(root)?;
    let Some(slug) = scope::branch_wave(root, &scan.waves) else {
        let branch = scope::current_branch(root).unwrap_or_else(|| "?".to_string());
        let report = format!(
            "{mode_line}\n{}\n",
            ta("gate-not-wave", targs!("branch" => branch))
        );
        return Ok((report, 0));
    };
    let wave = scan.waves.iter().find(|w| w.slug == slug).unwrap();
    // A wave called off is outside judgement, and §6.3-a says EVERY
    // court says so aloud. Review 0037 R-6: this one judged it in
    // silence, so after a cancellation nothing could be committed on
    // its branch at all -- with the hook `keel init` installs by
    // default, the branch was simply frozen.
    if let Some(why) = &wave.cancelled {
        let report = format!(
            "{mode_line}\n{}\n",
            ta(
                "gate-cancelled",
                targs!("wave" => wave.slug.clone(), "why" => why.clone()),
            )
        );
        return Ok((report, 0));
    }

    // The one court that physically runs the toolchain asks the home
    // first (review 0017 R-4): an adapter this release does not
    // serve -- or none at all -- is a word aloud and a pass, never a
    // blind cargo run over a foreign language.
    if !config.rust_adapter() {
        let name = config
            .adapter
            .clone()
            .unwrap_or_else(|| t("gate-adapter-absent-name"));
        let report = format!(
            "{mode_line}\n{}\n",
            ta("gate-adapter-unjudged", targs!("name" => name))
        );
        return Ok((report, 0));
    }

    let verdict = judge(root, wave, &subject, mutant)?;
    let (words, guilty) = match verdict {
        Verdict::Pass(words) => (words, false),
        Verdict::Refuse(words) => (words, true),
    };
    let code = if guilty && config.mode == "strict" {
        1
    } else {
        0
    };
    let soft_line = if guilty && config.mode == "soft" {
        format!("{}\n", t("gate-soft"))
    } else {
        String::new()
    };
    Ok((format!("{mode_line}\n{words}\n{soft_line}"), code))
}

enum Verdict {
    Pass(String),
    Refuse(String),
}

/// The judgement proper, mode-blind: what the message claims against
/// what the tests really do.
fn judge(
    root: &Path,
    wave: &docs::Wave,
    subject: &str,
    mutant: Option<(String, String)>,
) -> Result<Verdict, Refusal> {
    if let Some(rest) = subject.strip_prefix("red: ") {
        let scenario = rest.split_whitespace().next().unwrap_or("");
        return judge_red(root, wave, scenario, mutant);
    }
    if let Some((head, _)) = subject.split_once(':') {
        let head = head.trim();
        if docs::slug_ok(head) {
            if let Some((_, transform)) = wave.transforms.iter().find(|(n, _)| n == head) {
                return judge_work(root, wave, head, transform);
            }
            return Ok(Verdict::Refuse(ta(
                "gate-unknown-slug",
                targs!("slug" => head.to_string(), "wave" => wave.slug.clone()),
            )));
        }
        // The capitalized twin of a birth or a transform -- the
        // likeliest field typo -- does not walk past as "outside the
        // judgement" (review R-3).
        let lower = head.to_lowercase();
        if docs::slug_ok(&lower)
            && (lower == "red" || wave.transforms.iter().any(|(n, _)| *n == lower))
        {
            return Ok(Verdict::Refuse(ta(
                "gate-case",
                targs!("head" => head.to_string()),
            )));
        }
    }
    Ok(Verdict::Pass(t("gate-outside")))
}

/// The named exception of §6.3 (the operator's decision of
/// 2026-09-04): a court over our own battery or tooling cannot be
/// seen failing without breaking the thing it guards, so it MAY be
/// born green -- if the commit records the mutant it was proven with.
/// The guarantee survives: proving that the test can fail is still
/// required, only the proof moves from the hook's eye into the
/// message, where a reviewer meets it.
///
/// What counts is a line `mutant: <what was broken> -> <how the probe
/// named it>`, with words on both sides of the arrow: an exception
/// that costs nothing is not an exception, it is a hole.
pub fn mutant_line(message: &str) -> Option<(String, String)> {
    for line in message.lines() {
        let Some(rest) = line.trim().strip_prefix("mutant:") else {
            continue;
        };
        // Both arrows: the tool prints "→" in its own verdict, so
        // refusing it while showing it was a trap of our own making
        // (review 0037 R-20).
        let Some((broke, named)) = rest.split_once("->").or_else(|| rest.split_once('→')) else {
            continue;
        };
        let (broke, named) = (broke.trim(), named.trim());
        if !broke.is_empty() && !named.is_empty() {
            return Some((broke.to_string(), named.to_string()));
        }
    }
    None
}

fn judge_red(
    root: &Path,
    wave: &docs::Wave,
    scenario: &str,
    mutant: Option<(String, String)>,
) -> Result<Verdict, Refusal> {
    let Some((name, sc)) = wave.scenarios.iter().find(|(n, _)| n == scenario) else {
        return Ok(Verdict::Refuse(ta(
            "gate-red-unknown",
            targs!("slug" => scenario.to_string(), "wave" => wave.slug.clone()),
        )));
    };
    if sc.withdrawn.is_some() {
        return Ok(Verdict::Refuse(ta(
            "gate-red-withdrawn",
            targs!("scenario" => name.clone()),
        )));
    }
    let found = tags::scan(&adapter::test_files(root)?)?;
    let mine: Vec<_> = found.iter().filter(|t| t.scenario == *name).collect();
    match mine.len() {
        0 => Ok(Verdict::Refuse(ta(
            "gate-red-untagged",
            targs!("scenario" => name.clone()),
        ))),
        1 => match adapter::run_test(root, mine[0])? {
            Outcome::Failed => Ok(Verdict::Pass(ta(
                "gate-red-pass",
                targs!("scenario" => name.clone(), "test" => mine[0].test.clone()),
            ))),
            // The §6.3 exception: green, but the mutant is recorded.
            Outcome::Green => Ok(match mutant {
                Some((broke, named)) => Verdict::Pass(ta(
                    "gate-red-mutant",
                    targs!(
                        "scenario" => name.clone(),
                        "test" => mine[0].test.clone(),
                        "broke" => broke,
                        "named" => named
                    ),
                )),
                None => Verdict::Refuse(ta(
                    "gate-red-green",
                    targs!("scenario" => name.clone(), "test" => mine[0].test.clone()),
                )),
            }),
            Outcome::BuildBroken(words) => Ok(Verdict::Refuse(ta(
                "gate-red-broken",
                targs!("scenario" => name.clone(), "words" => words),
            ))),
            Outcome::NotRun => Ok(Verdict::Refuse(ta(
                "gate-red-notrun",
                targs!("scenario" => name.clone(), "test" => mine[0].test.clone()),
            ))),
        },
        n => Ok(Verdict::Refuse(ta(
            "gate-red-many-tags",
            targs!("scenario" => name.clone(), "count" => n as u64),
        ))),
    }
}

fn judge_work(
    root: &Path,
    wave: &docs::Wave,
    slug: &str,
    transform: &docs::Transform,
) -> Result<Verdict, Refusal> {
    let docs::TransformKind::Implements(scenarios) = &transform.kind else {
        // A chore carries no promises to run (§2.11).
        return Ok(Verdict::Pass(t("gate-chore")));
    };
    let wave_path = root.join("keel/waves").join(format!("{}.md", wave.slug));
    let revs = crate::rev::scenario_revs(&wave_path)?;
    let found = tags::scan(&adapter::test_files(root)?)?;

    let mut checked: u64 = 0;
    let mut live: u64 = 0;
    for scenario in scenarios {
        let withdrawn = wave
            .scenarios
            .iter()
            .find(|(n, _)| n == scenario)
            .is_some_and(|(_, sc)| sc.withdrawn.is_some());
        if withdrawn {
            continue;
        }
        live += 1;
        let current = revs
            .iter()
            .find(|(n, _)| n == scenario)
            .map(|(_, r)| r.clone())
            .unwrap_or_default();
        let mine: Vec<_> = found.iter().filter(|t| t.scenario == *scenario).collect();
        if mine.is_empty() {
            return Ok(Verdict::Refuse(ta(
                "gate-work-untagged",
                targs!("transform" => slug.to_string(), "scenario" => scenario.clone()),
            )));
        }
        for tag in mine {
            if !crate::rev::matches(&tag.rev, &current) {
                return Ok(Verdict::Refuse(ta(
                    "gate-work-stale",
                    targs!("transform" => slug.to_string(), "scenario" => scenario.clone(), "recorded" => tag.rev.clone(), "actual" => current.clone()),
                )));
            }
            match adapter::run_test(root, tag)? {
                Outcome::Green => checked += 1,
                Outcome::Failed => {
                    return Ok(Verdict::Refuse(ta(
                        "gate-work-red",
                        targs!("transform" => slug.to_string(), "scenario" => scenario.clone(), "test" => tag.test.clone()),
                    )));
                }
                Outcome::BuildBroken(words) => {
                    return Ok(Verdict::Refuse(ta(
                        "gate-work-broken",
                        targs!("transform" => slug.to_string(), "words" => words),
                    )));
                }
                Outcome::NotRun => {
                    return Ok(Verdict::Refuse(ta(
                        "gate-work-notrun",
                        targs!("transform" => slug.to_string(), "scenario" => scenario.clone(), "test" => tag.test.clone()),
                    )));
                }
            }
        }
    }
    if live == 0 {
        // A vacuum is not a quiet zero: no live scenario was judged
        // (§2.12; review R-10).
        return Ok(Verdict::Pass(ta(
            "gate-work-vacuum",
            targs!("transform" => slug.to_string()),
        )));
    }
    Ok(Verdict::Pass(ta(
        "gate-work-pass",
        targs!("transform" => slug.to_string(), "count" => checked),
    )))
}

/// The commit-msg hook text keel installs -- flat sh, replaceable by
/// rewriting with the same command.
const HOOK: &str = "#!/bin/sh\n# keel gate -- the commit judged by the machine (Keel v2, journal A3).\nexec keel gate \"$1\"\n";

/// Writes `.git/hooks/commit-msg` calling `keel gate`. A repeated
/// call over our own hook is quietly the same file; a foreign hook
/// is never overwritten -- a refusal aloud (§9.7). This is the one
/// thing the module writes.
/// Where git will actually read the commit-msg hook -- the shared
/// directory of a worktree, `core.hooksPath` where someone set it.
/// Review 0037 R-10: the hooks-off road looked at a hard-wired
/// `.git/hooks` and so said "not installed" over a hook standing
/// somewhere else entirely.
pub fn hook_path(root: &Path) -> Option<std::path::PathBuf> {
    let out = crate::scope::git_at(root)
        .args(["rev-parse", "--git-path", "hooks"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let hooks = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!hooks.is_empty()).then(|| root.join(hooks).join("commit-msg"))
}

/// Whether the hook standing there is the one this release writes.
pub fn hook_is_ours(path: &Path) -> bool {
    std::fs::read_to_string(path).is_ok_and(|text| text == HOOK)
}

pub fn install_hook(root: &Path) -> Result<String, Refusal> {
    // --git-path hooks answers with the directory git will actually
    // read -- the shared hooks of the common dir in a worktree, and
    // core.hooksPath where someone set it (review R-1): "installed"
    // about a file git never reads would be the very lie this gate
    // exists to stop.
    let out = crate::scope::git_at(root)
        .args(["rev-parse", "--git-path", "hooks"])
        .output()
        .map_err(|e| Refusal {
            file: root.to_path_buf(),
            reason: ta("scope-git-failed", targs!("error" => e.to_string())),
            instead: t("scope-git-failed-instead"),
        })?;
    if !out.status.success() {
        return Err(Refusal {
            file: root.to_path_buf(),
            reason: ta(
                "scope-git-failed",
                targs!("error" => String::from_utf8_lossy(&out.stderr).trim().to_string()),
            ),
            instead: t("scope-git-failed-instead"),
        });
    }
    let hooks = root.join(String::from_utf8_lossy(&out.stdout).trim());
    let path = hooks.join("commit-msg");

    if path.is_file() {
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        if existing == HOOK {
            return Ok(t("gate-hook-already"));
        }
        return Err(Refusal {
            file: path,
            reason: t("gate-hook-foreign"),
            instead: t("gate-hook-foreign-instead"),
        });
    }

    write_hook(&hooks, &path).map_err(|e| Refusal {
        file: path.clone(),
        reason: ta("docs-unreadable", targs!("error" => e.to_string())),
        instead: t("docs-unreadable-instead"),
    })?;
    let shown = path.strip_prefix(root).unwrap_or(&path);
    Ok(ta(
        "gate-hook-installed",
        targs!("path" => shown.display().to_string()),
    ))
}

fn write_hook(hooks: &Path, path: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(hooks)?;
    std::fs::write(path, HOOK)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok(())
}
