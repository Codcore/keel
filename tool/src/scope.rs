//! Chapter 4: scope (contract tool-scope). Files are named before
//! the work -- and the branch is judged against the names, both ways
//! (§4.4); git is asked as a command of the system, and its refusal
//! is a refusal aloud, never silence. The module writes nothing.

use crate::docs::{ScopeLine, Wave};
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::collections::BTreeSet;
use std::path::Path;
use std::process::Command;

/// The one hand by which this tool calls git (wave 0021): a plain
/// `git -C <root>`, deaf to the repository the environment names.
/// A git hook hands its children GIT_DIR, GIT_WORK_TREE and their
/// kin -- and those outrank `-C`, so a court that inherited them
/// judged the repository that spawned it instead of the project it
/// was given. The 0020 review measured the price: check reported a
/// stranger's findings, review refused, gate lost its subject, and
/// close GREENED on an unproven wave. `git -c` travels the same way
/// (GIT_CONFIG_PARAMETERS), so it goes too. What stays is what a
/// person or a CI chose on purpose: GIT_CONFIG_GLOBAL,
/// GIT_CONFIG_SYSTEM, GIT_AUTHOR_* and the rest.
pub fn git_at(root: &Path) -> Command {
    let mut command = Command::new("git");
    command.arg("-C").arg(root);
    forget_the_hook(&mut command);
    command
}

/// What a git hook leaves in the environment for its children: the
/// repository it runs for, and the `-c` settings of the git command
/// that fired it. Anyone who spawns a child that may itself talk to
/// git strips these -- the courts through `git_at`, and the battery
/// through the adapter (review 0021 R-3: without it `keel close`
/// handed the whole test suite a stranger's repository, and a byte
/// of a sandbox reached that stranger).
pub fn forget_the_hook(command: &mut Command) {
    for name in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_COMMON_DIR",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        "GIT_PREFIX",
        "GIT_CEILING_DIRECTORIES",
        "GIT_CONFIG_PARAMETERS",
        "GIT_CONFIG_COUNT",
    ] {
        command.env_remove(name);
    }
}

