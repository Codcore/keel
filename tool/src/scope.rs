//! Chapter 4: scope (contract tool-scope). Files are named before
//! the work -- and the branch is judged against the names, both ways
//! (§4.4); git is asked as a command of the system, and its refusal
//! is a refusal aloud, never silence. The module writes nothing.

use crate::config::Config;
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
/// What git answers the same way for as long as this process lives,
/// remembered so it is asked once (wave 0061).
///
/// Measured on keel's own tree before the wave: one `keel check`
/// spawned **4604** git processes and needed **387** different
/// answers -- 92% repeats, at ~13 ms a process, which is the whole
/// minute the court took. Three kinds of question make it up, and all
/// three are constant within one run: what a file held AT A COMMIT
/// (a commit does not change), the list of commits that touched a
/// path, and what git says about the tree itself.
///
/// The key carries the ROOT, because one process may judge several
/// trees -- every probe with two sandboxes does -- and an answer of
/// one tree must never be handed to the court of another.
///
/// The memory dies with the process, which is the whole of its
/// lifetime: keel is a CLI, one run per command. Nothing is written
/// to disk, and nothing survives to be stale.
static REMEMBERED: std::sync::OnceLock<
    std::sync::Mutex<std::collections::HashMap<(std::path::PathBuf, String), String>>,
> = std::sync::OnceLock::new();

/// One git call, asked once per (tree, question).
///
/// **Only an ANSWER is remembered, never a failure.** A git that
/// stumbled once -- a busy `index.lock`, a moment of trouble -- would
/// otherwise poison the whole run: "no history here" is exactly the
/// verdict §5.6 softens to, so a cached failure would let a wave pass
/// on a revision nobody could prove, quietly and once per process.
/// A failure that repeats costs a few processes; a failure that is
/// believed costs a court.
/// The questions this memory is allowed to answer, by name.
///
/// Review 0061 R-3 measured what a nameless memory costs: a mutant
/// that widened it to everything `git_line` asks -- `diff`, `status`,
/// `rev-list`, `merge-base` -- passed the whole battery in silence,
/// because the only guard was a ratio of processes to answers. The
/// border of this memory is not "what is cheap to remember" but "what
/// cannot change while one command runs": a commit's content, the
/// list of commits that touched a path, and whether git serves this
/// tree at all. Anything about the WORKING state -- the branch, the
/// diff, the status -- is asked fresh, every time.
fn may_be_remembered(args: &[&str]) -> bool {
    matches!(
        args,
        ["show", _]
            | ["log", "--format=%H", "--", _]
            | ["rev-parse", "--git-dir"]
            | ["rev-parse", "--is-shallow-repository"]
            | ["rev-parse", "--show-toplevel"]
    )
}

pub(crate) fn remembered(root: &Path, args: &[&str]) -> Option<String> {
    // A question outside the named border goes straight to git, and
    // no answer of it is ever kept: the border holds by a court here,
    // not by the attention of whoever adds the next caller.
    if !may_be_remembered(args) {
        return git_at(root)
            .args(args)
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).into_owned());
    }
    let key = (root.to_path_buf(), args.join("\u{1f}"));
    let memory = REMEMBERED.get_or_init(Default::default);
    if let Ok(seen) = memory.lock()
        && let Some(answer) = seen.get(&key)
    {
        return Some(answer.clone());
    }
    let answer = git_at(root)
        .args(args)
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).into_owned())?;
    if let Ok(mut seen) = memory.lock() {
        seen.insert(key, answer.clone());
    }
    Some(answer)
}

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
        // And keel's own word about the branch (review 0040 R-3): a
        // battery run from a court must judge the project, not the
        // branch its parent was told about.
        "KEEL_BRANCH",
        // And the launcher's word about the ref (wave 0050): the pin
        // court no longer reads it, and the project's tests must not
        // hear it either (global review 2026-09-06, bugs cut R-8).
        "KEEL_RUNNING_REF",
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
    // Asked only where git will not say. Review 0035 R-3: asking the
    // environment FIRST made it a switch that turns courts off --
    // KEEL_BRANCH=plan/x silenced the form court, KEEL_BRANCH=main
    // silenced scope on a real wave branch, and both looked honest in
    // the verdict. Where git knows the branch, git is the answer; the
    // environment answers only the case §4.10 named, where git knows
    // nothing.
    let from_git = branch_by_git(root);
    if from_git.is_some() {
        return from_git;
    }
    // The flag first, then the variable: what a person typed means
    // more than what a hook left behind. The flag lives HERE and not
    // in the environment (review 0040 R-3): written into KEEL_BRANCH
    // it was inherited by every child this process starts -- and
    // `keel close` starts the project's whole battery, whose own
    // probes then judged themselves on a branch nobody was on. Three
    // closed waves came back red, measured.
    if let Some(named) = NAMED_BRANCH.get() {
        return Some(named.clone());
    }
    std::env::var("KEEL_BRANCH")
        .ok()
        .map(|named| named.trim().to_string())
        .filter(|named| !named.is_empty())
}

