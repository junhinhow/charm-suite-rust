//! Horizontal and vertical bar charts rendered with block characters.

use std::fmt::Write as _;

use lipgloss::Style;

use crate::{go_max, pad_like_go, to_i64, truncate_bytes, BAR_GLYPH};

/// A bar chart rendered in the terminal.
///
/// The chart can be laid out horizontally (one row per series, the default) or
/// vertically (one column per series).
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
    /// The maximum width of the chart.
    pub width: usize,
    /// The thickness of each bar (in rows for horizontal charts).
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
    /// bars one character wide and the numeric values visible.
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
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]).width(20);
    /// assert_eq!(chart.width, 20);
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// Sets the thickness of each bar (in rows for horizontal charts).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::BarChart;
    ///
    /// let chart = BarChart::new(&["a"], &[1.0]).bar_width(3);
    /// assert_eq!(chart.bar_width, 3);
    /// ```
    #[must_use]
    pub fn bar_width(mut self, w: usize) -> Self {
        self.bar_width = w;
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

    /// Renders one row per series, with the label on the left.
    fn render_horizontal(&self) -> String {
        let max_val = self.max_value();

        // Longest label, measured in bytes (like the Go original).
        let mut max_label_len = 0usize;
        for label in &self.labels {
            if label.len() > max_label_len {
                max_label_len = label.len();
            }
        }

        let mut bar_max_width = to_i64(self.width) - to_i64(max_label_len) - 2;
        if self.show_values {
            bar_max_width -= 10;
        }
        if bar_max_width < 5 {
            bar_max_width = 5;
        }

        let mut result = String::new();
        for (i, val) in self.values.iter().enumerate() {
            let label = if i < self.labels.len() {
                self.labels[i].as_str()
            } else {
                ""
            };

            let padded_label = pad_like_go(label, max_label_len, false);
            let mut bar_len = (val / max_val * bar_max_width as f64).round() as i64;
            if bar_len < 0 {
                bar_len = 0;
            }

            let bar = BAR_GLYPH.to_string().repeat(bar_len as usize);

            let mut line = self.label_style.render(&padded_label);
            line.push(' ');
            line.push_str(&self.bar_style.render(&bar));
            if self.show_values {
                line.push(' ');
                let _ = write!(line, "{val:.1}");
            }

            for row in 0..self.bar_width {
                result.push_str(&line);
                if i < self.values.len() - 1 || row < self.bar_width.saturating_sub(1) {
                    result.push('\n');
                }
            }
        }

        self.style.render(&result)
    }

    /// Renders one column per series, with the labels on the bottom row.
    fn render_vertical(&self) -> String {
        let max_val = self.max_value();

        let chart_height = 10usize;
        let col_width = self.bar_width + 1;

        let mut rows: Vec<String> = vec![String::new(); chart_height + 1];

        for (row, out) in rows.iter_mut().enumerate().take(chart_height) {
            let threshold = (chart_height - row) as f64 / chart_height as f64;
            for val in &self.values {
                let normalized = val / max_val;
                if normalized >= threshold {
                    out.push_str(
                        &self
                            .bar_style
                            .render(&BAR_GLYPH.to_string().repeat(self.bar_width)),
                    );
                } else {
                    out.push_str(&" ".repeat(self.bar_width));
                }
                out.push(' ');
            }
        }

        // Label row.
        let Some(label_row) = rows.last_mut() else {
            return String::new();
        };
        for i in 0..self.values.len() {
            let mut label = if i < self.labels.len() {
                self.labels[i].clone()
            } else {
                String::new()
            };
            if label.len() > col_width {
                label = format!("{}.", truncate_bytes(&label, col_width - 1));
            }
            label_row.push_str(
                &self
                    .label_style
                    .render(&pad_like_go(&label, col_width, true)),
            );
        }

        let mut result = String::new();
        for (i, row) in rows.iter().enumerate() {
            result.push_str(row);
            if i < rows.len() - 1 {
                result.push('\n');
            }
        }

        self.style.render(&result)
    }

    /// The largest value of the series, or `1.0` when every value is zero.
    fn max_value(&self) -> f64 {
        let mut max_val = 0.0f64;
        for v in &self.values {
            max_val = go_max(max_val, *v);
        }
        if max_val == 0.0 {
            max_val = 1.0;
        }
        max_val
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(!chart.contains('\u{2588}'));
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
    fn zero_width_still_draws_a_minimum_bar() {
        let chart = BarChart::new(&["a"], &[1.0]).width(0).render();
        // The bar area never shrinks below five cells.
        assert!(chart.contains('\u{2588}'));
    }

    #[test]
    fn vertical_chart_has_eleven_rows() {
        let chart = BarChart::new(&["a", "b"], &[1.0, 2.0]).vertical().render();
        assert_eq!(chart.lines().count(), 11);
    }

    #[test]
    fn bar_width_adds_rows_to_horizontal_charts() {
        let chart = BarChart::new(&["a"], &[1.0]).bar_width(3).render();
        assert_eq!(chart.lines().count(), 3);
    }

    #[test]
    fn zero_bar_width_renders_nothing() {
        assert_eq!(BarChart::new(&["a"], &[1.0]).bar_width(0).render(), "");
    }

    #[test]
    fn long_vertical_labels_are_truncated() {
        let chart = BarChart::new(&["alpha"], &[1.0]).vertical().render();
        let last = charm_core::strip_ansi(&chart);
        let last = last.lines().last().unwrap_or_default();
        assert_eq!(last, "a.");
    }
}
