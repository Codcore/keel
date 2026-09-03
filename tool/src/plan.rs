//! Rung 12, the skeleton hand (contract tool-plan; §10.2, §8.2,
//! §8.5): the plan is written by hand -- the tool hands the form,
//! the reminders and the number court, never the content. Both
//! skeletons are born deliberately red: check leads the author
//! until the form becomes a promise, and the unfinished never
//! merges by accident.

use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::Path;
use std::process::Command;

/// The `keel plan <slug>` birth: a full-form wave skeleton, the
/// §8.2 branches, the §10.2 author's pass -- and the §8.5/§8.8
/// number court over disk waves and every git branch first.
pub fn wave(root: &Path, slug: &str) -> Result<String, Refusal> {
    let waves = root.join("keel/waves");
    let file = waves.join(format!("{slug}.md"));
    if !crate::docs::slug_ok(slug) {
        return Err(refuse_slug(&file, slug));
    }
    // The number is the digits before the first hyphen (§8.5). A
    // number that is there yet does not fit the counting gets its
    // true reason (review 0013 R-3), never "starts with no number".
    let head = slug.split('-').next().unwrap_or("");
    if head.is_empty() || !head.chars().all(|c| c.is_ascii_digit()) {
        return Err(Refusal {
            file: file.clone(),
            reason: ta("plan-no-number", targs!("slug" => slug.to_string())),
            instead: t("plan-no-number-instead"),
        });
    }
    let Ok(number) = head.parse::<u64>() else {
        return Err(Refusal {
            file: file.clone(),
            reason: ta("plan-number-huge", targs!("head" => head.to_string())),
            instead: t("plan-number-huge-instead"),
        });
    };
    keel_dirs(root, &waves, &t("what-waves"))?;
    if file.is_file() {
        return Err(Refusal {
            file,
            reason: ta("plan-exists", targs!("slug" => slug.to_string())),
            instead: t("plan-exists-instead"),
        });
    }
    // The number court reads numbers off the file names of
    // keel/waves/ (§8.5: the file's name testifies by itself) and
    // off the branches: a document broken inside hides no number,
    // and one deliberately red skeleton does not block the next
    // birth.
    let mut taken: Vec<u64> = wave_file_numbers(&waves);
    let branches_read = branch_numbers(root, slug, &mut taken);
    if taken.contains(&number) {
        let next = taken.iter().max().unwrap_or(&number) + 1;
        // The instead tells the truth about what was searched
        // (review 0013 R-1): "every branch" is said only where git
        // actually answered.
        let instead = if branches_read {
            ta(
                "plan-number-taken-instead",
                targs!("next" => format!("{next:04}")),
            )
        } else {
            ta(
                "plan-number-taken-instead-disk",
                targs!("next" => format!("{next:04}")),
            )
        };
        return Err(Refusal {
            file,
            reason: ta(
                "plan-number-taken",
                targs!("number" => format!("{number:04}")),
            ),
            instead,
        });
    }

    let skeleton = format!(
        "---\n# {}\n# depends_on: []\nscenarios:\n  first-promise: {{covers: []}}\ntransforms:\n  first-work:\n    implements: [first-promise]\n    files:\n      - path/named-by-hand\n---\n\n## Why\n\n{}\n\n## scenario: first-promise\n\n{}\n\n## transform: first-work\n\n{}\n",
        ta("plan-skel-header", targs!("slug" => slug.to_string())),
        t("plan-skel-why"),
        t("plan-skel-scenario"),
        t("plan-skel-transform"),
    );
    write_new(&file, &skeleton)?;

    let shown = file.strip_prefix(root).unwrap_or(&file);
    let mut out = ta(
        "plan-created",
        targs!("file" => shown.display().to_string()),
    );
    out.push('\n');
    out.push_str(&ta("plan-branches", targs!("slug" => slug.to_string())));
    out.push('\n');
    if !branches_read {
        // Where git is silent only the disk judged -- said aloud,
        // never a quiet narrowing of the §8.8 search.
        out.push_str(&t("plan-branches-unread"));
        out.push('\n');
    }
    out.push_str(&t("plan-cuts"));
    out.push('\n');
    out.push_str(&t("plan-next"));
    out.push('\n');
    Ok(out)
}

