//! Scenario test of wave 0002-config-and-language, relocated from a
//! unit test in src/i18n.rs by wave 0006: the adapter reads tags in
//! tests/*.rs, and the closure court must see this proof. The body
//! is the same; the tag is unchanged.

use keel::i18n::I18n;

/// proves: missing-key-falls-back@e73fdd -- "Output languages": no
/// translation means English text, not a hole.
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
