//! Styled ASCII art banners for terminal user interfaces.
//!
//! `charm-banner` turns plain text into large ASCII art built with Unicode
//! block elements (`▀`, `▄`, `█`, `░`) and renders it with three built-in
//! bitmap fonts: **Small** (3 rows), **Medium** (5 rows) and **Large**
//! (7 rows).
//!
//! This crate is a Rust port of the Go library
//! [`charm-banner`](https://github.com/junhinhow/charm-banner) and keeps its
//! behavior byte-for-byte compatible, including the font data, which was
//! extracted programmatically from the original Go source.
//!
//! # Quick start
//!
//! ```
//! use charm_banner::{render, Font, RenderOption};
//!
//! // The default font is `Medium`.
//! let banner = render("HELLO", &[]);
//! assert_eq!(banner.lines().count(), 5);
//!
//! // Pick another font.
//! let small = render("HI", &[RenderOption::Font(Font::Small)]);
//! assert_eq!(small.lines().count(), 3);
//! ```
//!
//! # Styling
//!
//! Any [`lipgloss::Style`] can be applied to the rendered banner:
//!
//! ```
//! use charm_banner::{render, RenderOption};
//! use lipgloss::Style;
//!
//! let style = Style::new().bold(true);
//! let banner = render("CHARM", &[RenderOption::Style(style)]);
//! assert!(!banner.is_empty());
//! ```
//!
//! # Options
//!
//! The canonical API is [`render`], which takes a slice of [`RenderOption`].
//! A builder-style alternative is available through [`RenderOptions`] and
//! [`render_with`]:
//!
//! ```
//! use charm_banner::{render_with, Font, RenderOptions};
//!
//! let opts = RenderOptions::new().with_font(Font::Large);
//! let banner = render_with("GO", &opts);
//! assert_eq!(banner.lines().count(), 7);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod fonts;

use std::collections::HashMap;
use std::fmt;

pub use fonts::{LARGE_FONT, MEDIUM_FONT, SMALL_FONT};

/// A bitmap pattern describing how a single character is drawn.
///
/// Each entry of [`CharPattern::rows`] is one line of the rendered banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CharPattern {
    /// The lines of the pattern, from top to bottom.
    pub rows: &'static [&'static str],
}

/// The size of the bitmap font used to draw a banner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Font {
    /// Small font: 3 rows tall.
    Small,
    /// Medium font: 5 rows tall. This is the default.
    #[default]
    Medium,
    /// Large font: 7 rows tall.
    Large,
}

impl Font {
    /// Number of rows of a banner drawn with this font.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::Font;
    ///
    /// assert_eq!(Font::Small.height(), 3);
    /// assert_eq!(Font::Medium.height(), 5);
    /// assert_eq!(Font::Large.height(), 7);
    /// ```
    pub const fn height(self) -> usize {
        match self {
            Font::Small => 3,
            Font::Medium => 5,
            Font::Large => 7,
        }
    }

    /// Nominal number of columns of a glyph drawn with this font.
    ///
    /// The nominal width is the one advertised by the Go original
    /// (`fontWidth`), which is not used while rendering. The glyphs of the Go
    /// fonts are in fact 3, 6 and 9 columns wide respectively.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::Font;
    ///
    /// assert_eq!(Font::Small.width(), 3);
    /// assert_eq!(Font::Medium.width(), 5);
    /// assert_eq!(Font::Large.width(), 7);
    /// ```
    pub const fn width(self) -> usize {
        match self {
            Font::Small => 3,
            Font::Medium => 5,
            Font::Large => 7,
        }
    }

