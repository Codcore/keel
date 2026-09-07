//! Scenario test of wave 0061: the court asks git once for the same
//! answer.
//!
//! Measured before the plan, on keel's own tree: `keel check` spawned
//! **4604** git processes and needed **387** different answers -- 92%
//! of them repeats, and 3546 of those a `git show` of a file at a
//! commit, the same one up to 72 times over. At ~13 ms a process that
//! is the whole minute the court takes.
//!
//! The probe asks the only question that can hold this: HOW MANY
//! processes, against how many different answers. It asks it the way
//! the fault was measured -- with a `git` of its own on PATH that
//! writes down every call and then hands the real one through.

mod common;

use common::keel_sandbox;
use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn executable(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
}

/// A `git` that counts. It writes its whole argument line to `log`
/// and then execs the real one, so what the court gets back is the
/// truth and what the probe reads is the price.
fn counting_git(dir: &Path, log: &Path) -> std::path::PathBuf {
    let shim = dir.join("shim");
    std::fs::create_dir_all(&shim).unwrap();
    let real = ["/usr/bin/git", "/bin/git", "/opt/homebrew/bin/git"]
        .into_iter()
        .find(|c| Path::new(c).is_file())
        .unwrap_or("/usr/bin/git");
    std::fs::write(
        shim.join("git"),
        format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" >> '{}'\nexec {real} \"$@\"\n",
            log.display()
        ),
    )
    .unwrap();
    executable(&shim.join("git"));
    shim
}

