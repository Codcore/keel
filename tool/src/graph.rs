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

    let mut answered: BTreeSet<&str> = BTreeSet::new();
    for (_, scenario) in &wave.scenarios {
        if scenario.withdrawn.is_none() {
            for cut in &scenario.covers {
                answered.insert(cut.as_str());
            }
        }
    }
    for (cut, _) in &wave.decisions {
        answered.insert(cut.as_str());
    }
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
pub fn cross_findings(waves: &[Wave]) -> Vec<(String, String, String)> {
    use crate::i18n::{t, ta};
    use crate::targs;
    use std::collections::{BTreeMap, BTreeSet};

    let mut out = Vec::new();
    let slugs: BTreeSet<&str> = waves.iter().map(|w| w.slug.as_str()).collect();
    let everyones_scenarios: BTreeSet<&str> = waves
        .iter()
        .flat_map(|w| w.scenarios.iter().map(|(n, _)| n.as_str()))
        .collect();

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
            if let Some(successor) = &scenario.superseded_by
                && !everyones_scenarios.contains(successor.as_str())
            {
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
