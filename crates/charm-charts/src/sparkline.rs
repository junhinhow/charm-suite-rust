//! Compact line chart rendered with Unicode block characters.

use lipgloss::Style;

use crate::{display_width, go_max, go_min, BLOCKS};

const MIN_WIDTH: usize = 1;
const MAX_WIDTH: usize = 500;
const MIN_HEIGHT: usize = 1;
const MAX_HEIGHT: usize = 8;

/// A compact line chart drawn with Unicode block characters.
///
/// The data is automatically resampled to fit the requested width and scaled
/// between the smallest and the largest value of the resampled series. Each
/// row of a multi-row chart adds eight sub-levels of resolution.
///
/// # Examples
///
/// ```
/// use charm_charts::Sparkline;
///
/// let line = Sparkline::new(&[1.0, 5.0, 3.0, 8.0]).render();
/// assert_eq!(line.chars().count(), 40);
/// ```
#[derive(Debug, Clone)]
pub struct Sparkline {
    /// The raw data points to plot.
    pub data: Vec<f64>,
    /// The display width, in characters (`1..=500`).
    pub width: usize,
    /// The display height, in rows (`1..=8`).
    pub height: usize,
    /// The style applied to the rendered rows.
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
    /// The width is clamped to the `1..=500` range, exactly like the Go
    /// implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::Sparkline;
    ///
    /// let line = Sparkline::new(&[1.0, 2.0, 3.0]).width(10);
    /// assert_eq!(line.width, 10);
    /// assert_eq!(Sparkline::new(&[1.0]).width(0).width, 1);
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w.clamp(MIN_WIDTH, MAX_WIDTH);
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
        self.height = h.clamp(MIN_HEIGHT, MAX_HEIGHT);
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
    /// The label is always printed on the first row; the remaining rows are
    /// indented so the columns stay aligned.
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
    /// Returns an empty string when there is no data to plot.
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

        let width = self.width.clamp(MIN_WIDTH, MAX_WIDTH);
        let height = self.height.clamp(MIN_HEIGHT, MAX_HEIGHT);
        let resampled = resample(&self.data, width);

        let Some(&first) = resampled.first() else {
            return String::new();
        };
        let mut min_val = first;
        let mut max_val = first;
        for v in &resampled {
            min_val = go_min(min_val, *v);
            max_val = go_max(max_val, *v);
        }

        let mut span = max_val - min_val;
        if span == 0.0 {
            span = 1.0;
        }

        // Each row of the chart represents `BLOCKS.len() - 1` sub-levels.
        let steps = BLOCKS.len() - 1;
        let levels = height * steps;
        let mut rows: Vec<String> = vec![String::new(); height];
        for v in &resampled {
            let normalized = (v - min_val) / span;
            let cell_level = (normalized * levels as f64) as usize;

            for (row, out) in rows.iter_mut().enumerate() {
                let row_from_bottom = height - 1 - row;
                let base = row_from_bottom * steps;
                if cell_level <= base {
                    out.push(BLOCKS[0]);
                } else {
                    let local_level = cell_level - base;
                    out.push(BLOCKS[local_level.min(steps)]);
                }
            }
        }

        let prefix = if self.label.is_empty() {
            String::new()
        } else {
            format!("{} ", self.label)
        };
        let indent = " ".repeat(display_width(&prefix));
        let lines: Vec<String> = rows
            .iter()
            .enumerate()
            .map(|(i, row)| {
                if i == 0 {
                    format!("{prefix}{row}")
                } else {
                    format!("{indent}{row}")
                }
            })
            .collect();

        self.style.render(&lines.join("\n"))
    }
}

/// Reduces or expands the data to fit the target width.
///
/// This is a direct port of the Go `resample` helper: when there are more
/// points than columns, each bucket holds the mean of the values it covers;
/// when there are fewer points, the series is expanded by linear
/// interpolation between the original points.
///
/// * `data` is empty or `width` is `0`: returns an empty vector.
pub(crate) fn resample(data: &[f64], width: usize) -> Vec<f64> {
    if data.is_empty() || width == 0 {
        return Vec::new();
    }
    if data.len() == width {
        return data.to_vec();
    }
    if data.len() > width {
        return reduce(data, width);
    }
    expand(data, width)
}

/// Averages each bucket of `data` into one of `width` points.
fn reduce(data: &[f64], width: usize) -> Vec<f64> {
    let ratio = data.len() as f64 / width as f64;
    (0..width)
        .map(|i| {
            let start = (i as f64 * ratio) as usize;
            let mut end = ((i + 1) as f64 * ratio) as usize;
            if end > data.len() {
                end = data.len();
            }
            if end <= start {
                end = start + 1;
            }
            let sum = data[start..end].iter().fold(0.0, |acc, v| acc + v);
            sum / (end - start) as f64
        })
        .collect()
}

