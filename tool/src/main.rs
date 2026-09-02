//! CLI: тонка обгортка над бібліотекою. Команди зшиті з лупом
//! методики; кожна відмова — причина плюс «що робити натомість».

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cmd = args
        .first()
        .map_or("(порожньо)".to_string(), |c| format!("\"{c}\""));
    eprintln!(
        "відмова: команда {cmd} ще не існує\n  причина: перший щабель самонаведення — CLI виростає хвилями\n  натомість: keel check [тека]"
    );
    ExitCode::from(2)
}
