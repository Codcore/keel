//! CLI: a thin wrapper over the library. Commands are stitched to
//! the methodology loop; every refusal carries a reason plus "what to
//! do instead". CLI frame refusals -- before the config is read --
//! are English: the project language is not known yet.

use keel::i18n::{t, ta};
use keel::targs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("check") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::check::run(&root, &config) {
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
        Some("rev") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::rev::report(&root, &config) {
                Ok((report, findings)) => {
                    print!("{report}");
                    if findings == 0 {
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
        Some("gate") => {
            let Some(message_file) = args.get(1).map(PathBuf::from) else {
                eprintln!(
                    "{}\n  {}\n  {}",
                    t("main-gate-no-message"),
                    t("main-gate-no-message-reason"),
                    t("main-usage")
                );
                return ExitCode::from(2);
            };
            let root = args
                .get(2)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::gate::run(&root, &message_file) {
                Ok((report, code)) => {
                    print!("{report}");
                    ExitCode::from(code as u8)
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some("map") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::map::draw(&root) {
                Ok(report) => {
                    print!("{report}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some("close") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::close::judge(&root) {
                Ok((report, blockers)) => {
                    print!("{report}");
                    if blockers == 0 {
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
        Some("review") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::review::package(&root) {
                Ok(report) => {
                    print!("{report}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some("init") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            // init is the one command that runs before a config can
            // be counted on: the language falls back to the config
            // court's own reading of whatever stands.
            let lang = keel::config::read(&root)
                .map(|c| c.lang)
                .unwrap_or_default();
            keel::i18n::init(&lang);
            match keel::init::run(&root) {
                Ok((report, failed)) => {
                    print!("{report}");
                    if failed == 0 {
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
        Some("plan") => {
            let Some(slug) = args.get(1) else {
                eprintln!(
                    "{}\n  {}\n  {}",
                    t("main-plan-no-slug"),
                    t("main-plan-no-slug-reason"),
                    t("main-usage")
                );
                return ExitCode::from(2);
            };
            let root = args
                .get(2)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::plan::wave(&root, slug) {
                Ok(report) => {
                    print!("{report}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some("new") => {
            if args.get(1).map(String::as_str) != Some("contract") {
                eprintln!(
                    "{}\n  {}\n  {}",
                    t("main-new-unknown"),
                    t("main-new-unknown-reason"),
                    t("main-usage")
                );
                return ExitCode::from(2);
            }
            let Some(slug) = args.get(2) else {
                eprintln!(
                    "{}\n  {}\n  {}",
                    t("main-new-no-slug"),
                    t("main-new-no-slug-reason"),
                    t("main-usage")
                );
                return ExitCode::from(2);
            };
            let root = args
                .get(3)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::plan::contract(&root, slug) {
                Ok(report) => {
                    print!("{report}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some("status") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::status::report(&root) {
                Ok((report, refusals)) => {
                    print!("{report}");
                    if refusals == 0 {
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
        Some("next") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::next::step(&root) {
                Ok(report) => {
                    print!("{report}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some("trust") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::trust::record(&root) {
                Ok(report) => {
                    print!("{report}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some("hook") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            match keel::gate::install_hook(&root) {
                Ok(words) => {
                    println!("{words}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
            }
        }
        Some(other) => {
            eprintln!(
                "{}\n  {}\n  {}",
                ta(
                    "main-unknown-command",
                    targs!("command" => other.to_string())
                ),
                t("main-unknown-command-reason"),
                t("main-usage")
            );
            ExitCode::from(2)
        }
        None => {
            eprintln!(
                "{}\n  {}\n  {}",
                t("main-no-command"),
                t("main-no-command-reason"),
                t("main-usage")
            );
            ExitCode::from(2)
        }
    }
}
