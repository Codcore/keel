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
            let write_mode = args.get(1).map(String::as_str) == Some("--write");
            let root = args
                .get(if write_mode { 2 } else { 1 })
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            if write_mode {
                return match keel::rev::write(&root) {
                    Ok((report, _)) => {
                        print!("{report}");
                        ExitCode::SUCCESS
                    }
                    Err(refusal) => {
                        eprintln!("{refusal}");
                        ExitCode::from(2)
                    }
                };
            }
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
            // init runs before a config can be counted on -- but a
            // broken keel.toml never steers the call silently
            // (review 0014 R-1, §7.9): the refusal is said aloud
            // and the frame still lands in the default language.
            let lang = match keel::config::read(&root) {
                Ok(config) => config.lang,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    String::new()
                }
            };
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
            // --for <agent> asks for the step in that agent's own
            // answer shape (wave 0025): the session hooks of the two
            // tools take context differently, and the hook must not
            // guess. Without it, the plain step, exactly as before.
            let mut agent: Option<String> = None;
            let mut where_from: Option<String> = None;
            let mut rest = args.iter().skip(1);
            while let Some(word) = rest.next() {
                match word.as_str() {
                    "--for" => match rest.next() {
                        Some(named) => agent = Some(named.clone()),
                        None => {
                            eprintln!("{}", t("main-usage"));
                            return ExitCode::from(2);
                        }
                    },
                    // --for=<agent> is the other half of the same
                    // spelling, and an unknown flag is a refusal, not
                    // a directory: swallowing it made "--forx" a path
                    // with a puzzling word (review 0025 R-12).
                    other if other.starts_with("--for=") => {
                        agent = Some(other["--for=".len()..].to_string());
                    }
                    other if other.starts_with('-') => {
                        eprintln!("{}", t("main-usage"));
                        return ExitCode::from(2);
                    }
                    other => where_from = Some(other.to_string()),
                }
            }
            let root = where_from.map_or_else(|| PathBuf::from("."), PathBuf::from);
            let config = match keel::config::read(&root) {
                Ok(config) => config,
                Err(refusal) => {
                    // Asked for by a hook, the config's own refusal is
                    // still the word the agent needs, and it rides in
                    // the agent's shape with a green exit -- the same
                    // law as in next::step_for, and for the same
                    // measured reason: in Cursor an exit code of 2
                    // means "block the action". Asked for by a
                    // person, nothing changes.
                    if let Some(named) = &agent {
                        // The words of the shaping hand live in i18n
                        // too, and this road runs before the config
                        // could name a language: the default one.
                        keel::i18n::init("");
                        match keel::next::say_for(named, &format!("{refusal}")) {
                            Ok(said) => {
                                print!("{said}");
                                return ExitCode::SUCCESS;
                            }
                            Err(refusal) => {
                                eprintln!("{refusal}");
                                return ExitCode::from(2);
                            }
                        }
                    }
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&config.lang);
            let said = match &agent {
                Some(named) => keel::next::step_for(&root, named),
                None => keel::next::step(&root),
            };
            match said {
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
        Some("update") => {
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
            let (word, lacked) = keel::generated::write(&root, &config);
            println!("{}", t("update-title"));
            println!("  {word}");
            if lacked == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(1)
            }
        }
        Some("version") => {
            let root = args
                .get(1)
                .map_or_else(|| PathBuf::from("."), PathBuf::from);
            // The lamp looks with the unpinned eye (tool-version):
            // it must answer exactly where the courts refuse, and a
            // broken config never steers it silently -- its row
            // carries the reason, the config court the full refusal.
            let lang = keel::config::read_unpinned(&root)
                .map(|config| config.lang)
                .unwrap_or_default();
            keel::i18n::init(&lang);
            print!("{}", keel::version::report(&root));
            ExitCode::SUCCESS
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
