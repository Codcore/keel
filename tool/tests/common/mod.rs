//! One hand for every probe's sandbox (wave 0030).
//!
//! Before this, `sandbox()` lived in twenty-six copies and each one
//! removed its directory ON ENTRY -- cleaning up after the previous
//! run of the same name and never after itself. By wave 0029 that had
//! left 11,511 directories holding 20 GB in /tmp, and the disk hit
//! 100% in the middle of a wave.
//!
//! Every test binary compiles this module separately, so a hand one
//! probe does not call is dead code there and nowhere else: the
//! allow below says that once, instead of each file pretending to
//! use what it does not.
#![allow(dead_code)]

use std::path::{Path, PathBuf};

/// A fresh sandbox for one case -- the shape every probe calls.
pub fn sandbox(name: &str) -> Sandbox {
    Sandbox::new(name)
}

/// The same, already carrying the methodology's own two directories.
/// Twenty-one of the twenty-six probes built these inside their own
/// copy of `sandbox()`, so the shape is kept rather than pushed into
/// twenty-one call sites.
pub fn keel_sandbox(name: &str) -> Sandbox {
    let sandbox = Sandbox::new(name);
    std::fs::create_dir_all(sandbox.join("keel/waves")).unwrap();
    std::fs::create_dir_all(sandbox.join("keel/contracts")).unwrap();
    // Six of the twenty-one also made this one, and dropping it
    // narrowed the fixture in silence: "keel/reviews is absent"
    // proves a slightly different world from "keel/reviews is empty"
    // (review 0030 R-6).
    std::fs::create_dir_all(sandbox.join("keel/reviews")).unwrap();
    sandbox
}

/// The branch does the work its wave declared, and does it lawfully.
///
/// Since wave 0068 the closing court is not narrower than `keel
/// check`: a wave whose file is never touched (§4.4), whose promise
/// was never born red (§6.3), or whose transform no commit closes
/// (§6.2) is unfinished, and the court says so before it spends a
/// battery. A sandbox that calls `keel close` therefore has to BE
/// what it depicts -- a branch that did its work -- or the probe
/// measures the court's complaint about the fixture instead of the
/// thing it came for.
///
/// The declaration is READ, not passed in: the hand asks
/// `keel::docs::scan` for the wave the branch is named after and
/// keeps exactly what that wave says. A fixture whose wave file
/// changes stays lawful without anybody remembering a second place --
/// and a hand given the file by hand is precisely how the first cut
/// of this wave produced a sandbox that touched one file and declared
/// another.
///
/// A branch that names no wave is left alone: several probes are
/// about that very state, and inventing work there would erase the
/// thing they measure.
pub fn did_the_work(dir: &Path) {
    let branch = std::process::Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_default();
    let Ok(scan) = keel::docs::scan(dir) else {
        return;
    };
    let Some(wave) = scan.waves.iter().find(|wave| wave.slug == branch) else {
        return;
    };
    // Red first, and every promise of it: a tagged test the branch
    // never saw fail proves nothing (§6.3), and the sandboxes write
    // the test and its tag in one breath.
    for (_, transform) in &wave.transforms {
        if let keel::docs::TransformKind::Implements(promises) = &transform.kind {
            for promise in promises {
                in_git(dir, &["add", "-A"]);
                in_git(
                    dir,
                    &[
                        "commit",
                        "-q",
                        "--allow-empty",
                        "-m",
                        &format!("red: {promise}"),
                    ],
                );
            }
        }
    }
    // Then the work, one commit per transform under its own slug:
    // that is what closes a transform (§6.2), and a commit named
    // anything else leaves it open however much the files changed.
    for (slug, transform) in &wave.transforms {
        // `one new in <dir>/` promises ONE new file there (§4.1), and
        // two such lines over one directory promise two. The hand
        // counts the lines per directory rather than writing one file
        // per line into the same name -- which is how the first cut
        // kept a wave saying "2 new" while the branch added 1 (review
        // R-11).
        let mut new_in: std::collections::BTreeMap<String, usize> =
            std::collections::BTreeMap::new();
        for row in &transform.files {
            match row {
                keel::docs::ScopeLine::Path(path) => append_to(&dir.join(path)),
                keel::docs::ScopeLine::OneNewIn(place) => {
                    *new_in
                        .entry(place.trim_end_matches('/').to_string())
                        .or_default() += 1;
                }
            }
        }
        for (place, count) in new_in {
            let place = dir.join(place);
            std::fs::create_dir_all(&place).unwrap();
            for nth in 0..count {
                append_to(&place.join(format!("from-the-branch-{nth}.txt")));
            }
        }
        in_git(dir, &["add", "-A"]);
        in_git(
            dir,
            &[
                "commit",
                "-q",
                "--allow-empty",
                "-m",
                &format!("{slug}: the files this transform declared"),
            ],
        );
    }
}

