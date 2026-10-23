//! Vertical menu with a cursor and a highlighted selection.

use lipgloss::{Color, Style};

/// Default width of the sidebar, in columns.
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
/// by as many spaces as the cursor occupies in columns, so the labels stay
/// aligned.
///
/// Every line is exactly [`width`](Sidebar::width) columns wide. Items that do
/// not fit are truncated, never wrapped, and the highlight covers the whole
/// line.
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
    /// Width of the sidebar, in columns. Zero is rendered as one column.
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
    /// The first item is selected. The default width is 20 columns, the
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
    /// The slice is borrowed from the menu, so it cannot change its state. The
    /// Go original returns a copy for the same guarantee.
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

    /// Appends items to the end of the menu.
    ///
    /// Returns the index of the first item added, or `None` when `items` is
    /// empty (the Go original returns `-1`).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// let mut sidebar = Sidebar::new(&["a", "b"]);
    /// assert_eq!(sidebar.add(&["c"]), Some(2));
    /// assert_eq!(sidebar.items().len(), 3);
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

    /// Removes the item at `index` and returns its label.
    ///
    /// Returns an empty string when `index` is out of range, as in Go. The
    /// selection follows the removal: removing an item before the selected one
    /// keeps the same item selected, and removing the selected item selects the
    /// next one (or the last one, when it was the final item).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_nav::Sidebar;
    ///
    /// let mut sidebar = Sidebar::new(&["a", "b", "c"]);
    /// sidebar.set_selected(2);
    /// assert_eq!(sidebar.remove(0), "a");
    /// assert_eq!(sidebar.items()[sidebar.selected()], "c");
    /// assert_eq!(sidebar.remove(9), "");
    /// ```
    pub fn remove(&mut self, index: usize) -> String {
        if index >= self.items.len() {
            return String::new();
        }
        let removed = self.items.remove(index);

        if index < self.selected {
            self.selected -= 1;
        } else if self.selected >= self.items.len() && !self.items.is_empty() {
            self.selected = self.items.len() - 1;
        }

        removed
    }

    /// Renders the styled sidebar.
    ///
    /// Each item occupies one line of exactly [`width`](Sidebar::width)
    /// columns. The cursor is measured in display columns, not bytes, and the
    /// labels are truncated to fit. An empty sidebar renders as an empty
    /// string.
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

        // One column of padding on each side of the content.
        let width = self.width.max(1);
        let avail = width.saturating_sub(2);

        // The cursor takes the same number of columns on every item, even when
        // it is multibyte (for example "▸ " is 4 bytes and 2 columns).
        let cursor_width = lipgloss::utils::width_visible(&self.cursor).min(avail);
        let text_width = avail - cursor_width;

        let selected_prefix = Style::truncate_visible_line(&self.cursor, cursor_width);
        let unselected_prefix = " ".repeat(cursor_width);

        let lines: Vec<String> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let (prefix, style) = if index == self.selected {
                    (&selected_prefix, &self.selected_style)
                } else {
                    (&unselected_prefix, &self.unselected_style)
                };

                let text = Style::truncate_visible_line(item, text_width);

                // The left padding is part of the styled text, so the highlight
                // of the selected item covers the whole line.
                let mut line = format!(" {prefix}{text}");
                let used = lipgloss::utils::width_visible(&line);
                if used < width {
                    line.push_str(&" ".repeat(width - used));
                }

                style.render(&line)
            })
            .collect();

        lines.join("\n")
    }
}
