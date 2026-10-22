//! Color palette viewer and generator for the terminal.
//!
//! The crate renders color swatches, the 256-color ANSI grid, true-color
//! gradients and named themes, and can compare several themes side by side.
//! It is a faithful port of the Go `charm-palette` library.
//!
//! Every function returns a plain `String` containing ANSI escape sequences;
//! the caller decides where to print it.
//!
//! # Examples
//!
//! ```
//! use charm_core::Rgb;
//! use charm_palette::show_palette;
//!
//! let swatches = show_palette(&[Rgb::rgb(255, 0, 0), Rgb::rgb(0, 255, 0)], 24);
//! assert_eq!(swatches.lines().count(), 2);
//! ```
//!
//! # Known differences from the Go original
//!
//! * The Go implementation receives `image/color.Color` values and reads them
//!   through `RGBA()`, which returns **premultiplied** channels. This port uses
//!   the raw channels of [`Rgb`] instead, so the swatches keep the colour the
//!   caller asked for. For fully opaque colours — the common case — both
//!   implementations produce exactly the same output.
//! * The hexadecimal label rendered next to a swatch always shows the opaque
//!   `#rrggbb` triplet, even for translucent colours.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::BTreeMap;

use charm_core::Rgb;
use lipgloss::{Color, Style};

/// Converts a `usize` into an `i32`, saturating instead of wrapping around.
fn cells(width: usize) -> i32 {
    i32::try_from(width).unwrap_or(i32::MAX)
}

/// Renders `text` centered inside a block of `width` cells.
///
/// This reproduces the exact layout of the Go lipgloss, which differs from the
/// Rust `lipgloss` port in two subtle ways:
///
/// * the Go implementation gives the extra cell of an odd gap to the **right**
///   side (`left = gap / 2`), while the Rust port rounds the left padding up;
/// * the Go implementation styles the padding with the *whitespace* style
///   (the background colour, when there is one), while the Rust port leaves
///   the padding unstyled.
///
/// * `style` styles the content;
/// * `whitespace` styles the padding.
fn render_centered(text: &str, width: usize, style: &Style, whitespace: &Style) -> String {
    let lines = wrap_text(text, width);
    let widest = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);

    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let gap = width.max(widest).saturating_sub(line.chars().count());
        let left = gap / 2;
        let right = gap - left;

        if left > 0 {
            out.push_str(&whitespace.render(&" ".repeat(left)));
        }
        out.push_str(&style.render(line));
        if right > 0 {
            out.push_str(&whitespace.render(&" ".repeat(right)));
        }
        if i + 1 < lines.len() {
            out.push('\n');
        }
    }
    out
}

/// Wraps `text` into lines of at most `width` characters.
///
/// The Go lipgloss breaks on spaces and hard-breaks words that are longer than
/// the line width. The palette only renders single tokens (hexadecimal values
/// and short labels), so a plain hard break is enough.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    if text.chars().count() <= width {
        return vec![text.to_owned()];
    }
    text.chars()
        .collect::<Vec<char>>()
        .chunks(width)
        .map(|chunk| chunk.iter().collect())
        .collect()
}

/// Renders color swatches with the given total width.
///
/// Each color is drawn as a filled block with its hexadecimal value below it.
/// The width is divided equally between the swatches, with a minimum of four
/// columns per swatch.
///
/// # Examples
///
/// ```
/// use charm_core::Rgb;
/// use charm_palette::show_palette;
///
/// let out = show_palette(&[Rgb::rgb(0xff, 0x00, 0x00)], 12);
/// assert!(out.contains("#ff0000"));
/// ```
pub fn show_palette(colors: &[Rgb], width: usize) -> String {
    if colors.is_empty() {
        return String::new();
    }

    let mut swatch_width = width / colors.len();
    if swatch_width < 4 {
        swatch_width = 4;
    }

    let mut top_row = String::new();
    let mut bottom_row = String::new();

    for color in colors {
        let hex = color.to_hex();

        let swatch = Style::new()
            .background(color.to_lipgloss())
            .width(cells(swatch_width))
            .render(&" ".repeat(swatch_width));

        let label = render_centered(
            &hex,
            swatch_width,
            &Style::new().foreground(color.to_lipgloss()),
            &Style::new(),
        );

        top_row.push_str(&swatch);
        bottom_row.push_str(&label);
    }

    format!("{top_row}\n{bottom_row}")
}

