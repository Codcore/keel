//! Мова виводу інструмента: ключі в коді + один Fluent-файл на мову,
//! вшитий у бінарник; вибір за lang з keel.toml; fallback — англійська
//! (NEW-CONCEPT, «Конфіг → Мови виводу інструмента»).

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

/// Вшиті файли — артефакт нашої збірки; зламаний переклад ловить тест
/// embedded_bundles_parse ще до релізу, тож тут паніка чесна.
fn bundle(lang: &str, source: &str) -> FluentBundle<FluentResource> {
    let id: LanguageIdentifier = lang
        .parse()
        .unwrap_or_else(|e| panic!("embedded language id {lang}: {e}"));
    let resource = FluentResource::try_new(source.to_string())
        .unwrap_or_else(|(_, errors)| panic!("embedded {lang}.ftl is broken: {errors:?}"));
    let mut bundle = FluentBundle::new_concurrent(vec![id]);
    // Без невидимих ізолюючих знаків довкола підстановок: вивід — для
    // терміналів і для порівнянь у тестах, не для двонапрямного HTML.
    bundle.set_use_isolating(false);
    bundle
        .add_resource(resource)
        .unwrap_or_else(|errors| panic!("embedded {lang}.ftl has clashes: {errors:?}"));
    bundle
}

impl I18n {
    /// Мови, вшиті в цей реліз.
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

    /// Для тестів: зібрати з довільних джерел.
    pub fn from_sources(lang: &str, lang_src: &str, en_src: &str) -> I18n {
        I18n {
            lang: bundle(lang, lang_src),
            en: bundle("en", en_src),
        }
    }

    /// Текст ключа: мова проєкту → англійська → сам ключ (діра у
    /// звіті заборонена).
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

/// Мова процесу: ставиться раз у main після читання конфіга; поки не
/// поставлена (бібліотечні тести), діє англійська.
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

/// Аргументи повідомлення одним виразом: targs!("name" => value, ...).
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

    /// proves: missing-key-falls-back@e73fdd — тримає розділ «Мови
    /// виводу» концепту: нема перекладу — англійський текст, не діра.
    #[test]
    fn missing_key_falls_back() {
        // Українському джерелу бракує ключа — приходить англійський.
        let i = I18n::from_sources("uk", "", "hello = Hello");
        assert_eq!(i.text("hello"), "Hello");

        // Ключа нема ніде — повертається сам ключ, а не паніка й не
        // порожнеча: діра у звіті заборонена.
        let real = I18n::embedded("uk");
        assert_eq!(real.text("no-such-key-ever"), "no-such-key-ever");
    }

    /// Обидва вшиті переклади читаються — зламаний .ftl не доїде до
    /// релізу мовчки (застереження трансформи speak-by-keys).
    #[test]
    fn embedded_bundles_parse() {
        for lang in ["en", "uk"] {
            let _ = I18n::embedded(lang);
        }
    }
}