/// Spreads `data` over `width` points with linear interpolation.
fn expand(data: &[f64], width: usize) -> Vec<f64> {
    if data.len() == 1 {
        return vec![data[0]; width];
    }
    if width == 1 {
        return vec![data[0]];
    }

    let last = data.len() - 1;
    let step = last as f64 / (width - 1) as f64;
    (0..width)
        .map(|i| {
            let pos = i as f64 * step;
            let low = (pos.floor() as usize).min(last);
            let high = (pos.ceil() as usize).min(last);
            let frac = pos - low as f64;
            data[low] * (1.0 - frac) + data[high] * frac
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &str) -> String {
        charm_core::strip_ansi(text)
    }

    #[test]
    fn empty_data_renders_nothing() {
        assert_eq!(Sparkline::new(&[]).render(), "");
    }

    #[test]
    fn zero_width_is_clamped_to_one_column() {
        let out = Sparkline::new(&[1.0, 2.0, 3.0]).width(0).render();
        assert_eq!(display_width(&out), 1);
    }

    #[test]
    fn output_fills_the_requested_width() {
        for width in [1, 5, 10, 30] {
            let out = Sparkline::new(&[1.0, 2.0, 3.0]).width(width).render();
            assert_eq!(display_width(&out), width, "width {width}");
        }
    }

    #[test]
    fn flat_series_renders_spaces() {
        let out = Sparkline::new(&[4.0, 4.0, 4.0]).width(3).render();
        assert_eq!(out, "   ");
    }

    #[test]
    fn height_is_clamped() {
        assert_eq!(Sparkline::new(&[1.0]).height(0).height, 1);
        assert_eq!(Sparkline::new(&[1.0]).height(99).height, 8);
    }

    #[test]
    fn rendered_height_follows_the_clamped_value() {
        for (height, rows) in [(0, 1), (1, 1), (3, 3), (8, 8), (100, 8)] {
            let out = Sparkline::new(&[1.0, 2.0, 3.0]).height(height).render();
            assert_eq!(out.lines().count(), rows, "height {height}");
        }
    }

    #[test]
    fn multi_row_has_one_line_per_row() {
        let out = Sparkline::new(&[1.0, 5.0, 3.0]).height(3).render();
        assert_eq!(out.lines().count(), 3);
    }

    #[test]
    fn label_stays_on_the_first_row_and_rows_stay_aligned() {
        for height in [1, 2, 5] {
            let out = plain(
                &Sparkline::new(&[1.0, 5.0, 3.0])
                    .width(10)
                    .height(height)
                    .label("CPU")
                    .render(),
            );
            let lines: Vec<&str> = out.split('\n').collect();
            assert_eq!(lines.len(), height);
            assert!(lines[0].starts_with("CPU "));
            for line in &lines[1..] {
                assert!(line.starts_with("    "), "{line:?}");
            }
            for line in &lines {
                assert_eq!(display_width(line), 14, "{line:?}");
            }
        }
    }

    #[test]
    fn a_single_point_renders_one_glyph_per_column() {
        assert_eq!(Sparkline::new(&[7.0]).width(4).render().chars().count(), 4);
    }

    #[test]
    fn resample_averages_buckets() {
        assert_eq!(resample(&[1.0, 2.0, 3.0, 4.0], 2), vec![1.5, 3.5]);
        assert_eq!(resample(&[1.0, 2.0, 3.0], 3), vec![1.0, 2.0, 3.0]);
        assert_eq!(resample(&[2.0, 4.0], 1), vec![3.0]);
    }

    #[test]
    fn resample_expands_with_linear_interpolation() {
        let expanded = resample(&[1.0, 2.0, 3.0, 4.0], 7);
        assert_eq!(expanded.len(), 7);
        assert_eq!(expanded[0], 1.0);
        assert_eq!(expanded[6], 4.0);
        assert!((expanded[1] - 1.5).abs() < 1e-12);
    }

    #[test]
    fn resample_of_a_single_point_repeats_it() {
        assert_eq!(resample(&[7.0], 4), vec![7.0; 4]);
    }

    #[test]
    fn resample_of_empty_data_or_zero_width_is_empty() {
        assert_eq!(resample(&[], 10), Vec::<f64>::new());
        assert_eq!(resample(&[1.0, 2.0], 0), Vec::<f64>::new());
    }
}
