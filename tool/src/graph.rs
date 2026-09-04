//! Chapter 3: the graph links (contract tool-graph).

use crate::docs::Wave;

/// The forty-cut vocabulary, embedded: the methodology and the tool
/// version together (operator's ruling), so a slug is part of the
/// release, not a file to quietly edit.
static CUTS: [&str; 40] = [
    "functional.completeness",
    "functional.correctness",
    "functional.appropriateness",
    "performance.time-behaviour",
    "performance.capacity",
    "performance.resource-utilisation",
    "compatibility.co-existence",
    "compatibility.interoperability",
    "interaction.appropriateness-recognisability",
    "interaction.learnability",
    "interaction.operability",
    "interaction.user-error-protection",
    "interaction.user-engagement",
    "interaction.inclusivity",
    "interaction.user-assistance",
    "interaction.self-descriptiveness",
    "reliability.faultlessness",
    "reliability.fault-tolerance",
    "reliability.availability",
    "reliability.recoverability",
    "security.confidentiality",
    "security.integrity",
    "security.non-repudiation",
    "security.accountability",
    "security.authenticity",
    "security.resistance",
    "maintainability.modularity",
    "maintainability.reusability",
    "maintainability.analysability",
    "maintainability.modifiability",
    "maintainability.testability",
    "flexibility.adaptability",
    "flexibility.scalability",
    "flexibility.installability",
    "flexibility.replaceability",
    "safety.operational-constraints",
    "safety.risk-identification",
    "safety.fail-safe",
    "safety.hazard-warning",
    "safety.safe-integration",
];

pub fn cuts() -> &'static [&'static str] {
    &CUTS
}

/// In-wave judgement: §3.4 slugs, §10.3 silence (withdrawn covers do
/// not count, §2.12), implements into nowhere.
pub fn wave_findings(wave: &Wave) -> Vec<(String, String)> {
    use crate::docs::TransformKind;
    use crate::i18n::{t, ta};
    use crate::targs;
    use std::collections::BTreeSet;

    let known: BTreeSet<&str> = CUTS.iter().copied().collect();
    let mut out = Vec::new();

    for (name, scenario) in &wave.scenarios {
        for cut in &scenario.covers {
            if !known.contains(cut.as_str()) {
                out.push((
                    ta(
                        "graph-unknown-cut",
                        targs!("slug" => cut.clone(), "holder" => name.clone()),
                    ),
                    t("graph-unknown-cut-instead"),
                ));
            }
        }
    }
    for (cut, _) in &wave.decisions {
        if !known.contains(cut.as_str()) {
            out.push((
                ta(
                    "graph-unknown-cut",
                    targs!("slug" => cut.clone(), "holder" => "decisions".to_string()),
                ),
                t("graph-unknown-cut-instead"),
            ));
        }
    }

    // §10.3 wants exactly one answer per cut: silence is a finding,
    // and so is a double -- two live covers, or a live cover next to
    // a decision of the same wave. A dead cover does not count
    // (§2.12), so "dead cover + decision" stays the lawful pair.
    let mut live_covers: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
    for (name, scenario) in &wave.scenarios {
        if scenario.withdrawn.is_none() {
            for cut in &scenario.covers {
                live_covers.entry(cut.as_str()).or_default().push(name);
            }
        }
    }
    let decided: BTreeSet<&str> = wave.decisions.iter().map(|(c, _)| c.as_str()).collect();
    for (cut, holders) in &live_covers {
        if holders.len() > 1 {
            let named = holders
                .iter()
                .map(|h| format!("\"{h}\""))
                .collect::<Vec<_>>()
                .join(", ");
            out.push((
                ta(
                    "graph-double-cover",
                    targs!("slug" => cut.to_string(), "count" => holders.len() as u64, "holders" => named),
                ),
                t("graph-double-cover-instead"),
            ));
        }
        if decided.contains(cut) {
            out.push((
                ta(
                    "graph-double-decided",
                    targs!("slug" => cut.to_string(), "holder" => holders[0].to_string()),
                ),
                t("graph-double-decided-instead"),
            ));
        }
    }

    let mut answered: BTreeSet<&str> = live_covers.keys().copied().collect();
    answered.extend(decided);
    let missing: Vec<&str> = CUTS
        .iter()
        .copied()
        .filter(|c| !answered.contains(c))
        .collect();
    if !missing.is_empty() {
        out.push((
            ta("graph-silence", targs!("missing" => missing.join(", "))),
            t("graph-silence-instead"),
        ));
    }

    for (transform_name, transform) in &wave.transforms {
        if let TransformKind::Implements(list) = &transform.kind {
            for scenario_name in list {
                if !wave.scenarios.iter().any(|(n, _)| n == scenario_name) {
                    out.push((
                        ta(
                            "graph-implements-missing",
                            targs!("transform" => transform_name.clone(), "scenario" => scenario_name.clone()),
                        ),
                        t("graph-implements-missing-instead"),
                    ));
                }
            }
        }
    }
    out
}