/// The branch `--branch` named, held for this process alone.
static NAMED_BRANCH: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// The frame says the branch it was given, once, before any command
/// looks. A second call is ignored: a flag is given once.
pub fn name_the_branch(named: &str) {
    let named = named.trim();
    if !named.is_empty() {
        let _ = NAMED_BRANCH.set(named.to_string());
    }
}

/// Whether a branch was named by flag -- so a caller can say aloud
/// that git already knew one and the word was not used.
pub fn branch_was_named() -> bool {
    NAMED_BRANCH.get().is_some()
}

/// The branch as git alone tells it, for a caller that needs to know
/// whether git has an answer at all (review 0040 R-9).
pub fn branch_by_git_public(root: &Path) -> Option<String> {
    branch_by_git(root)
}

/// The branch as git alone tells it.
fn branch_by_git(root: &Path) -> Option<String> {
    // Both questions are about the TREE, and neither changes while
    // this process runs: measured 13 calls each in a sandbox with
    // eight waves (wave 0061).
    let top = remembered(root, &["rev-parse", "--show-toplevel"])?;
    let top = std::fs::canonicalize(top.trim()).ok()?;
    if top != std::fs::canonicalize(root).ok()? {
        return None;
    }
    // The BRANCH is asked fresh every time, and stays out of the
    // memory above (review 0061 R-2). Where the toplevel of a tree is
    // a fact about the tree, the branch is a fact about its WORKING
    // STATE: a checkout changes it, and a library caller may check
    // one out between two questions. The wave's own prose said the
    // memory holds only what does not change within a run; the branch
    // does not belong in that sentence, and thirteen processes saved
    // are not worth a court answering about a branch that has moved.
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

/// What a `plan/<name>` branch plans, by its name alone (§8.2). The
/// name may be no wave at all -- that is the caller's to say aloud.
pub fn plan_branch(root: &Path) -> Option<String> {
    current_branch(root).and_then(|b| b.strip_prefix("plan/").map(str::to_string))
}

/// A `spike/*` branch: research, outside the methodology (§4.13).
pub fn spike_branch(root: &Path) -> Option<String> {
    current_branch(root).and_then(|b| b.strip_prefix("spike/").map(str::to_string))
}

/// §4.9: a plan branch carries the plan and nothing else. Everything
/// it changed against the base that is not the methodology's own
/// furniture (§4.8 -- the `keel/` directory, the config, and the
/// files this release generates) is a finding by name.
///
/// The conformance audit (ВАЖКА-4) measured the paragraph held by
/// nothing at all: a branch called `plan/<wave>` is not named after a
/// wave, so the scope court was skipped entirely -- and code laid
/// down there is seen by nobody, since the work branch no longer
/// carries it in its diff.
pub fn plan_findings(
    root: &Path,
    config: &Config,
) -> Result<Vec<(String, String, String)>, Refusal> {
    let (base, _) = compare_base(root)?;
    let changed_raw = git_line(
        root,
        &["diff", "--name-only", "--no-renames", &base, "HEAD"],
    )?;
    let mut out = Vec::new();
    let locks = crate::adapter::lockfiles(root);
    let leavings = crate::adapter::leavings(root);
    for file in changed_raw.lines().map(str::trim) {
        if file.is_empty() || furniture(root, config, file, &locks, &leavings) {
            continue;
        }
        // The finding is hung on the file it accuses: review 0036
        // R-14 measured it addressed to `keel/waves/<name>.md` of a
        // wave that may not exist at all.
        out.push((
            file.to_string(),
            ta("scope-plan-code", targs!("file" => file.to_string())),
            t("scope-plan-code-instead"),
        ));
    }
    Ok(out)
}

/// The methodology's own files, §4.8 as written: the `keel/`
/// directory and the tool's own config, and every generated file
/// **in the form the tool left it** -- its digest the recorded one
/// or the one this release writes. Edited by a hand it is code, as
/// the paragraph's second sentence says in as many words. Review
/// 0036 R-12 read this by NAME (the skills, the CI file, AGENTS.md),
/// and a hand-edited generated file went on being furniture on the
/// plan branch while `keel update`'s own files were drift on the
/// wave branch (global review 2026-09-06, methodology R-6; wave
/// 0052). One reading now, `generated::is_furniture`, for both
/// courts. A project's own file under a generated name that the tool
/// never wrote is code here -- named in the contract.
/// A path the tongue's runner left, not the wave's work (wave 0057):
/// `tests/__pycache__/a.pyc` is pytest's, wherever it sits. WHERE a
/// leaving may be met is the adapter's word, not a guess from its
/// shape: `__pycache__/` and `node_modules/` at any depth, a build
/// directory at its own place alone. Review 0057 R-7 measured the
/// guess: a project's own `docs/target/notes.md` and
/// `vendor/node_modules/mine/index.js` fell silently out of scope.
fn left_by_runner(file: &str, leavings: &[crate::adapter::Leaving]) -> bool {
    leavings.iter().any(|left| {
        let name = left.path.trim_end_matches('/');
        if name.is_empty() {
            return false;
        }
        if left.anywhere {
            file.split('/').any(|part| part == name)
        } else {
            file.starts_with(&format!("{name}/"))
        }
    })
}

fn furniture(
    root: &Path,
    config: &Config,
    file: &str,
    locks: &[String],
    leavings: &[crate::adapter::Leaving],
) -> bool {
    file.starts_with("keel/")
        || file == "keel.toml"
        || crate::generated::is_furniture(root, config, file)
        // What the runner leaves is the runner's, not the wave's
        // (queue after 0055, bugs R-21): a stranger who ran their
        // battery once and committed everything had `keel check`
        // call pytest's `.pyc` files drift -- and the frame had told
        // them this tongue leaves nothing worth ignoring.
        || left_by_runner(file, leavings)
        // The tongue's own, named by the adapter (wave 0055): a lock
        // file the runner writes without being asked is not this
        // wave's work, and the first build through the hook made one
        // in a stranger's project (final review 2026-09-06, bugs
        // R-20). Asked ONCE per comparison, not once per file: the
        // question costs a manifest read and a directory walk, and
        // `keel check` grew three to seven percent slower before this
        // (review 0055 R-13).
        || locks.iter().any(|lock| lock == file)
}

/// The contracts the branch changed against the base: a fact of the
/// branch the weight must read too (§6.8, §5.7) -- `docs::weight`
/// reads the declared files alone, and a light wave whose branch
/// changed a contract rode to one PR (global review 2026-09-06,
/// methodology R-3; wave 0052).
pub fn contracts_changed(root: &Path) -> Result<Vec<String>, Refusal> {
    let (base, _) = compare_base(root)?;
    let changed = git_line(
        root,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            &base,
            "HEAD",
            "--",
            "keel/contracts/",
        ],
    )?;
    Ok(changed
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        // A contract is a DOCUMENT (§2.9): `keel init` leaves
        // `keel/contracts/.gitkeep` so the standing empty directory
        // outlives git, and all three courts then called a light
        // wave's branch a contract change and led it to a full wave
        // -- on the first day in a stranger's project (final review
        // 2026-09-06, bugs R-8; wave 0055).
        .filter(|l| l.ends_with(".md"))
        .map(str::to_string)
        .collect())
}

