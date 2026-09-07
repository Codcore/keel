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

/// The same paragraph as one line: the norm is wrapped by hand, so a
/// sentence this probe looks for lives across a line break as often as
/// not, and a `contains` over the raw text would hold only until the
/// next reflow.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
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
    // §6.3-а ends where it always did -- at the withdrawn promise of
    // §2.12 -- and no longer with the sentence that called the
    // rollback uncovered (asserted by the ending, not by a phrase the
    // tool never says: dead_assert_test reads `contains` only).
    let word = paragraph(&uk, "**§6.3-а.");
    assert!(
        word.trim_end().ends_with("(§2.12)."),
        "§6.3-а (uk) ends at §2.12, without the sentence that called the rollback uncovered:\n{word}"
    );
    let word = paragraph(&en, "**§6.3-a.");
    assert!(
        word.trim_end().ends_with("(§2.12)."),
        "§6.3-a (en) ends at §2.12 too (review 0054 R-12):\n{word}"
    );
    // The revert clause names its condition (review 0054 R-3): only a
    // revert under the new wave's slug is read as work.
    for (tongue, text, head) in [("uk", &uk, "**§6.3-б."), ("en", &en, "**§6.3-b.")] {
        let word = paragraph(text, head);
        assert!(
            word.contains("§8.4") && word.contains("§6.2"),
            "{tongue}: §6.3 says a revert is work only under the slug (§8.4), else outside judgement (§6.2):\n{word}"
        );
    }

    // --- §2.11: the exception of wave 0058, and its width ---
    //
    // Held here because nothing else holds it (review 0058 R-5): a
    // mutant that took the paragraph out of the English text and out
    // of the root copy walked the whole battery, and both "the
    // methodology of this binary" rows stayed green -- the skeleton
    // compares chapters and numbers, not sentences, and
    // `translated_from` holds the Ukrainian side alone.
    let root = repo_file("METHODOLOGY.md");
    for (tongue, text, mark, price) in [
        ("uk", &uk, "Виняток — контракт", "план окремо, робота окремо"),
        ("en", &en, "The exception is a contract", "plan apart, work apart"),
        ("root copy", &root, "The exception is a contract", "plan apart, work apart"),
    ] {
        let word = flat(paragraph(text, "**§2.11."));
        assert!(
            word.contains(mark) && word.contains("§6.8") && word.contains(price),
            "{tongue}: §2.11 names the exception of a contract, its paragraph \
             (§6.8) and its price:\n{word}"
        );
        assert!(
            word.contains("§9.9"),
            "{tongue}: and says who holds what the machine cannot tell apart \
             -- a change of the promises under a chore:\n{word}"
        );
    }
    // The width of it, said aloud rather than left to be discovered
    // (review 0058 R-4): one contract row makes a wave of any number
    // of chores full.
    for (tongue, text, width) in [
        ("uk", &uk, "безумовний"),
        ("en", &en, "unconditional"),
        ("root copy", &root, "unconditional"),
    ] {
        let word = flat(paragraph(text, "**§2.11."));
        assert!(
            word.contains(width),
            "{tongue}: §2.11 says the exception is unconditional:\n{word}"
        );
    }

    // --- §4.11 and §10.5 carry the mark of a text-held rule ---
    for (tongue, text) in [("uk", &uk), ("en", &en), ("root copy", &root)] {
        let mark = if tongue == "uk" { "текстове" } else { "textual" };
        for head in ["**§4.11.", "**§10.5."] {
            let word = flat(paragraph(text, head));
            assert!(
                word.contains(mark) && word.contains("§9.9") && word.contains("§7.10"),
                "{tongue}: {head} marks itself text-held, held by the reviewer \
                 (§9.9), because of §7.10:\n{word}"
            );
        }
    }

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

    // And the court a person actually runs says so over this very
    // tree: the library call above is the same rule read through the
    // library, which is a weaker fact than the one this probe's head
    // claims (final review 2026-09-06, tests R-6). `keel check` is
    // run here, and its two method rows are read.
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_keel"))
        .args(["check", repo.to_str().unwrap()])
        .output()
        .expect("keel check runs over this repository");
    let said = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    // The exit code is the whole tree's and says nothing here -- a
    // wave in progress reddens it by design; the rows do the talking.
    let rows: Vec<&str> = said
        .lines()
        .filter(|line| {
            line.contains("методика цього бінарника") || line.contains("methodology of this binary")
        })
        .collect();
    assert_eq!(
        rows.len(),
        2,
        "keel check gives the norm a row per tongue (wave 0029):\n{said}"
    );
    for row in rows {
        assert!(
            row.trim_start().starts_with("зелене") || row.trim_start().starts_with("green"),
            "and both rows are green over this tree:\n{row}"
        );
    }
}
