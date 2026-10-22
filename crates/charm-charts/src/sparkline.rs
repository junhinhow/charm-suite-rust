//! Compact line chart rendered with Unicode block characters.

use lipgloss::Style;

use crate::{go_max, go_min, to_i64, BLOCKS};

/// A compact line chart drawn with Unicode block characters.
///
/// The data is automatically resampled to fit the requested width and scaled
/// between the smallest and the largest value of the resampled series.
///
/// # Examples
///
/// ```
/// use charm_charts::Sparkline;
///
/// let line = Sparkline::new(&[1.0, 5.0, 3.0, 8.0]).render();
/// assert_eq!(line.chars().count(), 4);
/// ```
#[derive(Debug, Clone)]
pub struct Sparkline {
    /// The raw data points to plot.
    pub data: Vec<f64>,
    /// The display width, in characters.
    pub width: usize,
    /// The display height, in rows.
    pub height: usize,
    /// The style applied to the rendered blocks.
    pub style: Style,
    /// An optional label rendered before the chart.
    pub label: String,
}

impl Sparkline {
    /// Creates a new sparkline with the given data points.
    ///
    /// The default configuration is a width of `40`, a height of `1` and no
    /// label.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::Sparkline;
    ///
    /// let line = Sparkline::new(&[3.0, 1.0, 2.0]);
    /// assert_eq!(line.width, 40);
    /// assert_eq!(line.height, 1);
    /// ```
    pub fn new(data: &[f64]) -> Self {
        Self {
            data: data.to_vec(),
            width: 40,
            height: 1,
            style: Style::new(),
            label: String::new(),
        }
    }

    /// Sets the display width, in characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::Sparkline;
    ///
    /// let line = Sparkline::new(&[1.0, 2.0, 3.0]).width(10);
    /// assert_eq!(line.width, 10);
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// Sets the display height, in rows.
    ///
    /// The height is clamped to the `1..=8` range, exactly like the Go
    /// implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::Sparkline;
    ///
    /// let line = Sparkline::new(&[1.0, 2.0, 3.0]).height(99);
    /// assert_eq!(line.height, 8);
    /// ```
    #[must_use]
    pub fn height(mut self, h: usize) -> Self {
        self.height = h.clamp(1, 8);
        self
    }

    /// Sets the style used to render the sparkline characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::Sparkline;
    /// use lipgloss::Color;
    ///
    /// let line = Sparkline::new(&[1.0, 2.0])
    ///     .style(lipgloss::Style::new().foreground(Color("212".to_string())));
    /// assert!(!line.render().is_empty());
    /// ```
    #[must_use]
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }

    /// Sets the label rendered before the sparkline.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::Sparkline;
    ///
    /// let line = Sparkline::new(&[1.0, 2.0]).label("cpu").render();
    /// assert!(line.starts_with("cpu "));
    /// ```
    #[must_use]
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Renders the sparkline as a styled string.
    ///
    /// Returns an empty string when there is no data to plot, or when the
    /// configured width resamples the data down to nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::Sparkline;
    ///
    /// assert_eq!(Sparkline::new(&[]).render(), "");
    /// ```
    pub fn render(&self) -> String {
        if self.data.is_empty() {
            return String::new();
        }

        let resampled = resample(&self.data, self.width);
        if resampled.is_empty() {
            // The Go original panics here (it indexes `resampled[0]`).
            return String::new();
        }

        let mut min_val = resampled[0];
        let mut max_val = resampled[0];
        for v in &resampled {
            min_val = go_min(min_val, *v);
            max_val = go_max(max_val, *v);
        }

        let mut span = max_val - min_val;
        if span == 0.0 {
            span = 1.0;
        }

        if self.height == 1 {
            let mut line = String::new();
            if !self.label.is_empty() {
                line.push_str(&self.label);
                line.push(' ');
            }
            let max_index = to_i64(BLOCKS.len()) - 1;
            for v in &resampled {
                let normalized = (v - min_val) / span;
                let idx = (normalized * max_index as f64) as i64;
                line.push(BLOCKS[idx.clamp(0, max_index) as usize]);
            }
            return self.style.render(&line);
        }

        if self.height == 0 {
            // Degenerate height: the Go original produces an empty block list
            // and only keeps the (un-styled) label.
            return if self.label.is_empty() {
                String::new()
            } else {
                format!("{}\n", self.label)
            };
        }

        // Multi-row rendering.
        let levels = BLOCKS.len() * self.height;
        let max_index = to_i64(BLOCKS.len()) - 1;
        let mut rows: Vec<String> = vec![String::new(); self.height];
        for v in &resampled {
            let normalized = (v - min_val) / span;
            let cell_level = (normalized * (levels - 1) as f64) as i64;

            for (row, out) in rows.iter_mut().enumerate() {
                let row_from_bottom = self.height - 1 - row;
                let row_base = to_i64(row_from_bottom * (BLOCKS.len() - 1));
                let local_level = cell_level - row_base;

                if local_level <= 0 {
                    out.push(' ');
                } else if local_level >= max_index {
                    out.push(BLOCKS[BLOCKS.len() - 1]);
                } else {
                    out.push(BLOCKS[local_level as usize]);
                }
            }
        }

        let mut result = String::new();
        if !self.label.is_empty() {
            result.push_str(&self.label);
            result.push('\n');
        }
        for (i, row) in rows.iter().enumerate() {
            result.push_str(&self.style.render(row));
            if i < rows.len() - 1 {
                result.push('\n');
            }
        }
        result
    }
}

