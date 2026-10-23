//! Horizontal and vertical bar charts rendered with block characters.

use lipgloss::Style;

use crate::{center, display_width, fit_label, go_max, pad_left, BAR_GLYPH};

const MIN_WIDTH: usize = 1;
const MAX_WIDTH: usize = 500;
const MIN_BAR_WIDTH: usize = 1;
const MAX_BAR_WIDTH: usize = 20;
const VERTICAL_HEIGHT: usize = 10;

/// A bar chart rendered in the terminal.
///
/// The chart can be laid out horizontally (one group of rows per series, the
/// default) or vertically (one column per series).
///
/// # Examples
///
/// ```
/// use charm_charts::BarChart;
///
/// let chart = BarChart::new(&["api", "web"], &[10.0, 20.0]).render();
/// assert!(chart.contains("api"));
/// ```
#[derive(Debug, Clone)]
pub struct BarChart {
    /// The labels rendered next to (or below) the bars.
    pub labels: Vec<String>,
    /// The values plotted by the chart.
    pub values: Vec<f64>,
    /// The maximum width of the chart (`1..=500`).
    pub width: usize,
    /// The thickness of each bar, in rows for horizontal charts or in columns
    /// for vertical charts (`1..=20`).
    pub bar_width: usize,
    /// Whether the chart is rendered horizontally.
    pub horizontal: bool,
    /// The style applied to the whole chart.
    pub style: Style,
    /// The style applied to the bar fills.
    pub bar_style: Style,
    /// The style applied to the labels.
    pub label_style: Style,
    /// Whether the numeric value is displayed next to each bar.
    pub show_values: bool,
}

impl BarChart {
    /// Creates a new bar chart with the given labels and values.
    ///
    /// The default configuration renders a horizontal chart of width `50` with
    /// bars one row thick and the numeric values visible.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]);
    /// assert!(chart.horizontal);
    /// assert_eq!(chart.width, 50);
    /// assert_eq!(chart.bar_width, 1);
    /// ```
    pub fn new(labels: &[&str], values: &[f64]) -> Self {
        Self {
            labels: labels.iter().map(|l| (*l).to_owned()).collect(),
            values: values.to_vec(),
            width: 50,
            bar_width: 1,
            horizontal: true,
            style: Style::new(),
            bar_style: Style::new().foreground(lipgloss::Color("212".to_string())),
            label_style: Style::new().foreground(lipgloss::Color("252".to_string())),
            show_values: true,
        }
    }

