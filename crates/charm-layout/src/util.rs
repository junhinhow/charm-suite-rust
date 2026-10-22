//! Internal helpers shared by the flex and the grid engines.
//!
//! These are direct ports of the unexported `joinWithSpacer`,
//! `joinVerticalWithSpacer`, `spacerH` and `spacerV` helpers of the Go
//! original.

use lipgloss::Position;

/// Borrows a slice of owned strings as a slice of string slices.
///
/// [`lipgloss::join_horizontal`] and [`lipgloss::join_vertical`] take
/// `&[&str]`, so the rendered blocks need to be re-borrowed.
pub(crate) fn as_refs(items: &[String]) -> Vec<&str> {
    items.iter().map(String::as_str).collect()
}

/// Joins blocks horizontally, inserting `spacer` between every pair of blocks.
///
/// Port of `joinWithSpacer`.
pub(crate) fn join_with_spacer(items: &[String], spacer: &str, pos: Position) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut interleaved: Vec<&str> = Vec::with_capacity(items.len() * 2 - 1);
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            interleaved.push(spacer);
        }
        interleaved.push(item.as_str());
    }
    lipgloss::join_horizontal(pos, &interleaved)
}

/// Joins blocks vertically, inserting `spacer` between every pair of blocks.
///
/// Port of `joinVerticalWithSpacer`.
pub(crate) fn join_vertical_with_spacer(items: &[String], spacer: &str) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut interleaved: Vec<&str> = Vec::with_capacity(items.len() * 2 - 1);
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            interleaved.push(spacer);
        }
        interleaved.push(item.as_str());
    }
    lipgloss::join_vertical(lipgloss::LEFT, &interleaved)
}

/// Returns a horizontal spacer of `width` spaces.
///
/// Port of `spacerH`. The repetition is bounded by `lipgloss` so that an
/// absurdly large gap cannot exhaust the memory.
pub(crate) fn spacer_h(width: usize) -> String {
    lipgloss::security::safe_repeat(' ', width)
}

/// Returns a vertical spacer of `height` newlines.
///
/// Port of `spacerV`. The repetition is bounded by `lipgloss` so that an
/// absurdly large gap cannot exhaust the memory.
pub(crate) fn spacer_v(height: usize) -> String {
    lipgloss::security::safe_str_repeat("\n", height)
}

/// Converts a count into an `i64` used by the layout arithmetic.
///
/// Values above `i64::MAX` are clamped to `i64::MAX`.
pub(crate) fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Converts a dimension into the `i32` expected by `lipgloss`.
///
/// Negative values are clamped to zero and values above `i32::MAX` are clamped
/// to `i32::MAX`.
pub(crate) fn to_i32(value: i64) -> i32 {
    i32::try_from(value.max(0)).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacers_have_expected_shape() {
        assert_eq!(spacer_h(3), "   ");
        assert_eq!(spacer_h(0), "");
        assert_eq!(spacer_v(2), "\n\n");
        assert_eq!(spacer_v(0), "");
    }

    #[test]
    fn join_with_spacer_inserts_between_items() {
        let items = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        assert_eq!(join_with_spacer(&items, "-", lipgloss::TOP), "a-b-c");
    }

    #[test]
    fn join_with_spacer_handles_empty_input() {
        assert_eq!(join_with_spacer(&[], "-", lipgloss::TOP), "");
        assert_eq!(join_vertical_with_spacer(&[], "-"), "");
    }

    #[test]
    fn join_vertical_with_spacer_inserts_between_items() {
        let items = vec!["a".to_string(), "b".to_string()];
        // The vertical join pads every line to the width of the widest one.
        assert_eq!(join_vertical_with_spacer(&items, "\n"), "a\n \n \nb");
    }

    #[test]
    fn to_i32_clamps_to_the_valid_range() {
        assert_eq!(to_i32(-10), 0);
        assert_eq!(to_i32(0), 0);
        assert_eq!(to_i32(42), 42);
        assert_eq!(to_i32(i64::MAX), i32::MAX);
    }
}
