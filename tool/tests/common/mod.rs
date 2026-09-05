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