    /// Sets the maximum width of the chart.
    ///
    /// The width is clamped to the `1..=500` range, like the Go original.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]).width(20);
    /// assert_eq!(chart.width, 20);
    /// assert_eq!(BarChart::new(&["a"], &[1.0]).width(0).width, 1);
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w.clamp(MIN_WIDTH, MAX_WIDTH);
        self
    }

    /// Sets the thickness of each bar.
    ///
    /// In horizontal charts the thickness is a number of rows; in vertical
    /// charts it is a number of columns. The value is clamped to `1..=20`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]).bar_width(3);
    /// assert_eq!(chart.bar_width, 3);
    /// assert_eq!(BarChart::new(&["a"], &[1.0]).bar_width(0).bar_width, 1);
    /// ```
    #[must_use]
    pub fn bar_width(mut self, w: usize) -> Self {
        self.bar_width = w.clamp(MIN_BAR_WIDTH, MAX_BAR_WIDTH);
        self
    }

    /// Switches the chart to the horizontal layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]).vertical().horizontal();
    /// assert!(chart.horizontal);
    /// ```
    #[must_use]
    pub fn horizontal(mut self) -> Self {
        self.horizontal = true;
        self
    }

    /// Switches the chart to the vertical layout.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]).vertical();
    /// assert!(!chart.horizontal);
    /// ```
    #[must_use]
    pub fn vertical(mut self) -> Self {
        self.horizontal = false;
        self
    }

    /// Sets the style applied to the whole chart.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    /// use lipgloss::Color;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0])
    ///     .style(lipgloss::Style::new().foreground(Color("15".to_string())));
    /// assert!(!chart.render().is_empty());
    /// ```
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Sets the style applied to the bar fills.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    /// use lipgloss::Color;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0])
    ///     .bar_style(lipgloss::Style::new().foreground(Color("9".to_string())));
    /// assert!(!chart.render().is_empty());
    /// ```
    #[must_use]
    pub fn bar_style(mut self, s: Style) -> Self {
        self.bar_style = s;
        self
    }

    /// Sets the style applied to the labels.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    /// use lipgloss::Color;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0])
    ///     .label_style(lipgloss::Style::new().foreground(Color("8".to_string())));
    /// assert!(!chart.render().is_empty());
    /// ```
    #[must_use]
    pub fn label_style(mut self, s: Style) -> Self {
        self.label_style = s;
        self
    }

    /// Enables or disables the numeric value next to each bar.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]).show_values(false);
    /// assert!(!chart.show_values);
    /// ```
    #[must_use]
    pub fn show_values(mut self, show: bool) -> Self {
        self.show_values = show;
        self
    }

    /// Renders the bar chart as a styled string.
    ///
    /// Returns an empty string when there are no values to plot.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// assert_eq!(BarChart::new(&["a"], &[]).render(), "");
    /// ```
    pub fn render(&self) -> String {
        if self.values.is_empty() {
            return String::new();
        }
        if self.horizontal {
            self.render_horizontal()
        } else {
            self.render_vertical()
        }
    }

    /// Renders one group of rows per series, with the label on the left.
    fn render_horizontal(&self) -> String {
        let width = self.width.clamp(MIN_WIDTH, MAX_WIDTH);
        let thickness = self.bar_width.clamp(MIN_BAR_WIDTH, MAX_BAR_WIDTH);
        let max_val = self.max_value();

        let label_width = self.max_label_width();
        let value_width = if self.show_values {
            self.max_value_width()
        } else {
            0
        };

        // Space left for the bars once the label and value columns are fixed.
        let mut bar_area = width.saturating_sub(label_width + 1);
        if self.show_values {
            bar_area = bar_area.saturating_sub(value_width + 1);
        }
        let bar_area = bar_area.max(1);

        let mut lines = Vec::with_capacity(self.values.len() * thickness);
        for (i, val) in self.values.iter().enumerate() {
            let scaled = (val / max_val * bar_area as f64).round();
            let bar_len = if scaled <= 0.0 {
                0
            } else {
                (scaled as usize).min(bar_area)
            };
            let bar = BAR_GLYPH.to_string().repeat(bar_len);

            // The bar cell has a fixed width, so the values line up.
            let bar_cell = format!(
                "{}{}",
                self.bar_style.render(&bar),
                " ".repeat(bar_area - bar_len)
            );

            // The label and the value appear on the first row of the group;
            // the other rows of the thick bar keep the empty columns.
            for row in 0..thickness {
                let label_part = if row == 0 {
                    pad_left(self.label_at(i), label_width)
                } else {
                    " ".repeat(label_width)
                };

                let mut line = format!("{} {bar_cell}", self.label_style.render(&label_part));
                if self.show_values {
                    let value_part = if row == 0 {
                        pad_left(&format_value(*val), value_width)
                    } else {
                        " ".repeat(value_width)
                    };
                    line.push(' ');
                    line.push_str(&value_part);
                }
                lines.push(line);
            }
        }

        self.style.render(&lines.join("\n"))
    }

    /// Renders one column per series, with the values above and the labels
    /// below the bars.
    fn render_vertical(&self) -> String {
        let width = self.width.clamp(MIN_WIDTH, MAX_WIDTH);
        let bar_width = self.bar_width.clamp(MIN_BAR_WIDTH, MAX_BAR_WIDTH);
        let max_val = self.max_value();
        let count = self.values.len();

        // Natural column width: the widest of the bar, the value and the label,
        // so nothing is cut when there is room.
        let mut col_width = bar_width;
        if self.show_values {
            col_width = col_width.max(self.max_value_width());
        }
        col_width = col_width.max(self.max_label_width());

        // The requested width is the limit: the columns shrink to fit it.
        let available = width.saturating_sub(count - 1).max(count);
        let limit = available / count;
        if col_width > limit {
            col_width = limit;
        }
        let col_width = col_width.max(1);

        // A bar cannot be wider than its column.
        let drawn = bar_width.min(col_width);
        let pad = col_width - drawn;
        let left = pad / 2;

        let mut lines = Vec::with_capacity(VERTICAL_HEIGHT + 2);

        if self.show_values {
            let row: Vec<String> = self
                .values
                .iter()
                .map(|v| center(&fit_label(&format_value(*v), col_width), col_width))
                .collect();
            lines.push(row.join(" "));
        }

        let fill = BAR_GLYPH.to_string().repeat(drawn);
        for r in 0..VERTICAL_HEIGHT {
            let threshold = (VERTICAL_HEIGHT - r) as f64 / VERTICAL_HEIGHT as f64;
            let cells: Vec<String> = self
                .values
                .iter()
                .map(|val| {
                    let bar = if val / max_val >= threshold {
                        self.bar_style.render(&fill)
                    } else {
                        " ".repeat(drawn)
                    };
                    format!("{}{bar}{}", " ".repeat(left), " ".repeat(pad - left))
                })
                .collect();
            lines.push(cells.join(" "));
        }

        let labels: Vec<String> = (0..count)
            .map(|i| {
                let label = fit_label(self.label_at(i), col_width);
                self.label_style.render(&center(&label, col_width))
            })
            .collect();
        lines.push(labels.join(" "));

        self.style.render(&lines.join("\n"))
    }

    /// The largest value of the series, or `1.0` when every value is zero.
    fn max_value(&self) -> f64 {
        let mut max_val = 0.0;
        for v in &self.values {
            max_val = go_max(max_val, *v);
        }
        if max_val == 0.0 {
            max_val = 1.0;
        }
        max_val
    }

    /// The label of the series at `index`, or an empty string when missing.
    fn label_at(&self, index: usize) -> &str {
        self.labels.get(index).map_or("", String::as_str)
    }

    /// The widest label among the plotted series, in columns.
    fn max_label_width(&self) -> usize {
        (0..self.values.len())
            .map(|i| display_width(self.label_at(i)))
            .max()
            .unwrap_or(0)
    }

    /// The widest formatted value, in columns.
    fn max_value_width(&self) -> usize {
        self.values
            .iter()
            .map(|v| display_width(&format_value(*v)))
            .max()
            .unwrap_or(0)
    }
}