/// The current branch by git's word. None wherever git serves no
/// name for this root: no repository, no git at all, a detached
/// head, or a git tree whose top is not the root itself -- a parent
/// repository's branch would judge paths that never meet the
/// declared names, so it does not get to judge (review R-4). The
/// caller says aloud that scope was not compared.
pub fn current_branch(root: &Path) -> Option<String> {
    // Named by the environment when git will not say it. §4.10 says
    // that where git knows no branch it must be named explicitly, and
    // until wave 0035 there was no way to name it -- so in CI on a
    // `pull_request` event, where actions/checkout leaves a detached
    // HEAD, the scope court was skipped entirely and a file no
    // transform declared went unseen (conformance audit ВАЖКА-5).
    //
    // The environment is asked FIRST: where a person or a workflow
    // has said which branch this is, that is the answer, and git
    // guessing something else would only argue with them.
    if let Ok(named) = std::env::var("KEEL_BRANCH") {
        let named = named.trim().to_string();
        if !named.is_empty() {
            return Some(named);
        }
    }
    let top = git_at(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if !top.status.success() {
        return None;
    }
    let top = std::fs::canonicalize(String::from_utf8_lossy(&top.stdout).trim()).ok()?;
    if top != std::fs::canonicalize(root).ok()? {
        return None;
    }
    let out = git_at(root)
        .args(["branch", "--show-current"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let name = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if name.is_empty() { None } else { Some(name) }
}

/// The wave the current branch is named after (§8.2) -- or None,
/// with no guessing between near-misses.
pub fn branch_wave(root: &Path, waves: &[Wave]) -> Option<String> {
    let branch = current_branch(root)?;
    waves
        .iter()
        .find(|w| w.slug == branch)
        .map(|w| w.slug.clone())
}

/// The comparison base: the merge-base with main -- the local one,
/// or origin/main on a fresh clone that has no local main -- or,
/// where main never existed at all, the first commit of the branch.
/// Returns the sha and whether main gave it, so the report can say
/// what it took (the wave's own caveat).
pub fn compare_base(root: &Path) -> Result<(String, bool), Refusal> {
    for main in ["main", "origin/main"] {
        if let Ok(sha) = git_line(root, &["merge-base", main, "HEAD"]) {
            return Ok((sha, true));
        }
    }
    let roots = git_line(root, &["rev-list", "--max-parents=0", "HEAD"])?;
    let sha = roots.lines().last().unwrap_or("").trim().to_string();
    Ok((sha, false))
}

/// The both-ways comparison (§4.3-§4.6): what the branch changed
/// against the base, next to the union of the wave transforms'
/// files. keel/ is outside the comparison (§4.8); `one new in`
/// counts strictly (§4.1).
pub fn findings(root: &Path, wave: &Wave) -> Result<Vec<(String, String)>, Refusal> {
    let (base, _) = compare_base(root)?;
    // Renames are read as a departure plus an arrival, whatever the
    // host machine's diff.renames fancies: both names meet the
    // declared list, and the verdict is the same on every machine
    // (review R-2).
    let changed_raw = git_line(
        root,
        &["diff", "--name-only", "--no-renames", &base, "HEAD"],
    )?;
    let added_raw = git_line(
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
    let changed: BTreeSet<&str> = changed_raw.lines().map(str::trim).collect();
    let added: BTreeSet<&str> = added_raw.lines().map(str::trim).collect();

    let mut declared: BTreeSet<&str> = BTreeSet::new();
    // Every `one new in` line is a promise of one file: two lines
    // over one directory promise two (§4.1 -- "need two, write two
    // lines"; review R-1).
    let mut dirs: std::collections::BTreeMap<&str, u64> = Default::default();
    for (_, transform) in &wave.transforms {
        for line in &transform.files {
            match line {
                ScopeLine::Path(p) => {
                    declared.insert(p.as_str());
                }
                ScopeLine::OneNewIn(d) => *dirs.entry(d.as_str()).or_insert(0) += 1,
            }
        }
    }

    let mut out = Vec::new();

    // Drift (§4.6): touched yet never declared. A *new* file inside a
    // `one new in` directory is judged by the count below, not here;
    // an old file changed there is drift like anywhere else -- the
    // promise spoke only of one new file.
    for file in &changed {
        if file.is_empty() || file.starts_with("keel/") || declared.contains(file) {
            continue;
        }
        if added.contains(file) && dirs.keys().any(|d| file.starts_with(d)) {
            continue;
        }
        out.push((
            ta("scope-drift", targs!("file" => file.to_string())),
            t("scope-drift-instead"),
        ));
    }

    // The other way (§4.4): declared yet untouched, judged across the
    // whole branch (§4.5), not any single commit. keel/ stays outside
    // the comparison on this side too (§4.8; review R-3).
    for file in &declared {
        if file.starts_with("keel/") {
            continue;
        }
        if !changed.contains(file) {
            out.push((
                ta("scope-untouched", targs!("file" => file.to_string())),
                t("scope-untouched-instead"),
            ));
        }
    }

    // `one new in <dir>/`: as many new files as there are lines --
    // fewer is a finding, more is a finding, the exact count is
    // silence (§4.1). One line keeps the crisp zero/two words.
    for (dir, promised) in &dirs {
        let new_here: Vec<&str> = added
            .iter()
            .copied()
            .filter(|f| !f.is_empty() && f.starts_with(dir))
            .collect();
        let found = new_here.len() as u64;
        if found == *promised {
            continue;
        }
        if *promised == 1 {
            if found == 0 {
                out.push((
                    ta("scope-one-new-none", targs!("dir" => dir.to_string())),
                    t("scope-one-new-none-instead"),
                ));
            } else {
                out.push((
                    ta(
                        "scope-one-new-many",
                        targs!("dir" => dir.to_string(), "files" => new_here.join(", ")),
                    ),
                    t("scope-one-new-many-instead"),
                ));
            }
        } else {
            out.push((
                ta(
                    "scope-one-new-count",
                    targs!("dir" => dir.to_string(), "promised" => *promised, "found" => found),
                ),
                t("scope-one-new-count-instead"),
            ));
        }
    }

    Ok(out)
}

/// One git call, one trimmed answer; a non-zero exit is a refusal
/// that carries git's own words.
pub(crate) fn git_line(root: &Path, args: &[&str]) -> Result<String, Refusal> {
    let refuse = |error: String| Refusal {
        file: root.to_path_buf(),
        reason: ta("scope-git-failed", targs!("error" => error)),
        instead: t("scope-git-failed-instead"),
    };
    // quotePath off: a Ukrainian filename compares as itself, not as
    // git's octal-escaped quotation of it.
    let out = git_at(root)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .output()
        .map_err(|e| refuse(e.to_string()))?;
    if !out.status.success() {
        return Err(refuse(
            String::from_utf8_lossy(&out.stderr).trim().to_string(),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}
