//! Locale codes: normalization, POSIX pseudo-locales and base languages.

/// Locale used when no other can be detected or when the input is empty.
pub(crate) const DEFAULT_LOCALE: &str = "en";

/// Canonical storage key for a locale: [`normalize_locale`] when that yields
/// a value, otherwise the trimmed input.
pub(crate) fn locale_key(locale: &str) -> String {
    let key = normalize_locale(locale);
    if key.is_empty() {
        locale.trim().to_owned()
    } else {
        key
    }
}

/// Extracts the locale code from an environment variable value.
///
/// The encoding (`.UTF-8`) and the modifier (`@euro`) are removed before the
/// POSIX pseudo-locales `C` and `POSIX` are recognized, so `C.UTF-8` maps to
/// `en` as well. Any other value is normalized.
///
/// Examples: `pt_BR.UTF-8` becomes `pt-BR`, `PT_br.UTF-8` becomes `pt-BR`,
/// `C.UTF-8` becomes `en`.
pub(crate) fn parse_locale(raw: &str) -> String {
    let raw = raw.trim();
    if raw.is_empty() {
        return DEFAULT_LOCALE.to_owned();
    }

    let base = strip_encoding_and_modifier(raw);
    if base.is_empty() {
        return DEFAULT_LOCALE.to_owned();
    }
    if matches!(base.to_lowercase().as_str(), "c" | "posix") {
        return DEFAULT_LOCALE.to_owned();
    }

    let normalized = normalize_locale(base);
    if normalized.is_empty() {
        DEFAULT_LOCALE.to_owned()
    } else {
        normalized
    }
}

/// Canonicalizes a locale identifier to the `ll-Ssss-CC` form: language in
/// lowercase, script in title case and region in uppercase.
///
/// Examples: `PT_br` becomes `pt-BR`, `zh_hant_tw` becomes `zh-Hant-TW`.
pub(crate) fn normalize_locale(locale: &str) -> String {
    let locale = strip_encoding_and_modifier(locale);
    let parts: Vec<&str> = locale
        .split(['-', '_'])
        .filter(|part| !part.is_empty())
        .collect();

    let normalized: Vec<String> = parts
        .iter()
        .enumerate()
        .map(|(index, part)| {
            if index == 0 {
                part.to_lowercase()
            } else if is_script_subtag(part) {
                title_case(part)
            } else if is_region_subtag(part) {
                part.to_uppercase()
            } else {
                part.to_lowercase()
            }
        })
        .collect();

    normalized.join("-")
}

/// Returns the language subtag of a locale: `pt-BR` becomes `pt`.
pub(crate) fn base_language(locale: &str) -> String {
    let locale = locale.trim();
    locale
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_lowercase()
}

/// Removes the encoding (`.UTF-8`) and the modifier (`@euro`) of a locale.
fn strip_encoding_and_modifier(locale: &str) -> &str {
    locale.split(['.', '@']).next().unwrap_or(locale).trim()
}

/// Reports whether `part` is an ISO 15924 script code (four ASCII letters).
pub(crate) fn is_script_subtag(part: &str) -> bool {
    part.len() == 4 && part.bytes().all(|b| b.is_ascii_alphabetic())
}

/// Reports whether `part` is an ISO 3166-1 region code (two ASCII letters or
/// three digits).
fn is_region_subtag(part: &str) -> bool {
    (part.len() == 2 && part.bytes().all(|b| b.is_ascii_alphabetic()))
        || (part.len() == 3 && part.bytes().all(|b| b.is_ascii_digit()))
}

/// Uppercases the first character and lowercases the rest.
fn title_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{base_language, locale_key, normalize_locale, parse_locale};

    #[test]
    fn normalize_locale_table() {
        let cases = [
            ("pt_BR", "pt-BR"),
            ("PT_br", "pt-BR"),
            ("PT-BR", "pt-BR"),
            ("pt", "pt"),
            ("EN", "en"),
            ("zh_Hant_TW", "zh-Hant-TW"),
            ("zh-hant-tw", "zh-Hant-TW"),
            ("es_419", "es-419"),
            ("pt_BR.UTF-8", "pt-BR"),
            ("de_DE.UTF-8@euro", "de-DE"),
            ("  pt-BR  ", "pt-BR"),
            ("", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(normalize_locale(input), expected, "normalize({input:?})");
        }
    }

    #[test]
    fn base_language_table() {
        let cases = [
            ("pt-BR", "pt"),
            ("pt_BR", "pt"),
            ("pt", "pt"),
            ("zh-Hant-TW", "zh"),
            ("", ""),
        ];
        for (input, expected) in cases {
            assert_eq!(base_language(input), expected, "base({input:?})");
        }
    }

    #[test]
    fn locale_key_normalizes_and_keeps_empty_input() {
        assert_eq!(locale_key("PT_br"), "pt-BR");
        assert_eq!(locale_key("  "), "");
    }

    #[test]
    fn parse_locale_table() {
        let cases = [
            ("pt_BR.UTF-8", "pt-BR"),
            ("PT_br.UTF-8", "pt-BR"),
            ("en_US", "en-US"),
            ("C", "en"),
            ("c", "en"),
            ("POSIX", "en"),
            ("posix", "en"),
            ("C.UTF-8", "en"),
            ("c.UTF-8", "en"),
            ("POSIX.UTF-8", "en"),
            ("posix.UTF-8", "en"),
            ("de_DE.UTF-8@euro", "de-DE"),
            ("zh_Hant_TW", "zh-Hant-TW"),
            ("", "en"),
        ];
        for (input, expected) in cases {
            assert_eq!(parse_locale(input), expected, "parse_locale({input:?})");
        }
    }
}
