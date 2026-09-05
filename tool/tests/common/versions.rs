//! A world for judging the installer: a repository shaped like
//! keel's, with two versions, and a stub `cargo` so a probe pays
//! milliseconds instead of a release build.
//!
//! What is faked is named rather than hidden: only `cargo`. The
//! clone, the fetch, the checkout, the layout on disk and every
//! refusal are the real script against a real git repository.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct World {
    pub dir: PathBuf,
    pub repo: PathBuf,
    pub home: PathBuf,
    pub bin: PathBuf,
    pub stub: PathBuf,
    /// The ref of the older version, and the crate version it builds.
    pub old_ref: String,
    pub old_version: String,
    pub new_version: String,
}

pub fn git(dir: &Path, args: &[&str]) -> String {
    let mut command = Command::new("git");
    for name in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE", "GIT_PREFIX"] {
        command.env_remove(name);
    }
    let out = command
        .arg("-C")
        .arg(dir)
        .args(["-c", "user.email=keel@test", "-c", "user.name=keel-test"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "git {args:?}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn crate_file(version: &str) -> String {
    format!("[package]\nname = \"keel\"\nversion = \"{version}\"\n")
}

pub fn world(dir: &Path) -> World {
    let repo = dir.join("source");
    fs::create_dir_all(repo.join("tool")).unwrap();
    fs::write(repo.join("tool/Cargo.toml"), crate_file("1.0.0")).unwrap();
    git(&repo, &["init", "-q", "-b", "main"]);
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "the old one"]);
    git(&repo, &["tag", "v1.0.0"]);
    fs::write(repo.join("tool/Cargo.toml"), crate_file("2.0.0")).unwrap();
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "the new one"]);
    git(&repo, &["tag", "v2.0.0"]);

    // The stub: it reads the tree's own crate version and writes a
    // binary that answers with it, so two refs give two versions.
    let stub = dir.join("stub");
    fs::create_dir_all(&stub).unwrap();
    let script = "#!/bin/sh\n\
         root=\"$(pwd)\"\n\
         for word in \"$@\"; do case \"$word\" in */tool/Cargo.toml) root=\"${word%/tool/Cargo.toml}\";; esac; done\n\
         version=$(grep '^version' \"$root/tool/Cargo.toml\" | head -1 | cut -d'\"' -f2)\n\
         out=\"$root/tool/target/release\"\n\
         mkdir -p \"$out\"\n\
         printf '#!/bin/sh\\nif [ \"$1\" = \"--version\" ]; then echo \"keel %s\"; else echo \"ran %s: $*\"; fi\\n' \"$version\" \"$version\" > \"$out/keel\"\n\
         chmod +x \"$out/keel\"\n";
    fs::write(stub.join("cargo"), script).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(stub.join("cargo")).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(stub.join("cargo"), perms).unwrap();
    }

    World {
        dir: dir.to_path_buf(),
        repo,
        home: dir.join("home"),
        bin: dir.join("bin"),
        stub,
        old_ref: "v1.0.0".to_string(),
        old_version: "1.0.0".to_string(),
        new_version: "2.0.0".to_string(),
    }
}

/// The real install.sh, run against that world.
pub fn install(world: &World, git_ref: Option<&str>) -> (String, i32) {
    let installer = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("install.sh");
    let path = format!(
        "{}:{}",
        world.stub.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let out = Command::new("sh")
        .arg(installer)
        .current_dir(&world.dir)
        .env("PATH", path)
        .env("KEEL_REPO", &world.repo)
        .env("KEEL_HOME", &world.home)
        .env("KEEL_BIN", &world.bin)
        .env("KEEL_REF", git_ref.unwrap_or(""))
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

/// What the installed `keel` answers in a given project.
pub fn run_in(world: &World, project: &Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(world.bin.join("keel"))
        .args(args)
        .current_dir(project)
        .env("KEEL_HOME", &world.home)
        .env("KEEL_REPO", &world.repo)
        .env(
            "PATH",
            format!(
                "{}:{}",
                world.stub.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
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
