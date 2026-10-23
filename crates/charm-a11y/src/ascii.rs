use std::collections::{HashMap, HashSet};
use std::sync::{LazyLock, Mutex, PoisonError};

use crate::ascii_table::FALLBACKS;
use crate::profile::Profile;
use crate::strip::strip_ansi;

static GLYPHS: LazyLock<HashMap<char, &'static str>> = LazyLock::new(|| {
    FALLBACKS
        .iter()
        .filter_map(|(glyph, ascii)| glyph.chars().next().map(|c| (c, *ascii)))
        .collect()
});

static INTERNED: LazyLock<Mutex<HashSet<&'static str>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Replaces known Unicode glyphs with their ASCII equivalents.
///
/// Characters that have no ASCII equivalent in the table are kept as they are.
/// Multi-character replacements are intentional (`…` becomes `...`), so the
/// width of the result can differ from the input.
///
/// # Examples
///
/// ```
/// assert_eq!(charm_a11y::transliterate("┌─┐ → ✓"), "+-+ -> v");
/// assert_eq!(charm_a11y::transliterate("世界"), "世界");
/// ```
pub fn transliterate(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match GLYPHS.get(&c) {
            Some(ascii) => out.push_str(ascii),
            None => out.push(c),
        }
    }
    out
}

/// Transliterates a `'static` border part, returning a `'static` result.
///
/// Single known glyphs reuse the table value. Parts that mix glyphs are
/// interned once, so the leak is bounded by the number of distinct parts.
pub(crate) fn transliterate_static(s: &'static str) -> &'static str {
    if !s.chars().any(|c| GLYPHS.contains_key(&c)) {
        return s;
    }
    let mut chars = s.chars();
    if let (Some(only), None) = (chars.next(), chars.next()) {
        if let Some(ascii) = GLYPHS.get(&only) {
            return ascii;
        }
    }
    intern(&transliterate(s))
}

fn intern(s: &str) -> &'static str {
    let mut set = INTERNED.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(found) = set.get(s) {
        return found;
    }
    let leaked: &'static str = Box::leak(s.to_owned().into_boxed_str());
    set.insert(leaked);
    leaked
}

impl Profile {
    /// Adapts `s` to the profile.
    ///
    /// ANSI and OSC sequences are always removed. Without Unicode support, the
    /// glyphs in the transliteration table are replaced with ASCII. Other
    /// glyphs are preserved.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_a11y::Profile;
    ///
    /// let ascii = Profile::default();
    /// assert_eq!(ascii.sanitize("┌─┐\n│✓│"), "+-+\n|v|");
    /// ```
    pub fn sanitize(&self, s: &str) -> String {
        let out = strip_ansi(s);
        if self.unicode {
            out
        } else {
            transliterate(&out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{transliterate, transliterate_static, FALLBACKS};
    use crate::profile::Profile;

    const REQUIRED: &[(&str, &str)] = &[
        ("─", "-"),
        ("│", "|"),
        ("┌", "+"),
        ("┐", "+"),
        ("└", "+"),
        ("┘", "+"),
        ("├", "+"),
        ("┤", "+"),
        ("┬", "+"),
        ("┴", "+"),
        ("┼", "+"),
        ("╭", "+"),
        ("╮", "+"),
        ("╰", "+"),
        ("╯", "+"),
        ("▀", "^"),
        ("▄", "_"),
        ("█", "#"),
        ("░", "."),
        ("▒", ":"),
        ("▓", "#"),
        ("▸", ">"),
        ("▾", "v"),
        ("●", "o"),
        ("○", "o"),
        ("✓", "v"),
        ("✗", "x"),
        ("…", "..."),
        ("→", "->"),
        ("←", "<-"),
        ("↑", "^"),
        ("↓", "v"),
        ("━", "-"),
        ("┃", "|"),
        ("═", "="),
        ("║", "|"),
        ("▶", ">"),
        ("◀", "<"),
        ("•", "*"),
        ("≤", "<="),
        ("≥", ">="),
        ("⚠", "(!)"),
    ];

    #[test]
    fn transliterate_required_glyphs() {
        let profile = Profile::default();
        for (glyph, want) in REQUIRED {
            assert_eq!(transliterate(glyph), *want, "glyph {glyph}");
            assert_eq!(
                profile.sanitize(&format!("[{glyph}]")),
                format!("[{want}]"),
                "glyph {glyph}"
            );
        }
    }

    #[test]
    fn ascii_table_is_complete_and_ascii() {
        for (glyph, want) in FALLBACKS {
            assert_eq!(transliterate(glyph), *want, "glyph {glyph}");
            assert!(want.is_ascii(), "{glyph} maps to non-ASCII {want:?}");
        }
    }

    #[test]
    fn sanitize_mixed_text() {
        let profile = Profile::default();
        let input = "\x1b[1m┌─┐\x1b[0m\n│✓│\n└─┘ → 100% …";
        assert_eq!(profile.sanitize(input), "+-+\n|v|\n+-+ -> 100% ...");
    }

    #[test]
    fn sanitize_preserves_unicode_and_strips_ansi() {
        let profiles = [
            Profile {
                unicode: true,
                ..Profile::default()
            },
            Profile {
                no_color: true,
                unicode: true,
                ..Profile::default()
            },
        ];
        for profile in profiles {
            assert_eq!(profile.sanitize("\x1b[31m┌─┐ ✓ 世界\x1b[0m"), "┌─┐ ✓ 世界");
        }
    }

    #[test]
    fn sanitize_unknown_glyph_is_preserved() {
        assert_eq!(Profile::default().sanitize("世界"), "世界");
    }

    #[test]
    fn sanitize_empty() {
        assert_eq!(Profile::default().sanitize(""), "");
    }

    #[test]
    fn sanitize_is_deterministic() {
        let profile = Profile::default();
        let input = "┌─┐ ✓ → … █";
        let want = profile.sanitize(input);
        for i in 0..100 {
            assert_eq!(profile.sanitize(input), want, "iteration {i}");
        }
    }

    #[test]
    fn sanitize_does_not_change_ascii_text() {
        let got = Profile::default().sanitize("ASCII puro: a[0] -> b");
        assert_eq!(got, "ASCII puro: a[0] -> b");
    }

    #[test]
    fn transliterate_static_reuses_table_and_interns_mixed_parts() {
        assert_eq!(transliterate_static("─"), "-");
        assert_eq!(transliterate_static("═══"), "===");
        assert_eq!(transliterate_static("═══"), "===");
        assert_eq!(transliterate_static("世界"), "世界");
        assert_eq!(transliterate_static(""), "");
    }
}
