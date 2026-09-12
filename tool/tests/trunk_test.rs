//! Scenario test of wave 0072: the trunk is the one git names.
//!
//! Issue #51, from a live project on keel 1.3.0 that moved to
//! `development` as its default branch: `keel check` reddened every
//! plan branch on §4.9, naming files the branch never touched --
//! exactly the files the default branch has and `main` does not.
//!
//! git already knew the answer. `scope::trunk` asked it LAST: it
//! walked `main`, `master`, `origin/main`, `origin/master` first, so
//! in any repository where one of those resolves, `refs/…/HEAD` was
//! never asked at all.

mod common;

use common::{Sandbox, keel_sandbox};

use std::fs;
use std::path::Path;
use std::process::Command;

fn write(dir: &Path, rel: &str, text: &str) {
    let path = dir.join(rel);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args([
            "-c",
            "user.email=keel@test",
            "-c",
            "user.name=keel-test",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn keel(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
        out.status.code().unwrap_or(-1),
    )
}

fn all_decided() -> String {
    let mut block = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        block.push_str(&format!("  {cut}: \"n/a, бо ця пісочниця грає інше\"\n"));
    }
    block
}

/// A project whose default branch is NOT `main`, arranged the way
/// issue #51 describes: the trunk carries a file `main` does not, and
/// the plan branch is cut from the trunk, touching only `keel/`.
///
/// The remote is a real bare repository, because `refs/…/HEAD` is a
/// remote-tracking symref and a sandbox that fakes it would be
/// measuring the fixture, not the tool.
fn project(name: &str, default_branch: &str, remote: &str) -> Sandbox {
    let home = keel_sandbox(name);
    let bare = home.join("origin.git");
    let work = home.join("work");
    fs::create_dir_all(&bare).unwrap();
    fs::create_dir_all(&work).unwrap();

    write(&work, "keel.toml", "lang = \"en\"\nadapter = \"rust\"\n");
    write(
        &work,
        "Cargo.toml",
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    write(&work, "src/lib.rs", "pub fn a() {}\n");
    fs::create_dir_all(work.join("keel/contracts")).unwrap();

    git(&work, &["init", "-q", "-b", "main"]);
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "base"]);

    // The trunk goes ahead of `main` by one file -- the shape of a
    // project that promotes `main` from its default branch on purpose.
    git(&work, &["checkout", "-q", "-b", default_branch]);
    write(&work, "DESIGN.md", "what the default branch carries\n");
    git(&work, &["add", "-A"]);
    git(
        &work,
        &["commit", "-q", "-m", "the default branch moves on"],
    );

    Command::new("git")
        .args(["init", "--bare", "-q"])
        .arg(&bare)
        .output()
        .unwrap();
    git(&work, &["remote", "add", remote, bare.to_str().unwrap()]);
    git(&work, &["push", "-q", remote, "main", default_branch]);
    git(
        &work,
        &[
            "symbolic-ref",
            &format!("refs/remotes/{remote}/HEAD"),
            &format!("refs/remotes/{remote}/{default_branch}"),
        ],
    );

    // A plan branch cut from the trunk, carrying the plan and nothing
    // else.
    git(&work, &["checkout", "-q", "-b", "plan/0001-a-wave"]);
    write(
        &work,
        "keel/waves/0001-a-wave.md",
        &format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: work\nтіло\n",
            all_decided()
        ),
    );
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "the plan"]);
    home
}

