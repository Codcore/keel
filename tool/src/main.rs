//! CLI: a thin wrapper over the library. Commands are stitched to
//! the methodology loop; every refusal carries a reason plus "what to
//! do instead". CLI frame refusals -- before the config is read --
//! are English: the project language is not known yet.

use keel::i18n::{t, ta};
use keel::targs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// The single optional directory of a reading command, and whether
/// anything else was typed beside it.
fn one_path(args: &[String]) -> (PathBuf, bool) {
    let mut root = PathBuf::from(".");
    let mut seen = false;
    let mut extra = false;
    for word in args.iter().skip(1) {
        if seen {
            extra = true;
            break;
        }
        seen = true;
        root = PathBuf::from(word);
    }
    (root, extra)
}

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
        // setup is init on a project that already has answers: the
        // same wizard, the same flags, the current config as the
        // defaults (wave 0032 -- until now a keel.toml was edited by
        // hand or not at all).
        Some("init") | Some("setup") => {
            // The answers of the wizard, given as flags (wave 0026):
            // the question-free road, and the road the probe drives,
            // since the drawing of questions needs a pty.
            let mut given: Vec<(String, String)> = Vec::new();
            let mut root = PathBuf::from(".");
            let mut named_root = false;
            let mut no_ask = false;
            let mut rest = args.iter().skip(1);
            while let Some(word) = rest.next() {
                let mut answer = |field: &str, value: Option<&String>| match value {
                    Some(value) => {
                        given.push((field.to_string(), value.clone()));
                        true
                    }
                    None => false,
                };
                let ok = match word.as_str() {
                    "--no-ask" => {
                        no_ask = true;
                        true
                    }
                    "--hooks" => answer("hooks", Some(&"yes".to_string())),
                    "--no-hooks" => answer("hooks", Some(&"no".to_string())),
                    "--lang" | "--adapter" | "--mode" | "--agents" | "--version" | "--ci"
                    | "--trust" => answer(word.trim_start_matches("--"), rest.next()),
                    other if other.starts_with("--") && other.contains('=') => {
                        let (flag, value) = other.split_once('=').unwrap();
                        answer(flag.trim_start_matches("--"), Some(&value.to_string()))
                    }
                    other if other.starts_with('-') => false,
                    other => {
                        // Two paths is a typo, not a choice: before
                        // this wave the first won, and the new
                        // parsing silently made it the last (review
                        // 0026 R-12). Neither is worth guessing.
                        if named_root {
                            eprintln!("{}", t("main-usage"));
                            return ExitCode::from(2);
                        }
                        named_root = true;
                        root = PathBuf::from(other);
                        true
                    }
                };
                if !ok {
                    eprintln!("{}", t("main-usage"));
                    return ExitCode::from(2);
                }
            }
            let setup = args.first().map(String::as_str) == Some("setup");
            // init runs before a config can be counted on -- but a
            // broken keel.toml never steers the call silently
            // (review 0014 R-1, §7.9): the refusal is said aloud
            // and the frame still lands in the default language.
            // setup reads it unpinned: a pin that no longer matches
            // this binary is the commonest reason to run setup, and
            // saying "refusal" while doing the work anyway is the
            // very thing review 0032 R-1 caught.
            let read = if setup {
                keel::config::read_unpinned(&root)
            } else {
                keel::config::read(&root)
            };
            let lang = match read {
                Ok(config) => config.lang,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    String::new()
                }
            };
            keel::i18n::init(&lang);
            let mut answers = match keel::ask::from_flags(&given) {
                Ok(answers) => answers,
                Err(refusal) => {
                    // Judged before a single byte is written.
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            // The silence, which is the wave's first law: questions
            // only where BOTH ends are terminals. A tool that asks in
            // CI, in a test sandbox or in a pipe simply hangs. An
            // existing config is a fact (§7.9) and is never asked
            // about, so the wizard runs only where the file is about
            // to be born.
            // The questions are drawn on STDERR by the library, and
            // read from the terminal -- so the law watches stdin and
            // stderr, not stdout (review 0026 R-2, measured: with
            // 2>file the questions went into the file, the terminal
            // saw nothing, and the config was born from answers
            // nobody watched being asked). With stdout piped the
            // questions still show and still work, so that road is
            // no longer barred either.
            let listening = std::io::IsTerminal::is_terminal(&std::io::stdin())
                && std::io::IsTerminal::is_terminal(&std::io::stderr());
            // Flags answer their own questions and silence no others
            // (review 0026 R-4: one flag used to silence all five).
            // setup asks even where a config stands -- that is the
            // whole point of it -- and seeds its defaults from the
            // answers already given.
            if setup && root.join("keel.toml").is_file() {
                // read_unpinned, not read: a version pin that no
                // longer matches this binary is the commonest reason
                // to run setup at all, and refusing there would deny
                // the person the very fix they came for.
                //
                // And an UNREADABLE config stops the command dead.
                // Review 0032 R-1: swallowing the error let setup
                // overwrite a config it had never read -- trust,
                // digests and every answer gone, reported as "born",
                // exit 0. A court that says "refusal" and does the
                // opposite is worse than no court.
                match keel::config::read_unpinned(&root) {
                    Ok(config) => answers = keel::ask::from_config(&config, &answers),
                    Err(refusal) => {
                        eprintln!("{refusal}");
                        return ExitCode::from(2);
                    }
                }
            }
            if !no_ask && listening && (setup || !root.join("keel.toml").is_file()) {
                // init asks only what a flag has not answered; setup
                // asks EVERYTHING, showing the current answer as the
                // default -- otherwise it can change nothing that is
                // already set, which is every answer it exists to
                // change. Review 0032 R-6 measured it asking two of
                // eight questions on a full config.
                let asking = if setup {
                    keel::ask::ask_with_defaults(&keel::ask::questions(), &answers)
                } else {
                    keel::ask::ask_unanswered(&keel::ask::questions(), &answers)
                };
                match asking {
                    Ok(asked) => answers = asked,
                    Err(refusal) => {
                        eprintln!("{refusal}");
                        return ExitCode::from(2);
                    }
                }
            }
            let done = if setup {
                keel::init::setup(&root, &answers)
            } else {
                keel::init::run(&root, &answers)
            };
            match done {
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
        // The concept, carried since wave 0032: a text the binary
        // held and the mouth would not give.
        Some("concept") => {
            let (root, extra) = one_path(&args);
            if extra {
                eprintln!("{}", t("main-usage"));
                return ExitCode::from(2);
            }
            let lang = match keel::config::read_unpinned(&root) {
                Ok(config) => config.lang,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&lang);
            println!("{}", t("speak-concept-title"));
            println!();
            println!("{}", keel::speak::concept());
            println!(
                "{}",
                ta(
                    "speak-concept-source",
                    targs!("version" => env!("CARGO_PKG_VERSION").to_string())
                )
            );
            ExitCode::SUCCESS
        }
        Some("cuts") => {
            // The mouth reads no document from disk: it serves what
            // this release was built with, so a project that has
            // neither file still hears both (wave 0027).
            let (root, extra) = one_path(&args);
            if extra {
                eprintln!("{}", t("main-usage"));
                return ExitCode::from(2);
            }
            // The language now decides WHICH NORMATIVE DOCUMENT a
            // person reads, not merely the chrome of the output, so a
            // config that does not parse -- or names a language this
            // release does not carry -- is a refusal here exactly as
            // it is everywhere else in the frame. It used to be
            // swallowed, and `lang = "ua"` (the commonest typo: the
            // language code is uk, the domain is ua) silently handed
            // a Ukrainian project forty English questions (review
            // 0028 R-5).
            let lang = match keel::config::read_unpinned(&root) {
                Ok(config) => config.lang,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&lang);
            match keel::speak::cuts_report(&lang) {
                Ok(said) => {
                    print!("{said}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    // The judged list and the read list drifted: that
                    // is a finding, not a quiet difference.
                    eprintln!("{refusal}");
                    ExitCode::from(1)
                }
            }
        }
        Some("method") => {
            // A paragraph number looks like one (§N.M or N.M); a
            // chapter is asked for by name; anything else that is not
            // an existing directory is a typo, and a typo is refused
            // rather than swallowed into "show me the contents"
            // (review 0027 R-8).
            let looks_like_a_paragraph = |word: &str| {
                if word.starts_with('§') {
                    return true;
                }
                word.split_once('.').is_some_and(|(head, tail)| {
                    !head.is_empty()
                        && !tail.is_empty()
                        && head.chars().all(|c| c.is_ascii_digit())
                        && tail.chars().all(|c| c.is_ascii_digit())
                })
            };
            let mut asked: Option<String> = None;
            let mut root = PathBuf::from(".");
            let mut named_root = false;
            for word in args.iter().skip(1) {
                if asked.is_none() && (looks_like_a_paragraph(word) || !Path::new(word).is_dir()) {
                    asked = Some(word.clone());
                    continue;
                }
                if named_root {
                    eprintln!("{}", t("main-usage"));
                    return ExitCode::from(2);
                }
                named_root = true;
                root = PathBuf::from(word);
            }
            // As for `cuts` (review 0028 R-5): the language decides
            // WHICH NORMATIVE DOCUMENT is read, so a config that does
            // not parse or names a language this release does not
            // carry is a refusal, not a silent fall back to English.
            let lang = match keel::config::read_unpinned(&root) {
                Ok(config) => config.lang,
                Err(refusal) => {
                    eprintln!("{refusal}");
                    return ExitCode::from(2);
                }
            };
            keel::i18n::init(&lang);
            match keel::speak::method(&lang, asked.as_deref()) {
                Ok(said) => {
                    print!("{said}");
                    ExitCode::SUCCESS
                }
                Err(refusal) => {
                    eprintln!("{refusal}");
                    ExitCode::from(2)
                }
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
