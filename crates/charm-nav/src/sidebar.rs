//! Vertical menu with a cursor and a highlighted selection.

use lipgloss::{Color, Style};

/// Default width of the sidebar container.
pub const DEFAULT_WIDTH: usize = 20;

/// Default cursor drawn in front of the selected item.
pub const DEFAULT_CURSOR: &str = "\u{25B8} ";

/// Foreground colour used by the default selected style.
pub const ACCENT_COLOR: &str = "#7D56F4";

/// Background colour used by the default selected style.
pub const ACCENT_BACKGROUND_COLOR: &str = "#1A1A2E";

/// Foreground colour used by the default unselected style.
pub const MUTED_COLOR: &str = "#666666";

/// A vertical menu with a cursor indicator and a selection highlight.
///
/// The selected item is rendered with [`selected_style`](Sidebar::selected_style)
/// and prefixed by the [`cursor`](Sidebar::cursor). The other items are
/// rendered with [`unselected_style`](Sidebar::unselected_style) and prefixed
/// by as many spaces as the cursor has bytes, so the labels stay aligned.
///
/// # Examples
///
/// ```
/// use charm_nav::Sidebar;
///
/// let mut sidebar = Sidebar::new(&["Dashboard", "Users"]);
/// assert_eq!(sidebar.selected(), 0);
///
/// sidebar.next();
/// assert_eq!(sidebar.selected(), 1);
/// ```
#[derive(Debug, Clone)]
pub struct Sidebar {
    /// Width of the sidebar container.
    pub width: usize,
    /// Prefix drawn in front of the selected item.
    pub cursor: String,
    /// Style applied to the selected item.
    pub selected_style: Style,
    /// Style applied to the unselected items.
    pub unselected_style: Style,
    /// Labels of the menu items.
    items: Vec<String>,
    /// Index of the selected item.
    selected: usize,
}

impl Sidebar {
    /// Creates a new sidebar with the given labels.
    ///
    /// The first item is selected. The default width is 20 characters, the
    /// default cursor is `"\u{25B8} "`, the selected style is bold on a dark
    /// background and the unselected style is dimmed.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// let sidebar = Sidebar::new(&["Dashboard"]);
    /// assert_eq!(sidebar.width, 20);
    /// assert_eq!(sidebar.cursor, "\u{25B8} ");
    /// ```
    #[must_use]
    pub fn new(items: &[&str]) -> Self {
        Self {
            width: DEFAULT_WIDTH,
            cursor: DEFAULT_CURSOR.to_string(),
            selected_style: Style::new()
                .bold(true)
                .foreground(Color(ACCENT_COLOR.to_string()))
                .background(Color(ACCENT_BACKGROUND_COLOR.to_string())),
            unselected_style: Style::new().foreground(Color(MUTED_COLOR.to_string())),
            items: items.iter().map(|item| (*item).to_string()).collect(),
            selected: 0,
        }
    }

    /// Sets the index of the selected item.
    ///
    /// Out of range indexes are ignored, exactly like in the Go original.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// let mut sidebar = Sidebar::new(&["a", "b"]);
    /// sidebar.set_selected(1);
    /// assert_eq!(sidebar.selected(), 1);
    ///
    /// sidebar.set_selected(42);
    /// assert_eq!(sidebar.selected(), 1);
    /// ```
    pub fn set_selected(&mut self, index: usize) {
        if index < self.items.len() {
            self.selected = index;
        }
    }

    /// Returns the index of the selected item.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// assert_eq!(Sidebar::new(&["a"]).selected(), 0);
    /// ```
    #[must_use]
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Moves the selection to the next item, wrapping around.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// let mut sidebar = Sidebar::new(&["a", "b"]);
    /// sidebar.next();
    /// sidebar.next();
    /// assert_eq!(sidebar.selected(), 0);
    /// ```
    pub fn next(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = (self.selected + 1) % self.items.len();
    }

    /// Moves the selection to the previous item, wrapping around.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// let mut sidebar = Sidebar::new(&["a", "b"]);
    /// sidebar.previous();
    /// assert_eq!(sidebar.selected(), 1);
    /// ```
    pub fn previous(&mut self) {
        if self.items.is_empty() {
            return;
        }
        self.selected = (self.selected + self.items.len() - 1) % self.items.len();
    }