    /// Lowercase name of the font (`"small"`, `"medium"` or `"large"`).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::Font;
    ///
    /// assert_eq!(Font::Large.name(), "large");
    /// ```
    pub const fn name(self) -> &'static str {
        match self {
            Font::Small => "small",
            Font::Medium => "medium",
            Font::Large => "large",
        }
    }

    /// The glyph map of this font.
    ///
    /// Characters that are not present in the map fall back to the pattern of
    /// the space character, exactly like the Go original.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::Font;
    ///
    /// let glyphs = Font::Medium.glyphs();
    /// assert!(glyphs.contains_key(&'A'));
    /// assert!(glyphs.contains_key(&' '));
    /// ```
    pub fn glyphs(self) -> &'static HashMap<char, CharPattern> {
        match self {
            Font::Small => &SMALL_FONT,
            Font::Medium => &MEDIUM_FONT,
            Font::Large => &LARGE_FONT,
        }
    }
}

/// An option that customizes how a banner is rendered.
///
/// Options are applied in order, so the last one wins. This mirrors the
/// variadic `Option` list of the Go original.
// The style is kept by value (as in the Go original, where `WithStyle` takes a
// `lipgloss.Style`), which makes the enum bigger than its variants suggest.
#[allow(clippy::large_enum_variant)]
#[derive(Clone)]
pub enum RenderOption {
    /// Applies a lipgloss style to the whole rendered banner.
    Style(lipgloss::Style),
    /// Selects the bitmap font used to draw the banner.
    Font(Font),
}

impl fmt::Debug for RenderOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderOption::Style(_) => f.write_str("RenderOption::Style(<lipgloss::Style>)"),
            RenderOption::Font(font) => f.debug_tuple("RenderOption::Font").field(font).finish(),
        }
    }
}

/// A builder-style collection of [`RenderOption`] values.
///
/// This is a convenience wrapper around the canonical [`RenderOption`] API;
/// both produce exactly the same output.
///
/// # Examples
///
/// ```
/// use charm_banner::{render_with, Font, RenderOptions};
///
/// let opts = RenderOptions::new().with_font(Font::Small);
/// assert_eq!(render_with("A", &opts).lines().count(), 3);
/// ```
#[derive(Clone, Default)]
pub struct RenderOptions {
    font: Font,
    style: Option<lipgloss::Style>,
}

impl fmt::Debug for RenderOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RenderOptions")
            .field("font", &self.font)
            .field("style", &self.style.as_ref().map(|_| "<lipgloss::Style>"))
            .finish()
    }
}

impl RenderOptions {
    /// Creates the default options: the `Medium` font and no style.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::RenderOptions;
    ///
    /// let opts = RenderOptions::new();
    /// assert_eq!(opts.font(), charm_banner::Font::Medium);
    /// ```
    pub fn new() -> Self {
        Self {
            font: Font::default(),
            style: None,
        }
    }

    /// Selects the bitmap font.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::{Font, RenderOptions};
    ///
    /// let opts = RenderOptions::new().with_font(Font::Large);
    /// assert_eq!(opts.font(), Font::Large);
    /// ```
    #[must_use]
    pub fn with_font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    /// Sets the lipgloss style applied to the banner.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::RenderOptions;
    /// use lipgloss::Style;
    ///
    /// let opts = RenderOptions::new().with_style(Style::new().bold(true));
    /// assert!(opts.style().is_some());
    /// ```
    #[must_use]
    pub fn with_style(mut self, style: lipgloss::Style) -> Self {
        self.style = Some(style);
        self
    }

    /// The font selected by these options.
    pub const fn font(&self) -> Font {
        self.font
    }

    /// The style selected by these options, if any.
    pub fn style(&self) -> Option<&lipgloss::Style> {
        self.style.as_ref()
    }

    /// Converts the options into the canonical [`RenderOption`] list.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_banner::{render, Font, RenderOptions};
    ///
    /// let opts = RenderOptions::new().with_font(Font::Small);
    /// let banner = render("HI", &opts.to_options());
    /// assert_eq!(banner.lines().count(), 3);
    /// ```
    pub fn to_options(&self) -> Vec<RenderOption> {
        let mut options = Vec::with_capacity(2);
        options.push(RenderOption::Font(self.font));
        if let Some(style) = &self.style {
            options.push(RenderOption::Style(style.clone()));
        }
        options
    }
}