/// The slugs the branch's commits carry as their subjects' heads --
/// `<slug>: …` against the base (§6.2: a transform is closed by a
/// commit under its slug, and several are allowed, §2.4). Nobody
/// read this before wave 0052: `next` called a transform done by its
/// touched files, and a branch with all its work in one `wip:`
/// commit was "time for the PR" (global review 2026-09-06,
/// methodology R-9).
pub fn slug_commits(root: &Path) -> Result<BTreeSet<String>, Refusal> {
    let (base, _) = compare_base(root)?;
    let subjects = git_line(root, &["log", "--format=%s", &format!("{base}..HEAD")])?;
    Ok(subjects
        .lines()
        .filter_map(|line| line.split_once(':'))
        .map(|(head, _)| head.trim().to_string())
        .filter(|head| !head.is_empty())
        .collect())
}

/// Whether a file stands in the trunk -- the fact of a merge (§6.5):
/// `Some(true)` where the trunk carries it, `Some(false)`
/// where a trunk exists and does not carry it, `None` where no trunk can be
/// asked at all -- and the caller says that aloud rather than
/// claiming a merge it cannot see (wave 0052, methodology R-5).
pub fn stands_in_main(root: &Path, rel: &str) -> Option<bool> {
    let trunk = trunk_for_the_merge_fact(root)?;
    let there = git_at(root)
        .args(["cat-file", "-e", &format!("{trunk}:{rel}")])
        .output()
        .ok()?;
    Some(there.status.success())
}