/// One line at the end of a file, with the comment leader its tongue
/// uses: the declared file is source the runner will compile.
///
/// A declared name may be a DIRECTORY -- §4.3 allows writing one --
/// and this hand cannot satisfy such a row. Measured (review R2-3):
/// §4.4 reads the declared name LITERALLY, so a file written inside
/// `lib/` does not answer a declared `lib`; it answers nothing and
/// adds a drift finding of its own. Writing to the directory itself
/// is `IsADirectory`, which used to end the probe with a panic from
/// inside this hand (review R-11).
///
/// So a directory that already stands is left alone, and the row
/// stays unanswered -- which is the truth about it. A sandbox that
/// wants its declaration answered names a file, or says `one new in
/// <dir>/`, which is what a branch adding a file there actually does.
/// A name that is no directory yet becomes a FILE of that name: that
/// is precisely what §4.4 reads.
fn append_to(path: &Path) {
    if path.is_dir() {
        return;
    }
    let leader = match path.extension().and_then(|kind| kind.to_str()) {
        Some("rs" | "js" | "ts" | "mjs" | "cjs" | "jsx" | "tsx") => "//",
        Some("txt" | "md") => "",
        _ => "#",
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    let mut body = std::fs::read_to_string(path).unwrap_or_default();
    body.push_str(&format!("\n{leader} the work of this branch\n"));
    std::fs::write(path, body).unwrap();
}

/// git, inside a sandbox, with an identity of its own: a machine
/// running the battery may have no global one.
pub fn in_git(dir: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .args(["-c", "user.email=probe@keel", "-c", "user.name=probe"])
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// A sandbox that removes itself when its test ends.
///
/// The cleanup hangs on `Drop`, so it happens on the ordinary way out
/// AND while a panic unwinds -- unlike a sweep written as the last
/// line of a test body, which a panic skips. But a FAILING test keeps
/// its sandbox: that is the one a person opens to find out what
/// happened, and a cleanup that eats the evidence is worse than a
/// leak.
#[must_use = "a sandbox dropped at once takes its directory with it"]
pub struct Sandbox {
    path: PathBuf,
}

impl Sandbox {
    /// A fresh directory named by this process and this case.
    ///
    /// The path is built here, from this prefix and the process id:
    /// the hand can only ever remove what it made itself. That is the
    /// line reviewer 0026 crossed by hand when he deleted ten
    /// thousand directories belonging to other sessions.
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("keel-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl std::ops::Deref for Sandbox {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<std::ffi::OsStr> for Sandbox {
    fn as_ref(&self) -> &std::ffi::OsStr {
        self.path.as_os_str()
    }
}

impl AsRef<Path> for Sandbox {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        if std::thread::panicking() {
            // The evidence stays, and says where it is.
            eprintln!("sandbox kept for inspection: {}", self.path.display());
            return;
        }
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

pub mod versions;

/// Whether this machine can judge a case at all -- and, when it
/// cannot, what it lacked, in words a person reads in a log.
///
/// Wave 0044: three probes carried `assert!(have_mix(), ...)` while
/// the head of their own file said they "say so and stop rather than
/// pretending". An assert is a failure, not a stop -- so on a runner
/// with no elixir the battery went red and `keel close` called two
/// proven scenarios unproven. A probe that has judged NOTHING must
/// not report that it judged and found fault.
pub enum Machine {
    Has,
    Lacks(String),
}

impl Machine {
    /// Can this case be judged here? Where it cannot, the reason is
    /// said aloud first -- to stderr, which is where `cargo test`
    /// shows it and where a runner's log keeps it -- and the caller
    /// returns having judged nothing, which is not the same as
    /// having judged and found nothing wrong.
    ///
    /// Under a DECLARED runner it is not a stop either: GitHub Actions
    /// and its kin set `CI`, and a runner that promised the whole
    /// battery and lacks a tongue is the runner's fault, not the
    /// machine's shape -- eleven probes skipped themselves on this
    /// repository's own runner in silence, and its battery was
    /// narrower than the courts' (global review 2026-09-06, tests
    /// R-10; wave 0053). There a missing tool fails the probe by name.
    pub fn ready(self) -> bool {
        match self {
            Machine::Has => true,
            Machine::Lacks(why) => {
                if std::env::var_os("CI").is_some_and(|set| !set.is_empty()) {
                    panic!(
                        "on a declared runner (CI is set) a missing tool is a fall, \
                         not a skip: {why}"
                    );
                }
                eprintln!("skipped, and said aloud: {why}");
                false
            }
        }
    }
}

/// Is this runner on the machine? Asked by running it, not by
/// looking for a file: a binary on PATH that cannot start is not a
/// runner this probe can use.
pub fn machine_has(tool: &str) -> Machine {
    match std::process::Command::new(tool).arg("--version").output() {
        Ok(out) if out.status.success() => Machine::Has,
        Ok(out) => Machine::Lacks(format!(
            "`{tool} --version` left with {} -- the tool is on PATH and \
             will not run, so this probe judged nothing",
            out.status.code().unwrap_or(-1)
        )),
        Err(why) => Machine::Lacks(format!(
            "`{tool}` is not on this machine ({why}) -- this probe \
             judged nothing"
        )),
    }
}
