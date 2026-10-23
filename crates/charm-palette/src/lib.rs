//! Color palette viewer and generator for the terminal.
//!
//! The crate renders color swatches, the 256-color ANSI grid, true-color
//! gradients and named themes, and can compare several themes side by side.
//! It is a faithful port of the Go `charm-palette` library.
//!
//! Every function returns a plain `String` containing ANSI escape sequences;
//! the caller decides where to print it. The `try_*` variants return an
//! [`Error`] instead of an empty string when the input is invalid.
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
//! * The Go implementation receives `image/color.Color` values. This port takes
//!   [`charm_core::Rgb`], which cannot be nil, so there is no `ErrNilColor`
//!   counterpart.
//! * The Go `*Err` functions are the `try_*` functions here, and they return
//!   [`Error`] instead of `error`.
//! * [`try_show_true_color`] reports [`Error::InvalidDimensions`] where the Go
//!   `ShowTrueColorErr` reports `ErrInvalidWidth`.
//! * [`compare_themes`] takes the themes without names. The Go function does
//!   not use names either; the columns are titled by position.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;

use std::collections::{BTreeMap, BTreeSet};

use charm_core::Rgb;
use lipgloss::{Color, Style};

pub use error::Error;

/// Width of a `#rrggbb` label, in columns.
const HEX_WIDTH: usize = 7;
/// Smallest width of the label column in themes.
const MIN_LABEL_WIDTH: usize = 6;
/// Largest width of the label column in themes.
const MAX_LABEL_WIDTH: usize = 24;
/// Width of the swatch drawn by [`try_show_theme`], in columns.
const THEME_SWATCH_WIDTH: usize = 12;
/// Width of each theme column in [`try_compare_themes`], in columns.
const COMPARE_COLUMN_WIDTH: usize = 16;

/// Renders color swatches with the given total width.
///
/// Each color is drawn as a filled block with its hexadecimal value below it.
/// The width is divided equally between the swatches when there is room for
/// the `#rrggbb` label of every color; otherwise the swatches are stacked
/// vertically, one per row, so the requested width is never exceeded.
///
/// Invalid input (no colors, zero width) returns an empty string; use
/// [`try_show_palette`] to get the reason.
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
    try_show_palette(colors, width).unwrap_or_default()
}

/// Renders color swatches like [`show_palette`], reporting invalid input.
///
/// # Errors
///
/// Returns [`Error::NoColors`] when `colors` is empty and
/// [`Error::InvalidWidth`] when `width` is zero.
///
/// # Examples
///
/// ```
/// use charm_core::Rgb;
/// use charm_palette::{try_show_palette, Error};
///
/// assert_eq!(try_show_palette(&[], 10), Err(Error::NoColors));
/// assert!(try_show_palette(&[Rgb::black()], 10).is_ok());
/// ```
pub fn try_show_palette(colors: &[Rgb], width: usize) -> Result<String, Error> {
    if colors.is_empty() {
        return Err(Error::NoColors);
    }
    if width == 0 {
        return Err(Error::InvalidWidth(width));
    }

    let per_swatch = width / colors.len();
    if per_swatch < HEX_WIDTH {
        return Ok(render_palette_vertical(colors, width));
    }
    Ok(render_palette_horizontal(colors, per_swatch))
}

/// Draws every swatch in one row, with the hexadecimal labels below them.
fn render_palette_horizontal(colors: &[Rgb], cell: usize) -> String {
    let mut top = String::new();
    let mut base = String::new();

    for color in colors {
        top.push_str(&swatch(*color, cell));

        let (left, right) = gap(HEX_WIDTH, cell);
        let label = Style::new()
            .foreground(contrast(*color).to_lipgloss())
            .render(&color.to_hex());
        base.push_str(&" ".repeat(left));
        base.push_str(&label);
        base.push_str(&" ".repeat(right));
    }

    format!("{top}\n{base}")
}

