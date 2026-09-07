//! Scenario test of wave 0041: the launcher runs what the project
//! pinned.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;
use common::versions::{install, run_in, world};

use std::fs;
use std::process::Command;

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

/// The shapes review 0041 measured and the wave had not: a pin two
/// versions answer to, a pin in the other legal TOML quotes, a ref
/// with a slash in it, a positional project path, and a checksum
/// record that is gone or empty.
#[test]
fn the_launcher_is_never_silently_wrong() {
    let dir = sandbox("neverwrong");
    let w = world(&dir);
    install(&w, Some(&w.old_ref));
    install(&w, Some("v2.0.0"));
    install(&w, Some("v2.0.0-again"));

    let project = |name: &str, body: &str| {
        let at = dir.join(name);
        fs::create_dir_all(&at).unwrap();
        fs::write(at.join("keel.toml"), body).unwrap();
        at
    };

    // R-1: two versions answer to "2.0.0". Picking one by glob order
    // is the silent wrong binary this launcher exists to prevent.
    let ambiguous = project("ambiguous", "lang = \"uk\"\nversion = \"2.0.0\"\n");
    let (said, code) = run_in(&w, &ambiguous, &["--version"]);
    assert_eq!(code, 2, "an ambiguous pin refuses:\n{said}");
    assert!(
        said.contains("v2.0.0") && said.contains("v2.0.0-again"),
        "naming both refs that answer to it:\n{said}"
    );

    // And the ref is unique, so pinning it runs exactly that one --
    // which is the only pin that can work on keel itself, where every
    // ref answers 0.1.0.
    let by_ref = project("byref", "lang = \"uk\"\nversion = \"v2.0.0-again\"\n");
    let (said, code) = run_in(&w, &by_ref, &["--version"]);
    assert_eq!(code, 0, "a pin naming a ref runs it:\n{said}");
    assert!(said.contains("keel 2.0.0"), "the right one:\n{said}");

    // R-2: single quotes are a legal TOML string, and the tool reads
    // them. A pin the launcher cannot read is a refusal, never a
    // shrug that runs whatever is current.
    let quoted = project("quoted", "lang = \"uk\"\nversion = 'v1.0.0'\n");
    let (said, code) = run_in(&w, &quoted, &["--version"]);
    assert_eq!(code, 0, "the other legal quotes are read too:\n{said}");
    assert!(
        said.contains(&format!("keel {}", w.old_version)),
        "and name the same version:\n{said}"
    );
    let broken = project("brokenpin", "lang = \"uk\"\nversion = nonsense\n");
    let (said, code) = run_in(&w, &broken, &["--version"]);
    assert_eq!(code, 2, "a version line nobody can read refuses:\n{said}");
    assert!(
        !said.contains(&format!("keel {}", w.new_version)),
        "rather than running another version in silence:\n{said}"
    );

    // R-4: a ref with a slash is a real ref -- `plan/0041-...` is a
    // branch of keel's own repository.
    install(&w, Some("rel/2.0"));
    let sloped = project("sloped", "lang = \"uk\"\nversion = \"rel/2.0\"\n");
    let (said, code) = run_in(&w, &sloped, &["--version"]);
    assert_eq!(code, 0, "a ref with a slash installs and runs:\n{said}");
    assert!(said.contains("keel 2.0.0"), "as itself:\n{said}");

    // R-5: a positional project path is the form keel's own probes
    // use, and the launcher used to ignore it.
    let plain = project("plainhere", "lang = \"uk\"\n");
    let old_one = project(
        "oldone",
        &format!("lang = \"uk\"\nversion = \"{}\"\n", w.old_ref),
    );
    let (said, _) = run_in(&w, &plain, &["version", old_one.to_str().unwrap()]);
    assert!(
        said.contains(&format!("keel {}", w.old_version)),
        "the pin read is the pin of the project named on the line:\n{said}"
    );

    // R-12: the pin is found by walking UP from the project, and the
    // first `version =` line is the one read -- a keel.toml is a
    // config, and a second line further down is not an override.
    // Deep inside a project pinned to the OLD version, so falling
    // through to whatever is current would be visible.
    let deep = dir.join("oldpinned").join("a").join("b");
    fs::create_dir_all(&deep).unwrap();
    fs::write(
        dir.join("oldpinned").join("keel.toml"),
        format!("lang = \"uk\"\nversion = \"{}\"\n", w.old_ref),
    )
    .unwrap();
    let (said, code) = run_in(&w, &deep, &["--version"]);
    assert_eq!(code, 0, "the pin of the project above is found:\n{said}");
    assert!(
        said.contains(&format!("keel {}", w.old_version)),
        "and honoured, rather than falling through to whatever is \
         current:\n{said}"
    );

    // The pin is the TOP-LEVEL version, and a `version` key inside a
    // section is not one: a line-based reader that takes the last
    // match would take the wrong line.
    let sectioned = project(
        "sectioned",
        &format!(
            "lang = \"uk\"\nversion = \"{}\"\n\n[generated]\nversion = \"v2.0.0\"\n",
            w.old_ref
        ),
    );
    let (said, _) = run_in(&w, &sectioned, &["--version"]);
    assert!(
        said.contains(&format!("keel {}", w.old_version)),
        "the pin is the first version line, at the top level:\n{said}"
    );

    // R-11: the launcher hands the arguments on, whole.
    let (said, _) = run_in(&w, &plain, &["check", "--json", "a b"]);
    assert!(
        said.contains("args: check --json a b"),
        "every argument reaches the binary, spaces and all:\n{said}"
    );

    // R-3: a checksum record that is gone, or one that says nothing,
    // is not a pass -- the installer itself wrote an empty one on a
    // machine with no sha256 tool, and the gate turned off in silence.
    let home = w.home.join("versions").join(&w.old_ref);
    let swapped = "#!/bin/sh\necho \"a different tool\"\n";
    fs::write(home.join("keel"), swapped).unwrap();
    fs::remove_file(home.join(".keel-sum")).unwrap();
    let (said, code) = run_in(&w, &quoted, &["--version"]);
    assert_eq!(code, 2, "a missing checksum record refuses:\n{said}");
    assert!(
        !said.contains("a different tool"),
        "and runs nothing:\n{said}"
    );
    fs::write(home.join(".keel-sum"), "\n").unwrap();
    let (said, code) = run_in(&w, &quoted, &["--version"]);
    assert_eq!(code, 2, "and an empty one refuses too:\n{said}");
    assert!(
        !said.contains("a different tool"),
        "and runs nothing:\n{said}"
    );
}