/// Displays the 256 ANSI colors in an organized grid.
///
/// The grid is split in the standard colors (`0-15`), the 6x6x6 color cube
/// (`16-231`) and the grayscale ramp (`232-255`).
///
/// # Examples
///
/// ```
/// use charm_palette::show_ansi256;
///
/// let out = show_ansi256();
/// assert!(out.contains("Standard Colors (0-15)"));
/// assert!(out.contains("Color Cube (16-231)"));
/// assert!(out.contains("Grayscale (232-255)"));
/// ```
pub fn show_ansi256() -> String {
    let mut sb = String::new();

    sb.push_str("  Standard Colors (0-15)\n");
    // Standard colors 0-7.
    for i in 0..8u32 {
        let style = Style::new().background(Color(i.to_string()));
        sb.push_str(&render_centered(&format!("{i:>3}"), 4, &style, &style));
    }
    sb.push('\n');
    // Bright colors 8-15.
    for i in 8..16u32 {
        let style = Style::new()
            .background(Color(i.to_string()))
            .foreground(Color("0".to_string()));
        sb.push_str(&render_centered(
            &format!("{i:>3}"),
            4,
            &style,
            &Style::new().background(Color(i.to_string())),
        ));
    }
    sb.push_str("\n\n");

    // Color cube 16-231.
    sb.push_str("  Color Cube (16-231)\n");
    for row in 0..12usize {
        for col in 0..18usize {
            let idx = 16 + row * 18 + col;
            if idx > 231 {
                break;
            }
            let fg = if row > 5 { "0" } else { "15" };
            let style = Style::new()
                .background(Color(idx.to_string()))
                .foreground(Color(fg.to_string()));
            sb.push_str(&render_centered(
                &format!("{idx:>3}"),
                4,
                &style,
                &Style::new().background(Color(idx.to_string())),
            ));
        }
        sb.push('\n');
    }
    sb.push('\n');

    // Grayscale ramp 232-255.
    sb.push_str("  Grayscale (232-255)\n");
    for i in 232..256u32 {
        let fg = if i > 243 { "0" } else { "15" };
        let style = Style::new()
            .background(Color(i.to_string()))
            .foreground(Color(fg.to_string()));
        sb.push_str(&render_centered(
            &format!("{i:>3}"),
            4,
            &style,
            &Style::new().background(Color(i.to_string())),
        ));
    }
    sb.push('\n');

    sb
}

/// Renders a rainbow gradient using true color.
///
/// The hue varies horizontally and the lightness varies vertically.
///
/// # Examples
///
/// ```
/// use charm_palette::show_true_color;
///
/// assert_eq!(show_true_color(0, 0), "");
/// assert_eq!(show_true_color(4, 2).lines().count(), 2);
/// ```
pub fn show_true_color(width: usize, height: usize) -> String {
    if width == 0 || height == 0 {
        return String::new();
    }

    let mut sb = String::new();

    for y in 0..height {
        for x in 0..width {
            let hue = x as f64 / width as f64 * 360.0;
            let saturation = 1.0;
            let lightness = 0.2 + (y as f64 / height as f64) * 0.6;

            let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
            let hex = format!("#{r:02x}{g:02x}{b:02x}");

            let style = Style::new().background(Color(hex));
            sb.push_str(&style.render(" "));
        }
        sb.push('\n');
    }

    sb
}