/// Stacks the swatches one per row, for widths too small for the horizontal
/// layout.
fn render_palette_vertical(colors: &[Rgb], width: usize) -> String {
    colors
        .iter()
        .map(|color| {
            let hex = color.to_hex();
            if width > HEX_WIDTH {
                let swatch_width = width - HEX_WIDTH - 1;
                format!("{} {hex}", swatch(*color, swatch_width))
            } else {
                truncate(&hex, width)
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
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
    for i in 0..8u32 {
        sb.push_str(&grid_cell(i, None));
    }
    sb.push('\n');
    for i in 8..16u32 {
        sb.push_str(&grid_cell(i, Some("0")));
    }
    sb.push_str("\n\n");

    sb.push_str("  Color Cube (16-231)\n");
    for row in 0..12u32 {
        for col in 0..18u32 {
            let index = 16 + row * 18 + col;
            if index > 231 {
                break;
            }
            let fg = if row > 5 { "0" } else { "15" };
            sb.push_str(&grid_cell(index, Some(fg)));
        }
        sb.push('\n');
    }
    sb.push('\n');

    sb.push_str("  Grayscale (232-255)\n");
    for index in 232..256u32 {
        let fg = if index > 243 { "0" } else { "15" };
        sb.push_str(&grid_cell(index, Some(fg)));
    }
    sb.push('\n');

    sb
}

/// One four-column cell of the ANSI grid: the index, right-aligned, on its
/// background color.
fn grid_cell(index: u32, foreground: Option<&str>) -> String {
    let mut style = Style::new().background(Color(index.to_string()));
    if let Some(fg) = foreground {
        style = style.foreground(Color(fg.to_string()));
    }
    style.render(&format!("{index:>3} "))
}

/// Renders a rainbow gradient using true color.
///
/// The hue varies horizontally and the lightness varies vertically. A zero
/// width or height returns an empty string; use [`try_show_true_color`] to get
/// the reason.
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
    try_show_true_color(width, height).unwrap_or_default()
}

/// Renders a rainbow gradient like [`show_true_color`], reporting invalid
/// dimensions.
///
/// # Errors
///
/// Returns [`Error::InvalidDimensions`] when `width` or `height` is zero.
///
/// # Examples
///
/// ```
/// use charm_palette::{try_show_true_color, Error};
///
/// assert_eq!(
///     try_show_true_color(10, 0),
///     Err(Error::InvalidDimensions { width: 10, height: 0 })
/// );
/// ```
pub fn try_show_true_color(width: usize, height: usize) -> Result<String, Error> {
    if width == 0 || height == 0 {
        return Err(Error::InvalidDimensions { width, height });
    }

    let mut sb = String::new();
    for y in 0..height {
        for x in 0..width {
            let hue = x as f64 / width as f64 * 360.0;
            let saturation = 1.0;
            let lightness = 0.2 + (y as f64 / height as f64) * 0.6;

            let (r, g, b) = hsl_to_rgb(hue, saturation, lightness);
            let hex = format!("#{r:02x}{g:02x}{b:02x}");

            sb.push_str(&Style::new().background(Color(hex)).render(" "));
        }
        sb.push('\n');
    }

    Ok(sb)
}

/// Renders a named theme with its named colors.
///
/// Each color is drawn as a swatch, followed by its name and its hexadecimal
/// value. The colors are sorted alphabetically by name, so the output is
/// deterministic. An empty theme returns an empty string; use
/// [`try_show_theme`] to get the reason.
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
    try_show_theme(name, colors).unwrap_or_default()
}

/// Renders a named theme like [`show_theme`], reporting invalid input.
///
/// # Errors
///
/// Returns [`Error::NoColors`] when `colors` is empty.
///
/// # Examples
///
/// ```
/// use std::collections::BTreeMap;
///
/// use charm_palette::{try_show_theme, Error};
///
/// let empty = BTreeMap::new();
/// assert_eq!(try_show_theme("Empty", &empty), Err(Error::NoColors));
/// ```
pub fn try_show_theme(name: &str, colors: &BTreeMap<String, Rgb>) -> Result<String, Error> {
    if colors.is_empty() {
        return Err(Error::NoColors);
    }

    let label_width = label_column_width(colors.keys().map(String::as_str));

    let title = Style::new()
        .bold(true)
        .foreground(Color("15".to_string()))
        .padding_2(0, 1)
        .render(&format!("Theme: {name}"));

    let mut sb = String::new();
    sb.push_str(&title);
    sb.push_str("\n\n");

    for (label, color) in colors {
        let label_styled = Style::new()
            .foreground(contrast(*color).to_lipgloss())
            .render(&pad_right(&truncate(label, label_width), label_width));
        let hex_styled = Style::new()
            .foreground(Color("8".to_string()))
            .render(&color.to_hex());

        sb.push_str("  ");
        sb.push_str(&swatch(*color, THEME_SWATCH_WIDTH));
        sb.push_str("  ");
        sb.push_str(&label_styled);
        sb.push_str("  ");
        sb.push_str(&hex_styled);
        sb.push('\n');
    }

    Ok(sb)
}

/// Renders several themes side by side for comparison.
///
/// Each theme becomes a column titled `Theme 1`, `Theme 2`, and so on, and
/// each color name becomes a row. Colors missing from a theme are shown as a
/// dashed placeholder. Names are sorted alphabetically. Without themes, an
/// empty string is returned; use [`try_compare_themes`] to get the reason.
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
/// let out = compare_themes(&[&a, &b]);
/// assert!(out.contains("Theme 1"));
/// assert!(out.contains("Theme 2"));
/// ```
pub fn compare_themes(themes: &[&BTreeMap<String, Rgb>]) -> String {
    try_compare_themes(themes).unwrap_or_default()
}

/// Renders several themes side by side like [`compare_themes`], reporting
/// invalid input.
///
/// # Errors
///
/// Returns [`Error::NoThemes`] when `themes` is empty.
///
/// # Examples
///
/// ```
/// use charm_palette::{try_compare_themes, Error};
///
/// assert_eq!(try_compare_themes(&[]), Err(Error::NoThemes));
/// ```
pub fn try_compare_themes(themes: &[&BTreeMap<String, Rgb>]) -> Result<String, Error> {
    if themes.is_empty() {
        return Err(Error::NoThemes);
    }

    let keys: BTreeSet<&str> = themes
        .iter()
        .flat_map(|theme| theme.keys().map(String::as_str))
        .collect();
    let label_width = label_column_width(keys.iter().copied());

    let mut sb = String::new();

    sb.push_str(&" ".repeat(label_width + 2));
    for index in 0..themes.len() {
        let header = center(&format!("Theme {}", index + 1), COMPARE_COLUMN_WIDTH);
        sb.push_str(&Style::new().bold(true).render(&header));
        sb.push_str("  ");
    }
    sb.push('\n');

    for key in keys {
        sb.push_str(&label_cell(key, label_width));
        for theme in themes {
            match theme.get(key) {
                None => sb.push_str(&"-".repeat(COMPARE_COLUMN_WIDTH)),
                Some(color) => sb.push_str(&swatch(*color, COMPARE_COLUMN_WIDTH)),
            }
            sb.push_str("  ");
        }
        sb.push('\n');
    }

    Ok(sb)
}

/// The label column of a comparison row, dimmed and padded to `width`.
fn label_cell(text: &str, width: usize) -> String {
    let padded = pad_right(&truncate(text, width), width);
    format!(
        "{}  ",
        Style::new()
            .foreground(Color("8".to_string()))
            .render(&padded)
    )
}

/// A filled block of `width` columns in the given color.
fn swatch(color: Rgb, width: usize) -> String {
    Style::new()
        .background(color.to_lipgloss())
        .render(&" ".repeat(width))
}

/// The foreground color with the best contrast over `color`: black or white,
/// chosen by the WCAG relative luminance.
fn contrast(color: Rgb) -> Rgb {
    if relative_luminance(color) > 0.179 {
        Rgb::black()
    } else {
        Rgb::white()
    }
}

/// The WCAG relative luminance of a color.
fn relative_luminance(color: Rgb) -> f64 {
    0.2126 * linearize(color.r) + 0.7152 * linearize(color.g) + 0.0722 * linearize(color.b)
}

/// Linearizes an 8-bit channel, as specified by WCAG.
fn linearize(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.03928 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Width of the label column: the widest label, clamped to the theme limits.
fn label_column_width<'a>(labels: impl Iterator<Item = &'a str>) -> usize {
    let widest = labels.map(charm_core::display_width).max().unwrap_or(0);
    widest.clamp(MIN_LABEL_WIDTH, MAX_LABEL_WIDTH)
}

/// Splits the free columns around `content` into a left and a right gap, with
/// the extra column (if any) on the right.
fn gap(content: usize, width: usize) -> (usize, usize) {
    let free = width.saturating_sub(content);
    let left = free / 2;
    (left, free - left)
}

/// Centers `text` in `width` columns.
fn center(text: &str, width: usize) -> String {
    let (left, right) = gap(charm_core::display_width(text), width);
    format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
}

/// Pads `text` with spaces on the right until it spans `width` columns.
fn pad_right(text: &str, width: usize) -> String {
    let current = charm_core::display_width(text);
    format!("{text}{}", " ".repeat(width.saturating_sub(current)))
}

/// Keeps at most `max` columns of `text`, ending it with `…` when it is cut.
fn truncate(text: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    if charm_core::display_width(text) <= max {
        return text.to_owned();
    }
    if max == 1 {
        return "…".to_owned();
    }
    format!("{}…", take_columns(text, max - 1))
}

/// The longest prefix of `text` that fits in `columns` columns.
fn take_columns(text: &str, columns: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let mut buf = [0u8; 4];
        let step = charm_core::display_width(ch.encode_utf8(&mut buf));
        if used + step > columns {
            break;
        }
        used += step;
        out.push(ch);
    }
    out
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

    fn plain(text: &str) -> String {
        charm_core::strip_ansi(text)
    }

    #[test]
    fn empty_palette_is_an_error() {
        assert_eq!(show_palette(&[], 40), "");
        assert_eq!(try_show_palette(&[], 40), Err(Error::NoColors));
    }

    #[test]
    fn zero_width_is_an_error() {
        assert_eq!(
            try_show_palette(&[Rgb::black()], 0),
            Err(Error::InvalidWidth(0))
        );
        assert_eq!(show_palette(&[Rgb::black()], 0), "");
    }

    #[test]
    fn horizontal_palette_has_two_rows_and_fills_the_width() {
        let out = show_palette(&[Rgb::rgb(0xff, 0, 0), Rgb::rgb(0, 0xff, 0)], 40);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in lines {
            assert_eq!(charm_core::display_width(line), 40);
        }
    }

    #[test]
    fn palette_shows_the_hexadecimal_value() {
        let out = plain(&show_palette(&[Rgb::rgb(0xff, 0, 0)], 40));
        assert!(out.contains("#ff0000"));
    }

    #[test]
    fn narrow_palette_stacks_the_colors_and_respects_the_width() {
        let out = plain(&show_palette(
            &[Rgb::rgb(0x28, 0x28, 0x28), Rgb::rgb(0xfa, 0xbd, 0x2f)],
            8,
        ));
        assert_eq!(out, " #282828\n #fabd2f");
    }

    #[test]
    fn very_narrow_palette_truncates_the_label() {
        let out = plain(&show_palette(&[Rgb::rgb(0x28, 0x28, 0x28)], 5));
        assert_eq!(out, "#282…");
    }

    #[test]
    fn contrast_is_black_on_light_and_white_on_dark() {
        assert_eq!(contrast(Rgb::rgb(240, 230, 140)), Rgb::black());
        assert_eq!(contrast(Rgb::rgb(15, 23, 42)), Rgb::white());
    }

    #[test]
    fn palette_label_uses_the_contrast_color() {
        let out = show_palette(&[Rgb::rgb(15, 23, 42)], 20);
        assert!(!out.contains("\x1b[38;2;15;23;42m"));
        assert!(out.contains("\x1b[38;2;255;255;255m"));
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
    fn ansi256_grid_has_the_expected_shape() {
        let out = plain(&show_ansi256());
        let lines: Vec<&str> = out.lines().collect();
        let cube = lines
            .iter()
            .position(|l| l.contains("Color Cube"))
            .unwrap_or_default();
        let gray = lines
            .iter()
            .position(|l| l.contains("Grayscale"))
            .unwrap_or_default();
        let rows = &lines[cube + 1..gray - 1];
        assert_eq!(rows.len(), 12);
        for row in rows {
            assert_eq!(charm_core::display_width(row), 18 * 4);
        }
    }

    #[test]
    fn true_color_with_zero_size_is_an_error() {
        assert_eq!(show_true_color(0, 10), "");
        assert_eq!(
            try_show_true_color(10, 0),
            Err(Error::InvalidDimensions {
                width: 10,
                height: 0
            })
        );
    }

    #[test]
    fn true_color_has_one_line_per_row() {
        assert_eq!(show_true_color(8, 3).lines().count(), 3);
    }

    #[test]
    fn empty_theme_is_an_error() {
        let colors = BTreeMap::new();
        assert_eq!(show_theme("empty", &colors), "");
        assert_eq!(try_show_theme("empty", &colors), Err(Error::NoColors));
    }

    #[test]
    fn theme_shows_the_title_and_the_labels() {
        let mut colors = BTreeMap::new();
        colors.insert("primary".to_string(), Rgb::rgb(0xff, 0x79, 0xc6));
        let out = plain(&show_theme("Dracula", &colors));
        assert!(out.contains("Theme: Dracula"));
        assert!(out.contains("primary"));
        assert!(out.contains("#ff79c6"));
    }

    #[test]
    fn theme_rows_are_sorted_alphabetically() {
        let mut colors = BTreeMap::new();
        colors.insert("text".to_string(), Rgb::white());
        colors.insert("background".to_string(), Rgb::black());
        colors.insert("primary".to_string(), Rgb::rgb(1, 2, 3));
        let out = plain(&show_theme("Order", &colors));
        let background = out.find("background").unwrap_or_default();
        let primary = out.find("primary").unwrap_or_default();
        let text = out.find("text").unwrap_or_default();
        assert!(background < primary && primary < text);
    }

    #[test]
    fn long_theme_labels_are_truncated_inside_their_column() {
        let mut colors = BTreeMap::new();
        colors.insert(
            "nome-de-etapa-muito-comprido".to_string(),
            Rgb::rgb(1, 2, 3),
        );
        colors.insert("configuração".to_string(), Rgb::rgb(4, 5, 6));
        let out = plain(&show_theme("Longos", &colors));
        assert!(out.contains('…'));
        // Indent, swatch, gap, label column and hex value.
        let row_width = 2 + THEME_SWATCH_WIDTH + 2 + 24 + 2 + HEX_WIDTH;
        for line in out.lines().filter(|line| line.contains('#')) {
            assert!(charm_core::display_width(line) <= row_width, "{line:?}");
        }
    }

    #[test]
    fn empty_theme_list_is_an_error() {
        assert_eq!(compare_themes(&[]), "");
        assert_eq!(try_compare_themes(&[]), Err(Error::NoThemes));
    }

    #[test]
    fn compare_themes_renders_a_header_per_theme() {
        let mut a = BTreeMap::new();
        a.insert("primary".to_string(), Rgb::rgb(0xff, 0, 0));
        let mut b = BTreeMap::new();
        b.insert("primary".to_string(), Rgb::rgb(0, 0xff, 0));
        let out = plain(&compare_themes(&[&a, &b]));
        assert!(out.contains("Theme 1"));
        assert!(out.contains("Theme 2"));
        assert!(out.contains("primary"));
    }

    #[test]
    fn compare_themes_marks_missing_colors() {
        let mut a = BTreeMap::new();
        a.insert("primary".to_string(), Rgb::rgb(0xff, 0, 0));
        let b = BTreeMap::new();
        let out = plain(&compare_themes(&[&a, &b]));
        assert!(out.contains("----------------"));
    }

    #[test]
    fn compare_themes_rows_have_the_same_width_as_the_header() {
        let mut a = BTreeMap::new();
        a.insert("background".to_string(), Rgb::rgb(15, 23, 42));
        a.insert("primary".to_string(), Rgb::rgb(99, 102, 241));
        let mut b = BTreeMap::new();
        b.insert("background".to_string(), Rgb::black());
        b.insert("primary".to_string(), Rgb::rgb(220, 38, 38));
        let out = plain(&compare_themes(&[&a, &b]));
        let lines: Vec<&str> = out.lines().collect();
        let expected = charm_core::display_width(lines[0]);
        for line in &lines {
            assert_eq!(charm_core::display_width(line), expected, "{line:?}");
        }
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
    fn truncate_keeps_whole_characters() {
        assert_eq!(truncate("abcdef", 4), "abc…");
        assert_eq!(truncate("abc", 4), "abc");
        assert_eq!(truncate("abc", 1), "…");
        assert_eq!(truncate("日本語", 4), "日…");
    }

    #[test]
    fn gap_gives_the_extra_column_to_the_right() {
        assert_eq!(gap(3, 4), (0, 1));
        assert_eq!(gap(3, 6), (1, 2));
    }

    fn six_colors() -> Vec<Rgb> {
        vec![
            Rgb::rgb(255, 89, 94),
            Rgb::rgb(255, 202, 58),
            Rgb::rgb(138, 201, 38),
            Rgb::rgb(25, 130, 196),
            Rgb::rgb(106, 76, 147),
            Rgb::rgb(1, 2, 3),
        ]
    }

    #[test]
    fn small_palette_never_exceeds_the_requested_width() {
        let out = plain(&show_palette(&six_colors(), 20));
        assert_ne!(out, "");
        for line in out.lines() {
            assert!(charm_core::display_width(line) <= 20, "{line:?}");
        }
        for color in six_colors() {
            assert!(out.contains(&color.to_hex()), "missing {}", color.to_hex());
        }
    }

    #[test]
    fn horizontal_palette_fills_the_requested_width_exactly() {
        let out = show_palette(&six_colors()[..5], 80);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in lines {
            assert_eq!(charm_core::display_width(line), 80, "{line:?}");
        }
    }

    #[test]
    fn translucent_colors_keep_their_channels() {
        let out = plain(&show_palette(&[Rgb::new(255, 0, 0, 128)], 20));
        assert!(out.contains("#ff0000"));
    }

    #[test]
    fn theme_labels_use_the_contrast_color() {
        let mut colors = BTreeMap::new();
        colors.insert("dark".to_string(), Rgb::rgb(15, 23, 42));
        colors.insert("light".to_string(), Rgb::rgb(240, 230, 140));
        let out = show_theme("Contraste", &colors);
        assert!(!out.contains("\x1b[38;2;15;23;42m"));
        assert!(out.contains("\x1b[38;2;255;255;255m"));
        assert!(!out.contains("\x1b[38;2;240;230;140m"));
        assert!(out.contains("\x1b[38;2;0;0;0m"));
    }

    #[test]
    fn true_color_rows_have_the_requested_width() {
        let out = show_true_color(4, 2);
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 2);
        for line in lines {
            assert_eq!(charm_core::display_width(line), 4);
        }
    }

    #[test]
    fn compare_themes_header_sits_over_the_swatch_column() {
        let mut a = BTreeMap::new();
        a.insert("background".to_string(), Rgb::rgb(15, 23, 42));
        a.insert("primary".to_string(), Rgb::rgb(99, 102, 241));
        let mut b = BTreeMap::new();
        b.insert("background".to_string(), Rgb::black());
        b.insert("primary".to_string(), Rgb::rgb(220, 38, 38));
        let out = plain(&compare_themes(&[&a, &b]));
        let header = out.lines().next().unwrap_or_default();
        let column = header.find("Theme 1").unwrap_or_default();
        let start = charm_core::display_width(&header[..column]);
        let swatch_start = label_column_width(["background", "primary"].into_iter()) + 2;
        assert!(start >= swatch_start);
        assert!(start + "Theme 1".len() <= swatch_start + COMPARE_COLUMN_WIDTH);
    }

    #[test]
    fn compare_themes_long_labels_keep_the_rows_aligned() {
        let mut colors = BTreeMap::new();
        colors.insert("configuração".to_string(), Rgb::rgb(99, 102, 241));
        colors.insert(
            "nome-de-etapa-muito-comprido".to_string(),
            Rgb::rgb(168, 85, 247),
        );
        let out = plain(&compare_themes(&[&colors]));
        let lines: Vec<&str> = out.lines().collect();
        let expected = charm_core::display_width(lines[0]);
        for line in lines {
            assert_eq!(charm_core::display_width(line), expected, "{line:?}");
        }
    }

    #[test]
    fn hsl_to_rgb_covers_every_sector() {
        let cases = [
            (0.0, 255, 0, 0),
            (60.0, 255, 255, 0),
            (120.0, 0, 255, 0),
            (180.0, 0, 255, 255),
            (240.0, 0, 0, 255),
            (300.0, 255, 0, 255),
            (360.0, 255, 0, 0),
        ];
        for (h, r, g, b) in cases {
            assert_eq!(hsl_to_rgb(h, 1.0, 0.5), (r, g, b), "hue {h}");
        }
        let (r, g, b) = hsl_to_rgb(123.0, 0.0, 0.5);
        assert!(r == g && g == b && (127..=128).contains(&r));
    }
}