/// Whether the branch's own work is already in the trunk -- HEAD an
/// ancestor of it (§6.5: "its file AND its work arrive in the trunk by one
/// PR"; review 0052 R-6 measured a wave file put on the trunk by hand
/// calling the unmerged work closed). None where no trunk can be
/// asked.
pub fn work_in_trunk(root: &Path) -> Option<bool> {
    let trunk = trunk_for_the_merge_fact(root)?;
    let out = git_at(root)
        .args(["merge-base", "--is-ancestor", "HEAD", &trunk])
        .output()
        .ok()?;
    Some(out.status.success())
}

/// The remote this clone actually has: `origin` when it is there,
/// otherwise the only one, and nothing at all when there are none or
/// several (review 0031 R-8: `origin` was assumed and a project
/// pushed to `upstream` was told its work did not exist).
pub fn remote_name(root: &Path) -> Option<String> {
    let all = git_line(root, &["remote"]).ok()?;
    let mut names = all.lines().map(str::trim).filter(|name| !name.is_empty());
    let first = names.next()?;
    if all.lines().any(|name| name.trim() == "origin") {
        return Some("origin".to_string());
    }
    names.next().is_none().then(|| first.to_string())
}

/// Who named the trunk. The verdict says it aloud, because the first
/// run after wave 0072 gives different findings in a project whose
/// `refs/…/HEAD` is missing or stale, and a person must be able to
/// read the cause in the line itself instead of hunting a keel
/// regression.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrunkSource {
    /// `keel.toml` named it.
    Named,
    /// git named it: `refs/remotes/<remote>/HEAD`.
    Git,
    /// Nobody named it: taken by name, `main` then `master`.
    Guess,
}

/// The trunk, whole: what it is called, who called it that, and the
/// ref a comparison can actually use.
#[derive(Clone, Debug)]
pub struct Trunk {
    /// The branch's NAME, with no remote in front of it. `check`
    /// composes `{remote}/{name}` for the freshness line itself; a
    /// hand that handed back a ref would make it build
    /// `origin/origin/main`, and that line would vanish.
    pub name: String,
    pub source: TrunkSource,
    /// The remote this clone has, when it has one.
    pub remote: Option<String>,
    /// The LOCAL branch of that name when it exists, else
    /// `{remote}/{name}`; None where neither stands, and then there
    /// is no base and the courts say so.
    pub reference: Option<String>,
}

/// What one asking of the trunk produced: the answer, when there is
/// one, and what git said that this hand would not take.
///
/// The two are kept together because the second outlives the first:
/// where nothing else answers there is no `Trunk` at all, and a
/// verdict that then says "nobody names it" is lying about the cause
/// -- git did name one (review 0072 round three, R-4).
#[derive(Clone, Debug, Default)]
struct Resolved {
    trunk: Option<Trunk>,
    refused: Option<(String, RefusedBecause)>,
    /// What the LIST of names alone would have answered -- `main`,
    /// then `master`. Kept beside the answer because the merge fact
    /// is read only where it agrees with what git said (see
    /// `trunk_for_the_merge_fact`).
    by_name: Option<String>,
}