/// Renders a named theme with its named colors.
///
/// Each color is displayed as a block, followed by its name and its
/// hexadecimal value.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use charm_core::Rgb;
/// use charm_palette::show_theme;
///
/// let mut colors = BTreeMap::new();
/// colors.insert("primary".to_string(), Rgb::rgb(0xff, 0x79, 0xc6));
/// let out = show_theme("Dracula", &colors);
/// assert!(out.contains("Theme: Dracula"));
/// assert!(out.contains("primary"));
/// ```
pub fn show_theme(name: &str, colors: &BTreeMap<String, Rgb>) -> String {
    if colors.is_empty() {
        return String::new();
    }

    let title_style = Style::new()
        .bold(true)
        .foreground(Color("15".to_string()))
        .padding_2(0, 1);

    let mut sb = String::new();
    sb.push_str(&title_style.render(&format!("Theme: {name}")));
    sb.push_str("\n\n");

    for (label, color) in colors {
        let hex = color.to_hex();

        let swatch = Style::new()
            .background(color.to_lipgloss())
            .width(12)
            .render(&" ".repeat(12));

        let label_styled = Style::new()
            .foreground(color.to_lipgloss())
            .width(12)
            .render(label);

        let hex_styled = Style::new()
            .foreground(Color("8".to_string()))
            .width(12)
            .render(&hex);

        sb.push_str("  ");
        sb.push_str(&swatch);
        sb.push_str("\n  ");
        sb.push_str(&label_styled);
        sb.push_str("\n  ");
        sb.push_str(&hex_styled);
        sb.push_str("\n\n");
    }

    sb
}

/// Renders several themes side by side for comparison.
///
/// Each theme is rendered as a column with its colors stacked. Colors that
/// are missing from a theme are replaced by a dashed placeholder.
///
/// The theme names are accepted for API symmetry, but — like the Go original
/// — the columns are titled by their position (`Theme 1`, `Theme 2`, ...).
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use charm_core::Rgb;
/// use charm_palette::compare_themes;
///
/// let mut a = BTreeMap::new();
/// a.insert("primary".to_string(), Rgb::rgb(0xff, 0x00, 0x00));
/// let mut b = BTreeMap::new();
/// b.insert("primary".to_string(), Rgb::rgb(0x00, 0xff, 0x00));
///
/// let out = compare_themes(&[("A", &a), ("B", &b)]);
/// assert!(out.contains("Theme 1"));
/// assert!(out.contains("Theme 2"));
/// ```
pub fn compare_themes(themes: &[(&str, &BTreeMap<String, Rgb>)]) -> String {
    if themes.is_empty() {
        return String::new();
    }

    let col_width = 16usize;
    let mut sb = String::new();

    // Collect every unique key.
    let mut all_keys: BTreeMap<&str, bool> = BTreeMap::new();
    for (_, theme) in themes {
        for key in theme.keys() {
            all_keys.insert(key.as_str(), true);
        }
    }

    // Header with the theme index.
    for (i, _) in themes.iter().enumerate() {
        let header = render_centered(
            &format!("Theme {}", i + 1),
            col_width,
            &Style::new().bold(true),
            &Style::new(),
        );
        sb.push_str(&header);
        sb.push_str("  ");
    }
    sb.push('\n');

    // One row per key, with the swatch of each theme.
    for key in all_keys.keys() {
        sb.push_str(
            &Style::new()
                .foreground(Color("8".to_string()))
                .render(&format!("{key:<12}")),
        );
        sb.push('\n');

        for (_, theme) in themes {
            match theme.get(*key) {
                None => {
                    sb.push_str(
                        &Style::new()
                            .width(cells(col_width))
                            .render(&"-".repeat(col_width)),
                    );
                }
                Some(color) => {
                    let swatch = Style::new()
                        .background(color.to_lipgloss())
                        .width(cells(col_width))
                        .render(&" ".repeat(col_width));
                    sb.push_str(&swatch);
                }
            }
            sb.push_str("  ");
        }
        sb.push('\n');
    }

    sb
}

