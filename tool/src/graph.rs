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
    let _ = wave;
    todo!("transform graph-checks")
}

/// Cross-wave judgement: depends_on existence and cycles (§7.2), a
/// superseded_by successor unknown to any wave.
pub fn cross_findings(waves: &[Wave]) -> Vec<(String, String, String)> {
    let _ = waves;
    todo!("transform graph-checks")
}