/// Renders `text` as a stylized banner.
///
/// The text is upper-cased, every character is replaced by its bitmap glyph
/// (characters without a glyph fall back to the space glyph) and the glyphs
/// are joined by a single blank column. The result is optionally styled with
/// the [`RenderOption::Style`] option.
///
/// # Examples
///
/// ```
/// use charm_banner::{render, Font, RenderOption};
///
/// // Default font: Medium.
/// let banner = render("HELLO", &[]);
/// assert_eq!(banner.lines().count(), 5);
///
/// // Small font.
/// let small = render("HELLO", &[RenderOption::Font(Font::Small)]);
/// assert_eq!(small.lines().count(), 3);
///
/// // Lower case input is upper-cased.
/// assert_eq!(render("abc", &[]), render("ABC", &[]));
/// ```
pub fn render(text: &str, opts: &[RenderOption]) -> String {
    let mut font = Font::default();
    let mut style: Option<lipgloss::Style> = None;

    for opt in opts {
        match opt {
            RenderOption::Style(s) => style = Some(s.clone()),
            RenderOption::Font(f) => font = *f,
        }
    }

    render_inner(text, font, style.as_ref())
}

/// Renders `text` as a stylized banner using builder-style options.
///
/// This is the convenience counterpart of [`render`]; both functions produce
/// exactly the same output.
///
/// # Examples
///
/// ```
/// use charm_banner::{render_with, Font, RenderOptions};
///
/// let opts = RenderOptions::new().with_font(Font::Large);
/// let banner = render_with("GO", &opts);
/// assert_eq!(banner.lines().count(), 7);
/// ```
pub fn render_with(text: &str, opts: &RenderOptions) -> String {
    render_inner(text, opts.font, opts.style())
}

/// Pattern used when a font has no glyph for a character and no space glyph.
static EMPTY_PATTERN: CharPattern = CharPattern { rows: &[] };

fn render_inner(text: &str, font: Font, style: Option<&lipgloss::Style>) -> String {
    let glyphs = font.glyphs();
    let upper = to_upper(text);
    let height = font.height();
    let mut lines: Vec<String> = vec![String::new(); height];

    // The Go original compares the *byte* offset of the rune with the *byte*
    // length of the string, so a blank column is also appended after a
    // multi-byte character. Reproduced here for byte-for-byte parity.
    let last_offset = upper.len().saturating_sub(1);

    for (offset, ch) in upper.char_indices() {
        let pattern = glyphs
            .get(&ch)
            .unwrap_or_else(|| glyphs.get(&' ').unwrap_or(&EMPTY_PATTERN));

        for (row, line) in lines.iter_mut().enumerate() {
            if let Some(pattern_row) = pattern.rows.get(row) {
                line.push_str(pattern_row);
            }
        }

        // Blank column between the glyphs.
        if offset < last_offset {
            for line in &mut lines {
                line.push(' ');
            }
        }
    }

    let result = lines.join("\n");

    match style {
        Some(style) => style.render(&result),
        None => result,
    }
}

