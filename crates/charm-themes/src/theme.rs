//! Theme and syntax-highlighting color definitions.

use crate::lipgloss::LipglossStyles;
use charm_core::Rgb;

/// A complete color palette that can be applied across terminal UI code.
///
/// A theme assigns a color to every semantic role used by the charm
/// ecosystem: base background/foreground, accents for headings and emphasis,
/// status colors for feedback messages, and a muted tone for disabled or
/// secondary text.
///
/// # Examples
///
/// ```
/// use charm_themes::Theme;
/// use charm_themes::themes;
///
/// let theme = themes::gruvbox();
/// assert_eq!(theme.name, "gruvbox");
/// assert_eq!(theme.background.to_hex(), "#282828");
/// assert_eq!(theme.primary.to_hex(), "#fabd2f");
///
/// // Themes can be compared and cloned.
/// assert_eq!(theme, themes::gruvbox());
/// assert_ne!(theme, themes::nord());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Theme {
    /// Human readable identifier of the theme (e.g. `"gruvbox"`).
    pub name: String,

    /// Main background color of the application.
    pub background: Rgb,

    /// Default text color.
    pub foreground: Rgb,

    /// Primary accent, used for headings and titles.
    pub primary: Rgb,

    /// Secondary accent, used for subtitles and secondary information.
    pub secondary: Rgb,

    /// Accent used for highlights and emphasis.
    pub accent: Rgb,

    /// Color used for positive/confirmations.
    pub success: Rgb,

    /// Color used for cautionary messages.
    pub warning: Rgb,

    /// Color used for failures and errors.
    pub error: Rgb,

    /// Color used for informational messages.
    pub info: Rgb,

    /// Color used for disabled, dimmed or de-emphasized content.
    pub muted: Rgb,

    /// Colors used for syntax highlighting of code blocks.
    pub syntax: SyntaxColors,
}

/// Colors used for syntax highlighting of code blocks.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let syntax = themes::dracula().syntax;
/// assert_eq!(syntax.keyword.to_hex(), "#ff79c6");
/// assert_eq!(syntax.string.to_hex(), "#f1fa8c");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SyntaxColors {
    /// Color of language keywords.
    pub keyword: Rgb,

    /// Color of string literals.
    pub string: Rgb,

    /// Color of numeric literals.
    pub number: Rgb,

    /// Color of comments.
    pub comment: Rgb,

    /// Color of function names.
    pub function: Rgb,

    /// Color of type names.
    pub r#type: Rgb,

    /// Color of operators.
    pub operator: Rgb,

    /// Color of constants.
    pub constant: Rgb,
}

impl Theme {
    /// Derives a set of [`lipgloss::Style`] values from this theme.
    ///
    /// This mirrors the `Theme.Lipgloss()` method of the original Go
    /// implementation: every style is created from scratch, with the theme
    /// colors applied to the matching semantic role.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_themes::themes;
    ///
    /// let styles = themes::nord().lipgloss();
    /// let rendered = styles.title.render("Nord");
    /// assert!(rendered.contains("Nord"));
    /// ```
    pub fn lipgloss(&self) -> LipglossStyles {
        LipglossStyles::from_theme(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::themes;

    #[test]
    fn theme_fields_are_public_and_readable() {
        let theme = themes::catppuccin();
        assert_eq!(theme.name, "catppuccin");
        assert_eq!(theme.background.to_hex(), "#1e1e2e");
        assert_eq!(theme.foreground.to_hex(), "#cdd6f4");
        assert_eq!(theme.primary.to_hex(), "#cba6f7");
        assert_eq!(theme.secondary.to_hex(), "#a6e3a1");
        assert_eq!(theme.accent.to_hex(), "#f5c2e7");
        assert_eq!(theme.success.to_hex(), "#a6e3a1");
        assert_eq!(theme.warning.to_hex(), "#f9e2af");
        assert_eq!(theme.error.to_hex(), "#f38ba8");
        assert_eq!(theme.info.to_hex(), "#89b4fa");
        assert_eq!(theme.muted.to_hex(), "#585b70");
    }

    #[test]
    fn syntax_fields_are_public_and_readable() {
        let syntax = themes::catppuccin().syntax;
        assert_eq!(syntax.keyword.to_hex(), "#cba6f7");
        assert_eq!(syntax.string.to_hex(), "#a6e3a1");
        assert_eq!(syntax.number.to_hex(), "#fab387");
        assert_eq!(syntax.comment.to_hex(), "#585b70");
        assert_eq!(syntax.function.to_hex(), "#89b4fa");
        assert_eq!(syntax.r#type.to_hex(), "#f9e2af");
        assert_eq!(syntax.operator.to_hex(), "#89dceb");
        assert_eq!(syntax.constant.to_hex(), "#fab387");
    }

    #[test]
    fn theme_is_cloneable_and_comparable() {
        let a = themes::dracula();
        let b = a.clone();
        assert_eq!(a, b);
        assert_ne!(a, themes::gruvbox());
        assert!(format!("{a:?}").contains("dracula"));
    }

    #[test]
    fn lipgloss_styles_are_derived_from_the_theme() {
        let theme = themes::nord();
        let styles = theme.lipgloss();
        assert_eq!(
            styles.base.get_foreground(),
            Some(theme.foreground.to_lipgloss())
        );
        assert_eq!(
            styles.title.get_foreground(),
            Some(theme.primary.to_lipgloss())
        );
    }
}
