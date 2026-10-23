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
    /// The slice is borrowed from the bar, so it cannot change its state. The
    /// Go original returns a copy for the same guarantee.
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

    /// Appends tabs to the end of the bar.
    ///
    /// Returns the index of the first tab added, or `None` when `items` is
    /// empty (the Go original returns `-1`).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// let mut tabs = Tabs::new(&["Home"]);
    /// assert_eq!(tabs.add(&["Settings", "About"]), Some(1));
    /// assert_eq!(tabs.items().len(), 3);
    /// assert_eq!(tabs.add(&[]), None);
    /// ```
    pub fn add(&mut self, items: &[&str]) -> Option<usize> {
        if items.is_empty() {
            return None;
        }
        let first = self.items.len();
        self.items
            .extend(items.iter().map(|item| (*item).to_string()));
        Some(first)
    }

    /// Removes the tab at `index` and returns its label.
    ///
    /// Returns an empty string when `index` is out of range, as in Go. The
    /// active tab follows the removal: removing a tab before the active one
    /// keeps the same tab active, and removing the active tab selects the
    /// next one (or the last one, when it was the final tab).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Tabs;
    ///
    /// let mut tabs = Tabs::new(&["Home", "Settings", "About"]);
    /// tabs.set_active(2);
    /// assert_eq!(tabs.remove(0), "Home");
    /// assert_eq!(tabs.active(), 1);
    /// assert_eq!(tabs.items()[tabs.active()], "About");
    /// assert_eq!(tabs.remove(9), "");
    /// ```
    pub fn remove(&mut self, index: usize) -> String {
        if index >= self.items.len() {
            return String::new();
        }
        let removed = self.items.remove(index);

        if index < self.active {
            self.active -= 1;
        } else if self.active >= self.items.len() && !self.items.is_empty() {
            self.active = self.items.len() - 1;
        }

        removed
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