    /// Returns the labels of the menu items.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// let sidebar = Sidebar::new(&["a"]);
    /// assert_eq!(sidebar.items().len(), 1);
    /// ```
    #[must_use]
    pub fn items(&self) -> &[String] {
        &self.items
    }

    /// Renders the styled sidebar.
    ///
    /// An empty sidebar renders as an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use lipgloss::utils::strip_ansi;
    /// use charm_nav::Sidebar;
    ///
    /// let sidebar = Sidebar::new(&["Dashboard"]);
    /// let rendered = strip_ansi(&sidebar.render());
    /// assert!(rendered.contains("\u{25B8} Dashboard"));
    /// ```
    pub fn render(&self) -> String {
        if self.items.is_empty() {
            return String::new();
        }

        let container_style = Style::new()
            .width(to_i32(self.width))
            .padding_left(1)
            .padding_right(1);

        // The Go original pads the unselected items with `len(Cursor)` spaces,
        // that is, with the number of *bytes* of the cursor.
        let padding = " ".repeat(self.cursor.len());

        let lines: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                if index == self.selected {
                    self.selected_style
                        .render(&format!("{}{}", self.cursor, item))
                } else {
                    self.unselected_style.render(&format!("{padding}{item}"))
                }
            })
            .collect();

        container_style.render(&lines.join("\n"))
    }
}

/// Converts an `usize` dimension into the `i32` expected by `lipgloss`.
///
/// Values above `i32::MAX` are clamped instead of overflowing.
fn to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use lipgloss::utils::strip_ansi;

    #[test]
    fn new_uses_the_go_defaults() {
        let sidebar = Sidebar::new(&["a"]);
        assert_eq!(sidebar.width, 20);
        assert_eq!(sidebar.cursor, "\u{25B8} ");
        assert_eq!(sidebar.selected(), 0);
    }

    #[test]
    fn empty_sidebar_renders_empty_string() {
        assert_eq!(Sidebar::new(&[]).render(), "");
    }

    #[test]
    fn set_selected_ignores_out_of_range_indexes() {
        let mut sidebar = Sidebar::new(&["a", "b"]);
        sidebar.set_selected(9);
        assert_eq!(sidebar.selected(), 0);
        sidebar.set_selected(1);
        assert_eq!(sidebar.selected(), 1);
    }

    #[test]
    fn next_wraps_around() {
        let mut sidebar = Sidebar::new(&["a", "b"]);
        sidebar.next();
        sidebar.next();
        assert_eq!(sidebar.selected(), 0);
    }

    #[test]
    fn previous_wraps_around() {
        let mut sidebar = Sidebar::new(&["a", "b"]);
        sidebar.previous();
        assert_eq!(sidebar.selected(), 1);
    }

    #[test]
    fn next_and_previous_on_empty_sidebar_do_nothing() {
        let mut sidebar = Sidebar::new(&[]);
        sidebar.next();
        sidebar.previous();
        assert_eq!(sidebar.selected(), 0);
    }

    #[test]
    fn render_marks_the_selected_item_with_the_cursor() {
        let mut sidebar = Sidebar::new(&["Dashboard", "Users"]);
        sidebar.set_selected(1);
        let rendered = strip_ansi(&sidebar.render());
        let lines: Vec<&str> = rendered.lines().collect();
        assert!(lines[0].contains("  Dashboard"));
        assert!(lines[1].contains("\u{25B8} Users"));
    }

    #[test]
    fn unselected_items_are_padded_with_the_cursor_width() {
        let sidebar = Sidebar::new(&["Dashboard", "Users"]);
        let rendered = strip_ansi(&sidebar.render());
        let lines: Vec<&str> = rendered.lines().collect();
        // The cursor occupies 4 bytes, so the unselected item is shifted by 4
        // columns, keeping the labels aligned.
        assert!(lines[1].contains("    Users"));
    }

    #[test]
    fn items_returns_the_labels() {
        let sidebar = Sidebar::new(&["a", "b"]);
        assert_eq!(sidebar.items(), ["a".to_string(), "b".to_string()]);
    }
}
