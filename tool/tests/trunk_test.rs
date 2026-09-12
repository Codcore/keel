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

/// proves: the-trunk-is-the-one-git-names@3dcdca
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

    // --- a default branch may be named like a wave and still be the
    // trunk ---
    //
    // The guard that keeps a clone of a working tree from judging a
    // branch against itself must read a FACT, not the shape of a
    // name: `2024-rewrite` looks exactly like a wave slug, and a
    // project may well call its default branch that. Measured before
    // this side existed: such a project lost its trunk entirely and
    // `keel check` said "this clone knows no main trunk".
    let home = project("trunkyear", "2024-rewrite", "origin");
    let work = home.join("work");
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("trunk: 2024-rewrite"),
        "a branch is a branch of the work when it CARRIES the wave \
         file it is named after, not when its name is shaped like a \
         slug:\n{out}"
    );
    assert!(
        !out.contains("DESIGN.md"),
        "and the comparison runs against it:\n{out}"
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
    // One line, both facts: the NAME the courts speak of and the REF
    // it resolved to. This clone has no local `main`, only
    // `origin/main` -- and a verdict that said one on one line and the
    // other on the next was read as two answers (R2-5).
    assert!(
        out.contains("main (origin/main)"),
        "the trunk line carries the name and the ref it resolved \
         to:\n{out}"
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
    // Without the key, git says `development` and the list of names
    // says `main`: two answers, and §6.5 is not a place to pick one
    // by guesswork (round five, R-1). The court says it cannot see
    // the fact -- never that it saw one.
    let (guessing, _) = keel(&["status", work.to_str().unwrap()]);
    assert!(
        !guessing.contains("merging closed it"),
        "where the sources disagree the merge fact is not claimed:\n{guessing}"
    );
    // With the key, the project has answered, and the fact is read.
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\ntrunk = \"development\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "name the trunk"]);
    let (standing, _) = keel(&["status", work.to_str().unwrap()]);
    assert!(
        standing.contains("0003-a-chore") && standing.contains("merging closed it"),
        "the merge fact is seen from the trunk itself, whatever the \
         trunk is called, once the project has named it:\n{standing}"
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

    // --- a clone parked on ANY name: a base equal to the head is
    // not a base ---
    //
    // Review 0072 round three measured the way in that no guard over
    // names can close: `git clone` copies the source's HEAD, and a
    // tree parked on `wip` -- or `feature/x`, or anything at all --
    // hands the clone a trunk that stands exactly where the branch
    // being judged stands. The comparison then compares nothing, and
    // every line reads "compared" over it.
    let home = project("trunkparked", "development", "origin");
    let source = home.join("work");
    git(&source, &["checkout", "-q", "-b", "0001-a-wave"]);
    write(&source, "src/lib.rs", "pub fn a() {}\npub fn b() {}\n");
    write(&source, "src/smuggled.rs", "pub fn smuggled() {}\n");
    git(&source, &["add", "-A"]);
    git(
        &source,
        &["commit", "-q", "-m", "work: and a file nobody named"],
    );
    // The tree is parked on a name that says nothing about waves.
    git(&source, &["checkout", "-q", "-b", "wip"]);
    let clone = home.join("parked");
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
    git(&clone, &["checkout", "-q", "0001-a-wave"]);
    let (out, _) = keel(&["check", clone.to_str().unwrap()]);
    // And this is the wave's named border, not a solved case: no
    // measure over refs tells `wip` from a trunk, so keel compares
    // against it -- and SAYS so by name. Review round four measured
    // what the other road costs: a line claiming the courts did not
    // judge, printed beside their own red findings.
    assert!(
        out.contains("trunk: wip") && out.contains("named by git"),
        "the verdict names the branch it compared against and who \
         named it, so a reader sees at once that the clone's git was \
         asked and what it answered:\n{out}"
    );

    // --- the key answers, and the line says the key ---
    //
    // Round three R-3: the refused answer was printed over the key's,
    // so the line said "taken by name" about a trunk the key had
    // named, and told the reader to do what they had already done.
    let home = project("trunkkeywins", "development", "origin");
    let work = home.join("work");
    // The refused answer must be a REAL one, or this side proves
    // nothing: an unpushed branch never gets past `rev-parse
    // --verify` and the arm under test is never reached.
    git(&work, &["push", "-q", "origin", "plan/0001-a-wave"]);
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/plan/0001-a-wave",
        ],
    );
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\ntrunk = \"development\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "name the trunk"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("named by keel.toml"),
        "the key answered, so the key is what the line names:\n{out}"
    );
    assert!(
        !out.contains("taken without git"),
        "and it does not describe an answer it did not use:\n{out}"
    );

    // --- where nothing answers, the refusal is still the reason ---
    //
    // Round three R-4: the refused name died together with the
    // missing answer, and the verdict fell back to "nobody named
    // one" -- which is exactly the lie the round before had paid to
    // remove.
    let home = project("trunknothing", "development", "origin");
    let source = home.join("work");
    // The bare remote refuses to delete the branch its own HEAD
    // points at, so move that first.
    git(
        &home.join("origin.git"),
        &["symbolic-ref", "HEAD", "refs/heads/development"],
    );
    git(&source, &["branch", "-q", "-D", "main"]);
    git(&source, &["push", "-q", "origin", "--delete", "main"]);
    git(&source, &["checkout", "-q", "-b", "0002-b-wave"]);
    write(
        &source,
        "keel/waves/0002-b-wave.md",
        &format!(
            "---\ntransforms:\n  work:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: work\nтіло\n",
            all_decided()
        ),
    );
    write(&source, "src/lib.rs", "pub fn a() {}\npub fn c() {}\n");
    git(&source, &["add", "-A"]);
    git(&source, &["commit", "-q", "-m", "work: the wave"]);
    let clone = home.join("nothing");
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
    let (out, _) = keel(&["check", clone.to_str().unwrap()]);
    assert!(
        out.contains("git names 0002-b-wave"),
        "the silence has a cause, and the cause is what git named and \
         this hand would not take:\n{out}"
    );

    // --- the mark is hex, and the alphabet is part of the measure ---
    //
    // Round three MY-A: widening `a..f` to `a..z` left the whole
    // battery green, so half of the measure was held by nothing.
    let home = project("trunkalphabet", "development", "origin");
    let work = home.join("work");
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\n\n[trust]\nci = \"zzzzzzzzzzzz\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(
        &work,
        &["commit", "-q", "-m", "twelve letters are not twelve hex"],
    );
    let (out, code) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("root key"),
        "twelve characters that are not hex are not keel's mark:\n{out}"
    );
    assert_ne!(code, 0, "and it is refused, not swallowed:\n{out}");

    // --- a plan branch is a branch of the work by its PREFIX, and no
    // wave file is named after it ---
    //
    // Review round four, MY-A: removing the `plan/` and `spike/` half
    // of the guard left the whole battery green, so half the measure
    // was held by nothing. `keel/waves/plan/0001-a-wave.md` does not
    // exist and never will -- the prefix is the fact here.
    let home = project("trunkplanhead", "development", "origin");
    let work = home.join("work");
    git(&work, &["push", "-q", "origin", "plan/0001-a-wave"]);
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/plan/0001-a-wave",
        ],
    );
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("git names plan/0001-a-wave"),
        "a plan branch is never a trunk, and the verdict says what it \
         would not take:\n{out}"
    );

    // --- and the question goes to the REF, not to the working tree ---
    //
    // Review round four, MY-C: asking `HEAD:keel/waves/…` instead of
    // `{ref}:keel/waves/…` left the battery green, while the contract
    // promises the answer does not depend on what is checked out.
    // Here the ref carries the wave file and the checked-out tree
    // does not.
    let home = project("trunkbyref", "development", "origin");
    let work = home.join("work");
    git(&work, &["checkout", "-q", "-b", "0001-a-wave"]);
    git(&work, &["push", "-q", "origin", "0001-a-wave"]);
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/0001-a-wave",
        ],
    );
    // A plan branch cut from the trunk: its tree carries no
    // `keel/waves/0001-a-wave.md` at all.
    git(&work, &["checkout", "-q", "development"]);
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
    assert!(
        !work.join("keel/waves/0001-a-wave.md").exists(),
        "the fixture must really lack the file in the working tree"
    );
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("git names 0001-a-wave"),
        "the branch carries its own wave file AT THE REF, and that is \
         what decides -- not what happens to be checked out:\n{out}"
    );

    // --- the merge fact needs a witness, and a parked clone is not
    // one ---
    //
    // Review 0072 round five measured the worst of the border: a tree
    // parked on `wip` hands a clone that branch as its trunk, HEAD is
    // an ancestor of it, and `keel close` called an UNMERGED wave
    // closed. Before this wave the list of names was asked first and
    // `origin/main` stood right there, so it could not happen.
    let home = project("trunkmergefact", "development", "origin");
    let source = home.join("work");
    git(&source, &["checkout", "-q", "development"]);
    write(
        &source,
        "keel/waves/0004-a-chore.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
            all_decided()
        ),
    );
    write(
        &source,
        "keel/reviews/0004-a-chore.md",
        "рецензія: знахідок нема\n",
    );
    git(&source, &["checkout", "-q", "-b", "0004-a-chore"]);
    git(&source, &["add", "-A"]);
    git(
        &source,
        &["commit", "-q", "-m", "tidy: the chore, not merged anywhere"],
    );
    // The tree is parked on a working name standing exactly here.
    git(&source, &["checkout", "-q", "-b", "wip"]);
    let clone = home.join("parkedfact");
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
    let (said, _) = keel(&["status", clone.to_str().unwrap()]);
    assert!(
        !said.contains("merging closed it"),
        "no merge happened, and a trunk this clone only guessed at is \
         no witness that one did:\n{said}"
    );
    // And the source, where git and the names agree, still sees it.
    let (source_says, _) = keel(&["status", source.to_str().unwrap()]);
    assert!(
        source_says.contains("0004-a-chore"),
        "the source still speaks of the wave:\n{source_says}"
    );

    // --- a bare name in refs/…/HEAD is still an answer ---
    //
    // Review 0072 round six: `refs/remotes/origin/HEAD` pointed at a
    // local ref gives a bare `development` from `symbolic-ref
    // --short`, and cutting the remote's prefix with `?` dropped the
    // answer in silence.
    let home = project("trunkbarehead", "development", "origin");
    let work = home.join("work");
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/heads/development",
        ],
    );
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("trunk: development"),
        "a bare name is the name, not nothing:\n{out}"
    );

    // --- and a limit about the trunk does not leak onto spike/ ---
    //
    // Review 0072 round six: the line was pushed from two places, and
    // the second had no guard, so research branches were told §4.9
    // and §4.12 went unjudged -- where §4.13 says they never are.
    let home = project("trunkspike", "development", "origin");
    let work = home.join("work");
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\ntrunk = \"nosuchbranch\"\n",
    )
    .unwrap();
    git(&work, &["checkout", "-q", "-b", "spike/0010-a-look"]);
    git(&work, &["add", "-A"]);
    git(&work, &["commit", "-q", "-m", "a look"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        !out.contains("no branch of that name"),
        "research is outside the methodology (§4.13): a limit about \
         the trunk has nothing to say here:\n{out}"
    );
}

