//! Мова виводу інструмента: ключі + один Fluent-файл на мову,
//! вшитий у бінарник; вибір за lang з keel.toml; fallback — англійська.

pub struct I18n(());

impl I18n {
    /// Вшиті мови цього релізу.
    pub fn embedded(lang: &str) -> I18n {
        let _ = lang;
        todo!("трансформа speak-by-keys")
    }

    /// Для тестів: зібрати з довільних джерел.
    pub fn from_sources(lang: &str, lang_src: &str, en_src: &str) -> I18n {
        let _ = (lang, lang_src, en_src);
        todo!("трансформа speak-by-keys")
    }

    /// Текст ключа: мова проєкту → англійська → сам ключ.
    pub fn text(&self, key: &str) -> String {
        let _ = key;
        todo!("трансформа speak-by-keys")
    }
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
