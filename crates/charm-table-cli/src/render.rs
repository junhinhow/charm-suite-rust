//! Table rendering.
//!
//! Reproduces `renderTable` and `truncate` of the Go original: column widths
//! come from rune counts, `--max-width` shrinks the columns by the same integer
//! arithmetic, and cells are styled with `lipgloss`.

use clap::ValueEnum;
use lipgloss::{Border, Color, Style};

/// Border styles selectable with `--style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum BorderStyle {
    /// Rounded corners (default).
    Rounded,
    /// Thick lines.
    Thick,
    /// Double lines.
    Double,
    /// Single lines. Uses the same characters as the Go `NormalBorder`.
    Ascii,
}

impl BorderStyle {
    /// Returns the `lipgloss` border drawn for this style.
    pub(crate) fn border(self) -> Border {
        match self {
            Self::Rounded => lipgloss::rounded_border(),
            Self::Thick => lipgloss::thick_border(),
            Self::Double => lipgloss::double_border(),
            Self::Ascii => lipgloss::normal_border(),
        }
    }
}

/// Options that control [`render_table`].
#[derive(Debug, Clone)]
pub(crate) struct RenderOptions {
    /// Border style.
    pub(crate) style: BorderStyle,
    /// Maximum table width. `None` means no limit.
    pub(crate) max_width: Option<usize>,
    /// Header foreground color: an ANSI index such as `"63"` or a hex value
    /// such as `"#ff0000"`.
    pub(crate) color: String,
}

/// Renders headers and rows as a bordered table, without a trailing newline.
///
/// # Examples
///
/// ```text
/// let options = RenderOptions {
///     style: BorderStyle::Rounded,
///     max_width: None,
///     color: "63".to_owned(),
/// };
/// let table = render_table(&["a".to_owned()], &[vec!["1".to_owned()]], &options);
/// assert!(table.starts_with('╭'));
/// ```
pub(crate) fn render_table(
    headers: &[String],
    rows: &[Vec<String>],
    options: &RenderOptions,
) -> String {
    let border = options.style.border();
    let widths = column_widths(headers, rows, options.max_width);
    let header_style = Style::new()
        .bold(true)
        .foreground(Color(options.color.clone()))
        .padding_2(0, 1);
    let cell_style = Style::new().padding_2(0, 1);

    let mut out = String::new();
    push_rule(
        &mut out,
        &widths,
        (
            border.top_left,
            border.top,
            border.middle_top,
            border.top_right,
        ),
    );
    out.push('\n');
    push_row(&mut out, &border, headers, &widths, &header_style);
    push_rule(
        &mut out,
        &widths,
        (
            border.middle_left,
            border.bottom,
            border.middle,
            border.middle_right,
        ),
    );
    out.push('\n');
    for row in rows {
        push_row(&mut out, &border, row, &widths, &cell_style);
    }
    push_rule(
        &mut out,
        &widths,
        (
            border.bottom_left,
            border.bottom,
            border.middle_bottom,
            border.bottom_right,
        ),
    );
    out
}

/// Computes the content width of each column.
///
/// The width is the longest header or cell in the column. When `max_width` is
/// set and the table is wider, every column loses the same share of the excess
/// and is kept at no less than 3 characters, as the Go original does. The
/// result can therefore still exceed `max_width` when columns hit that floor.
fn column_widths(headers: &[String], rows: &[Vec<String>], max_width: Option<usize>) -> Vec<usize> {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(cell.chars().count());
        }
    }

    if let Some(max) = max_width {
        let total: usize = widths.iter().map(|w| w + 3).sum();
        if total > max {
            let per_column = (total - max) / widths.len() + 1;
            for width in &mut widths {
                *width = width.saturating_sub(per_column).max(3);
            }
        }
    }
    widths
}