/// Converts HSL values to RGB.
///
/// * `h`: hue, from `0` to `360`
/// * `s`: saturation, from `0` to `1`
/// * `l`: lightness, from `0` to `1`
#[allow(clippy::many_single_char_names)]
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_palette_renders_nothing() {
        assert_eq!(show_palette(&[], 40), "");
    }

    #[test]
    fn palette_has_two_rows() {
        let out = show_palette(&[Rgb::rgb(0xff, 0, 0), Rgb::rgb(0, 0xff, 0)], 40);
        assert_eq!(out.lines().count(), 2);
    }

    #[test]
    fn palette_shows_the_hexadecimal_value() {
        let out = show_palette(&[Rgb::rgb(0xff, 0, 0)], 40);
        assert!(out.contains("#ff0000"));
    }

    #[test]
    fn palette_minimum_swatch_width_is_four() {
        let out = show_palette(&[Rgb::rgb(0xff, 0, 0)], 1);
        let first = charm_core::strip_ansi(&out);
        let first = first.lines().next().unwrap_or_default();
        assert_eq!(first.chars().count(), 4);
    }

    #[test]
    fn ansi256_has_the_three_sections() {
        let out = show_ansi256();
        assert!(out.starts_with("  Standard Colors (0-15)\n"));
        assert!(out.contains("  Color Cube (16-231)\n"));
        assert!(out.contains("  Grayscale (232-255)\n"));
    }

    #[test]
    fn ansi256_renders_every_color() {
        let out = show_ansi256();
        for i in 0..256u32 {
            assert!(out.contains(&format!("{i:>3}")), "missing color {i}");
        }
    }

    #[test]
    fn true_color_with_zero_size_renders_nothing() {
        assert_eq!(show_true_color(0, 10), "");
        assert_eq!(show_true_color(10, 0), "");
    }

    #[test]
    fn true_color_has_one_line_per_row() {
        assert_eq!(show_true_color(8, 3).lines().count(), 3);
    }

    #[test]
    fn empty_theme_renders_nothing() {
        let colors = BTreeMap::new();
        assert_eq!(show_theme("empty", &colors), "");
    }

    #[test]
    fn theme_shows_the_title_and_the_labels() {
        let mut colors = BTreeMap::new();
        colors.insert("primary".to_string(), Rgb::rgb(0xff, 0x79, 0xc6));
        let out = show_theme("Dracula", &colors);
        assert!(out.contains("Theme: Dracula"));
        assert!(out.contains("primary"));
        assert!(out.contains("#ff79c6"));
    }

    #[test]
    fn empty_theme_list_renders_nothing() {
        assert_eq!(compare_themes(&[]), "");
    }

    #[test]
    fn compare_themes_renders_a_header_per_theme() {
        let mut a = BTreeMap::new();
        a.insert("primary".to_string(), Rgb::rgb(0xff, 0, 0));
        let mut b = BTreeMap::new();
        b.insert("primary".to_string(), Rgb::rgb(0, 0xff, 0));
        let out = charm_core::strip_ansi(&compare_themes(&[("A", &a), ("B", &b)]));
        assert!(out.contains("Theme 1"));
        assert!(out.contains("Theme 2"));
        assert!(out.contains("primary"));
    }

    #[test]
    fn compare_themes_marks_missing_colors() {
        let mut a = BTreeMap::new();
        a.insert("primary".to_string(), Rgb::rgb(0xff, 0, 0));
        let b = BTreeMap::new();
        let out = charm_core::strip_ansi(&compare_themes(&[("A", &a), ("B", &b)]));
        assert!(out.contains("----------------"));
    }

    #[test]
    fn hsl_to_rgb_matches_the_reference_values() {
        assert_eq!(hsl_to_rgb(0.0, 1.0, 0.5), (255, 0, 0));
        assert_eq!(hsl_to_rgb(120.0, 1.0, 0.5), (0, 255, 0));
        assert_eq!(hsl_to_rgb(240.0, 1.0, 0.5), (0, 0, 255));
        assert_eq!(hsl_to_rgb(0.0, 0.0, 1.0), (255, 255, 255));
        assert_eq!(hsl_to_rgb(0.0, 0.0, 0.0), (0, 0, 0));
    }

    #[test]
    fn wrap_text_hard_breaks_long_words() {
        assert_eq!(wrap_text("#282828", 4), vec!["#282", "828"]);
        assert_eq!(wrap_text("#282828", 5), vec!["#2828", "28"]);
        assert_eq!(wrap_text("#282828", 8), vec!["#282828"]);
        assert_eq!(wrap_text("#282828", 0), vec![""]);
    }

    #[test]
    fn render_centered_gives_the_extra_cell_to_the_right() {
        // The Go lipgloss uses `left = gap / 2`; the Rust port rounds up.
        let out = charm_core::strip_ansi(&render_centered("abc", 4, &Style::new(), &Style::new()));
        assert_eq!(out, "abc ");
    }

    #[test]
    fn render_centered_wraps_long_labels() {
        let out =
            charm_core::strip_ansi(&render_centered("#282828", 4, &Style::new(), &Style::new()));
        assert_eq!(out, "#282\n828 ");
    }
}
