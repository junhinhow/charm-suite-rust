//! Integer geometry shared by the flex and the grid engines.
//!
//! Port of the unexported helpers of the Go original (`geometry.go`).

use crate::util::{to_i32, to_i64};

/// Splits `total` units among the slices given by `weights`.
///
/// The slices always add up to `total`. Each slice receives the integer part
/// of its share; the remaining units go to the slices with the largest
/// fractions, with ties resolved in favour of the lower index. Null or
/// all-zero weights split the space evenly.
pub(crate) fn distribute(total: i64, weights: &[f64]) -> Vec<i64> {
    let count = weights.len();
    let mut slices = vec![0_i64; count];
    if count == 0 || total <= 0 {
        return slices;
    }

    let sum: f64 = weights.iter().sum();
    if sum <= 0.0 {
        let n = to_i64(count);
        let base = total / n;
        let rest = total % n;
        for (i, slice) in slices.iter_mut().enumerate() {
            *slice = base + i64::from(to_i64(i) < rest);
        }
        return slices;
    }

    let mut fractions = vec![0.0_f64; count];
    let mut used = 0_i64;
    for (i, weight) in weights.iter().enumerate() {
        let exact = total as f64 * *weight / sum;
        let whole = exact.floor();
        slices[i] = whole as i64;
        fractions[i] = exact - whole;
        used += whole as i64;
    }

    let mut order: Vec<usize> = (0..count).collect();
    order.sort_by(|&a, &b| fractions[b].total_cmp(&fractions[a]));

    let mut rest = total - used;
    for &idx in &order {
        if rest <= 0 {
            break;
        }
        slices[idx] += 1;
        rest -= 1;
    }

    // Floating point rounding can still leave the sum off by a unit; correct
    // it one step at a time, as the Go original does.
    for _ in 0..=count {
        let current: i64 = slices.iter().sum();
        if current == total {
            break;
        }
        if current < total {
            slices[order[0]] += 1;
            continue;
        }
        if let Some(&idx) = order.iter().find(|&&idx| slices[idx] > 0) {
            slices[idx] -= 1;
        }
    }

    slices
}

/// Returns `count` equal weights, used to split space evenly.
pub(crate) fn equal_weights(count: usize) -> Vec<f64> {
    vec![1.0; count]
}

/// Reduces `gap` so that `count - 1` gaps fit in `space`.
///
/// The result is never negative and never larger than `gap`.
pub(crate) fn limit_gap(gap: i64, space: i64, count: usize) -> i64 {
    if count <= 1 || gap <= 0 {
        return 0;
    }
    gap.min(space / (to_i64(count) - 1))
}

/// Normalizes a rendered block so that it fills exactly `width` columns and at
/// most `height` lines.
///
/// `lipgloss` wraps the content inside the width, but wide characters (CJK,
/// emoji) can still overflow it by one column, so the block is cut and padded
/// explicitly.
pub(crate) fn fit_block(block: &str, width: i64, height: i64) -> String {
    let mut lines: Vec<&str> = block.split('\n').collect();
    if height > 0 && to_i64(lines.len()) > height {
        lines.truncate(height as usize);
    }
    lines
        .into_iter()
        .map(|line| fit_line(line, width))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Fits one line to exactly `width` columns.
fn fit_line(line: &str, width: i64) -> String {
    if width <= 0 {
        return String::new();
    }
    let width = width as usize;
    let mut fitted = if charm_core::display_width(line) > width {
        lipgloss::Style::truncate_visible_line(line, width)
    } else {
        line.to_owned()
    };
    let current = charm_core::display_width(&fitted);
    if current < width {
        fitted.push_str(&" ".repeat(width - current));
    }
    fitted
}

/// Renders `content` inside a cell with exactly `width` columns and `height`
/// lines, truncating whatever does not fit.
pub(crate) fn render_cell(content: &str, width: i64, height: i64) -> String {
    if width <= 0 {
        return fit_block("", 0, height);
    }
    let style = lipgloss::Style::new()
        .width(to_i32(width))
        .height(to_i32(height));
    fit_block(&style.render(content), width, height)
}

/// Reports whether `ratio` can take part in the layout calculation.
pub(crate) fn ratio_is_valid(ratio: f64) -> bool {
    ratio.is_finite() && ratio >= 0.0
}

/// Returns the ratio that can be used in the calculation: negative, NaN and
/// infinite values count as zero.
pub(crate) fn safe_ratio(ratio: f64) -> f64 {
    if ratio_is_valid(ratio) {
        ratio
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distribute_preserves_the_total() {
        for total in [0, 1, 7, 38, 80, 1000] {
            let slices = distribute(total, &[0.3, 0.7, 0.2]);
            assert_eq!(slices.iter().sum::<i64>(), total, "total {total}");
        }
    }

    #[test]
    fn distribute_uses_the_largest_remainder() {
        // 38 * 0.3 = 11.4 and 38 * 0.7 = 26.6: the spare unit goes to 0.6.
        assert_eq!(distribute(38, &[0.3, 0.7]), vec![11, 27]);
    }

    #[test]
    fn distribute_breaks_ties_by_the_lower_index() {
        assert_eq!(distribute(40, &[1.0, 1.0, 1.0]), vec![14, 13, 13]);
    }

    #[test]
    fn distribute_splits_evenly_when_weights_are_zero() {
        assert_eq!(distribute(10, &[0.0, 0.0, 0.0]), vec![4, 3, 3]);
        assert_eq!(distribute(10, &[]), Vec::<i64>::new());
    }

    #[test]
    fn distribute_returns_zeros_for_non_positive_totals() {
        assert_eq!(distribute(0, &[1.0, 1.0]), vec![0, 0]);
        assert_eq!(distribute(-5, &[1.0, 1.0]), vec![0, 0]);
    }

    #[test]
    fn limit_gap_shrinks_to_fit() {
        assert_eq!(limit_gap(50, 10, 2), 10);
        assert_eq!(limit_gap(2, 10, 2), 2);
        assert_eq!(limit_gap(3, 10, 1), 0);
        assert_eq!(limit_gap(-1, 10, 3), 0);
    }

    #[test]
    fn ratio_helpers_reject_invalid_values() {
        assert!(ratio_is_valid(0.0));
        assert!(!ratio_is_valid(-0.1));
        assert!(!ratio_is_valid(f64::NAN));
        assert!(!ratio_is_valid(f64::INFINITY));
        assert_eq!(safe_ratio(-2.0), 0.0);
        assert_eq!(safe_ratio(0.5), 0.5);
    }

    #[test]
    fn fit_block_pads_and_truncates() {
        assert_eq!(fit_block("ab", 4, 2), "ab  ");
        assert_eq!(fit_block("abcdef", 3, 1), "abc");
        assert_eq!(fit_block("a\nb\nc", 1, 2), "a\nb");
    }
}