/// A branch of the WORK is never a trunk (§8.2, §4.13): a plan rides
/// `plan/…`, research rides `spike/…`, and a wave rides a branch
/// named after itself -- one that carries its own wave file.
///
/// Measured twice over. `git clone <tree>` copies the SOURCE's HEAD
/// into `refs/remotes/<remote>/HEAD`, so a clone taken while the tree
/// stood on a wave branch -- the clone keel's own briefing tells a
/// reviewer to make -- says the trunk IS that branch. The base then
/// equals HEAD, `git diff base HEAD` is empty for ever, and every
/// court reads "compared" over a comparison that never happened
/// (§4.10). A file no transform names was committed on such a clone
/// and drew no finding at all.
///
/// Two earlier measures were wrong, and each was measured wrong:
///
/// - "the branch we are standing on" (review 0072 round two): a
///   project standing on its own trunk, where the trunk is not called
///   `main`, lost its trunk entirely, and §6.5's merge fact flipped
///   with wherever HEAD happened to be. Where HEAD stands is not a
///   fact about the branch.
/// - "a name shaped like a wave slug" -- digits, a dash, a word: a
///   project whose default branch is `2024-rewrite` lost its trunk
///   the same way. The shape of a name is not a fact about it either.
///
/// What IS a fact: the branch carries `keel/waves/<its own name>.md`.
/// That is §8.2's rule read the only way a machine can read it, and
/// it is asked of the REF, not of the working tree, so the answer
/// does not depend on which branch happens to be checked out.
fn is_work_branch(root: &Path, at: &str, name: &str) -> bool {
    if name.starts_with("plan/") || name.starts_with("spike/") {
        return true;
    }
    git_line(
        root,
        &["cat-file", "-e", &format!("{at}:keel/waves/{name}.md")],
    )
    .is_ok()
}

/// What this repository calls its trunk, asked in this order -- and
/// the order is the wave's whole point (wave 0072, issue #51):
///
/// 1. what the project named in `keel.toml`. First, because the two
///    states where git answers WRONGLY are the ones only a project
///    can correct: a `refs/…/HEAD` left over from before a rename
///    (plain fetch never updates it), and a CI checkout built by
///    `init` + `remote add` + `fetch`, where it does not exist at all
///    and `git remote set-head` cannot help -- that directory is born
///    again every run.
/// 2. what git names: `refs/remotes/<remote>/HEAD`. The remote comes
///    from `remote_name`, never the literal `origin` -- review 0031
///    R-8 paid for that once already.
/// 3. `main`, else `master`, as they always were.
///
/// Before wave 0072 the list came first and `refs/…/HEAD` last, so in
/// any repository where `main` resolved git was never asked at all. A
/// project whose default branch is `development` had every plan
/// branch reddened on §4.9, named for files it had never touched.
///
/// And there were TWO hands: `check` asked git first, `scope` asked
/// the list first, so one verdict named two different trunks. This is
/// the one hand; `check` calls it too.
pub fn trunk_of(root: &Path, config: Option<&crate::config::Config>) -> Option<Trunk> {
    resolved(root, config).trunk
}

/// What git named and this hand would not take, whether or not
/// anything answered in its place -- and WHY, because the two reasons
/// are different things to tell a person.
pub fn trunk_refused(
    root: &Path,
    config: Option<&crate::config::Config>,
) -> Option<(String, RefusedBecause)> {
    resolved(root, config).refused
}

/// Why git's answer was not taken.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefusedBecause {
    /// It names a branch of the work (§8.2, §4.13).
    ItIsWork,
    /// The ref it points at is gone: a symref outlives its branch.
    ItIsGone,
}