/// Upper-cases `text` the way Go's `strings.ToUpper` does.
///
/// Go applies the *simple* Unicode uppercase mapping to every rune, so `ß`
/// stays `ß`. Rust's [`str::to_uppercase`] applies the *full* mapping and
/// would expand it to `SS`, changing the number of rendered glyphs.
fn to_upper(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for ch in text.chars() {
        let mut mapped = ch.to_uppercase();
        let first = mapped.next();
        let expanded = mapped.next().is_some();
        let upper = match (first, expanded) {
            (Some(single), false) => single,
            _ => ch,
        };
        out.push(upper);
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use lipgloss::utils::strip_ansi;

    #[test]
    fn font_heights_match_the_go_original() {
        assert_eq!(Font::Small.height(), 3);
        assert_eq!(Font::Medium.height(), 5);
        assert_eq!(Font::Large.height(), 7);
    }

    #[test]
    fn font_widths_match_the_go_original() {
        assert_eq!(Font::Small.width(), 3);
        assert_eq!(Font::Medium.width(), 5);
        assert_eq!(Font::Large.width(), 7);
    }

    #[test]
    fn default_font_is_medium() {
        assert_eq!(Font::default(), Font::Medium);
        assert_eq!(render("A", &[]).lines().count(), 5);
    }

    #[test]
    fn every_font_has_the_expected_number_of_glyphs() {
        for font in [Font::Small, Font::Medium, Font::Large] {
            assert_eq!(font.glyphs().len(), 41, "fonte {}", font.name());
        }
    }

    #[test]
    fn every_font_contains_the_expected_characters() {
        for font in [Font::Small, Font::Medium, Font::Large] {
            let glyphs = font.glyphs();
            for ch in 'A'..='Z' {
                assert!(glyphs.contains_key(&ch), "fonte {} sem `{ch}`", font.name());
            }
            for ch in '0'..='9' {
                assert!(glyphs.contains_key(&ch), "fonte {} sem `{ch}`", font.name());
            }
            for ch in [' ', '!', '.', '-', ':'] {
                assert!(glyphs.contains_key(&ch), "fonte {} sem `{ch}`", font.name());
            }
        }
    }

    #[test]
    fn glyph_rows_have_the_expected_height() {
        for font in [Font::Small, Font::Medium, Font::Large] {
            for (ch, pattern) in font.glyphs() {
                assert_eq!(
                    pattern.rows.len(),
                    font.height(),
                    "glifo `{ch}` da fonte {}",
                    font.name()
                );
            }
        }
    }

    #[test]
    fn unknown_character_falls_back_to_the_space_glyph() {
        let expected = render(" ", &[RenderOption::Font(Font::Small)]);
        let actual = render("@", &[RenderOption::Font(Font::Small)]);
        assert_eq!(expected, actual);
    }

    #[test]
    fn empty_text_renders_empty_lines() {
        let banner = render("", &[]);
        assert_eq!(banner, "\n\n\n\n");
        // `str::lines` drops the trailing empty line, `split` does not.
        assert_eq!(banner.split('\n').count(), 5);
    }

    #[test]
    fn text_is_upper_cased() {
        assert_eq!(render("abc", &[]), render("ABC", &[]));
    }

    #[test]
    fn last_option_wins() {
        let opts = [
            RenderOption::Font(Font::Small),
            RenderOption::Font(Font::Large),
        ];
        assert_eq!(render("A", &opts).lines().count(), 7);
    }

    #[test]
    fn style_is_applied_to_the_output() {
        // Sem TTY (CI) o `lipgloss` detecta `NoColor` e remove todo o SGR.
        // Forcamos truecolor para que a assercao seja independente do ambiente.
        lipgloss::renderer::set_color_profile(lipgloss::renderer::ColorProfileKind::TrueColor);

        let style = lipgloss::Style::new().bold(true);
        let styled = render("A", &[RenderOption::Style(style.clone())]);
        let plain = render("A", &[]);
        assert_eq!(strip_ansi(&styled), plain);
        assert!(styled.contains('\u{1b}'));
    }

    #[test]
    fn builder_options_match_the_canonical_api() {
        let opts = RenderOptions::new().with_font(Font::Large);
        assert_eq!(render_with("A", &opts), render("A", &opts.to_options()));
    }

    #[test]
    fn to_upper_keeps_the_simple_unicode_mapping() {
        assert_eq!(to_upper("abc"), "ABC");
        assert_eq!(to_upper("ß"), "ß");
        assert_eq!(to_upper("a-z"), "A-Z");
    }

    #[test]
    fn blank_column_is_added_between_glyphs() {
        let banner = render("AB", &[RenderOption::Font(Font::Small)]);
        let lines: Vec<&str> = banner.lines().collect();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "▄█▄ ██▄");
    }
}
