//! Terminal charts for the Charm Suite: sparklines, bar charts and progress
//! bars.
//!
//! The crate is a faithful port of the Go `charm-charts` library and keeps the
//! original rendering algorithm byte-for-byte compatible (after ANSI codes are
//! stripped). Every component follows the same shape:
//!
//! * a `new` constructor returning the default configuration;
//! * chainable `with_*`-style setters that consume `self` and return `Self`;
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
//! * The Go implementation panics when a [`Sparkline`] is rendered with a
//!   `width` of `0` (it indexes an empty resampled slice). This port returns an
//!   empty string instead.
//! * The Go implementation panics when a [`ProgressBar`] has a negative
//!   `value` assigned directly to the public field. This port clamps the value
//!   to the `0.0..=1.0` range instead.
//!
//! [`Sparkline`]: crate::Sparkline
//! [`ProgressBar`]: crate::ProgressBar

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

/// Converts a `usize` into an `i64`, saturating instead of wrapping around.
pub(crate) fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Pads `text` to `width` characters, mirroring Go's `fmt` verbs.
///
/// Go's `fmt` counts **runes** (not bytes) when padding strings, so
/// `format!("{:<8}", "é")` produces two spaces plus the two-byte character
/// (three cells wide in total). When `left_align` is `true` the padding is
/// appended (Go's `%-*s`), otherwise it is prepended (Go's `%*s`).
pub(crate) fn pad_like_go(text: &str, width: usize, left_align: bool) -> String {
    let len = text.chars().count();
    if len >= width {
        return text.to_owned();
    }
    let padding = " ".repeat(width - len);
    if left_align {
        format!("{text}{padding}")
    } else {
        format!("{padding}{text}")
    }
}

/// Truncates `text` to at most `bytes` **bytes**, mirroring Go's slice syntax.
///
/// Go slices strings by byte index and may therefore cut a multi-byte
/// character in half, producing an invalid UTF-8 sequence. This port keeps the
/// byte-oriented semantics but converts the result with
/// [`String::from_utf8_lossy`], so the returned string is always valid UTF-8.
pub(crate) fn truncate_bytes(text: &str, bytes: usize) -> String {
    if text.len() <= bytes {
        return text.to_owned();
    }
    String::from_utf8_lossy(&text.as_bytes()[..bytes]).into_owned()
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
    fn pad_like_go_counts_runes() {
        assert_eq!(pad_like_go("ab", 5, false), "   ab");
        assert_eq!(pad_like_go("ab", 5, true), "ab   ");
        assert_eq!(pad_like_go("abcdef", 3, false), "abcdef");
        assert_eq!(pad_like_go("é", 3, true), "é  ");
    }

    #[test]
    fn truncate_bytes_keeps_valid_utf8() {
        assert_eq!(truncate_bytes("abc", 2), "ab");
        assert_eq!(truncate_bytes("abc", 5), "abc");
        // Cuts a multi-byte character in half, like the Go original.
        assert_eq!(truncate_bytes("é", 1), "\u{fffd}");
    }
}
