//! Scenario test of wave 0041: the launcher runs what the project
//! pinned.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;
use common::versions::{install, run_in, world};

use std::fs;

/// proves: the-launcher-runs-what-the-project-pinned@09cc35 -- the
/// concept asks the `keel` command to read a project's pin and run
/// that version, and to refuse honestly with a ready command when it
/// cannot. Measured before the wave: the pin only FORBADE -- every
/// court refused while it differed, and a person had to close the gap
/// by hand.
#[test]
fn the_launcher_runs_what_the_project_pinned() {
    let dir = sandbox("launcher");
    let w = world(&dir);
    install(&w, Some(&w.old_ref));
    install(&w, Some("v2.0.0"));

    // A project pinned to the older version gets the older version,
    // though the newer one was installed last.
    let old_project = dir.join("old-project");
    fs::create_dir_all(&old_project).unwrap();
    fs::write(
        old_project.join("keel.toml"),
        format!("lang = \"uk\"\nversion = \"{}\"\n", w.old_version),
    )
    .unwrap();
    let (said, code) = run_in(&w, &old_project, &["--version"]);
    assert_eq!(code, 0, "the pinned version runs:\n{said}");
    assert!(
        said.contains(&format!("keel {}", w.old_version)),
        "and it is the one the project pinned, not the one installed \
         last:\n{said}"
    );

    // And a project pinned to the newer one gets that.
    let new_project = dir.join("new-project");
    fs::create_dir_all(&new_project).unwrap();
    fs::write(
        new_project.join("keel.toml"),
        format!("lang = \"uk\"\nversion = \"{}\"\n", w.new_version),
    )
    .unwrap();
    let (said, _) = run_in(&w, &new_project, &["--version"]);
    assert!(
        said.contains(&format!("keel {}", w.new_version)),
        "each project on its own version, at the same time:\n{said}"
    );

    // A pin nobody installed: a refusal naming what IS here and the
    // command that brings what is not -- never another version run in
    // silence.
    let stranger = dir.join("stranger");
    fs::create_dir_all(&stranger).unwrap();
    fs::write(
        stranger.join("keel.toml"),
        "lang = \"uk\"\nversion = \"7.7.7\"\n",
    )
    .unwrap();
    let (said, code) = run_in(&w, &stranger, &["--version"]);
    assert_eq!(code, 2, "an uninstalled pin refuses:\n{said}");
    assert!(
        said.contains("7.7.7"),
        "naming the pin that was asked for:\n{said}"
    );
    assert!(
        said.contains(&w.old_version) && said.contains(&w.new_version),
        "and what is installed here instead:\n{said}"
    );
    assert!(
        said.contains("KEEL_REF="),
        "with the command that installs it (NEW-CONCEPT: an honest \
         refusal carries the ready command):\n{said}"
    );
    assert!(
        !said.contains(&format!("keel {}", w.new_version)),
        "and no other version ran -- the wrong binary in silence is \
         worse than a refusal:\n{said}"
    );

    // A project with no pin at all gets the version that stands, and
    // is asked nothing new.
    let plain = dir.join("plain");
    fs::create_dir_all(&plain).unwrap();
    fs::write(plain.join("keel.toml"), "lang = \"uk\"\n").unwrap();
    let (said, code) = run_in(&w, &plain, &["--version"]);
    assert_eq!(code, 0, "an unpinned project just runs:\n{said}");
    assert!(
        said.contains(&format!("keel {}", w.new_version)),
        "on the version that was installed last:\n{said}"
    );

    // `-C <dir>` decides which project's pin is read: the launcher
    // must find the same project the tool will judge.
    let (said, _) = run_in(
        &w,
        &plain,
        &["-C", old_project.to_str().unwrap(), "--version"],
    );
    assert!(
        said.contains(&format!("keel {}", w.old_version)),
        "the pin read is the pin of the project named by -C:\n{said}"
    );

    // A binary that is not the one that was installed is named, not
    // run: the checksum is checked before the hand-over.
    let home = w.home.join("versions").join(&w.old_ref);
    fs::write(home.join("keel"), "#!/bin/sh\necho \"a different tool\"\n").unwrap();
    let (said, code) = run_in(&w, &old_project, &["--version"]);
    assert_eq!(code, 2, "a swapped binary refuses:\n{said}");
    assert!(
        !said.contains("a different tool"),
        "and does not run:\n{said}"
    );
}
