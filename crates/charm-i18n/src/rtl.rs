//! Right-to-left detection and visual reordering of text.

use crate::bidi::reverse_visual;
use crate::locale::{is_script_subtag, normalize_locale};

/// Language codes (ISO 639-1 and 639-3) written from right to left.
const RTL_LANGUAGES: &[&str] = &[
    "ar", "arc", "ckb", "dv", "fa", "he", "iw", "ji", "khw", "ks", "mzn", "nqo", "ps", "sd", "syr",
    "ug", "ur", "yi",
];

/// Script codes (ISO 15924) written from right to left.
const RTL_SCRIPTS: &[&str] = &[
    "Adlm", "Arab", "Armi", "Avst", "Chrs", "Elym", "Hatr", "Hung", "Mand", "Mani", "Mend", "Narb",
    "Nbat", "Nkoo", "Orkh", "Ougr", "Phlp", "Phnx", "Rohg", "Samr", "Sogo", "Syrc", "Thaa", "Yezi",
];

/// Reports whether `locale` is written from right to left.
///
/// The script subtag takes precedence over the language when it is present:
/// `az-Arab` is right to left, `az-Latn` is not.
///
/// # Examples
///
/// ```
/// assert!(charm_i18n::is_rtl("ar-EG"));
/// assert!(charm_i18n::is_rtl("az-Arab"));
/// assert!(!charm_i18n::is_rtl("pt-BR"));
/// ```
pub fn is_rtl(locale: &str) -> bool {
    let normalized = normalize_locale(locale);
    if normalized.is_empty() {
        return false;
    }

    let mut parts = normalized.split('-');
    let language = parts.next().unwrap_or_default();
    for part in parts {
        if is_script_subtag(part) {
            return RTL_SCRIPTS.contains(&part);
        }
    }
    RTL_LANGUAGES.contains(&language)
}

/// Reorders `text` into visual order (left to right) using the bidirectional
/// algorithm of the Unicode standard (UAX #9) over grapheme clusters.
///
/// Terminals that do not reorder bidirectional text display the returned
/// string correctly. Text without right-to-left characters is returned
/// unchanged.
///
/// # Examples
///
/// ```
/// assert_eq!(charm_i18n::reverse("Hello"), "Hello");
/// assert_eq!(charm_i18n::reverse("\u{5e9}\u{5dc}\u{5d5}\u{5dd}"), "\u{5dd}\u{5d5}\u{5dc}\u{5e9}");
/// ```
pub fn reverse(text: &str) -> String {
    reverse_visual(text)
}

#[cfg(test)]
mod tests {
    use super::{is_rtl, reverse};

    #[test]
    fn detects_rtl_locales() {
        let cases = [
            ("ar", true),
            ("ar-EG", true),
            ("he", true),
            ("he-IL", true),
            ("fa", true),
            ("fa-IR", true),
            ("ur", true),
            ("ps", true),
            ("dv", true),
            ("syr", true),
            ("nqo", true),
            ("yi", true),
            ("az-Arab", true),
            ("az-Latn", false),
            ("pt-BR", false),
            ("en", false),
            ("zh-Hant-TW", false),
            ("", false),
        ];
        for (locale, expected) in cases {
            assert_eq!(is_rtl(locale), expected, "is_rtl({locale:?})");
        }
    }

    #[test]
    fn reverses_hebrew_word_into_visual_order() {
        assert_eq!(
            reverse("\u{5e9}\u{5dc}\u{5d5}\u{5dd}"),
            "\u{5dd}\u{5d5}\u{5dc}\u{5e9}"
        );
    }

    #[test]
    fn reverse_keeps_ltr_text_and_empty_input() {
        assert_eq!(reverse("Hello"), "Hello");
        assert_eq!(reverse(""), "");
    }

    #[test]
    fn reverse_keeps_zwj_emoji_intact() {
        let family = "\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}";
        assert_eq!(reverse(family), family);
    }
}
