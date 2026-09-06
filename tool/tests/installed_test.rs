//! Scenario test of wave 0041: the lamp shows what stands here.
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

mod common;

use common::sandbox;
use common::versions::{install, world};

use std::fs;
use std::process::Command;

fn keel(home: &std::path::Path, args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(args)
        .env("KEEL_HOME", home)
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

/// proves: the-lamp-shows-what-stands-here@0fa368 -- the concept asks
/// `keel version` for three things: which version runs, where the pin
/// points, and **which versions stand locally**. The third had no
/// answer, because until this wave only one version could stand.
#[test]
fn the_lamp_shows_what_stands_here() {
    let dir = sandbox("lamp");
    let w = world(&dir);
    install(&w, Some(&w.old_ref));
    install(&w, Some("v2.0.0"));

    let project = dir.join("project");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("keel.toml"), "lang = \"uk\"\n").unwrap();

    let (said, code) = keel(&w.home, &["version", project.to_str().unwrap()]);
    assert_eq!(code, 0, "the lamp answers:\n{said}");
    for version in [&w.old_version, &w.new_version] {
        assert!(
            said.contains(version.as_str()),
            "and names version {version} as standing here:\n{said}"
        );
    }
    assert!(
        said.contains(&w.old_ref) && said.contains("v2.0.0"),
        "each by the ref it was installed under:\n{said}"
    );

    // The machine road carries the same list as a field, not prose.
    let (said, _) = keel(&w.home, &["version", "--json", project.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&said).unwrap();
    let installed = package["installed"]
        .as_array()
        .unwrap_or_else(|| panic!("the package lists what stands here:\n{said}"));
    let mut versions: Vec<&str> = installed
        .iter()
        .map(|one| one["version"].as_str().unwrap_or_default())
        .collect();
    versions.sort_unstable();
    assert_eq!(
        versions,
        vec![w.old_version.as_str(), w.new_version.as_str()],
        "and lists exactly those two:\n{said}"
    );
    assert!(
        installed
            .iter()
            .all(|one| one["ref"].as_str().is_some_and(|named| !named.is_empty())),
        "each with the ref it was installed under:\n{said}"
    );

    // A machine where nothing stands says so, rather than an empty
    // silence a harness would read as "one, unnamed".
    let empty = sandbox("lampempty");
    let (said, _) = keel(&empty, &["version", "--json", project.to_str().unwrap()]);
    let package: serde_json::Value = serde_json::from_str(&said).unwrap();
    assert_eq!(
        package["installed"].as_array().map(Vec::len),
        Some(0),
        "nothing standing is an empty list, not a missing field:\n{said}"
    );
    // And the prose says so too, rather than leaving a person to read
    // an absence (review 0041 R-14: the row was held by nothing).
    let (said, _) = keel(&empty, &["version", project.to_str().unwrap()]);
    assert!(
        said.contains("не стоїть жодної версії") || said.contains("no version stands here"),
        "the prose names the empty shelf:\n{said}"
    );
    // And it does NOT guess why: a version can stand and still not be
    // reached, which is a different sentence.
    assert!(
        !said.contains("launcher"),
        "without inventing a cause it did not measure:\n{said}"
    );
}