fn resolved(root: &Path, config: Option<&crate::config::Config>) -> Resolved {
    let asked = config
        .and_then(|config| config.trunk.as_deref())
        .map(str::trim)
        .filter(|named| !named.is_empty());
    // Remembered per tree AND per asked name: keel reads no branch
    // it has not just been shown, and it creates none, so the answer
    // cannot change while one command runs. Measured before the
    // memory: `trunk()` was called 12 times in one `keel check`, at
    // up to three git processes a call.
    //
    // This is NOT the memory of `may_be_remembered`, and it must not
    // become it: review 0061 R-3 drew that border at "what cannot
    // change while one command runs", and named the working state as
    // the outside. A decision composed of a config key and two refs
    // is remembered here, under its own name, where a reader can see
    // what is kept and why.
    static TRUNK: std::sync::OnceLock<
        std::sync::Mutex<std::collections::HashMap<(std::path::PathBuf, String), Resolved>>,
    > = std::sync::OnceLock::new();
    let key = (root.to_path_buf(), asked.unwrap_or_default().to_string());
    let memory = TRUNK.get_or_init(Default::default);
    if let Ok(seen) = memory.lock()
        && let Some(answer) = seen.get(&key)
    {
        return answer.clone();
    }

    let remote = remote_name(root);
    let named = asked.map(|named| (named.to_string(), TrunkSource::Named));
    let by_git = || {
        let remote = remote.as_deref()?;
        let head = git_line(
            root,
            &[
                "symbolic-ref",
                "--short",
                &format!("refs/remotes/{remote}/HEAD"),
            ],
        )
        .ok()?;
        let head = head.trim();
        // A symref survives the ref it points at: `symbolic-ref`
        // answers `origin/development` happily when that ref has been
        // deleted, and `merge-base` then dies on a name that is not
        // an object. So the answer is verified before it is believed.
        // A symref that outlived its ref is still an ANSWER, and the
        // verdict owes the cause: review 0072 round six measured this
        // `?` dropping it and the line saying "nobody names it" while
        // git had just named one.
        if stands(root, head).is_none() {
            // The PREFIX comes off, not the last slash, and a bare
            // name keeps itself: `origin/release/stable` is the
            // branch `release/stable`, and a symref pointed at a
            // local ref gives a name with no slash at all. Review
            // 0072 round seven measured both -- the first said "git
            // names stable", which never existed, and the second
            // dropped the answer entirely.
            let gone = head.strip_prefix(&format!("{remote}/")).unwrap_or(head);
            return Some((gone.to_string(), String::new()));
        }
        // The PREFIX comes off, not the last slash: a default branch
        // may carry one. Review 0072 R-2 measured `origin/HEAD ->
        // origin/release/stable` read as the branch `stable`, which
        // stands nowhere -- the verdict named a branch that does not
        // exist and both §4.9 and §4.12 died in silence; and where a
        // branch `stable` did happen to exist, the comparison ran
        // against a stranger.
        // The prefix comes off when it is there. It is not always:
        // `refs/remotes/origin/HEAD` may be pointed at a local ref by
        // hand, and then `--short` gives a bare name. Review 0072
        // round five measured that shape read as NOTHING -- the
        // answer died in silence and the verdict said "nobody names
        // it" while git had just named one.
        let name = head.strip_prefix(&format!("{remote}/")).unwrap_or(head);
        Some((name.to_string(), head.to_string()))
    };
    // The refs a name may live under, in the order they are asked.
    // A fresh clone that has never checked out its trunk has only
    // `origin/main`, so the local ref alone would leave it with no
    // trunk at all -- the battery measured exactly that when this
    // hand was first written the short way (`scope_test`,
    // `weight_of_the_branch_test`).
    //
    // The literal `origin` stands LAST and only where no remote is
    // configured at all. Review 0031 R-8 forbade ASSUMING origin --
    // a named remote is asked first, always -- but a tree that
    // carries `refs/remotes/origin/main` with no remote behind it
    // still has its trunk written there, and before this wave
    // `scope::trunk` read it.
    let refs_for = |name: &str| {
        let mut list = vec![name.to_string()];
        match remote.as_deref() {
            Some(remote) => list.push(format!("{remote}/{name}")),
            None => list.push(format!("origin/{name}")),
        }
        list
    };
    let by_name = || {
        ["main", "master"]
            .into_iter()
            .find(|name| refs_for(name).iter().any(|at| stands(root, at).is_some()))
            .map(|name| (name.to_string(), TrunkSource::Guess))
    };
    // A branch of the work is not a trunk, and saying WHY matters: a
    // person whose clone came from a working tree must read the cause
    // instead of "nobody names it" (review 0072 R2-2).
    let (by_git, refused) = match by_git() {
        // The ref is gone: `at` is empty, and the name is kept only
        // to say what git pointed at.
        Some((name, at)) if at.is_empty() => (None, Some((name, RefusedBecause::ItIsGone))),
        Some((name, at)) if is_work_branch(root, &at, &name) => {
            (None, Some((name, RefusedBecause::ItIsWork)))
        }
        Some((name, _)) => (Some((name, TrunkSource::Git)), None),
        None => (None, None),
    };
    // Asked always, not only when git is silent: the merge fact is
    // read only where this answer and git's agree (review 0072 round
    // five, R-1). It costs one `rev-parse --verify` per name, and the
    // whole resolution is remembered once per tree.
    let plainly = by_name().map(|(name, _)| name);
    let Some((name, source)) = named.or(by_git).or_else(by_name) else {
        // Nothing answered -- but what was refused is still worth
        // saying, and it is the only thing that explains the silence.
        let answer = Resolved {
            trunk: None,
            refused,
            by_name: plainly,
        };
        if let Ok(mut seen) = memory.lock() {
            seen.insert(key, answer.clone());
        }
        return answer;
    };

    // The ref the comparison uses: the LOCAL branch of that name
    // first, and this is not a detail. Before wave 0072 a project on
    // `main` compared against the local `main`, and handing back
    // `origin/main` instead would make one unpushed commit look like
    // the branch's own work -- the very shape issue #51 reported,
    // handed to the population the wave must leave alone.
    let reference = refs_for(&name)
        .into_iter()
        .find(|at| stands(root, at).is_some());
    let answer = Resolved {
        trunk: Some(Trunk {
            name,
            source,
            remote,
            reference,
        }),
        refused,
        by_name: plainly,
    };
    if let Ok(mut seen) = memory.lock() {
        seen.insert(key, answer.clone());
    }
    answer
}