/// proves: the-launcher-runs-where-the-hook-runs@fac3ce -- measured by
/// live use on 2026-09-07, and only there: with no launcher on PATH
/// the hook keel installs falls back to `current_exe`, which is a
/// binary and needs no PATH at all. The moment a person installs the
/// launcher -- the ordinary shape, the one `install.sh` makes -- the
/// hook of every repository leads to a SHELL SCRIPT, and a graphical
/// client runs hooks with a PATH of its own.
#[test]
fn the_launcher_runs_where_the_hook_runs() {
    let dir = sandbox("launcherpath");
    let w = world(&dir);
    install(&w, Some(&w.old_ref));
    let project = dir.join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(
        project.join("keel.toml"),
        format!("lang = \"en\"\nversion = \"{}\"\n", w.old_ref),
    )
    .unwrap();

    // The PATH a graphical client hands its hooks: no keel on it, and
    // none of the everyday tools either. Measured on the author's
    // machine before this wave: `dirname: command not found`, and the
    // commit court did not run at all.
    let out = Command::new(w.bin.join("keel"))
        .args(["--version"])
        .current_dir(&project)
        .env("KEEL_HOME", &w.home)
        .env("PATH", "/nonexistent")
        .output()
        .unwrap();
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !said.contains("command not found"),
        "the launcher brings the tools it needs: a hook run by a \
         client with its own PATH is the shape keel itself installs, \
         and there `command not found` is the whole court:\n{said}"
    );
    assert!(
        said.contains("keel"),
        "and it answers as keel, not as a broken script:\n{said}"
    );

    // And what a person put in front STAYS in front: the launcher
    // adds, never replaces. A stub `uname` earlier on PATH must still
    // be the one that answers.
    let mine = dir.join("mine");
    fs::create_dir_all(&mine).unwrap();
    fs::write(
        mine.join("uname"),
        "#!/bin/sh\necho \"the one the person chose\"\n",
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(mine.join("uname"), fs::Permissions::from_mode(0o755)).unwrap();
    }
    let out = Command::new("/bin/sh")
        .arg("-c")
        .arg("uname")
        .env("PATH", format!("{}:/usr/bin:/bin", mine.display()))
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("the one the person chose"),
        "the shape this wave writes keeps the person's own PATH in \
         front -- /usr/bin goes to the END, or the launcher would be \
         choosing which binaries a machine runs"
    );
}
