//! Horizontal tab bar with an active and an inactive styling.

use lipgloss::{Color, Style};

/// Default separator inserted between two consecutive tabs.
pub const DEFAULT_SEPARATOR: &str = " | ";

/// Foreground colour used by the default active style.
pub const ACCENT_COLOR: &str = "#7D56F4";

/// Foreground colour used by the default inactive style.
pub const MUTED_COLOR: &str = "#666666";

/// A horizontal bar of tabs.
///
/// The active tab is rendered with [`active_style`](Tabs::active_style) and
/// every other tab with [`inactive_style`](Tabs::inactive_style). Tabs are
/// joined by the [`separator`](Tabs::separator).
///
/// # Examples
///
/// ```
/// use charm_nav::Tabs;
///
/// let mut tabs = Tabs::new(&["Home", "Settings", "About"]);
/// assert_eq!(tabs.active(), 0);
///
/// tabs.next();
/// assert_eq!(tabs.active(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Tabs {
    /// Separator inserted between two consecutive tabs.
    pub separator: String,
    /// Style applied to the active tab.
    pub active_style: Style,
    /// Style applied to the inactive tabs.
    pub inactive_style: Style,
    /// Labels of the tabs.
    items: Vec<String>,
    /// Index of the active tab.
    active: usize,
}

impl Tabs {
    /// Creates a new tab bar with the given labels.
    ///
    /// The first tab is active. The default separator is `" | "`, the active
    /// style is bold and underlined, and the inactive style is dimmed.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// let tabs = Tabs::new(&["Home", "Settings"]);
    /// assert_eq!(tabs.items(), ["Home".to_string(), "Settings".to_string()]);
    /// assert_eq!(tabs.separator, " | ");
    /// ```
    #[must_use]
    pub fn new(items: &[&str]) -> Self {
        Self {
            separator: DEFAULT_SEPARATOR.to_string(),
            active_style: Style::new()
                .bold(true)
                .underline(true)
                .foreground(Color(ACCENT_COLOR.to_string())),
            inactive_style: Style::new().foreground(Color(MUTED_COLOR.to_string())),
            items: items.iter().map(|item| (*item).to_string()).collect(),
            active: 0,
        }
    }

    /// Sets the index of the active tab.
    ///
    /// Out of range indexes are ignored, exactly like in the Go original.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// let mut tabs = Tabs::new(&["Home", "Settings"]);
    /// tabs.set_active(1);
    /// assert_eq!(tabs.active(), 1);
    ///
    /// tabs.set_active(99);
    /// assert_eq!(tabs.active(), 1);
    /// ```
    pub fn set_active(&mut self, index: usize) {
        if index < self.items.len() {
            self.active = index;
        }
    }

    /// Returns the index of the active tab.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// assert_eq!(Tabs::new(&["Home"]).active(), 0);
    /// ```
    #[must_use]
    pub fn active(&self) -> usize {
        self.active
    }

    /// Moves the selection to the next tab, wrapping around.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// let mut tabs = Tabs::new(&["Home", "Settings"]);
    /// tabs.next();
    /// tabs.next();
    /// assert_eq!(tabs.active(), 0);
    /// ```
    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.active = (self.active + 1) % self.items.len();
    }

    /// Moves the selection to the previous tab, wrapping around.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// let mut tabs = Tabs::new(&["Home", "Settings"]);
    /// tabs.previous();
    /// assert_eq!(tabs.active(), 1);
    /// ```
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.active = (self.active + self.items.len() - 1) % self.items.len();
    }

    /// Returns the labels of the tabs.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// let tabs = Tabs::new(&["Home"]);
    /// assert_eq!(tabs.items().len(), 1);
    /// ```
    #[must_use]
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Renders the styled tab bar.
    ///
    /// An empty tab bar renders as an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use lipgloss::utils::strip_ansi;
    /// use charm_nav::Tabs;
    ///
    /// let tabs = Tabs::new(&["Home", "Settings"]);
    /// assert_eq!(strip_ansi(&tabs.render()), "Home | Settings");
    /// ```
    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        let parts: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if index == self.active {
                    self.active_style.render(item)
                } else {
                    self.inactive_style.render(item)
                }
            })
            .collect();

        parts.join(&self.separator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lipgloss::utils::strip_ansi;

    #[test]
    fn new_uses_the_go_defaults() {
        let tabs = Tabs::new(&["Home"]);
        assert_eq!(tabs.separator, " | ");
        assert_eq!(tabs.active(), 0);
    }

    #[test]
    fn empty_tabs_render_empty_string() {
        assert_eq!(Tabs::new(&[]).render(), "");
    }

    #[test]
    fn set_active_ignores_out_of_range_indexes() {
        let mut tabs = Tabs::new(&["a", "b"]);
        tabs.set_active(5);
        assert_eq!(tabs.active(), 0);
        tabs.set_active(1);
        assert_eq!(tabs.active(), 1);
    }

    #[test]
    fn next_wraps_around() {
        let mut tabs = Tabs::new(&["a", "b", "c"]);
        tabs.next();
        tabs.next();
        tabs.next();
        assert_eq!(tabs.active(), 0);
    }

    #[test]
    fn previous_wraps_around() {
        let mut tabs = Tabs::new(&["a", "b", "c"]);
        tabs.previous();
        assert_eq!(tabs.active(), 2);
    }

    #[test]
    fn next_and_previous_on_empty_tabs_do_nothing() {
        let mut tabs = Tabs::new(&[]);
        tabs.next();
        tabs.previous();
        assert_eq!(tabs.active(), 0);
    }

    #[test]
    fn render_joins_the_tabs_with_the_separator() {
        let tabs = Tabs::new(&["Home", "Settings", "About"]);
        assert_eq!(strip_ansi(&tabs.render()), "Home | Settings | About");
    }

    #[test]
    fn items_returns_the_labels() {
        let tabs = Tabs::new(&["Home", "Settings"]);
        assert_eq!(tabs.items(), ["Home".to_string(), "Settings".to_string()]);
    }
}