/// Writes a horizontal rule: `left`, a fill repeated for each column (plus 2
/// columns of padding), `joint` between columns, and `right`.
fn push_rule(
    out: &mut String,
    widths: &[usize],
    (left, fill, joint, right): (&str, &str, &str, &str),
) {
    out.push_str(left);
    for (index, width) in widths.iter().enumerate() {
        out.push_str(&fill.repeat(width + 2));
        if index + 1 < widths.len() {
            out.push_str(joint);
        }
    }
    out.push_str(right);
}

/// Writes one line of cells, followed by a newline.
fn push_row(out: &mut String, border: &Border, cells: &[String], widths: &[usize], style: &Style) {
    out.push_str(border.left);
    for (index, width) in widths.iter().enumerate() {
        let cell = cells.get(index).map_or("", String::as_str);
        let content = truncate(cell, *width);
        let width = i32::try_from(*width).unwrap_or(i32::MAX);
        out.push_str(&style.clone().width(width).render(&content));
        if index + 1 < widths.len() {
            out.push_str(border.left);
        }
    }
    out.push_str(border.right);
    out.push('\n');
}

/// Shortens `text` to `max_width` characters, ending with `...` when there is
/// room for it. A `max_width` of 0 leaves the text unchanged.
///
/// Lengths are counted in Unicode scalar values (Go runes), not bytes and not
/// terminal display columns.
///
/// # Examples
///
/// ```text
/// assert_eq!(truncate("Engenheira", 8), "Engen...");
/// assert_eq!(truncate("Bruno", 3), "Bru");
/// assert_eq!(truncate("São Paulo", 0), "São Paulo");
/// ```
pub(crate) fn truncate(text: &str, max_width: usize) -> String {
    if max_width == 0 || text.chars().count() <= max_width {
        return text.to_owned();
    }
    if max_width <= 3 {
        return text.chars().take(max_width).collect();
    }
    let mut out: String = text.chars().take(max_width - 3).collect();
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn truncate_keeps_short_text() {
        assert_eq!(truncate("abc", 3), "abc");
        assert_eq!(truncate("abc", 10), "abc");
        assert_eq!(truncate("abc", 0), "abc");
    }

    #[test]
    fn truncate_adds_ellipsis_when_room_allows() {
        assert_eq!(truncate("Engenheira", 8), "Engen...");
        assert_eq!(truncate("Engenheira", 4), "E...");
    }

    #[test]
    fn truncate_without_ellipsis_for_tiny_widths() {
        assert_eq!(truncate("Bruno", 3), "Bru");
        assert_eq!(truncate("Bruno", 1), "B");
    }

    #[test]
    fn truncate_counts_runes_not_bytes() {
        assert_eq!(truncate("ação", 3), "açã");
        assert_eq!(truncate("ação longa", 6), "açã...");
    }

    #[test]
    fn widths_use_longest_cell_per_column() {
        let headers = strings(&["a", "bb"]);
        let rows = vec![strings(&["cccc", "d"]), strings(&["e"])];
        assert_eq!(column_widths(&headers, &rows, None), vec![4, 2]);
    }

    #[test]
    fn widths_shrink_with_max_width_and_floor_at_three() {
        let headers = strings(&["nome", "descricao"]);
        let rows = vec![strings(&["x", "0123456789012345678901234567890123"])];
        let widths = column_widths(&headers, &rows, Some(20));
        assert_eq!(widths, vec![3, 21]);
    }

    #[test]
    fn widths_floor_is_applied_to_narrow_columns() {
        let headers = strings(&["a", "b", "c", "d"]);
        let rows = vec![strings(&[
            "1",
            "2",
            "3",
            "0123456789012345678901234567890123",
        ])];
        let widths = column_widths(&headers, &rows, Some(5));
        assert!(widths.iter().all(|w| *w >= 3));
    }

    #[test]
    fn render_rounded_single_cell() {
        let options = RenderOptions {
            style: BorderStyle::Rounded,
            max_width: None,
            color: "63".to_owned(),
        };
        let table = render_table(&strings(&["a"]), &[strings(&["1"])], &options);
        let plain = charm_core::strip_ansi(&table);
        assert_eq!(plain, "╭───╮\n│ a │\n├───┤\n│ 1 │\n╰───╯");
    }
}