/// Whether a name resolves to a commit in this tree. `symbolic-ref`
/// does not ask this and `merge-base` dies when the answer is no.
fn stands(root: &Path, name: &str) -> Option<String> {
    git_line(
        root,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("{name}^{{commit}}"),
        ],
    )
    .ok()
    .map(|_| name.to_string())
}

/// The trunk as a ref the comparison can use, or None where the name
/// stands nowhere -- and then the courts say "not judged" rather than
/// fall to the root commit in silence.
pub fn trunk(root: &Path) -> Option<String> {
    let config = crate::config::read(root).ok();
    trunk_of(root, config.as_ref())?.reference
}

/// The trunk a MERGE FACT may be read from, which is not always the
/// trunk a comparison is taken against.
///
/// Two questions, and review 0072 round five measured why they part.
/// `git clone` copies the source's HEAD into
/// `refs/remotes/<remote>/HEAD`, so a tree parked on `wip` hands the
/// clone `wip` as its trunk. The comparison against it merely misses
/// things and says which branch it used. §6.5 is worse: `HEAD` is an
/// ancestor of `wip`, so the work "is in the trunk", and `keel close`
/// called an unmerged wave CLOSED. Before this wave that could not
/// happen -- the list of names was asked first, and `origin/main`
/// stood right there.
///
/// So the merge fact is read only where the sources AGREE, or where
/// the project itself named the trunk:
///
/// - the key named it: believed, always. A project that says which
///   branch its work lands in has answered the question.
/// - git and the name list say the same branch: believed.
/// - they disagree, and no key: the fact is NOT SEEN (`None`), and
///   the courts say so -- "will close by the fact of merge", never
///   "closed". §4.10's rule, applied to §6.5: a fact taken without a
///   witness is worse than no fact.
///
/// The comparison keeps asking `trunk`, because there a wrong base
/// is visible in the verdict (it names the branch it used) and a
/// missing base costs the whole court.
fn trunk_for_the_merge_fact(root: &Path) -> Option<String> {
    let config = crate::config::read(root).ok();
    let asked = resolved(root, config.as_ref());
    let trunk = asked.trunk?;
    if trunk.source == TrunkSource::Named {
        return trunk.reference;
    }
    match (trunk.source, asked.by_name) {
        // The list itself answered: there is nothing to disagree with.
        (TrunkSource::Guess, _) => trunk.reference,
        // git answered, and the names say the same branch.
        (TrunkSource::Git, Some(plain)) if plain == trunk.name => trunk.reference,
        // git answered and the names said NOTHING -- and this is the
        // hardest corner of the wave, measured from both sides by two
        // rounds of review.
        //
        // Round six: a project whose trunk is `development` and which
        // has no `main` and no `master` anywhere loses the merge fact
        // entirely, and `keel next` stops saying the loop is done.
        // Round seven: believing git there gives a clone parked on
        // `wip` -- in that same project -- an UNMERGED wave called
        // closed, because `wip` carries the wave file the branch
        // itself wrote.
        //
        // No question of the refs tells `wip` from `development`. So
        // the fail-safe side wins (§4.10): a fact taken without a
        // witness is worse than no fact. The cost is real and it is
        // curable by one line -- the courts name the key, and with it
        // the fact is seen again.
        (TrunkSource::Git, None) => None,
        _ => None,
    }
}