/// Cross-wave judgement: depends_on existence and cycles (§7.2), a
/// superseded_by successor unknown to any wave.
pub fn cross_findings(waves: &[Wave], contracts: &[String]) -> Vec<(String, String, String)> {
    use crate::i18n::{t, ta};
    use crate::targs;
    use std::collections::{BTreeMap, BTreeSet};

    // A wave called off is outside judgement (§6.3-a): it neither
    // holds a scenario name nor answers for a link.
    let waves: Vec<&Wave> = waves.iter().filter(|w| w.cancelled.is_none()).collect();
    let waves = waves.as_slice();
    let mut out = Vec::new();
    let slugs: BTreeSet<&str> = waves.iter().map(|w| w.slug.as_str()).collect();
    let contracts: BTreeSet<&str> = contracts.iter().map(String::as_str).collect();
    let everyones_scenarios: BTreeSet<&str> = waves
        .iter()
        .flat_map(|w| w.scenarios.iter().map(|(n, _)| n.as_str()))
        .collect();

    // One name, one home. A test tag is a bare name, so two waves
    // sharing a scenario name make one test close both -- the bug
    // audit copied a wave under a new number and `keel close` said
    // "closed" to both, though the second had no test at all. The
    // norm never says the slugs are unique either (methodology audit
    // С-9); until it does, the machine says it.
    let mut homes: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    for wave in waves {
        for (name, scenario) in &wave.scenarios {
            // A withdrawn promise no longer holds its name: §2.12
            // retires it, and the next wave may reuse it honestly.
            if scenario.withdrawn.is_some() {
                continue;
            }
            homes.entry(name.as_str()).or_default().push(&wave.slug);
        }
    }
    // The same namespace holds the contracts: a tag `proves: x@rev`
    // is read as a scenario, so a contract wearing a scenario's name
    // answers a question nobody asked -- review 0035 R-17 measured
    // the finding coming back in the wrong noun entirely.
    for (name, wave_slug) in homes.iter().filter_map(|(n, w)| w.first().map(|s| (n, s))) {
        if !contracts.contains(name) {
            continue;
        }
        out.push((
            wave_slug.to_string(),
            ta(
                "graph-name-taken",
                targs!("name" => name.to_string(), "wave" => wave_slug.to_string()),
            ),
            t("graph-name-taken-instead"),
        ));
    }

    for (name, mut waves_with_it) in homes {
        if waves_with_it.len() < 2 {
            continue;
        }
        waves_with_it.sort_unstable();
        let where_ = waves_with_it.join(", ");
        out.push((
            waves_with_it[0].to_string(),
            ta(
                "graph-scenario-twice",
                targs!("scenario" => name.to_string(), "waves" => where_),
            ),
            t("graph-scenario-twice-instead"),
        ));
    }

    for wave in waves {
        for target in &wave.depends_on {
            if !slugs.contains(target.as_str()) {
                out.push((
                    wave.slug.clone(),
                    ta("graph-depends-missing", targs!("target" => target.clone())),
                    t("graph-depends-missing-instead"),
                ));
            }
        }
        for (name, scenario) in &wave.scenarios {
            let Some(successor) = &scenario.superseded_by else {
                continue;
            };
            // Nothing takes over from itself (§2.12; review R-5).
            if successor == name {
                out.push((
                    wave.slug.clone(),
                    ta("graph-superseded-self", targs!("scenario" => name.clone())),
                    t("graph-superseded-self-instead"),
                ));
            } else if !everyones_scenarios.contains(successor.as_str()) {
                out.push((
                    wave.slug.clone(),
                    ta(
                        "graph-superseded-missing",
                        targs!("scenario" => name.clone(), "successor" => successor.clone()),
                    ),
                    t("graph-superseded-missing-instead"),
                ));
            }
        }
    }

    // Cycles (§7.2): depth-first over depends_on edges that exist.
    let edges: BTreeMap<&str, Vec<&str>> = waves
        .iter()
        .map(|w| {
            (
                w.slug.as_str(),
                w.depends_on
                    .iter()
                    .map(String::as_str)
                    .filter(|t| slugs.contains(t))
                    .collect(),
            )
        })
        .collect();
    let mut reported: BTreeSet<Vec<String>> = BTreeSet::new();
    for start in slugs.iter() {
        let mut path: Vec<&str> = Vec::new();
        let mut stack: Vec<(&str, usize)> = vec![(start, 0)];
        let mut on_path: BTreeSet<&str> = BTreeSet::new();
        while let Some((node, next)) = stack.pop() {
            if next == 0 {
                if on_path.contains(node) {
                    let from = path.iter().position(|n| *n == node).unwrap_or(0);
                    let mut cycle: Vec<String> =
                        path[from..].iter().map(|s| s.to_string()).collect();
                    cycle.push(node.to_string());
                    let mut key = cycle.clone();
                    key.sort();
                    key.dedup();
                    if reported.insert(key) {
                        out.push((
                            cycle[0].clone(),
                            ta("graph-cycle", targs!("chain" => cycle.join(" -> "))),
                            t("graph-cycle-instead"),
                        ));
                    }
                    continue;
                }
                on_path.insert(node);
                path.push(node);
            }
            let children = edges.get(node).map(Vec::as_slice).unwrap_or(&[]);
            if next < children.len() {
                stack.push((node, next + 1));
                stack.push((children[next], 0));
            } else {
                on_path.remove(node);
                path.pop();
            }
        }
    }
    out
}