/// Reduces or expands the data to fit the target width.
///
/// This is a direct port of the Go `resample` helper: when the data already
/// fits into the width it is returned unchanged, otherwise each output bucket
/// holds the arithmetic mean of the source values it covers.
///
/// * `data` is empty: returns an empty vector.
/// * `width` is `0`: returns an empty vector (the Go original panics later,
///   when the caller indexes the first element).
pub(crate) fn resample(data: &[f64], width: usize) -> Vec<f64> {
    if data.is_empty() {
        return Vec::new();
    }
    if data.len() <= width {
        return data.to_vec();
    }
    let mut result = vec![0.0f64; width];
    let ratio = data.len() as f64 / width as f64;
    for (i, slot) in result.iter_mut().enumerate() {
        let start = (i as f64 * ratio) as usize;
        let mut end = ((i + 1) as f64 * ratio) as usize;
        if end > data.len() {
            end = data.len();
        }
        let mut sum = 0.0f64;
        for value in &data[start..end] {
            sum += value;
        }
        *slot = sum / (end - start) as f64;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_data_renders_nothing() {
        assert_eq!(Sparkline::new(&[]).render(), "");
    }

    #[test]
    fn zero_width_renders_nothing_instead_of_panicking() {
        // The Go original panics with an index out of range.
        assert_eq!(Sparkline::new(&[1.0, 2.0, 3.0]).width(0).render(), "");
    }

    #[test]
    fn flat_series_renders_spaces() {
        let out = Sparkline::new(&[4.0, 4.0, 4.0]).render();
        assert_eq!(out, "   ");
    }

    #[test]
    fn label_is_prefixed_with_a_space() {
        let out = Sparkline::new(&[1.0, 2.0]).label("cpu").render();
        assert!(out.starts_with("cpu "));
    }

    #[test]
    fn height_is_clamped() {
        assert_eq!(Sparkline::new(&[1.0]).height(0).height, 1);
        assert_eq!(Sparkline::new(&[1.0]).height(99).height, 8);
    }

    #[test]
    fn multi_row_has_one_line_per_row() {
        let out = Sparkline::new(&[1.0, 5.0, 3.0]).height(3).render();
        assert_eq!(out.lines().count(), 3);
    }

    #[test]
    fn multi_row_with_label_keeps_label_on_its_own_line() {
        let out = Sparkline::new(&[1.0, 5.0, 3.0])
            .height(3)
            .label("mem")
            .render();
        assert_eq!(out.lines().count(), 4);
        assert_eq!(out.lines().next(), Some("mem"));
    }

    #[test]
    fn zero_height_keeps_only_the_label() {
        let mut line = Sparkline::new(&[1.0, 2.0, 3.0]).height(1);
        line.height = 0;
        assert_eq!(line.render(), "");

        let mut labelled = Sparkline::new(&[1.0, 2.0, 3.0]).label("mem");
        labelled.height = 0;
        assert_eq!(labelled.render(), "mem\n");
    }

    #[test]
    fn a_single_point_renders_one_glyph() {
        assert_eq!(Sparkline::new(&[7.0]).render().chars().count(), 1);
    }

    #[test]
    fn resample_averages_buckets() {
        assert_eq!(resample(&[1.0, 2.0, 3.0, 4.0], 2), vec![1.5, 3.5]);
        assert_eq!(resample(&[1.0, 2.0, 3.0], 3), vec![1.0, 2.0, 3.0]);
        assert_eq!(resample(&[2.0, 4.0], 1), vec![3.0]);
    }

    #[test]
    fn resample_of_empty_data_is_empty() {
        assert_eq!(resample(&[], 10), Vec::<f64>::new());
    }

    #[test]
    fn resample_of_zero_width_is_empty() {
        assert_eq!(resample(&[1.0, 2.0], 0), Vec::<f64>::new());
    }
}
