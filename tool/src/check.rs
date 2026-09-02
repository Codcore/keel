//! Перший поверх перевірок: «документи читаються» (щабель 1
//! самонаведення). Це ще не весь check методики — і звіт каже про це
//! сам: зелене про неперевірене заборонене (урок №4 розбору нотаток).

use crate::docs::{self, Refusal};
use std::fmt::Write as _;
use std::path::Path;

pub struct Outcome {
    pub report: String,
    pub findings: usize,
}

/// Обходить документи під коренем і складає звіт по кожному файлу:
/// цілі — перевірені, зіпсовані — названі з причиною; наприкінці —
/// що́ цим поверхом перевірено, а що ще ні, і наступний крок.
pub fn run(root: &Path) -> Result<Outcome, Refusal> {
    let scan = docs::scan(root)?;

    // Рядок на кожен документ, у порядку шляхів. Шлях відмови
    // показуємо відносним до кореня — звіт читає людина.
    let mut rows: Vec<(String, Option<String>)> = Vec::new();
    for wave in &scan.waves {
        rows.push((format!("keel/waves/{}.md", wave.slug), None));
    }
    for contract in &scan.contracts {
        rows.push((format!("keel/contracts/{}.md", contract.slug), None));
    }
    for refusal in &scan.refusals {
        let shown = refusal.file.strip_prefix(root).unwrap_or(&refusal.file);
        rows.push((
            shown.display().to_string(),
            Some(format!(
                "{}\n           натомість: {}",
                refusal.reason, refusal.instead
            )),
        ));
    }
    rows.sort();

    let mut report = String::from("keel check — документи (щабель 1)\n\n");
    for (path, verdict) in &rows {
        match verdict {
            None => writeln!(report, "  зелене   {path} — шапка читається").unwrap(),
            Some(text) => writeln!(report, "  червоне  {path} — {text}").unwrap(),
        }
    }
    if rows.is_empty() {
        report.push_str("  документів ще нема\n");
    }

    let findings = scan.refusals.len();
    let documents = scan.waves.len() + scan.contracts.len();
    writeln!(
        report,
        "\nперевірено цим поверхом: шапки — словник і форма (глави 2–4, §7.9)\n\
         ще не перевірено: редакції (§5), scope (§4), тести (§7.5), шапка↔тіло (§7.7) — щаблі попереду\n\
         підсумок: документів {documents}, відмов {findings}"
    )
    .unwrap();
    let next = if findings > 0 {
        "полагодь названі файли і повтори keel check"
    } else if documents == 0 {
        "створи першу хвилю в keel/waves/"
    } else {
        "щабель 2: редакції (keel rev)"
    };
    writeln!(report, "наступний крок: {next}").unwrap();

    Ok(Outcome { report, findings })
}