/// proves: the-trunk-is-the-one-git-names@8231ad
#[test]
fn the_trunk_is_the_one_git_names() {
    // --- issue #51: the trunk is what git names, not what sorts
    // first in a list of names ---
    let home = project("trunkdefault", "development", "origin");
    let work = home.join("work");
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("DESIGN.md"),
        "the plan branch is judged against the DEFAULT branch, not \
         against main: DESIGN.md belongs to the trunk and this branch \
         never touched it\n{out}"
    );
    assert_eq!(code, 0, "and nothing else reddens either:\n{out}");

    // --- the remote is not spelled `origin`: review 0031 R-8 put
    // that into `check`, and merging the hands must not lose it ---
    let home = project("trunkupstream", "development", "upstream");
    let work = home.join("work");
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("DESIGN.md"),
        "and the remote may be called anything -- a clone pushed to \
         `upstream` gets the same answer:\n{out}"
    );

    // --- a project whose default branch IS main sees no change:
    // this is the population the wave must leave alone, and the local
    // ref must win over the remote-tracking one ---
    //
    // Review 0072 R-5b measured the first shape of this side empty:
    // the branch was cut BEFORE the unpushed commit, so both refs
    // gave the same merge-base and no choice of ref could change the
    // verdict. The branch must be cut from the LOCAL tip, after the
    // commit that was never pushed -- that is the whole shape the
    // wave calls its main cost.
    let home = project("trunkmain", "feature-x", "origin");
    let work = home.join("work");
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    git(&work, &["checkout", "-q", "main"]);
    write(&work, "LOCAL.md", "committed locally, never pushed\n");
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "local only"]);
    // Cut HERE: the plan branch's history carries the unpushed commit,
    // so a base taken from origin/main would call LOCAL.md the
    // branch's own work.
    git(&work, &["checkout", "-q", "-b", "plan/0002-b-wave"]);
    write(
        &work,
        "keel/waves/0002-b-wave.md",
        &format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: work\nтіло\n",
            all_decided()
        ),
    );
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "the plan"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("LOCAL.md"),
        "the LOCAL main is the trunk when it exists -- resolving the \
         remote HEAD to a NAME and looking for the local ref first is \
         what keeps this population still:\n{out}"
    );

    // --- a default branch may carry a slash ---
    //
    // Review 0072 R-2: `origin/HEAD -> origin/release/stable` cut at
    // the LAST slash gave the branch `stable`, which stands nowhere.
    // The verdict named a branch that does not exist and §4.9 died in
    // silence.
    let home = project("trunkslash", "release/stable", "origin");
    let work = home.join("work");
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("trunk: release/stable"),
        "the name comes off the remote's prefix, not off the last \
         slash:\n{out}"
    );
    assert!(
        !out.contains("DESIGN.md"),
        "and it is a real branch, so the comparison runs:\n{out}"
    );
    assert_eq!(code, 0, "nothing reddens:\n{out}");

    // --- a symref that outlived the ref it points at ---
    //
    // `git symbolic-ref` answers happily when the branch is gone, and
    // `merge-base` then dies on a name that is not an object. Review
    // 0072 R-5a measured this guard held by nothing: removing it left
    // the whole battery green.
    let home = project("trunkdangling", "development", "origin");
    let work = home.join("work");
    git(
        &work,
        &["update-ref", "-d", "refs/remotes/origin/development"],
    );
    git(&work, &["branch", "-q", "-D", "development"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("trunk: main"),
        "a symref pointing at nothing is not an answer -- the next \
         source is asked, and main is still here:\n{out}"
    );
    assert!(
        out.contains("§4.9"),
        "and §4.9 is judged, not skipped in silence:\n{out}"
    );

    // --- and where git is WRONG, the project may say so itself ---
    //
    // The two states git answers wrongly are the ones only the
    // project can correct, and both are in this side: the remote HEAD
    // still points at `main` because a plain fetch never updates it
    // after a rename, and a CI checkout built by `init` + `remote
    // add` + `fetch` has no such ref at all. `keel.toml` is asked
    // first for exactly this.
    let home = project("trunknamed", "development", "origin");
    let work = home.join("work");
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    let (stale, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        stale.contains("DESIGN.md"),
        "the fixture must really be in the broken state, or the key \
         below proves nothing:\n{stale}"
    );
    // The key is a ROOT key, and it goes at the TOP: a line appended
    // to the end of a file that already carries `[trust]` is read as
    // part of that table and swallowed in silence. The document says
    // so for the same reason.
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\ntrunk = \"development\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "name the trunk"]);
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("DESIGN.md"),
        "a project that names its own trunk is believed over a \
         `refs/…/HEAD` left behind by a rename:\n{out}"
    );
    assert_eq!(code, 0, "and nothing else reddens either:\n{out}");
    assert!(
        out.contains("trunk: development") && out.contains("named by keel.toml"),
        "and the verdict says WHERE the trunk came from -- on the PLAN \
         branch, which named no trunk at all before this wave:\n{out}"
    );

    // --- and a key written where it is not read is refused, not
    // swallowed ---
    //
    // Measured on the 1.3.0 binary: a root key appended to the END of
    // a file that carries `[trust]` is read as an entry of that
    // table, and a table of name-to-reason takes any name at all. The
    // person who was told "name the trunk in keel.toml" did exactly
    // that and got silence.
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\n\n[trust]\n\"cargo test\" = \"the battery\"\n\ntrunk = \"development\"\n",
    )
    .unwrap();
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert_ne!(
        code, 0,
        "silence is the one answer it must not give:\n{out}"
    );
    assert!(
        out.contains("root key") && out.contains("[trust]"),
        "the refusal names the key, the table that ate it, and where \
         the line belongs:\n{out}"
    );

    // --- and a clone of a working tree must not take the branch it
    // is standing on for the trunk ---
    //
    // `git clone --no-local <tree>` -- the clone keel's own briefing
    // tells a reviewer to make -- copies the SOURCE's HEAD into
    // `refs/remotes/origin/HEAD`. A clone taken while the tree stood
    // on a wave branch therefore says the trunk IS that wave branch,
    // the base becomes HEAD, and `git diff base HEAD` is empty for
    // ever: every court reads "compared" over a comparison that never
    // happened (§4.10). Measured on wave 0072's own branch before
    // this guard: a file no transform names drew no finding at all.
    let home = project("trunkclone", "development", "origin");
    let source = home.join("work");
    // The source stands on the wave branch, not on its trunk.
    git(&source, &["checkout", "-q", "-b", "0001-a-wave"]);
    write(&source, "src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    git(&source, &["add", "-A"]);
    git(&source, &["commit", "-q", "-m", "work: the declared file"]);
    let clone = home.join("clone");
    let out = Command::new("git")
        .args(["clone", "-q", "--no-local"])
        .arg(&source)
        .arg(&clone)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    // The fixture must really be in the shape that broke it.
    let head = Command::new("git")
        .arg("-C")
        .arg(&clone)
        .args(["symbolic-ref", "--short", "refs/remotes/origin/HEAD"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&head.stdout).trim(),
        "origin/0001-a-wave",
        "the clone must carry the source's HEAD, or this side proves nothing"
    );
    // A file no transform names: the court must see it.
    write(&clone, "src/smuggled.rs", "pub fn smuggled() {}\n");
    git(&clone, &["add", "-A"]);
    git(&clone, &["commit", "-q", "-m", "work: smuggled"]);
    let (out, code) = keel(&["check", clone.to_str().unwrap()]);
    assert!(
        out.contains("src/smuggled.rs"),
        "a base equal to HEAD compares nothing, and the smuggled file \
         walks straight through §4.4:\n{out}"
    );
    assert_ne!(code, 0, "and the verdict is red for it:\n{out}");
    assert!(
        out.contains("git names 0001-a-wave"),
        "and the line says WHY it took another trunk: git did name \
         one, and \"nobody names it\" would be a lie about the \
         cause:\n{out}"
    );

    // --- a key naming a branch that is not here says so, and does
    // not advise what has already been done ---
    //
    // Review 0072 R-8: a typo in the new key turned two courts off
    // and the only advice on screen was "name the trunk in
    // keel.toml".
    let home = project("trunkmissing", "development", "origin");
    let work = home.join("work");
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\ntrunk = \"nosuchbranch\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "a typo in the key"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("nosuchbranch") && out.contains("no branch of that name"),
        "the limit names the key and what it could not find:\n{out}"
    );
    assert!(
        !out.contains("this clone knows no main trunk"),
        "and the generic line, which tells a person to do what they \
         have already done, does not stand beside it:\n{out}"
    );

    // --- an honest entry of [trust] named like a root key is read ---
    //
    // Review 0072 R-3: the first court read the NAME alone, so a
    // project whose contract runs a command called `ci` had its
    // keel.toml refused by every command, and the way out the
    // refusal named (`"./ci"`) matched nothing at all. The value is
    // what tells them apart: both tables carry keel's own twelve
    // characters.
    let home = project("trunktrust", "development", "origin");
    let work = home.join("work");
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\n\n[trust]\nci = \"668c699a0a59\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "a command of our own"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("root key"),
        "an entry carrying keel's own mark is an entry, not a root \
         key put in the wrong place:\n{out}"
    );

    // --- and standing ON a trunk that is not called main changes
    // nothing ---
    //
    // Review 0072 round two measured the first guard breaking exactly
    // this: it refused git's answer whenever the trunk was the branch
    // HEAD stood on, so a project on its own `development` lost its
    // trunk and §6.5's merge fact flipped depending on where HEAD
    // happened to be. The name is a fact of the methodology; where
    // HEAD stands is not.
    let home = project("trunkstanding", "development", "origin");
    let work = home.join("work");
    git(&work, &["checkout", "-q", "development"]);
    write(
        &work,
        "keel/waves/0003-a-chore.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
            all_decided()
        ),
    );
    write(
        &work,
        "keel/reviews/0003-a-chore.md",
        "рецензія: свіже око дивилось, знахідок нема\n",
    );
    git(&work, &["add", "-A"]);
    git(
        &work,
        &["commit", "-q", "-m", "tidy: the chore lands in the trunk"],
    );
    let (standing, _) = keel(&["status", work.to_str().unwrap()]);
    assert!(
        standing.contains("0003-a-chore") && standing.contains("merging closed it"),
        "the merge fact is seen from the trunk itself, whatever the \
         trunk is called:\n{standing}"
    );
    // And the same tree, read from a branch beside it, says the same.
    git(&work, &["checkout", "-q", "-b", "elsewhere"]);
    let (beside, _) = keel(&["status", work.to_str().unwrap()]);
    assert!(
        beside.contains("0003-a-chore") && beside.contains("merging closed it"),
        "and the answer does not depend on where HEAD stands:\n{beside}"
    );

    // --- the mark is twelve characters, and the length is part of
    // the measure ---
    //
    // Review 0072 R2-3: removing `value.len() == 12` left the whole
    // battery green, so the very condition that decides between
    // refusing and swallowing was held by nothing.
    let home = project("trunklongmark", "development", "origin");
    let work = home.join("work");
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\n\n[trust]\nci = \"abcdef0123456\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "thirteen is not twelve"]);
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("root key"),
        "thirteen hex characters are not keel's mark, so this is a \
         root key that slid under a table:\n{out}"
    );
    assert_ne!(code, 0, "and it is refused, not swallowed:\n{out}");

    // --- the memory answers the question it was asked ---
    //
    // Review 0072 R2-6: the key carries the asked name, and nothing
    // held it -- a key of the tree alone passed the whole battery.
    // One process, one tree, two questions: the answers must differ.
    let home = project("trunkmemory", "development", "origin");
    let work = home.join("work");
    let asked = keel::config::Config {
        trunk: Some("main".to_string()),
        ..Default::default()
    };
    let by_git = keel::scope::trunk_of(&work, None).expect("git names one");
    let by_key = keel::scope::trunk_of(&work, Some(&asked)).expect("the key names one");
    assert_eq!(by_git.name, "development", "git names the default branch");
    assert_eq!(
        by_key.name, "main",
        "and the key names another -- one tree, one process, two \
         answers, because the memory is keyed by what was asked"
    );
}
