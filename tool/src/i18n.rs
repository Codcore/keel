//! The tool's output language: keys in code + one Fluent file per
//! language embedded in the binary; picked by lang from keel.toml;
//! fallback is English (NEW-CONCEPT, "Config -> Tool output
//! languages").

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource};
use std::sync::OnceLock;
use unic_langid::LanguageIdentifier;

static EN_FTL: &str = include_str!("../i18n/en.ftl");
static UK_FTL: &str = include_str!("../i18n/uk.ftl");

pub struct I18n {
    lang: FluentBundle<FluentResource>,
    en: FluentBundle<FluentResource>,
}

/// Embedded files are our build artifact; a broken translation is
/// caught by embedded_bundles_parse before any release, so panicking
/// here is honest.
fn bundle(lang: &str, source: &str) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = lang
        .parse()
        .unwrap_or_else(|e| panic!("embedded language id {lang}: {e}"));
    let resource = FluentResource::try_new(source.to_string())
        .unwrap_or_else(|(_, errors)| panic!("embedded {lang}.ftl is broken: {errors:?}"));
    let mut bundle = FluentBundle::new_concurrent(vec![id]);
    // No invisible isolation marks around placeables: the output is
    // for terminals and test comparisons, not bidirectional HTML.
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .unwrap_or_else(|errors| panic!("embedded {lang}.ftl has clashes: {errors:?}"));
    bundle
}

impl I18n {
    /// Languages embedded in this release.
    pub fn embedded(lang: &str) -> I18n {
        let picked = match lang {
            "uk" => bundle("uk", UK_FTL),
            _ => bundle("en", EN_FTL),
        };
        I18n {
            lang: picked,
            en: bundle("en", EN_FTL),
        }
    }

    /// For tests: build from arbitrary sources.
    pub fn from_sources(lang: &str, lang_src: &str, en_src: &str) -> I18n {
        I18n {
            lang: bundle(lang, lang_src),
            en: bundle("en", en_src),
        }
    }

    /// The text of a key: project language -> English -> the key
    /// itself (a hole in the report is forbidden).
    pub fn text(&self, key: &str) -> String {
        self.format(key, None)
    }

    pub fn with(&self, key: &str, args: &FluentArgs) -> String {
        self.format(key, Some(args))
    }

    fn format(&self, key: &str, args: Option<&FluentArgs>) -> String {
        for bundle in [&self.lang, &self.en] {
            if let Some(message) = bundle.get_message(key)
                && let Some(pattern) = message.value()
            {
                let mut errors = Vec::new();
                return bundle
                    .format_pattern(pattern, args, &mut errors)
                    .into_owned();
            }
        }
        key.to_string()
    }
}

/// The process language: set once in main after the config is read;
/// until set (library tests), English is in effect.
static CURRENT: OnceLock<I18n> = OnceLock::new();

pub fn init(lang: &str) {
    let _ = CURRENT.set(I18n::embedded(lang));
}

fn current() -> &'static I18n {
    CURRENT.get_or_init(|| I18n::embedded("en"))
}

pub fn t(key: &str) -> String {
    current().text(key)
}

pub fn ta(key: &str, args: FluentArgs) -> String {
    current().with(key, &args)
}

/// Message arguments in one expression: targs!("name" => value, ...).
#[macro_export]
macro_rules! targs {
    ($($k:literal => $v:expr),* $(,)?) => {{
        let mut args = fluent_bundle::FluentArgs::new();
        $(args.set($k, $v);)*
        args
    }};
}

#[cfg(test)]
mod tests {
    use super::I18n;

    /// proves: missing-key-falls-back@e73fdd -- holds the concept's
    /// "Output languages": no translation means English text, not a
    /// hole.
    #[test]
    fn missing_key_falls_back() {
        // The Ukrainian source lacks the key -- English arrives.
        let i = I18n::from_sources("uk", "", "hello = Hello");
        assert_eq!(i.text("hello"), "Hello");

        // The key exists nowhere -- the key itself comes back, not a
        // panic and not emptiness: a hole in the report is forbidden.
        let real = I18n::embedded("uk");
        assert_eq!(real.text("no-such-key-ever"), "no-such-key-ever");
    }

    /// Both embedded translations parse -- a broken .ftl cannot ship
    /// silently (a caveat of the speak-by-keys transform).
    #[test]
    fn embedded_bundles_parse() {
        for lang in ["en", "uk"] {
            let _ = I18n::embedded(lang);
        }
    }
}