/// The witness rule, arm by arm, and both reasons a refusal can have.
///
/// Review 0072 round seven measured six payments of rounds five and
/// six held by nothing at all: remove an arm, swap a reason, and the
/// whole battery stayed green. Each side below falls to exactly one
/// of those mutants.
///
/// proves: the-trunk-is-the-one-git-names@3dcdca
#[test]
fn the_merge_fact_is_read_only_with_a_witness() {
    // --- the key is believed on its own ---
    let home = project("witnesskey", "development", "origin");
    let work = home.join("work");
    git(&work, &["checkout", "-q", "development"]);
    write(
        &work,
        "keel/waves/0005-a-chore.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
            all_decided()
        ),
    );
    write(&work, "keel/reviews/0005-a-chore.md", "рецензія\n");
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\ntrunk = \"development\"\n",
    )
    .unwrap();
    git(&work, &["add", "-A"]);
    git(
        &work,
        &["commit", "-q", "-m", "tidy: the chore lands in the trunk"],
    );
    let (said, _) = keel(&["status", work.to_str().unwrap()]);
    assert!(
        said.contains("merging closed it"),
        "the key names the trunk, and that is a witness on its \
         own:\n{said}"
    );

    // --- git alone, with the names silent, is NOT a witness ---
    //
    // The fail-safe side (§4.10), and the cost is named in the
    // verdict: one line of keel.toml brings the fact back. Believing
    // git here gave a clone parked on `wip` an unmerged wave called
    // closed.
    fs::write(
        work.join("keel.toml"),
        "lang = \"en\"\nadapter = \"rust\"\n",
    )
    .unwrap();
    git(&work, &["branch", "-q", "-D", "main"]);
    git(&work, &["add", "-A"]);
    git(
        &work,
        &["commit", "-q", "-m", "tidy: no name to fall back on"],
    );
    let (said, _) = keel(&["status", work.to_str().unwrap()]);
    assert!(
        !said.contains("merging closed it"),
        "git alone is not a witness, and the court says the fact \
         cannot be seen instead of claiming one:\n{said}"
    );
    assert!(
        said.contains("keel.toml"),
        "and it names the one line that brings it back:\n{said}"
    );

    // --- git and the names AGREE: that is a witness ---
    //
    // The ordinary shape, and the one the wave must leave untouched:
    // `origin/HEAD` says `main` and the list says `main` too.
    let home = project("witnessagree", "release", "origin");
    let work = home.join("work");
    git(&work, &["checkout", "-q", "release"]);
    // git names `release` and the list of names finds `main` -- so to
    // make the two AGREE the key is not used and `main` is what both
    // must say: point the remote's HEAD at it.
    git(
        &work,
        &[
            "symbolic-ref",
            "refs/remotes/origin/HEAD",
            "refs/remotes/origin/main",
        ],
    );
    git(&work, &["checkout", "-q", "main"]);
    write(
        &work,
        "keel/waves/0006-a-chore.md",
        &format!(
            "---\ntransforms:\n  tidy:\n    chore: \"дрібниця\"\n    files:\n      - src/lib.rs\n{}---\n\n## transform: tidy\nтіло\n",
            all_decided()
        ),
    );
    write(&work, "keel/reviews/0006-a-chore.md", "рецензія\n");
    git(&work, &["add", "-A"]);
    git(
        &work,
        &["commit", "-q", "-m", "tidy: the chore lands in main"],
    );
    git(&work, &["checkout", "-q", "-b", "beside"]);
    let (said, _) = keel(&["status", work.to_str().unwrap()]);
    assert!(
        said.contains("merging closed it"),
        "two sources saying the same branch are a witness, and this \
         is the shape every ordinary project has:\n{said}"
    );

    // --- a refusal says WHICH of the two reasons it had ---
    let home = project("witnessgone", "development", "origin");
    let work = home.join("work");
    git(
        &work,
        &["update-ref", "-d", "refs/remotes/origin/development"],
    );
    git(&work, &["branch", "-q", "-D", "development"]);
    let (out, _) = keel(&["check", work.to_str().unwrap()]);
    assert!(
        out.contains("is gone") && out.contains("development"),
        "a branch that is gone is not a branch of the work, and the \
         words are not the same words:\n{out}"
    );
}
