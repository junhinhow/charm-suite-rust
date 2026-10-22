//! Visual styles used when rendering a diff.
//!
//! The styles are plain [`lipgloss::Style`] values, so they can be freely
//! combined with the rest of the Lip Gloss ecosystem.

use lipgloss::Style;

/// Colors and styles applied to each element of a rendered diff.
///
/// # Examples
///
/// ```
/// use charm_diff::DiffStyle;
///
/// let style = DiffStyle::github();
/// assert!(style.addition.render("+new line").contains("+new line"));
/// ```
#[derive(Clone, Debug)]
pub struct DiffStyle {
    /// Style applied to added lines.
    pub addition: Style,
    /// Style applied to removed lines.
    pub deletion: Style,
    /// Style applied to unchanged (context) lines.
    pub context: Style,
    /// Style applied to the hunk header (`@@ -a,b +c,d @@`).
    pub hunk_header: Style,
    /// Style applied to line numbers.
    pub line_number: Style,
    /// Style applied to the column separator of the side-by-side layout.
    pub separator: Style,
}

impl DiffStyle {
    /// Returns whether none of the styles applies a visible transformation.
    pub(crate) fn is_zero(&self) -> bool {
        [
            &self.addition,
            &self.deletion,
            &self.context,
            &self.hunk_header,
            &self.line_number,
            &self.separator,
        ]
        .iter()
        .all(|style| style.render("charm-diff-zero") == "charm-diff-zero")
    }

    /// Returns a GitHub-inspired style with colored foregrounds and
    /// backgrounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_diff::DiffStyle;
    ///
    /// let style = DiffStyle::github();
    /// assert!(style.addition.render("x").contains('x'));
    /// ```
    pub fn github() -> Self {
        Self {
            addition: Style::new()
                .foreground(lipgloss::Color("#22863a".to_string()))
                .background(lipgloss::Color("#e6ffec".to_string())),
            deletion: Style::new()
                .foreground(lipgloss::Color("#cb2431".to_string()))
                .background(lipgloss::Color("#ffeef0".to_string())),
            context: Style::new().foreground(lipgloss::Color("#6a737d".to_string())),
            hunk_header: Style::new()
                .foreground(lipgloss::Color("#6f42c1".to_string()))
                .background(lipgloss::Color("#f1e5ff".to_string())),
            line_number: Style::new().foreground(lipgloss::Color("#959da5".to_string())),
            separator: Style::new().foreground(lipgloss::Color("#d1d5da".to_string())),
        }
    }

    /// Returns a minimalist style that only uses basic ANSI colors.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_diff::DiffStyle;
    ///
    /// let style = DiffStyle::terminal();
    /// assert!(style.deletion.render("x").contains('x'));
    /// ```
    pub fn terminal() -> Self {
        Self {
            addition: Style::new().foreground(lipgloss::Color("2".to_string())),
            deletion: Style::new().foreground(lipgloss::Color("1".to_string())),
            context: Style::new().foreground(lipgloss::Color("8".to_string())),
            hunk_header: Style::new().foreground(lipgloss::Color("6".to_string())),
            line_number: Style::new().foreground(lipgloss::Color("8".to_string())),
            separator: Style::new().foreground(lipgloss::Color("8".to_string())),
        }
    }

    /// Returns the default style, which is the same as
    /// [`DiffStyle::terminal`].
    ///
    /// This is the equivalent of `DefaultStyle()` of the Go original. The
    /// [`Default`] trait implementation delegates to this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_diff::DiffStyle;
    ///
    /// assert_eq!(
    ///     DiffStyle::default().addition.render("x"),
    ///     DiffStyle::terminal().addition.render("x")
    /// );
    /// ```
    // The name mirrors `DefaultStyle()` of the Go original, hence the lint
    // about shadowing `Default::default`.
    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Self::terminal()
    }
}

impl Default for DiffStyle {
    fn default() -> Self {
        Self::default()
    }
}