/// Formats a value with one decimal place, like Go's `%.1f`.
fn format_value(value: f64) -> String {
    format!("{value:.1}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(chart: &BarChart) -> String {
        charm_core::strip_ansi(&chart.render())
    }

    fn plain_chart(labels: &[&str], values: &[f64]) -> BarChart {
        BarChart::new(labels, values)
            .style(Style::new())
            .bar_style(Style::new())
            .label_style(Style::new())
    }

    #[test]
    fn empty_values_render_nothing() {
        assert_eq!(BarChart::new(&["a"], &[]).render(), "");
    }

    #[test]
    fn horizontal_is_the_default_layout() {
        assert!(BarChart::new(&["a"], &[1.0]).horizontal);
    }

    #[test]
    fn the_last_toggle_wins() {
        assert!(
            BarChart::new(&["a"], &[1.0])
                .vertical()
                .horizontal()
                .horizontal
        );
        assert!(
            !BarChart::new(&["a"], &[1.0])
                .horizontal()
                .vertical()
                .horizontal
        );
    }

    #[test]
    fn negative_values_render_an_empty_bar() {
        let chart = BarChart::new(&["a"], &[-4.0]).render();
        assert!(!chart.contains(BAR_GLYPH));
    }

    #[test]
    fn missing_labels_are_rendered_as_blank() {
        let chart = BarChart::new(&["a"], &[1.0, 2.0]).render();
        assert_eq!(chart.lines().count(), 2);
    }

    #[test]
    fn extra_labels_are_ignored() {
        let chart = BarChart::new(&["a", "b"], &[1.0]).render();
        assert_eq!(chart.lines().count(), 1);
    }

    #[test]
    fn values_are_hidden_when_requested() {
        let chart = BarChart::new(&["a"], &[1.5]).show_values(false).render();
        assert!(!chart.contains("1.5"));
    }

    #[test]
    fn values_are_shown_by_default() {
        let chart = BarChart::new(&["a"], &[1.5]).render();
        assert!(chart.contains("1.5"));
    }

    #[test]
    fn zero_width_is_clamped_and_still_draws_a_bar() {
        let chart = BarChart::new(&["a"], &[1.0]).width(0).render();
        assert!(chart.contains(BAR_GLYPH));
    }

    #[test]
    fn zero_bar_width_is_clamped_to_one_row() {
        assert_eq!(BarChart::new(&["a"], &[1.0]).bar_width(0).bar_width, 1);
        assert_eq!(
            BarChart::new(&["a"], &[1.0])
                .bar_width(0)
                .render()
                .lines()
                .count(),
            1
        );
    }

    #[test]
    fn vertical_chart_has_a_value_row_when_values_are_shown() {
        let chart = BarChart::new(&["a", "b"], &[1.0, 2.0]).vertical().render();
        assert_eq!(chart.lines().count(), 12);
        let hidden = BarChart::new(&["a", "b"], &[1.0, 2.0])
            .vertical()
            .show_values(false)
            .render();
        assert_eq!(hidden.lines().count(), 11);
    }

    #[test]
    fn bar_width_adds_rows_to_horizontal_charts() {
        let chart = BarChart::new(&["a"], &[1.0]).bar_width(3).render();
        assert_eq!(chart.lines().count(), 3);
    }

    #[test]
    fn thick_horizontal_bars_repeat_the_fill_on_each_row() {
        let chart = plain_chart(&["Go", "Rust"], &[100.0, 50.0])
            .width(40)
            .bar_width(3);
        let text = plain(&chart);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines.len(), 6);
        assert!(lines[0].contains("Go"));
        assert!(!lines[1].contains("Go"));
        assert!(lines[1].contains(BAR_GLYPH));
        assert_eq!(
            lines[0].matches(BAR_GLYPH).count(),
            lines[1].matches(BAR_GLYPH).count()
        );
    }

    #[test]
    fn horizontal_rows_have_the_same_width() {
        let chart =
            plain_chart(&["Go", "Rust", "Python", "JS"], &[85.0, 72.0, 95.0, 68.0]).width(50);
        let text = plain(&chart);
        let lines: Vec<&str> = text.lines().collect();
        let expected = display_width(lines[0]);
        for line in &lines {
            assert_eq!(display_width(line), expected, "{line:?}");
        }
    }

    #[test]
    fn horizontal_labels_are_right_aligned() {
        let chart =
            plain_chart(&["Go", "Rust", "Python", "JS"], &[85.0, 72.0, 95.0, 68.0]).width(50);
        let text = plain(&chart);
        for (line, label) in text.lines().zip(["Go", "Rust", "Python", "JS"]) {
            let index = line.find(label).unwrap_or_default();
            assert_eq!(
                display_width(&line[..index]) + display_width(label),
                6,
                "{line:?}"
            );
        }
    }

    #[test]
    fn horizontal_chart_respects_the_width() {
        let chart = plain_chart(&["Programação", "Go"], &[85.0, 72.0]).width(20);
        for line in plain(&chart).lines() {
            assert!(display_width(line) <= 20, "{line:?}");
        }
    }

    #[test]
    fn vertical_chart_respects_the_width() {
        for width in [20, 30, 50, 120] {
            let chart = plain_chart(&["Go", "Rust", "Python", "JS"], &[85.0, 72.0, 95.0, 68.0])
                .width(width)
                .vertical()
                .bar_width(3);
            for line in plain(&chart).lines() {
                assert!(display_width(line) <= width, "width {width}: {line:?}");
            }
        }
    }

    #[test]
    fn vertical_chart_scales_with_the_width() {
        let labels = ["Go", "Rust", "Python", "JS"];
        let values = [85.0, 72.0, 95.0, 68.0];
        let narrow = plain_chart(&labels, &values)
            .width(20)
            .vertical()
            .bar_width(3);
        let wide = plain_chart(&labels, &values)
            .width(60)
            .vertical()
            .bar_width(3);
        assert!(display_width(&plain(&wide)) > display_width(&plain(&narrow)));
    }

    #[test]
    fn vertical_chart_shows_values_only_when_asked() {
        let with_values = plain(&plain_chart(&["Go", "Rust"], &[85.0, 72.0]).vertical());
        let first = with_values.lines().next().unwrap_or_default();
        assert!(first.contains("85.0") && first.contains("72.0"));

        let without = plain(
            &plain_chart(&["Go", "Rust"], &[85.0, 72.0])
                .vertical()
                .show_values(false),
        );
        assert!(!without.contains("85.0"));
    }

    #[test]
    fn vertical_labels_are_centered_under_their_columns() {
        let chart = plain_chart(&["Go", "Rust", "Python", "JS"], &[85.0, 72.0, 95.0, 68.0])
            .width(50)
            .vertical()
            .bar_width(3);
        let text = plain(&chart);
        let last = text.lines().last().unwrap_or_default();
        assert!(last.contains(" Go "), "{last:?}");
        assert!(last.contains("Rust"), "{last:?}");
    }

    #[test]
    fn long_vertical_labels_are_shortened_with_an_ellipsis() {
        let chart = plain_chart(&["alpha"], &[1.0])
            .vertical()
            .width(3)
            .show_values(false);
        let text = plain(&chart);
        let last = text.lines().last().unwrap_or_default();
        assert_eq!(last, "al…");
    }

    #[test]
    fn vertical_bar_wider_than_its_column_is_shrunk() {
        let chart = plain_chart(&["a", "b"], &[1.0, 2.0])
            .width(12)
            .vertical()
            .bar_width(20)
            .show_values(false);
        for line in plain(&chart).lines() {
            assert!(display_width(line) <= 12, "{line:?}");
        }
    }

    #[test]
    fn long_label_on_vertical_chart_keeps_the_width() {
        let chart = plain_chart(&["Programação"], &[10.0]).width(20).vertical();
        for line in plain(&chart).lines() {
            assert!(display_width(line) <= 20, "{line:?}");
        }
    }
}