fn keel_counting(dir: &Path, shim: &Path, args: &[&str]) -> (String, i32) {
    let mut all: Vec<&str> = args.to_vec();
    all.push(dir.to_str().unwrap());
    let path = format!(
        "{}:{}",
        shim.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(&all)
        .env("PATH", path)
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

fn decided(except: &[&str]) -> String {
    let mut out = String::from("decisions:\n");
    for cut in keel::graph::cuts() {
        if !except.contains(cut) {
            out.push_str(&format!("  {cut}: \"не про цю пісочницю\"\n"));
        }
    }
    out
}

/// A tree with HISTORY: one contract that many waves lean on, and a
/// history deep enough that asking per commit costs more than asking
/// per answer -- which is exactly the shape keel's own tree has.
fn project(name: &str) -> common::Sandbox {
    let dir = keel_sandbox(name);
    std::fs::write(dir.join("keel.toml"), "lang = \"uk\"\nadapter = \"rust\"\n").unwrap();
    std::fs::write(
        dir.join("Cargo.toml"),
        "[package]\nname = \"toy\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::write(dir.join("src/lib.rs"), "pub fn a() {}\n").unwrap();
    std::fs::write(
        dir.join("keel/contracts/kept.md"),
        "---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту, редакція 0\n",
    )
    .unwrap();
    git(&dir, &["init", "-q", "-b", "main"]);
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "base"]);
    // A history: every commit is another revision of the same
    // contract, and only the LAST one is what the waves record.
    for step in 1..=12 {
        std::fs::write(
            dir.join("keel/contracts/kept.md"),
            format!("---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту, редакція {step}\n"),
        )
        .unwrap();
        git(&dir, &["add", "-A"]);
        git(&dir, &["commit", "-q", "-m", &format!("kept: {step}")]);
    }
    dir
}

/// proves: the-court-asks-git-once-for-the-same-answer@3d7fb6
#[test]
fn the_court_asks_git_once_for_the_same_answer() {
    let dir = project("gitmemory");
    let rev = keel::rev::contract_rev(&dir.join("keel/contracts/kept.md")).unwrap();
    // Eight closed waves, every one leaning on the same contract at
    // the same revision: eight questions with ONE answer.
    for n in 1..=8 {
        let slug = format!("000{n}-a-wave");
        std::fs::write(
            dir.join(format!("keel/waves/{slug}.md")),
            format!(
                "---\ntransforms:\n  work{n}:\n    chore: \"дрібниця\"\n    contracts: [kept@{rev}]\n    files:\n      - src/lib.rs\n{}---\n\n## transform: work{n}\nтіло\n",
                decided(&[])
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join(format!("keel/reviews/{slug}.md")),
            "# Рецензія\n\nok\n",
        )
        .unwrap();
    }
    git(&dir, &["add", "-A"]);
    git(&dir, &["commit", "-q", "-m", "eight waves lean on it"]);

    let log = dir.join("git-calls.txt");
    let shim = counting_git(&dir, &log);
    let (said, _) = keel_counting(&dir, &shim, &["check"]);
    let calls: Vec<String> = std::fs::read_to_string(&log)
        .unwrap_or_default()
        .lines()
        .map(|line| line.to_string())
        .collect();
    let mut unique: Vec<&String> = calls.iter().collect();
    unique.sort();
    unique.dedup();
    assert!(
        !calls.is_empty(),
        "the shim was the git the court used:\n{said}"
    );
    // The promise, in the only shape that can hold it: a process per
    // ANSWER, not per question. Some slack for what git is asked
    // once and answers about the working tree -- but nowhere near the
    // 12x of the measurement that made this wave.
    assert!(
        calls.len() <= unique.len() * 2,
        "the court asks git {} times for {} different answers -- a \
         process per question is the minute this wave exists to \
         remove (measured before the plan: 4604 calls, 387 answers)\n\
         most repeated:\n{}",
        calls.len(),
        unique.len(),
        {
            let mut counted: std::collections::BTreeMap<&String, usize> = Default::default();
            for call in &calls {
                *counted.entry(call).or_default() += 1;
            }
            let mut rows: Vec<(&&String, &usize)> = counted.iter().collect();
            rows.sort_by(|a, b| b.1.cmp(a.1));
            rows.iter()
                .take(3)
                .map(|(call, times)| format!("  {times}× {call}"))
                .collect::<Vec<_>>()
                .join("\n")
        }
    );

    // And the SAME contract's history is walked once, not once per
    // wave that names it.
    let shows = calls.iter().filter(|c| c.contains(" show ")).count();
    assert!(
        shows <= 14,
        "the history of one file is read once -- thirteen revisions \
         plus a margin, not thirteen per wave that leans on it: {shows} \
         `show` calls"
    );

    // Nothing about the verdict itself changed: the court still says
    // what it said, and the memory is invisible in the words.
    assert!(
        said.contains("посилань на контракти звірено: 8"),
        "and every one of the eight references is still judged:\n{said}"
    );

    // --- and a memory that remembered a FAILURE would be worse than
    // no memory at all ---
    //
    // §5.6 judges a closed wave's old record against HISTORY, so "git
    // could not say" is the answer that decides. A single stumble --
    // a busy index.lock, a moment of trouble -- must cost exactly the
    // question it broke, and nothing after it. Remember the failure,
    // and one stumble decides every later question the same way,
    // silently and once per process.
    //
    // Two closed waves, two DIFFERENT old revisions, and a git that
    // fails the first history question and answers truly ever after:
    // exactly one finding, not two.
    let dir = project("gitstumble");
    let older = dir.join("older.md");
    let mut revs: Vec<String> = Vec::new();
    for step in [4, 5] {
        std::fs::write(
            &older,
            format!("---\nmodule: toy\nexports: [\"pub fn a()\"]\n---\n\nтіло контракту, редакція {step}\n"),
        )
        .unwrap();
        revs.push(keel::rev::contract_rev(&older).unwrap());
    }
    std::fs::remove_file(&older).unwrap();
    for (n, rev) in revs.iter().enumerate() {
        let slug = format!("000{}-a-wave", n + 1);
        std::fs::write(
            dir.join(format!("keel/waves/{slug}.md")),
            format!(
                "---\ntransforms:\n  work{n}:\n    chore: \"дрібниця\"\n    contracts: [kept@{rev}]\n    files:\n      - src/lib.rs\n{}---\n\n## transform: work{n}\nтіло\n",
                decided(&[])
            ),
        )
        .unwrap();
        std::fs::write(
            dir.join(format!("keel/reviews/{slug}.md")),
            "# Рецензія\n\nok\n",
        )
        .unwrap();
    }
    git(&dir, &["add", "-A"]);
    git(
        &dir,
        &["commit", "-q", "-m", "two closed waves, two old records"],
    );

    let shim = dir.join("stumble");
    std::fs::create_dir_all(&shim).unwrap();
    let once = dir.join("stumbled-once");
    std::fs::write(
        shim.join("git"),
        format!(
            "#!/bin/sh\ncase \"$*\" in *'log --format=%H'*) if [ ! -f '{}' ]; then : > '{}'; echo 'fatal: a moment of trouble' >&2; exit 128; fi ;; esac\nexec /usr/bin/git \"$@\"\n",
            once.display(),
            once.display()
        ),
    )
    .unwrap();
    executable(&shim.join("git"));
    let (stumbled, _) = keel_counting(&dir, &shim, &["check"]);
    assert!(once.exists(), "the shim really did stumble:\n{stumbled}");
    let broken = stumbled
        .lines()
        .filter(|line| line.contains("червоне") && line.contains("kept@"))
        .count();
    assert_eq!(
        broken, 1,
        "one stumble costs ONE question and nothing after it -- a \
         remembered failure would decide both waves the same way, \
         quietly, once per process:\n{stumbled}"
    );
    // And the direction of the loss is the safe one: the court gets
    // STRICTER where git could not answer, never quieter.
    assert!(
        stumbled.contains("стара редакція законна (§5.6)"),
        "and where history could not testify the court says so and \
         reddens, rather than passing the wave in silence:\n{stumbled}"
    );
}
