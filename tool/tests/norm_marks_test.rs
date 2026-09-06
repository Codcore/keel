//! Scenario test of wave 0054: the norm names what holds it.
//!
//! The constitution (rule 6) asks that a rule living only in the text
//! be marked as such. Measured before the plan: neither text of the
//! norm carried a single such mark; §7.1's prose half cannot be held
//! by the machine without breaking §7.10, §8.6 is held by nobody, and
//! §6.3-a ended by saying the rollback of a merged wave is not
//! covered (the operator's decisions of 2026-09-06).
//!
//! proves tags -- revisions per §5.3-§5.4, verified by `keel rev`.

use std::path::Path;

fn repo_file(rel: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(rel),
    )
    .unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// One paragraph of the norm: from its `**§N.M.` head to the next.
fn paragraph<'a>(text: &'a str, head: &str) -> &'a str {
    let start = text
        .find(head)
        .unwrap_or_else(|| panic!("the norm carries {head}"));
    let rest = &text[start + head.len()..];
    let end = rest.find("\n**§").map_or(rest.len(), |at| at);
    &rest[..end]
}

/// proves: the-norm-names-what-holds-it@b8e77f
#[test]
fn the_norm_names_what_holds_it() {
    let uk = repo_file("docs/uk/METHODOLOGY-V2.md");
    let en = repo_file("docs/en/METHODOLOGY-V2.md");

    // --- §7.1: the prose half is a text-held rule, and says so ---
    let word = paragraph(&uk, "**§7.1.**");
    assert!(
        word.contains("текстове") && word.contains("§9.9") && word.contains("§7.10"),
        "§7.1 (uk) marks its prose half as text-held, held by the reviewer (§9.9), \
         because of §7.10:\n{word}"
    );
    let word = paragraph(&en, "**§7.1.**");
    assert!(
        word.contains("textual") && word.contains("§9.9") && word.contains("§7.10"),
        "§7.1 (en) marks its prose half as textual:\n{word}"
    );

    // --- §8.6: the plan commit's paragraph is held by people ---
    let word = paragraph(&uk, "**§8.6.**");
    assert!(
        word.contains("текстове") && word.contains("§9.9"),
        "§8.6 (uk) is marked text-held:\n{word}"
    );
    let word = paragraph(&en, "**§8.6.**");
    assert!(
        word.contains("textual") && word.contains("§9.9"),
        "§8.6 (en) is marked textual:\n{word}"
    );

    // --- §6.3-б: rolling back a merged wave is a wave ---
    let word = paragraph(&uk, "**§6.3-б.");
    assert!(
        word.contains("superseded_by") && word.contains("§6.8") && word.contains("revert"),
        "§6.3-б (uk) says a rollback is a new wave with superseded_by and a revert under \
         its slug, and the fast fix is the light wave of §6.8:\n{word}"
    );
    let word = paragraph(&en, "**§6.3-b.");
    assert!(
        word.contains("superseded_by") && word.contains("§6.8") && word.contains("revert"),
        "§6.3-b (en) says the same:\n{word}"
    );
    let word = paragraph(&uk, "**§6.3-а.");
    assert!(
        !word.contains("не покриті"),
        "§6.3-а (uk) no longer says the rollback is uncovered:\n{word}"
    );

    // --- the translation records the Ukrainian revision as it now
    // stands, so the skeleton court and the record stay green ---
    let recorded = en
        .lines()
        .find_map(|line| {
            line.split_once("`translated_from: ")
                .and_then(|(_, rest)| rest.split_once('`'))
                .map(|(revision, _)| revision.trim().to_string())
        })
        .expect("the English text records translated_from");
    let standing = keel::rev::text_rev(&uk);
    assert!(
        keel::rev::matches(&recorded, &standing),
        "translated_from ({recorded}) is the Ukrainian text's revision ({standing})"
    );
    assert!(
        keel::speak::methods_agree().is_ok(),
        "the two skeletons agree and the record is current: {:?}",
        keel::speak::methods_agree().err().map(|r| r.reason)
    );
}