/// The `keel new contract <slug>` birth: the module scaffolding and
/// the commented §2.7-§2.8 vocabulary, deliberately promising
/// nothing yet.
pub fn contract(root: &Path, slug: &str) -> Result<String, Refusal> {
    let contracts = root.join("keel/contracts");
    let file = contracts.join(format!("{slug}.md"));
    if !crate::docs::slug_ok(slug) {
        return Err(refuse_slug(&file, slug));
    }
    keel_dirs(root, &contracts, &t("what-contracts"))?;
    if file.is_file() {
        return Err(Refusal {
            file,
            reason: ta("newc-exists", targs!("slug" => slug.to_string())),
            instead: t("newc-exists-instead"),
        });
    }

    let skeleton = format!(
        "---\n# {}\nmodule: crate::module-named-by-hand\n# exports:\n#   - \"pub fn ...\"\n# verify: \"...\"\n---\n\n{}\n",
        ta("newc-skel-header", targs!("slug" => slug.to_string())),
        t("newc-skel-body"),
    );
    write_new(&file, &skeleton)?;

    let shown = file.strip_prefix(root).unwrap_or(&file);
    let mut out = ta(
        "newc-created",
        targs!("file" => shown.display().to_string()),
    );
    out.push('\n');
    out.push_str(&t("newc-next"));
    out.push('\n');
    Ok(out)
}

/// The birth needs its directories: keel/ and the target directory
/// must exist -- the docs school's own words guide where they do
/// not (a birth never creates the project's structure by stealth).
fn keel_dirs(root: &Path, dir: &Path, what: &str) -> Result<(), Refusal> {
    let keel = root.join("keel");
    if !keel.is_dir() {
        return Err(Refusal {
            file: keel,
            reason: t("docs-keel-missing"),
            instead: t("docs-keel-missing-instead"),
        });
    }
    if !dir.is_dir() {
        return Err(Refusal {
            file: dir.to_path_buf(),
            reason: ta("docs-dir-missing", targs!("what" => what.to_string())),
            instead: t("docs-dir-missing-instead"),
        });
    }
    Ok(())
}

/// The numbers held by the wave files themselves: the file stem's
/// leading digits (§8.5). No header is parsed -- a broken document
/// hides no number.
fn wave_file_numbers(waves: &Path) -> Vec<u64> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(waves) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md")
                && let Some(stem) = path.file_stem()
                && let Some(number) = leading_number(&stem.to_string_lossy())
            {
                out.push(number);
            }
        }
    }
    out
}

fn refuse_slug(file: &Path, slug: &str) -> Refusal {
    Refusal {
        file: file.to_path_buf(),
        reason: ta("plan-bad-slug", targs!("slug" => slug.to_string())),
        instead: t("plan-bad-slug-instead"),
    }
}

/// The wave number: the digits before the first hyphen (§8.5).
fn leading_number(slug: &str) -> Option<u64> {
    let head = slug.split('-').next().unwrap_or("");
    if head.is_empty() || !head.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    head.parse().ok()
}

/// Numbers held by branches, local and remote (§8.8: the free number
/// is searched across all branches, not only main). A branch name's
/// wave is its last path segment -- plan/<wave> and origin/<wave>
/// alike. The wave's OWN branches are its name, never a rival
/// (review 0013 R-2): a branch spelled exactly as the born slug is
/// skipped. Returns whether git answered; false narrows the court
/// to the disk, and the caller says that aloud.
fn branch_numbers(root: &Path, slug: &str, taken: &mut Vec<u64>) -> bool {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads",
            "refs/remotes",
        ])
        .output();
    let Ok(out) = out else {
        return false;
    };
    if !out.status.success() {
        return false;
    }
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let last = line.trim().rsplit('/').next().unwrap_or("");
        if last == slug {
            continue;
        }
        if let Some(number) = leading_number(last) {
            taken.push(number);
        }
    }
    true
}

/// One whole new file or a refusal -- never half of one and never
/// over something that exists: the text lands in a dot-temp next to
/// its place (dot-files are outside every court) and arrives by
/// rename, so a failure mid-write leaves no stub (review 0013 R-4);
/// the refusal speaks of a birth, not of reading. The one home of
/// the write school (wave 0015): init asks here.
pub(crate) fn write_new(file: &Path, text: &str) -> Result<(), Refusal> {
    let refuse = |e: std::io::Error| Refusal {
        file: file.to_path_buf(),
        reason: ta("plan-write-failed", targs!("error" => e.to_string())),
        instead: t("plan-write-failed-instead"),
    };
    let name = file.file_name().map(|n| n.to_string_lossy().into_owned());
    let tmp = file.with_file_name(format!(".{}.tmp", name.unwrap_or_default()));
    std::fs::write(&tmp, text).map_err(refuse)?;
    std::fs::rename(&tmp, file)
        .inspect_err(|_| {
            let _ = std::fs::remove_file(&tmp);
        })
        .map_err(refuse)
}