/// The comparison base: the merge-base with the trunk -- whichever
/// branch `trunk_of` above names, resolved to the local ref when it
/// stands and the remote-tracking one when it does not -- or, where
/// no trunk can be found at all, the first commit of the branch.
/// Returns the sha and whether the trunk gave it, so the report can
/// say what it took (the wave's own caveat).
pub fn compare_base(root: &Path) -> Result<(String, bool), Refusal> {
    if let Some(trunk) = trunk(root)
        && let Ok(sha) = git_line(root, &["merge-base", &trunk, "HEAD"])
    {
        return Ok((sha, true));
    }
    let roots = git_line(root, &["rev-list", "--max-parents=0", "HEAD"])?;
    let sha = roots.lines().last().unwrap_or("").trim().to_string();
    Ok((sha, false))
}

/// The both-ways comparison (§4.3-§4.6): what the branch changed
/// against the base, next to the union of the wave transforms'
/// files. keel/ is outside the comparison (§4.8); `one new in`
/// counts strictly (§4.1).
pub fn findings(
    root: &Path,
    wave: &Wave,
    config: &Config,
) -> Result<Vec<(String, String)>, Refusal> {
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

    // Rows are compared by the name they mean, not by the spelling
    // (wave 0057): `./src/a.rs` and `src/a.rs` are one file. The
    // spelling is kept beside the name, because the words of a
    // finding must quote the row as the person wrote it.
    let mut declared: std::collections::BTreeMap<String, &str> = Default::default();
    // Every `one new in` line is a promise of one file: two lines
    // over one directory promise two (§4.1 -- "need two, write two
    // lines"; review R-1).
    let mut dirs: std::collections::BTreeMap<String, (u64, &str)> = Default::default();
    // A row that climbs out of the tree names no file of this wave.
    let mut outside: Vec<&str> = Vec::new();
    for (_, transform) in &wave.transforms {
        for line in &transform.files {
            let written = line.as_written();
            if crate::docs::outside_root(written) {
                outside.push(written);
                continue;
            }
            match line {
                ScopeLine::Path(p) => {
                    declared.insert(line.name(), p.as_str());
                }
                ScopeLine::OneNewIn(d) => {
                    let seen = dirs.entry(line.name()).or_insert((0, d.as_str()));
                    seen.0 += 1;
                }
            }
        }
    }

    let mut out = Vec::new();
    let locks = crate::adapter::lockfiles(root);
    // Asked ONCE per comparison, like the locks beside it (review
    // 0055 R-13: the question costs a manifest read).
    let leavings = crate::adapter::leavings(root);

    // Drift (§4.6): touched yet never declared. A *new* file inside a
    // `one new in` directory is judged by the count below, not here;
    // an old file changed there is drift like anywhere else -- the
    // promise spoke only of one new file.
    for file in &changed {
        if file.is_empty()
            || declared.contains_key(*file)
            || furniture(root, config, file, &locks, &leavings)
        {
            continue;
        }
        if added.contains(file) && dirs.keys().any(|d| file.starts_with(d.as_str())) {
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
    for (name, written) in &declared {
        if name.starts_with("keel/") {
            continue;
        }
        if !changed.contains(name.as_str()) {
            out.push((
                ta("scope-untouched", targs!("file" => written.to_string())),
                t("scope-untouched-instead"),
            ));
        }
    }

    // A row that leaves the root (`../x`, `/x`): not drift, not an
    // untouched file -- no file of this tree at all, and said so by
    // name rather than left to fail every comparison in silence
    // (wave 0057).
    for row in &outside {
        out.push((
            ta("scope-outside", targs!("file" => row.to_string())),
            t("scope-outside-instead"),
        ));
    }

    // `one new in <dir>/`: as many new files as there are lines --
    // fewer is a finding, more is a finding, the exact count is
    // silence (§4.1). One line keeps the crisp zero/two words.
    for (name, (promised, dir)) in &dirs {
        let new_here: Vec<&str> = added
            .iter()
            .copied()
            .filter(|f| !f.is_empty() && f.starts_with(name.as_str()))
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
