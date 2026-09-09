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
            let lost: Vec<&str> = needle
                .split(|c: char| !c.is_alphanumeric())
                .filter(|word| word.chars().count() >= 5)
                .filter(|word| word.chars().any(|c| ('а'..='я').contains(&c)))
                .filter(|word| !words.contains(&word.to_lowercase()))
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

    // --- and the same court in English, where it can be true -----
    // The comment above is right that a bare word-by-word reading of
    // English drowns in fixture text. What is evidence, and what the
    // final review of 2026-09-06 (tests R-5) counted by hand, is a
    // NEGATIVE assert whose whole phrase stands NOWHERE: not in the
    // tool's own lines, not in the documents it carries, not even in
    // the probe's own fixture. Nothing can print it, so the assert
    // is green for ever.
    //
    // Nine such phrases are alive all the same, and each says why:
    // most are lines the tool would BUILD out of parts (a name, a
    // path, a version, a runner's own banner, a shape it used to
    // print) if it regressed; one guards a sentence the norm used to
    // carry, and one a line of output a window must cut. They are
    // named here rather than left to a reader's judgement -- a court
    // with an unwritten exception is not a court.
    const GUARDS: [(&str, &str); 9] = [
        // The full name mix would print if the reader put a test
        // into a describe block it is not in.
        ("elixir_border_test.rs", "a group it works"),
        // The version line the launcher would print if it ran a
        // binary other than the pinned one.
        ("launcher_fetch_test.rs", "keel 1.0.0"),
        // The path check would build if it addressed a finding to a
        // wave file that does not exist.
        ("plan_branch_test.rs", "keel/waves/0099-nowhere.md"),
        // pytest's own banner rule, which the adapter must not pass
        // off as a reason.
        ("python_border_test.rs", "_____"),
        // The drift line the review package would build if it called
        // a planned file drift.
        ("review_test.rs", "src/a.rs — added after the anchor"),
        // A sentence §4.13 used to carry: a guard over the NORM's
        // text, which a person edits.
        ("rule_truth_test.rs", "the check on a PR"),
        // The toml crate's own words, which the config court echoes
        // when a file does not parse: the tool never writes them
        // itself, and the whole point of the assert is that no such
        // echo appears.
        ("config_quoting_test.rs", "TOML parse error"),
        // The shape the ci verdict had before wave 0070: the last
        // line the command happened to print, squeezed into
        // parentheses that read as a duration. The words are the
        // probe's own fixture, but the PARENTHESES are the old
        // format, and only a regression would put them back.
        ("red_carries_words_test.rs", "(tailwind: rebuilding)"),
        // The middle of a 200-line output, which the window must cut.
        // The fixture writes `line-$i` in a shell loop, so no line of
        // this probe spells the hundredth -- and printing it is
        // exactly what a window that stopped cutting would do.
        ("red_carries_words_test.rs", "line-100"),
    ];

    let mut corpus = String::new();
    for path in [
        tool.join("i18n/uk.ftl"),
        tool.join("i18n/en.ftl"),
        repo.join("QUALITY.md"),
        repo.join("README.md"),
        repo.join("METHODOLOGY.md"),
        repo.join("AGENTS.md"),
        repo.join("BACKLOG.md"),
        repo.join("install.sh"),
        repo.join("docs/uk/QUALITY.md"),
        repo.join("docs/uk/METHODOLOGY-V2.md"),
        repo.join("docs/en/METHODOLOGY-V2.md"),
        repo.join("docs/uk/NEW-CONCEPT.md"),
    ] {
        corpus.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
    }
    for entry in std::fs::read_dir(tool.join("src")).unwrap().flatten() {
        corpus.push_str(&std::fs::read_to_string(entry.path()).unwrap_or_default());
    }
    let corpus = corpus.to_lowercase();

    let mut mute: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(tool.join("tests")).unwrap().flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|kind| kind != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).unwrap();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            // EVERY `.contains("…")` of the line, not the first: an
            // assert may hold one phrase and deny another on one line
            // (`x.contains("a") && !x.contains("b")`), and reading
            // only the first needle left the second unjudged (review
            // 0055 R-8).
            let mut at = 0usize;
            while let Some(found) = line[at..].find(".contains(\"") {
                let call = at + found;
                at = call + ".contains(\"".len();
                let Some(rest) = line.get(at..) else { break };
                let Some((needle, _)) = rest.split_once('"') else {
                    break;
                };
                // Negative? The `!` stands before the receiver of THIS
                // call: walk back over the receiver's own name.
                let before = &line[..call];
                let receiver_start = before
                    .rfind(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
                    .map_or(0, |at| at + 1);
                let negative = before[..receiver_start].trim_end().ends_with('!');
                if !negative || needle.len() < 5 || needle.chars().any(|c| ('а'..='я').contains(&c))
                {
                    continue;
                }
                if GUARDS.contains(&(file.as_str(), needle)) {
                    continue;
                }
                let lowered = needle.to_lowercase();
                if corpus.contains(&lowered) {
                    continue;
                }
                // The probe's own fixture may write it, and then the
                // court under test really can print it back.
                let elsewhere = lines
                    .iter()
                    .enumerate()
                    .any(|(other, l)| other != index && l.to_lowercase().contains(&lowered));
                if !elsewhere {
                    mute.push(format!("{file}:{} — \"{needle}\"", index + 1));
                }
            }
        }
    }

    assert!(
        mute.is_empty(),
        "every negative assert can still fail: these hunt a phrase \
         that stands nowhere -- not in the tool's lines, not in its \
         documents, not in the probe's own fixture -- so nothing can \
         ever print it. Fix the assert, or name it among GUARDS above \
         with the reason it is alive:\n  {}",
        mute.join("\n  ")
    );
}
