//! Scenario tests of wave 0014-field-frame, transform frame-hand:
//! `keel init` sets the methodology's frame in one move -- and
//! never tramples anything that already stands.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("keel-0014i-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

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

fn keel(args: &[&str]) -> (String, String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .output()
        .unwrap();
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().unwrap_or(-1),
    )
}

/// proves: init-births-the-frame@bb34f2 -- holds the NEW-CONCEPT
/// frame row and §8.7: one move births the three keel/ directories
/// (each with .gitkeep), keel.toml with the commented config
/// vocabulary enabling nothing, and the commit-msg hook by gate's
/// hand; every piece is its own "born" line, the tail reminds of
/// §8.7 and leads to keel plan, and check reads the born frame
/// without refusals.
#[test]
fn init_births_the_frame() {
    let dir = sandbox("cleanbirth");
    git(&dir, &["init", "-q", "-b", "main"]);
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "a clean birth is green:\n{out}");
    for piece in ["keel/waves", "keel/contracts", "keel/reviews"] {
        assert!(
            dir.join(piece).join(".gitkeep").is_file(),
            "the directory {piece} is born with .gitkeep:\n{out}"
        );
    }
    let config = fs::read_to_string(dir.join("keel.toml")).unwrap();
    assert!(
        config.contains("# lang") && config.contains("# adapter") && config.contains("# mode"),
        "the config carries the commented vocabulary, enabling nothing:\n{config}"
    );
    // Nothing of the VOCABULARY is enabled -- the defaults stay with
    // config's own words. The [generated] bookkeeping the frame
    // keeps for itself (wave 0022) is not a setting: it enables
    // nothing and changes no behaviour.
    assert!(
        !config.lines().any(|l| {
            let t = l.trim();
            ["version", "lang", "adapter", "mode", "ci"]
                .iter()
                .any(|field| t.starts_with(field))
        }),
        "nothing of the vocabulary is enabled -- defaults stay with config's words:\n{config}"
    );
    let hook = fs::read_to_string(dir.join(".git/hooks/commit-msg")).unwrap();
    assert!(
        hook.contains("keel gate"),
        "the commit-msg hook is gate's own:\n{hook}"
    );
    assert!(
        out.contains("born") && out.contains("keel.toml") && out.contains("commit-msg"),
        "every piece is its own line by name:\n{out}"
    );
    assert!(
        out.contains("squash") && out.contains("rebase"),
        "the §8.7 reminder rides the tail -- the button holds the rule:\n{out}"
    );
    assert!(
        out.contains("keel plan"),
        "the tail leads onwards to the first wave:\n{out}"
    );
    let (out, err, code) = keel(&["check", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 0, "the born frame reads without refusals:\n{out}");
}

