//! Scenario test of wave 0034: a court that cannot fail is not a
//! court.

mod common;

/// A chore, not a scenario, and the wave says why: this judges our
/// own battery rather than keel's behaviour, and §6.3's red birth
/// means something only where a state exists in which the probe
/// fails before the work.
///
/// wave 0033 renamed a line a person reads and left a neighbouring
/// assert hunting the old words. Nothing printed them any more, so
/// the assert could never fail again -- and it was the one holding
/// review 0031 R-8. The reviewer proved it with a mutant: break the
/// guard, and main goes red while that branch stayed green.
///
/// This is a class, not an incident: it turned up heavy twice in two
/// waves. So the battery reads itself.
#[test]
fn a_court_that_cannot_fail_is_not_a_court() {
    let tool = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // Everything the tool can say: its own lines, and every document
    // it carries and serves whole.
    let repo = tool.parent().unwrap();
    let mut words = String::new();
    for path in [
        tool.join("i18n/uk.ftl"),
        tool.join("i18n/en.ftl"),
        repo.join("QUALITY.md"),
        repo.join("docs/uk/QUALITY.md"),
        repo.join("docs/uk/METHODOLOGY-V2.md"),
        repo.join("docs/en/METHODOLOGY-V2.md"),
        repo.join("docs/uk/NEW-CONCEPT.md"),
    ] {
        words.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
    }
    let words = words.to_lowercase();

    let mut dead: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(tool.join("tests")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        for (number, line) in text.lines().enumerate() {
            // A `contains` on a Ukrainian phrase is the shape that
            // died: the words of the tool, quoted in a probe. If the
            // tool no longer says them anywhere, the assert around
            // them can never fail.
            let Some((_, rest)) = line.split_once(".contains(\"") else {
                continue;
            };
            let Some((needle, _)) = rest.split_once('"') else {
                continue;
            };
            // Cyrillic only, and on purpose: the tool's Ukrainian
            // speech lives nowhere but its own lines and the
            // documents it carries, so a missing word is real
            // evidence. Widening this to English drowns the court in
            // fixture text -- probes write their own wave slugs and
            // their own caveats, and none of that is the tool
            // speaking. Measured: five findings, five false.
            let cyrillic = needle.chars().any(|c| ('а'..='я').contains(&c));
            if !cyrillic || needle.len() < 6 {
                continue;
            }
            // Word by word, and in one case. A line the tool builds
            // from a template ("конфіг: keel.toml (lang = uk)") never
            // appears whole in the vocabulary, and the briefing
            // SHOUTS its prohibitions while the probe reads them
            // lowered -- neither is a dead assert. A needle whose
            // every word is still spoken somewhere is alive; one
            // carrying a word the tool no longer says anywhere is
            // not.
            //
            // A word the probe itself PUT INTO its fixture is alive
            // too: a court that hands back a document's own text --
            // the reason a wave was called off, say -- is measured by
            // quoting that text, and the tool's vocabulary cannot
            // contain it by definition. Wave 0037 met exactly that.
            let lost: Vec<&str> = needle
                .split(|c: char| !c.is_alphanumeric())
                .filter(|word| word.chars().count() >= 5)
                .filter(|word| word.chars().any(|c| ('а'..='я').contains(&c)))
                .filter(|word| !words.contains(&word.to_lowercase()))
                .filter(|word| {
                    // Said by the probe somewhere other than this
                    // very assert: a fixture writes it, this line
                    // reads it back.
                    text.match_indices(*word).count() < 2
                })
                .collect();
            if !lost.is_empty() {
                dead.push(format!("{file}:{} — \"{needle}\" ({lost:?})", number + 1));
            }
        }
    }

    assert!(
        dead.is_empty(),
        "every assert can still fail: these hunt words the tool no \
         longer says anywhere, so they are green forever and hold \
         nothing:\n  {}",
        dead.join("\n  ")
    );
}
