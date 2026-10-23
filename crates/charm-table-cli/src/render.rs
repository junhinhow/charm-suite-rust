//! Table rendering.
//!
//! Mirrors `render.go` of the Go original. Column widths are measured in
//! terminal cells, cell text is sanitized (line breaks and tabs become spaces)
//! and truncated to the column width, and `--max-width` caps the widest
//! columns to a common limit. Cells are styled with `lipgloss`.

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
    /// Plain ASCII characters: `+`, `-` and `|`.
    Ascii,
}

impl BorderStyle {
    /// Returns the `lipgloss` border drawn for this style.
    pub(crate) fn border(self) -> Border {
        match self {
            Self::Rounded => lipgloss::rounded_border(),
            Self::Thick => lipgloss::thick_border(),
            Self::Double => lipgloss::double_border(),
            Self::Ascii => lipgloss::ascii_border(),
        }
    }
}

/// Options that control [`render_table`].
#[derive(Debug, Clone)]
pub(crate) struct RenderOptions {
    /// Border style.
    pub(crate) style: BorderStyle,
    /// Maximum table width, borders included. `None` means no limit.
    pub(crate) max_width: Option<usize>,
    /// Header foreground color: an ANSI index such as `"63"` or a hex value
    /// such as `"#ff0000"`.
    pub(crate) color: String,
}

/// Error raised when a table cannot be drawn.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(crate) enum RenderError {
    /// The table has no columns.
    #[error("nenhuma coluna para renderizar")]
    NoColumns,
    /// `--max-width` is too small even for the borders and the padding.
    #[error(
        "largura maxima {max_width} insuficiente para {columns} colunas: sao necessarias ao menos {minimum} colunas"
    )]
    MaxWidthTooSmall {
        /// Requested maximum width.
        max_width: usize,
        /// Number of columns in the table.
        columns: usize,
        /// Smallest width that can hold the table.
        minimum: usize,
    },
}

/// Renders headers and rows as a bordered table.
///
/// The result ends with a newline, like the Go original's output.
///
/// # Errors
///
/// Returns [`RenderError::NoColumns`] when there are no headers and
/// [`RenderError::MaxWidthTooSmall`] when `max_width` cannot hold the borders.
pub(crate) fn render_table(
    headers: &[String],
    rows: &[Vec<String>],
    options: &RenderOptions,
) -> Result<String, RenderError> {
    if headers.is_empty() {
        return Err(RenderError::NoColumns);
    }

    let border = options.style.border();
    let mut widths = column_widths(headers, rows);
    if let Some(max_width) = options.max_width {
        widths = fit_to_max_width(&widths, max_width)?;
    }

    // In lipgloss v2, `width` is the full width of the block, padding included,
    // so each cell is given its column width plus the 2 padding columns.
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
    push_row(&mut out, &border, headers, &widths, &header_style);
    push_rule(
        &mut out,
        &widths,
        (
            border.middle_left,
            border.top,
            border.middle,
            border.middle_right,
        ),
    );
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
    Ok(out)
}

/// Computes the width of each column: the widest sanitized header or cell, in
/// terminal cells.
fn column_widths(headers: &[String], rows: &[Vec<String>]) -> Vec<usize> {
    let mut widths: Vec<usize> = headers
        .iter()
        .map(|header| charm_core::display_width(&sanitize_cell(header)))
        .collect();
    for row in rows {
        for (width, cell) in widths.iter_mut().zip(row) {
            *width = (*width).max(charm_core::display_width(&sanitize_cell(cell)));
        }
    }
    widths
}

/// Shrinks the columns so that the whole table fits in `max_width`.
///
/// The borders take one cell per column plus one at each edge, and each column
/// takes two cells of padding. Every column wider than a common cap is cut down
/// to that cap, the cap being the largest value that still fits. Whatever room
/// is left is then given back, one cell at a time, to the cut columns.
fn fit_to_max_width(widths: &[usize], max_width: usize) -> Result<Vec<usize>, RenderError> {
    let columns = widths.len();
    let overhead = 3 * columns + 1;
    let minimum = overhead + columns;
    if max_width < minimum {
        return Err(RenderError::MaxWidthTooSmall {
            max_width,
            columns,
            minimum,
        });
    }
    let budget = max_width - overhead;

    let total: usize = widths.iter().sum();
    if total <= budget {
        return Ok(widths.to_vec());
    }

    let largest = widths.iter().copied().max().unwrap_or(0);
    let (mut low, mut high) = (1, largest);
    while low < high {
        let mid = (low + high).div_ceil(2);
        if sum_capped(widths, mid) <= budget {
            low = mid;
        } else {
            high = mid - 1;
        }
    }

    let mut fitted: Vec<usize> = widths.iter().map(|&width| width.min(low)).collect();
    let mut used: usize = fitted.iter().sum();
    for (index, &width) in widths.iter().enumerate() {
        if used >= budget {
            break;
        }
        if width > low {
            fitted[index] += 1;
            used += 1;
        }
    }
    Ok(fitted)
}

/// Sums the widths after capping each one at `cap`.
fn sum_capped(widths: &[usize], cap: usize) -> usize {
    widths.iter().map(|&width| width.min(cap)).sum()
}

/// Writes a horizontal rule: `left`, a fill repeated for each column (plus 2
/// columns of padding), `joint` between columns, and `right`, followed by a
/// newline.
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
    out.push('\n');
}