/// proves: init-never-tramples@b645d7 -- holds §9.7 and the trample
/// law: a standing piece is "already stands" by name and stays
/// byte-identical (a foreign keel.toml is a fact, not a content
/// judgement); a foreign commit-msg hook is a refusal aloud, never
/// a rewrite; without git the hook line refuses with its reason
/// while the rest of the frame still lands; failed pieces redden
/// the exit.
#[test]
fn init_never_tramples() {
    let dir = sandbox("standing");
    git(&dir, &["init", "-q", "-b", "main"]);
    // The foreign config enables English so the asserted words stay
    // readable; a uk one turns the whole report Ukrainian -- seen
    // live, the honest behaviour of the config court.
    let foreign_config = "# somebody else's config\nlang = \"en\"\n";
    write(&dir, "keel.toml", foreign_config);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    let foreign_hook = "#!/bin/sh\nexit 0\n";
    write(&dir, ".git/hooks/commit-msg", foreign_hook);
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 1,
        "a piece that did not stand reddens the exit:\n{out}"
    );
    // The foreign config keeps every byte of its own: the frame may
    // add its bookkeeping line (wave 0022, as keel trust does for
    // [trust]) and nothing else -- no setting, no comment, no order.
    let after = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let mine: String = after
        .split("[generated]")
        .next()
        .unwrap()
        .trim_end()
        .to_string();
    assert_eq!(
        mine,
        foreign_config.trim_end(),
        "the foreign keel.toml keeps its own bytes:\n{out}"
    );
    assert_eq!(
        fs::read_to_string(dir.join(".git/hooks/commit-msg")).unwrap(),
        foreign_hook,
        "the foreign hook stays byte-identical:\n{out}"
    );
    assert!(
        out.contains("already stands"),
        "the standing pieces are said by name:\n{out}"
    );
    assert!(
        out.contains("not ours"),
        "the foreign hook is a refusal aloud, never a rewrite:\n{out}"
    );
    assert!(
        dir.join("keel/contracts/.gitkeep").is_file()
            && dir.join("keel/reviews/.gitkeep").is_file(),
        "the rest of the frame still lands piece by piece:\n{out}"
    );

    // Without git the hook line refuses with its reason -- the
    // directories and the config still arrive.
    let dir = sandbox("gitless");
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(code, 1, "the hookless frame is honest red:\n{out}");
    assert!(
        dir.join("keel/waves/.gitkeep").is_file() && dir.join("keel.toml").is_file(),
        "the frame lands even where git is silent:\n{out}"
    );

    // The second run over a full frame changes nothing and is green.
    let dir = sandbox("secondrun");
    git(&dir, &["init", "-q", "-b", "main"]);
    let (_, _, code) = keel(&["init", dir.to_str().unwrap()]);
    assert_eq!(code, 0, "the first run is green");
    let config_before = fs::read_to_string(dir.join("keel.toml")).unwrap();
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let out = format!("{out}{err}");
    assert_eq!(
        code, 0,
        "the second run is green -- everything stands:\n{out}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("keel.toml")).unwrap(),
        config_before,
        "the second run changes not a byte:\n{out}"
    );
    assert!(
        !out.contains("born:"),
        "nothing is claimed born where everything stood:\n{out}"
    );
}

/// proves: init-never-tramples@b645d7 -- the second birth out of
/// review 0014 (R-1/R-2): a broken keel.toml does not steer the
/// call silently -- the config refusal is said aloud and the frame
/// still lands in the default language; and a standing directory
/// missing its .gitkeep is fed the file with its own word --
/// "builds what is missing" is true there too.
#[test]
fn init_never_tramples_second_birth() {
    // R-1: the broken config is a word, never a silent default.
    let dir = sandbox("brokenconfig");
    git(&dir, &["init", "-q", "-b", "main"]);
    let broken = "lang = [not a string\n";
    write(&dir, "keel.toml", broken);
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let all = format!("{out}{err}");
    // The frame still lands around the broken config -- that is
    // what the rows below prove -- and the exit reddens, because a
    // piece did not stand. This wave's own words, and 0014's: "the
    // count of pieces that did not stand reddens the exit, and the
    // rest of the frame is delivered regardless". Until wave 0024
    // the generated integrations were not counted when the config
    // refused, so a project with a broken keel.toml got a green
    // frame: a refusal aloud with a green exit is a half-truth.
    assert_eq!(
        code, 1,
        "the exit reddens for the piece that did not stand:\n{all}"
    );
    assert!(
        all.contains("keel.toml") && all.contains("refusal"),
        "the config refusal is said aloud, not swallowed (R-1):\n{all}"
    );
    assert!(
        all.contains("not judged") || all.contains("не судяться"),
        "and the frame says which piece it could not judge:\n{all}"
    );
    assert!(
        dir.join("keel/waves/.gitkeep").is_file(),
        "the frame landed in the default language:\n{all}"
    );
    assert_eq!(
        fs::read_to_string(dir.join("keel.toml")).unwrap(),
        broken,
        "the broken config itself is not touched:\n{all}"
    );

    // R-2: a standing directory without .gitkeep is fed the file --
    // a new empty file tramples nothing, and the empty directory
    // now outlives git.
    let dir = sandbox("feedkeep");
    git(&dir, &["init", "-q", "-b", "main"]);
    fs::create_dir_all(dir.join("keel/waves")).unwrap();
    let (out, err, code) = keel(&["init", dir.to_str().unwrap()]);
    let all = format!("{out}{err}");
    assert_eq!(code, 0, "feeding is green:\n{all}");
    assert!(
        dir.join("keel/waves/.gitkeep").is_file(),
        "the standing directory is fed its .gitkeep (R-2):\n{all}"
    );
    assert!(
        all.contains("fed"),
        "the feeding is its own word, not a painted 'already stands':\n{all}"
    );
}
