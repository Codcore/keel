//! Rung 13, the frame hand (contract tool-init; NEW-CONCEPT
//! "cross-cutting", §8.7): the methodology's frame in one move --
//! and never a trampled byte. Every piece is its own line: born,
//! already stands, or a refusal aloud; the frame lands piece by
//! piece, and a second run builds only what is missing.

use crate::ask::Answers;
use crate::gate;
use crate::i18n::{t, ta};
use crate::refusal::Refusal;
use crate::targs;
use std::path::Path;

/// The `keel init` report and the count of pieces that did not
/// stand: zero is a green exit, anything else is honest red while
/// the rest of the frame still landed.
pub fn run(root: &Path, answers: &Answers) -> Result<(String, usize), Refusal> {
    let mut report = t("init-title");
    report.push('\n');
    let mut failed = 0usize;

    // The three keel/ directories, each born with .gitkeep so an
    // empty one outlives git. A standing directory is a fact and
    // stays untouched -- except a missing .gitkeep, which is fed
    // with its own word (review 0014 R-2): a new empty file
    // tramples nothing, and "builds what is missing" stays true.
    for rel in ["keel/waves", "keel/contracts", "keel/reviews"] {
        let dir = root.join(rel);
        if dir.is_dir() {
            let keep = dir.join(".gitkeep");
            if keep.is_file() {
                report.push_str(&ta("init-stands", targs!("piece" => rel.to_string())));
            } else {
                match std::fs::write(&keep, "") {
                    Ok(()) => {
                        report.push_str(&ta("init-fed", targs!("piece" => rel.to_string())));
                    }
                    Err(e) => {
                        failed += 1;
                        report.push_str(&ta(
                            "init-failed",
                            targs!("piece" => rel.to_string(), "error" => e.to_string()),
                        ));
                    }
                }
            }
            report.push('\n');
            continue;
        }
        let born =
            std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(dir.join(".gitkeep"), ""));
        match born {
            Ok(()) => {
                report.push_str(&ta("init-born", targs!("piece" => rel.to_string())));
                report.push('\n');
            }
            Err(e) => {
                failed += 1;
                report.push_str(&ta(
                    "init-failed",
                    targs!("piece" => rel.to_string(), "error" => e.to_string()),
                ));
                report.push('\n');
            }
        }
    }

    // keel.toml with the commented config vocabulary (NEW-CONCEPT,
    // Config), enabling
    // nothing: the defaults stay with config's own words. An
    // existing file -- whoever's -- is a fact: not read, not
    // touched (its content is config's court, §7.9). The write
    // rides a dot-temp and a rename (the 0013 school): whole or
    // refused, never a stub.
    let config = root.join("keel.toml");
    if config.is_file() {
        report.push_str(&ta(
            "init-stands",
            targs!("piece" => "keel.toml".to_string()),
        ));
        report.push('\n');
    } else {
        // The wizard's hand writes it (wave 0026): answered fields
        // stand as lines, unanswered ones stay comments, so a default
        // never passes itself off as a choice.
        let text = crate::ask::config_text(answers);
        match crate::plan::write_new(&config, &text).map_err(|refusal| refusal.reason) {
            Ok(()) => {
                report.push_str(&ta("init-born", targs!("piece" => "keel.toml".to_string())));
                report.push_str(" — ");
                report.push_str(&t(if *answers == Answers::default() {
                    "init-config-default"
                } else {
                    "init-config-answered"
                }));
                report.push('\n');
            }
            Err(e) => {
                failed += 1;
                report.push_str(&ta(
                    "init-failed",
                    targs!("piece" => "keel.toml".to_string(), "error" => e),
                ));
                report.push('\n');
            }
        }
    }

    // The commit-msg hook by gate's own hand (§9.3) -- no double: a
    // foreign hook or a silent git is gate's refusal, said here as a
    // row, and the frame keeps landing around it.
    //
    // Unless the project answered `hooks = false`. Review 0035 named
    // this and set it aside: the answer went into keel.toml and the
    // hook was written anyway, so the question changed nothing. An
    // installed hook is not swept away by a later "no" -- removing
    // what a person may rely on is not this command's to do -- but it
    // is said aloud that nobody maintains it now.
    // The answer as it STANDS, not only as this call carries it:
    // review 0037 R-11 measured a second `keel init` without flags
    // re-installing the hook over a keel.toml that said no.
    let hooks_off = answers.hooks == Some(false)
        || (answers.hooks.is_none()
            && crate::config::read_unpinned(root).is_ok_and(|config| !config.hooks));
    if hooks_off {
        // Asking git where the hook would live, and whose it is
        // (review 0037 R-10): a hard-wired .git/hooks said "not
        // installed" over a hook in core.hooksPath or a worktree's
        // shared directory, and called a stranger's hook ours.
        let key = match gate::hook_path(root) {
            Some(path) if path.is_file() => {
                if gate::hook_is_ours(&path) {
                    "init-hook-off-standing"
                } else {
                    "init-hook-off-foreign"
                }
            }
            _ => "init-hook-off",
        };
        report.push_str("  ");
        report.push_str(&t(key));
        report.push('\n');
    } else {
        match gate::install_hook(root) {
            Ok(words) => {
                report.push_str("  ");
                report.push_str(&words);
                report.push('\n');
            }
            Err(refusal) => {
                failed += 1;
                let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
                // Where the refusal points at the root itself the
                // stripped name is empty -- the piece keeps its name.
                let shown = if shown.as_os_str().is_empty() {
                    std::path::Path::new("commit-msg")
                } else {
                    shown
                };
                report.push_str(&format!(
                    "  {:<8} {} — {}\n           {}: {}\n",
                    t("word-red"),
                    shown.display(),
                    refusal.reason,
                    t("word-instead"),
                    refusal.instead
                ));
            }
        }
    }

    // The generated integrations (wave 0022): the block agents and
    // CI read lies in the repository, and the frame lays it -- but
    // never over a hand's edit.
    match crate::config::read_unpinned(root) {
        Ok(config) => {
            let (word, lacked) = crate::generated::write(root, &config);
            failed += lacked;
            report.push_str("  ");
            report.push_str(&word);
            report.push('\n');
        }
        Err(_) => {
            // The config court says that fault aloud already (main
            // prints its refusal, school 0014 R-1), so the frame
            // does not repeat the reason -- but it does count the
            // piece that did not stand (wave 0024). A keel.toml that
            // exists and does not read used to leave a GREEN frame:
            // the row said why and the exit code said all was well.
            // An empty agent list is exactly that case, and a
            // refusal aloud with a green exit is a half-truth.
            failed += 1;
            report.push_str("  ");
            report.push_str(&t("generated-unjudged-config"));
            report.push('\n');
        }
    }

    // The ignore rules (wave 0020, the third gift of the first
    // field): the frame advises and writes nothing of the project's
    // own -- .gitignore is not the methodology's frame, and the
    // frame tramples no byte (school 0014). The advice never
    // reddens the exit: it is no piece that failed to stand.
    report.push_str("  ");
    report.push_str(&ignore_row(root));
    report.push('\n');

    report.push('\n');
    report.push_str(&t("init-eight-seven"));
    report.push('\n');
    report.push_str(&t("init-next"));
    report.push('\n');
    Ok((report, failed))
}