/// Writes one line of cells, followed by a newline. Missing cells render empty.
fn push_row(out: &mut String, border: &Border, cells: &[String], widths: &[usize], style: &Style) {
    out.push_str(border.left);
    for (index, width) in widths.iter().enumerate() {
        let cell = cells.get(index).map_or("", String::as_str);
        let content = fit_text(cell, *width);
        let total = i32::try_from(width.saturating_add(2)).unwrap_or(i32::MAX);
        out.push_str(&style.clone().width(total).render(&content));
        if index + 1 < widths.len() {
            out.push_str(border.left);
        }
    }
    out.push_str(border.right);
    out.push('\n');
}

/// Sanitizes a cell and cuts it to `width` terminal cells.
///
/// A width of 0 yields an empty string.
fn fit_text(text: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    charm_core::truncate(&sanitize_cell(text), width)
}

/// Replaces line breaks and tabs with spaces, so that a cell cannot break the
/// layout of the table.
fn sanitize_cell(text: &str) -> String {
    text.replace("\r\n", " ").replace(['\r', '\n', '\t'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    fn options(style: BorderStyle, max_width: Option<usize>) -> RenderOptions {
        RenderOptions {
            style,
            max_width,
            color: "63".to_owned(),
        }
    }

    fn plain_lines(table: &str) -> Vec<String> {
        charm_core::strip_ansi(table)
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn widths_use_longest_cell_per_column() {
        let headers = strings(&["a", "bb"]);
        let rows = vec![strings(&["cccc", "d"]), strings(&["e"])];
        assert_eq!(column_widths(&headers, &rows), vec![4, 2]);
    }

    #[test]
    fn widths_are_measured_in_terminal_cells() {
        let headers = strings(&["idioma"]);
        let rows = vec![strings(&["\u{65e5}\u{672c}\u{8a9e}"])];
        assert_eq!(column_widths(&headers, &rows), vec![6]);
    }

    #[test]
    fn sanitize_replaces_line_breaks_and_tabs() {
        assert_eq!(sanitize_cell("a\r\nb\tc\nd\re"), "a b c d e");
    }

    #[test]
    fn fit_keeps_widths_that_already_fit() {
        assert_eq!(fit_to_max_width(&[4, 2], 20).ok(), Some(vec![4, 2]));
    }

    #[test]
    fn fit_caps_the_widest_columns_to_a_common_limit() {
        // budget = 30 - (3 * 3 + 1) = 20: the cap is 8 (8 + 8 + 3 = 19), and the
        // one spare cell goes to the first cut column.
        let fitted = fit_to_max_width(&[30, 30, 3], 30).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(fitted, vec![9, 8, 3]);
    }

    #[test]
    fn fit_rejects_limits_below_the_minimum() {
        let err = fit_to_max_width(&[1, 1, 1], 8).err();
        let text = err.map(|e| e.to_string()).unwrap_or_default();
        assert!(text.contains('8'), "{text}");
    }

    #[test]
    fn fit_text_never_splits_wide_glyphs() {
        let text = "\u{65e5}\u{672c}\u{8a9e}\u{30c6}\u{30ad}\u{30b9}\u{30c8}";
        let cut = fit_text(text, 5);
        assert!(charm_core::display_width(&cut) <= 5, "{cut:?}");
        assert_eq!(cut, "\u{65e5}...");
    }

    #[test]
    fn rounded_single_cell_table() {
        let table = render_table(
            &strings(&["a"]),
            &[strings(&["1"])],
            &options(BorderStyle::Rounded, None),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let plain = charm_core::strip_ansi(&table);
        assert_eq!(plain, "╭───╮\n│ a │\n├───┤\n│ 1 │\n╰───╯\n");
    }

    #[test]
    fn ascii_style_uses_plus_dash_and_pipe() {
        let table = render_table(
            &strings(&["a", "b"]),
            &[strings(&["1", "2"])],
            &options(BorderStyle::Ascii, None),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let plain = charm_core::strip_ansi(&table);
        assert!(plain.contains("+---+"), "{plain}");
        for drawn in ["─", "│", "╭", "╮", "╰", "╯", "├", "┤", "┬", "┴", "┼"] {
            assert!(!plain.contains(drawn), "{plain}");
        }
    }

    #[test]
    fn every_line_has_the_same_width_with_wide_glyphs() {
        let headers = strings(&["idioma", "valor"]);
        let rows = vec![
            strings(&["\u{65e5}\u{672c}\u{8a9e}", "42"]),
            strings(&["\u{1f389}", "ok"]),
        ];
        let table = render_table(&headers, &rows, &options(BorderStyle::Rounded, None))
            .unwrap_or_else(|e| panic!("{e}"));
        let widths: Vec<usize> = plain_lines(&table)
            .iter()
            .map(|line| charm_core::display_width(line))
            .collect();
        assert!(widths.windows(2).all(|pair| pair[0] == pair[1]), "{table}");
    }

    #[test]
    fn max_width_is_never_exceeded_for_representable_limits() {
        let headers = strings(&["nome", "descricao", "cidade"]);
        let rows = vec![strings(&[
            "Alice",
            "Uma descri\u{e7}\u{e3}o bem longa mesmo",
            "S\u{e3}o Paulo",
        ])];
        for limit in [20, 30, 40, 60] {
            let table = render_table(&headers, &rows, &options(BorderStyle::Rounded, Some(limit)))
                .unwrap_or_else(|e| panic!("{e}"));
            for line in plain_lines(&table) {
                assert!(
                    charm_core::display_width(&line) <= limit,
                    "limit {limit}: {line:?}"
                );
            }
        }
    }

    #[test]
    fn no_columns_is_an_error() {
        let err = render_table(&[], &[], &options(BorderStyle::Rounded, None)).err();
        assert_eq!(err, Some(RenderError::NoColumns));
    }
}
