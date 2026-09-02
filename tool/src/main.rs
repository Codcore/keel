//! CLI: тонка обгортка над бібліотекою. Команди зшиті з лупом
//! методики; кожна відмова — причина плюс «що робити натомість».

use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            match keel::check::run(&root) {
                Ok(outcome) => {
                    print!("{}", outcome.report);
                    if outcome.findings == 0 {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(1)
                    }
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some(other) => {
            eprintln!(
                "відмова: невідома команда \"{other}\"\n  причина: перший щабель самонаведення — команд поки одна\n  натомість: keel check [тека]"
            );
            ExitCode::from(2)
        }
        None => {
            eprintln!(
                "відмова: не названо команди\n  причина: keel не вгадує, що робити\n  натомість: keel check [тека]"
            );
            ExitCode::from(2)
        }
    }
}
