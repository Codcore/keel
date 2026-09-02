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

    let verdict = judge(root, wave, &subject)?;
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
fn judge(root: &Path, wave: &docs::Wave, subject: &str) -> Result<Verdict, Refusal> {
    if let Some(rest) = subject.strip_prefix("red: ") {
        let scenario = rest.split_whitespace().next().unwrap_or("");
        return judge_red(root, wave, scenario);
    }
    if let Some((head, _)) = subject.split_once(':') {
        let head = head.trim();
        if is_slug(head) {
            if let Some((_, transform)) = wave.transforms.iter().find(|(n, _)| n == head) {
                return judge_work(root, wave, head, transform);
            }
            return Ok(Verdict::Refuse(ta(
                "gate-unknown-slug",
                targs!("slug" => head.to_string(), "wave" => wave.slug.clone()),
            )));
        }
    }
    Ok(Verdict::Pass(t("gate-outside")))
}

fn judge_red(root: &Path, wave: &docs::Wave, scenario: &str) -> Result<Verdict, Refusal> {
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
            Outcome::Green => Ok(Verdict::Refuse(ta(
                "gate-red-green",
                targs!("scenario" => name.clone(), "test" => mine[0].test.clone()),
            ))),
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
    for scenario in scenarios {
        let withdrawn = wave
            .scenarios
            .iter()
            .find(|(n, _)| n == scenario)
            .is_some_and(|(_, sc)| sc.withdrawn.is_some());
        if withdrawn {
            continue;
        }
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
    Ok(Verdict::Pass(ta(
        "gate-work-pass",
        targs!("transform" => slug.to_string(), "count" => checked),
    )))
}

/// The slug shape of §1.2: lowercase latin, digits, hyphens.
fn is_slug(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// The commit-msg hook text keel installs -- flat sh, replaceable by
/// rewriting with the same command.
const HOOK: &str = "#!/bin/sh\n# keel gate -- the commit judged by the machine (Keel v2, journal A3).\nexec keel gate \"$1\"\n";

/// Writes `.git/hooks/commit-msg` calling `keel gate`. A repeated
/// call over our own hook is quietly the same file; a foreign hook
/// is never overwritten -- a refusal aloud (§9.7). This is the one
/// thing the module writes.
pub fn install_hook(root: &Path) -> Result<String, Refusal> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--git-dir"])
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
    let git_dir = root.join(String::from_utf8_lossy(&out.stdout).trim());
    let hooks = git_dir.join("hooks");
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