/// What the frame has to say about the ignore rules: four truths
/// and an honest fifth for a file it cannot read -- never a guess
/// and never a write. The adapter is asked through the config's own
/// home (school 0015/0017); the config is read unpinned, so a pin
/// this binary does not answer to still leaves the frame landing
/// (wave 0018's caveat).
fn ignore_row(root: &Path) -> String {
    // A config that cannot be read is not an unnamed adapter: the
    // rule is simply not judged, and the reason is said (review
    // 0020 R-2).
    let config = match crate::config::read_unpinned(root) {
        Ok(config) => config,
        Err(refusal) => return ta("init-ignore-unjudged", targs!("error" => refusal.reason)),
    };
    if !config.adapter_known() {
        // A named adapter is called by its name; "not named" belongs
        // to the absent one (review 0020 R-8; the 0017 R-3 school).
        return match config.adapter {
            Some(name) => ta("init-ignore-unknown-adapter", targs!("name" => name)),
            None => t("init-ignore-no-adapter"),
        };
    }
    // The rule names the directory THIS tongue builds into, asked of
    // the adapter that already knows (review 0042 R-3): the cargo
    // constant stood here, so an elixir project was told its build
    // directory is `_build/` and advised to ignore `target/` -- one
    // line contradicting itself, and following it left `_build` under
    // git, which is the very harm this reminder exists for.
    let rule = match crate::adapter::build_dir(root) {
        crate::adapter::BuildDir::At(path) => format!(
            "{}/",
            path.file_name().unwrap_or_default().to_string_lossy()
        ),
        _ => format!("{}/", crate::adapter::BUILD_DIR),
    };
    // Which directory to ask about is the adapter's answer: the
    // crate may live one level down (keel's own shape), a tongue may
    // build nothing at all, and a root the adapter cannot name is
    // said aloud, never guessed. Review 0038 R-18 caught the middle
    // case wearing the last one's words -- a ruby project was told
    // its crate could not be found.
    let build = match crate::adapter::build_dir(root) {
        crate::adapter::BuildDir::At(path) => path
            .strip_prefix(root)
            .unwrap_or(Path::new(""))
            .to_path_buf(),
        crate::adapter::BuildDir::Nothing => return t("init-ignore-nothing-built"),
        crate::adapter::BuildDir::Unknown => {
            let reason = crate::adapter::crate_root(root)
                .err()
                .map(|refusal| refusal.reason)
                .unwrap_or_default();
            return ta("init-ignore-no-crate", targs!("error" => reason));
        }
    };
    // Asked with the trailing slash git wants: a directory-only
    // rule (target/) matches a path only when the path is named as
    // a directory -- and before the first build there is no
    // directory on disk to speak for itself.
    let shown = format!("{}/", build.display());
    // git judges its own rules -- the root file, the nested ones
    // cargo writes beside a crate, the local exclude. Reading a
    // single file instead raised a false alarm on keel itself,
    // whose rule lives in tool/.gitignore (wave 0020, dogfood).
    // Asked through the frame's own git hand, deaf to the
    // repository a hook may have left in the environment (§gate).
    let out = crate::scope::git_at(root)
        .args(["check-ignore", "-v", "--"])
        .arg(&shown)
        .output();
    let out = match out {
        Ok(out) => out,
        Err(e) => return ta("init-ignore-unjudged", targs!("error" => e.to_string())),
    };
    let words = String::from_utf8_lossy(&out.stdout);
    let source = words
        .lines()
        .next()
        .and_then(|line| line.split(':').next())
        .unwrap_or("")
        .to_string();
    // Whether a rule TRAVELS is not a guess from the shape of the
    // path (review 0020 R-5): it travels only when a .gitignore of
    // the working tree gave it, and that file is not the one
    // core.excludesFile names -- a config file, global or local,
    // reaches no other clone.
    let named_by_config = crate::scope::git_at(root)
        .args(["config", "--get", "core.excludesFile"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_default();
    let travels = !source.is_empty()
        && !source.starts_with('/')
        && !source.contains(".git/")
        && Path::new(&source)
            .file_name()
            .is_some_and(|name| name == ".gitignore")
        && (named_by_config.is_empty() || source != named_by_config);
    match out.status.code() {
        // Ignored, and the rule comes from a file of the repository
        // -- it travels with every clone.
        Some(0) if travels => ta(
            "init-ignore-stands",
            targs!("path" => shown, "source" => source),
        ),
        // Ignored only here: an exclude of this clone, or the
        // person's global file -- neither travels (the first
        // field's R-4 school), so the advice stands.
        Some(0) => ta(
            "init-ignore-exclude-only",
            targs!("path" => shown, "source" => source, "rule" => rule),
        ),
        Some(1) => ta(
            "init-ignore-missing",
            targs!("path" => shown, "rule" => rule),
        ),
        _ => ta(
            "init-ignore-unjudged",
            targs!("error" => String::from_utf8_lossy(&out.stderr).trim().to_string()),
        ),
    }
}

/// `keel setup`: the wizard on a project that already answered once.
///
/// Named as a limit by review 0026 and unlifted for six waves --
/// until this one, a keel.toml was edited by hand or not at all. The
/// answers the wizard did not ask about survive, and so do the
/// sections it never asks about at all: `[trust]` and `[generated]`
/// belong to the machine and to the person, not to the wizard.
pub fn setup(root: &Path, answers: &crate::ask::Answers) -> Result<(String, usize), Refusal> {
    let config = root.join("keel.toml");
    // A project with no config at all gets the whole frame, not a
    // lonely keel.toml: review 0032 R-7 measured setup leaving a
    // directory with one file where init would have built the waves,
    // the contracts, the integrations and the workflow.
    if !config.is_file() {
        return run(root, answers);
    }
    // The person's file, edited -- not a new file with their words
    // dropped. Only what the wizard asked about moves.
    let old = std::fs::read_to_string(&config).unwrap_or_default();
    let mut text = if old.trim().is_empty() {
        crate::ask::config_text(answers)
    } else {
        crate::confedit::upsert_root(&old, &crate::ask::answered_rows(answers))
    };

    // What the wizard never asked about is carried across verbatim.
    {
        // Unpinned, for the same reason main.rs reads it unpinned.
        let kept = crate::config::read_unpinned(root).ok();
        if let Some(kept) = kept {
            // A trust line for a command this project no longer runs
            // is "a door opened in advance" -- keel check says so,
            // and review 0032 R-10 measured setup leaving one behind
            // whenever the ci command changed, turning the gate red
            // and sending the person back to editing by hand.
            // Every command this project really runs, not only the ci
            // line: a contract's `verify` is trusted by a person's
            // decision (§7.16), and the bug audit measured setup
            // dropping those silently -- a defect I introduced while
            // fixing R-10 of review 0032.
            let scan = crate::docs::scan(root);
            let contracts = scan.map(|scan| scan.contracts).unwrap_or_default();
            let live: Vec<String> = crate::trust::live_commands(&kept, &contracts)
                .into_iter()
                .map(|(_, command)| command)
                .chain(answers.ci.clone())
                .collect();
            // Removed, not merely filtered: the text being written is
            // the person's own config with every record already in
            // it, so a filtered list added nothing and took nothing
            // away (review 0034 R-4).
            text = crate::confedit::retain(&text, "trust", &live);
            let trust: Vec<(String, String)> = kept
                .trust
                .into_iter()
                .filter(|(command, _)| live.iter().any(|word| word == command))
                .collect();
            for (section, entries) in [("trust", trust), ("generated", kept.generated)] {
                if entries.is_empty() {
                    continue;
                }
                let rows: Vec<(String, String)> = entries.into_iter().collect();
                text = crate::confedit::upsert(&text, section, &rows);
            }
        }
    }

    // Through the same hand init writes with: a write that dies
    // half-way leaves the old file whole, not a stump (the 0013
    // school, which init::setup did not inherit -- review 0032 R-1).
    crate::plan::write_over(&config, &text)?;
    let mut report = ta("init-born", targs!("piece" => "keel.toml".to_string()));
    report.push('\n');

    // A changed answer that changes nothing is not a changed answer:
    // review 0032 R-9 measured `keel setup --agents claude,cursor`
    // writing the line and leaving the new agent with no files at
    // all, so the project declared an agent nothing was written for.
    let mut failed = 0;
    if let Ok(now) = crate::config::read_unpinned(root) {
        let (word, lacked) = crate::generated::write(root, &now);
        failed += lacked;
        report.push_str("  ");
        report.push_str(&word);
        report.push('\n');
    }
    Ok((report, failed))
}
