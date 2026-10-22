//! Trail of navigation items with a push/pop stack.

use lipgloss::{Color, Style};

/// Default separator inserted between two consecutive breadcrumb items.
pub const DEFAULT_SEPARATOR: &str = " > ";

/// Foreground colour used by the default active style.
pub const ACCENT_COLOR: &str = "#7D56F4";

/// Foreground colour used by the default inactive style.
pub const MUTED_COLOR: &str = "#666666";

/// A trail of navigation items.
///
/// The last item — the current location — is rendered with
/// [`active_style`](Breadcrumb::active_style) and every other item with
/// [`inactive_style`](Breadcrumb::inactive_style). The items are joined by
/// the [`separator`](Breadcrumb::separator).
///
/// # Examples
///
/// ```
/// use charm_nav::Breadcrumb;
///
/// let mut breadcrumb = Breadcrumb::new(&["Home"]);
/// breadcrumb.push("Settings");
/// assert_eq!(breadcrumb.items().len(), 2);
/// assert_eq!(breadcrumb.pop(), "Settings");
/// ```
#[derive(Debug, Clone)]
pub struct Breadcrumb {
    /// Separator inserted between two consecutive items.
    pub separator: String,
    /// Style applied to the current item.
    pub active_style: Style,
    /// Style applied to the previous items.
    pub inactive_style: Style,
    /// Labels of the breadcrumb items.
    items: Vec<String>,
}

impl Breadcrumb {
    /// Creates a new breadcrumb with the given labels.
    ///
    /// The default separator is `" > "`, the active style is bold and the
    /// inactive style is dimmed.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Breadcrumb;
    ///
    /// let breadcrumb = Breadcrumb::new(&["Home", "Settings"]);
    /// assert_eq!(breadcrumb.separator, " > ");
    /// ```
    #[must_use]
    pub fn new(items: &[&str]) -> Self {
        Self {
            separator: DEFAULT_SEPARATOR.to_string(),
            active_style: Style::new()
                .bold(true)
                .foreground(Color(ACCENT_COLOR.to_string())),
            inactive_style: Style::new().foreground(Color(MUTED_COLOR.to_string())),
            items: items.iter().map(|item| (*item).to_string()).collect(),
        }
    }

    /// Appends an item to the end of the breadcrumb.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Breadcrumb;
    ///
    /// let mut breadcrumb = Breadcrumb::new(&["Home"]);
    /// breadcrumb.push("Profile");
    /// assert_eq!(breadcrumb.items().last().map(String::as_str), Some("Profile"));
    /// ```
    pub fn push(&mut self, item: impl Into<String>) {
        self.items.push(item.into());
    }

    /// Removes the last item and returns it.
    ///
    /// Returns an empty string when the breadcrumb has no items, exactly like
    /// the Go original.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Breadcrumb;
    ///
    /// let mut breadcrumb = Breadcrumb::new(&["Home", "Settings"]);
    /// assert_eq!(breadcrumb.pop(), "Settings");
    /// assert_eq!(breadcrumb.pop(), "Home");
    /// assert_eq!(breadcrumb.pop(), "");
    /// ```
    pub fn pop(&mut self) -> String {
        self.items.pop().unwrap_or_default()
    }

    /// Returns the labels of the breadcrumb items.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Breadcrumb;
    ///
    /// let breadcrumb = Breadcrumb::new(&["Home"]);
    /// assert_eq!(breadcrumb.items(), ["Home".to_string()]);
    /// ```
    #[must_use]
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Renders the styled breadcrumb.
    ///
    /// The last item is the current location and receives the active style. An
    /// empty breadcrumb renders as an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use lipgloss::utils::strip_ansi;
    /// use charm_nav::Breadcrumb;
    ///
    /// let breadcrumb = Breadcrumb::new(&["Home", "Settings"]);
    /// assert_eq!(strip_ansi(&breadcrumb.render()), "Home > Settings");
    /// ```
    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        let last = self.items.len() - 1;
        let parts: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if index == last {
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
        let breadcrumb = Breadcrumb::new(&["Home"]);
        assert_eq!(breadcrumb.separator, " > ");
    }

    #[test]
    fn empty_breadcrumb_renders_empty_string() {
        assert_eq!(Breadcrumb::new(&[]).render(), "");
    }

    #[test]
    fn push_appends_to_the_end() {
        let mut breadcrumb = Breadcrumb::new(&["Home"]);
        breadcrumb.push("Settings");
        assert_eq!(
            breadcrumb.items(),
            ["Home".to_string(), "Settings".to_string()]
        );
    }

    #[test]
    fn pop_returns_the_removed_item() {
        let mut breadcrumb = Breadcrumb::new(&["Home", "Settings"]);
        assert_eq!(breadcrumb.pop(), "Settings");
        assert_eq!(breadcrumb.pop(), "Home");
    }

    #[test]
    fn pop_on_an_empty_breadcrumb_returns_an_empty_string() {
        let mut breadcrumb = Breadcrumb::new(&[]);
        assert_eq!(breadcrumb.pop(), "");
    }

    #[test]
    fn render_joins_the_items_with_the_separator() {
        let breadcrumb = Breadcrumb::new(&["Home", "Settings", "Profile"]);
        assert_eq!(
            strip_ansi(&breadcrumb.render()),
            "Home > Settings > Profile"
        );
    }
}
