//! Terminal charts for the Charm Suite: sparklines, bar charts and progress
//! bars.
//!
//! The crate is a faithful port of the Go `charm-charts` library. The layout
//! algorithms follow the Go original, so the output is identical after the
//! ANSI escape codes are stripped. Every component follows the same shape:
//!
//! * a `new` constructor returning the default configuration;
//! * chainable setters that consume `self` and return `Self`;
//! * a `render` method producing the styled string.
//!
//! # Examples
//!
//! ```
//! use charm_charts::{BarChart, ProgressBar, Sparkline};
//!
//! // A compact line chart.
//! let sparkline = Sparkline::new(&[1.0, 4.0, 2.0, 8.0, 5.0]).label("cpu").render();
//! assert!(sparkline.starts_with("cpu "));
//!
//! // A horizontal bar chart.
//! let chart = BarChart::new(&["api", "web"], &[12.0, 27.0]).render();
//! assert!(chart.contains("api"));
//!
//! // A progress bar.
//! let bar = ProgressBar::new(0.5).label("build").render();
//! assert!(bar.ends_with("  50%"));
//! ```
//!
//! # Known differences from the Go original
//!
//! * The Go implementation panics when a [`ProgressBar`] has a negative
//!   `value` assigned directly to the public field. This port clamps the value
//!   to the `0.0..=1.0` range instead.
//! * The Go implementation panics when a [`BarChart`] is rendered vertically
//!   with a `bar_width` larger than the column that `width` leaves for it. This
//!   port shrinks the bar to the column instead.
//!
//! [`ProgressBar`]: crate::ProgressBar
//! [`BarChart`]: crate::BarChart

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod barchart;
mod progress;
mod sparkline;

pub use barchart::BarChart;
pub use progress::ProgressBar;
pub use sparkline::Sparkline;

/// Block glyphs used by the sparkline renderer, from lowest to highest.
///
/// Extracted programmatically from the Go source (`sparkline.go`).
pub(crate) const BLOCKS: [char; 9] = [
    '\u{0020}', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
    '\u{2588}',
];

/// Full block glyph used to draw bar chart bars.
///
/// Extracted programmatically from the Go source (`barchart.go`).
pub(crate) const BAR_GLYPH: char = '\u{2588}';

/// Default glyph used to draw the filled portion of a progress bar.
///
/// Extracted programmatically from the Go source (`progress.go`).
pub(crate) const FILL_GLYPH: char = '\u{2588}';

/// Default glyph used to draw the empty portion of a progress bar.
///
/// Extracted programmatically from the Go source (`progress.go`).
pub(crate) const EMPTY_GLYPH: char = '\u{2591}';

/// Mirrors Go's `math.Max`.
///
/// Go propagates `NaN` and resolves the signed zero tie in favour of `+0`,
/// which differs from Rust's `f64::max`.
#[allow(clippy::float_cmp)]
pub(crate) fn go_max(x: f64, y: f64) -> f64 {
    if (x.is_infinite() && x.is_sign_positive()) || (y.is_infinite() && y.is_sign_positive()) {
        return f64::INFINITY;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if x == 0.0 && x == y {
        if x.is_sign_negative() {
            return y;
        }
        return x;
    }
    if x > y {
        x
    } else {
        y
    }
}

/// Mirrors Go's `math.Min`.
///
/// Go propagates `NaN` and resolves the signed zero tie in favour of `-0`,
/// which differs from Rust's `f64::min`.
#[allow(clippy::float_cmp)]
pub(crate) fn go_min(x: f64, y: f64) -> f64 {
    if (x.is_infinite() && x.is_sign_negative()) || (y.is_infinite() && y.is_sign_negative()) {
        return f64::NEG_INFINITY;
    }
    if x.is_nan() || y.is_nan() {
        return f64::NAN;
    }
    if x == 0.0 && x == y {
        if x.is_sign_negative() {
            return x;
        }
        return y;
    }
    if x < y {
        x
    } else {
        y
    }
}

/// Display width of `text` in terminal columns, ignoring ANSI codes.
pub(crate) fn display_width(text: &str) -> usize {
    charm_core::display_width(text)
}

/// Pads `text` with spaces on the left until it spans `width` columns.
pub(crate) fn pad_left(text: &str, width: usize) -> String {
    let current = display_width(text);
    if current >= width {
        return text.to_owned();
    }
    format!("{}{text}", " ".repeat(width - current))
}

/// Centers `text` in `width` columns. The extra column of an odd gap goes to
/// the right, like the Go original.
pub(crate) fn center(text: &str, width: usize) -> String {
    let current = display_width(text);
    if current >= width {
        return text.to_owned();
    }
    let left = (width - current) / 2;
    let right = width - current - left;
    format!("{}{text}{}", " ".repeat(left), " ".repeat(right))
}

/// Shortens `text` to fit `width` columns, ending it with `…` when it is cut.
///
/// The cut never splits a character.
pub(crate) fn fit_label(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if display_width(text) <= width {
        return text.to_owned();
    }
    if width == 1 {
        return "…".to_owned();
    }
    format!("{}…", truncate_chars(text, width - 1))
}

/// Drops characters from the end until the text fits `width` columns.
fn truncate_chars(text: &str, width: usize) -> String {
    let mut chars: Vec<char> = text.chars().collect();
    while !chars.is_empty() && display_width(&chars.iter().collect::<String>()) > width {
        chars.pop();
    }
    chars.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn go_max_matches_go_semantics() {
        assert_eq!(go_max(1.0, 2.0), 2.0);
        assert_eq!(go_max(0.0, -3.0), 0.0);
        assert!(go_max(f64::NAN, 1.0).is_nan());
        assert!(go_max(f64::INFINITY, 1.0).is_infinite());
        assert!(go_max(0.0, -0.0).is_sign_positive());
    }

    #[test]
    fn go_min_matches_go_semantics() {
        assert_eq!(go_min(1.0, 2.0), 1.0);
        assert_eq!(go_min(0.0, -3.0), -3.0);
        assert!(go_min(f64::NAN, 1.0).is_nan());
        assert!(go_min(f64::NEG_INFINITY, 1.0).is_infinite());
        assert!(go_min(0.0, -0.0).is_sign_negative());
    }

    #[test]
    fn pad_left_measures_display_width() {
        assert_eq!(pad_left("ab", 5), "   ab");
        assert_eq!(pad_left("abcdef", 3), "abcdef");
        assert_eq!(pad_left("日本", 6), "  日本");
    }

    #[test]
    fn center_gives_the_extra_column_to_the_right() {
        assert_eq!(center("ab", 5), " ab  ");
        assert_eq!(center("abc", 4), "abc ");
    }

    #[test]
    fn fit_label_keeps_short_text_and_cuts_long_text() {
        assert_eq!(fit_label("Go", 10), "Go");
        assert_eq!(fit_label("abcdef", 4), "abc…");
        assert_eq!(fit_label("abcdef", 1), "…");
        assert_eq!(fit_label("abcdef", 0), "");
    }

    #[test]
    fn fit_label_never_splits_a_character() {
        for (text, width) in [("Programação", 5), ("日本語", 4), ("Ação", 3)] {
            let fitted = fit_label(text, width);
            assert!(display_width(&fitted) <= width, "{fitted:?} too wide");
        }
    }
}
