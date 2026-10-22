//! Horizontal progress bar with an optional label and percentage.

use std::fmt::Write as _;

use lipgloss::Style;

use crate::{to_i64, EMPTY_GLYPH, FILL_GLYPH};

/// A horizontal progress bar.
///
/// The value is a ratio between `0.0` and `1.0`; the constructor clamps it to
/// that range, exactly like the Go implementation.
///
/// # Examples
///
/// ```
/// use charm_charts::ProgressBar;
///
/// let bar = ProgressBar::new(0.5).render();
/// assert!(bar.ends_with("  50%"));
/// ```
#[derive(Debug, Clone)]
pub struct ProgressBar {
    /// The progress ratio, between `0.0` and `1.0`.
    pub value: f64,
    /// The width of the bar, in characters.
    pub width: usize,
    /// The character used for the filled portion.
    pub fill_char: char,
    /// The character used for the empty portion.
    pub empty_char: char,
    /// The style applied to the filled portion.
    pub fill_style: Style,
    /// The style applied to the empty portion.
    pub empty_style: Style,
    /// The label rendered before the bar.
    pub label: String,
    /// Whether the percentage is displayed after the bar.
    pub show_pct: bool,
}

impl ProgressBar {
    /// Creates a new progress bar for the given value.
    ///
    /// The value is clamped to the `0.0..=1.0` range. The default bar is `40`
    /// characters wide and shows the percentage.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    ///
    /// let bar = ProgressBar::new(2.0);
    /// assert_eq!(bar.value, 1.0);
    /// assert_eq!(bar.width, 40);
    /// ```
    pub fn new(value: f64) -> Self {
        let value = value.clamp(0.0, 1.0);
        Self {
            value,
            width: 40,
            fill_char: FILL_GLYPH,
            empty_char: EMPTY_GLYPH,
            fill_style: Style::new().foreground(lipgloss::Color("212".to_string())),
            empty_style: Style::new().foreground(lipgloss::Color("240".to_string())),
            label: String::new(),
            show_pct: true,
        }
    }

    /// Sets the width of the bar, in characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    ///
    /// let bar = ProgressBar::new(0.5).width(10);
    /// assert_eq!(bar.width, 10);
    /// ```
    #[must_use]
    pub fn width(mut self, w: usize) -> Self {
        self.width = w;
        self
    }

    /// Sets the style applied to the filled portion.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    /// use lipgloss::Color;
    ///
    /// let bar = ProgressBar::new(0.5)
    ///     .fill_style(lipgloss::Style::new().foreground(Color("9".to_string())));
    /// assert!(!bar.render().is_empty());
    /// ```
    #[must_use]
    pub fn fill_style(mut self, s: Style) -> Self {
        self.fill_style = s;
        self
    }

    /// Sets the style applied to the empty portion.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    /// use lipgloss::Color;
    ///
    /// let bar = ProgressBar::new(0.5)
    ///     .empty_style(lipgloss::Style::new().foreground(Color("8".to_string())));
    /// assert!(!bar.render().is_empty());
    /// ```
    #[must_use]
    pub fn empty_style(mut self, s: Style) -> Self {
        self.empty_style = s;
        self
    }

    /// Sets the label rendered before the bar.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    ///
    /// let bar = ProgressBar::new(0.5).label("build");
    /// assert!(bar.render().starts_with("build "));
    /// ```
    #[must_use]
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Enables or disables the percentage displayed after the bar.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    ///
    /// let bar = ProgressBar::new(0.5).show_percentage(false);
    /// assert!(!bar.render().contains('%'));
    /// ```
    #[must_use]
    pub fn show_percentage(mut self, show: bool) -> Self {
        self.show_pct = show;
        self
    }

    /// Sets the character used for the filled portion.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    ///
    /// let bar = ProgressBar::new(0.5).fill_char('#');
    /// assert!(bar.render().contains('#'));
    /// ```
    #[must_use]
    pub fn fill_char(mut self, c: char) -> Self {
        self.fill_char = c;
        self
    }

    /// Sets the character used for the empty portion.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    ///
    /// let bar = ProgressBar::new(0.5).empty_char('.');
    /// assert!(bar.render().contains('.'));
    /// ```
    #[must_use]
    pub fn empty_char(mut self, c: char) -> Self {
        self.empty_char = c;
        self
    }

    /// Renders the progress bar as a styled string.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_charts::ProgressBar;
    ///
    /// assert!(ProgressBar::new(0.0).render().ends_with("   0%"));
    /// ```
    pub fn render(&self) -> String {
        let width = to_i64(self.width);
        let mut filled = (self.value * self.width as f64) as i64;
        if filled > width {
            filled = width;
        }
        if filled < 0 {
            // The Go original panics here (`strings.Repeat` with a negative
            // count); this port clamps instead.
            filled = 0;
        }
        let empty = width - filled;

        let mut result = String::new();
        if !self.label.is_empty() {
            result.push_str(&self.label);
            result.push(' ');
        }
        result.push_str(
            &self
                .fill_style
                .render(&self.fill_char.to_string().repeat(filled as usize)),
        );
        result.push_str(
            &self
                .empty_style
                .render(&self.empty_char.to_string().repeat(empty as usize)),
        );
        if self.show_pct {
            let _ = write!(result, " {:3.0}%", self.value * 100.0);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_is_clamped() {
        assert_eq!(ProgressBar::new(-1.0).value, 0.0);
        assert_eq!(ProgressBar::new(2.0).value, 1.0);
    }

    #[test]
    fn zero_width_renders_only_the_percentage() {
        let bar = ProgressBar::new(0.5).width(0).render();
        assert_eq!(bar, "  50%");
    }

    #[test]
    fn percentage_can_be_hidden() {
        let bar = ProgressBar::new(0.5).show_percentage(false).render();
        assert!(!charm_core::strip_ansi(&bar).contains('%'));
    }

    #[test]
    fn label_is_prefixed_with_a_space() {
        let bar = ProgressBar::new(0.5).label("build").render();
        assert!(bar.starts_with("build "));
    }

    #[test]
    fn custom_characters_are_used() {
        let bar = ProgressBar::new(0.5)
            .width(4)
            .fill_char('#')
            .empty_char('.')
            .render();
        let bar = charm_core::strip_ansi(&bar);
        assert!(bar.starts_with("##.."));
    }

    #[test]
    fn filled_portion_matches_the_ratio() {
        let bar = ProgressBar::new(0.25)
            .width(8)
            .show_percentage(false)
            .render();
        let bar = charm_core::strip_ansi(&bar);
        assert_eq!(bar.chars().filter(|c| *c == FILL_GLYPH).count(), 2);
        assert_eq!(bar.chars().filter(|c| *c == EMPTY_GLYPH).count(), 6);
    }

    #[test]
    fn negative_value_assigned_directly_is_clamped() {
        let mut bar = ProgressBar::new(0.5);
        bar.value = -0.5;
        let rendered = charm_core::strip_ansi(&bar.render());
        assert_eq!(rendered.chars().filter(|c| *c == FILL_GLYPH).count(), 0);
    }
}
