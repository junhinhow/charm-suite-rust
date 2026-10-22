//! Theme and syntax-highlighting color definitions.

use crate::bubbles::BubblesStyles;
use crate::glamour::StyleConfig;
use crate::lipgloss::LipglossStyles;
use charm_core::Rgb;

/// Error returned by [`Theme::validate`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum ThemeError {
    /// The theme has no name, or only whitespace as its name.
    #[error("theme name is empty")]
    MissingName,
}

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

    /// Derives the styles of the `bubbles` components from this theme.
    ///
    /// Mirrors `Theme.Bubbles()` of the Go implementation. Each call builds
    /// new styles, so call it once and reuse the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_themes::themes;
    ///
    /// let styles = themes::nord().bubbles();
    /// let header = styles.table.header.render("ID");
    /// assert!(charm_core::strip_ansi(&header).contains("ID"));
    /// ```
    pub fn bubbles(&self) -> BubblesStyles {
        BubblesStyles::from_theme(self)
    }

    /// Derives the `glamour` markdown style configuration from this theme.
    ///
    /// Mirrors `Theme.Glamour()` of the Go implementation. The result is
    /// plain data that serializes to the same JSON layout as the Go
    /// `ansi.StyleConfig`. This crate does not render markdown itself.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_themes::themes;
    ///
    /// let config = themes::nord().glamour();
    /// assert_eq!(config.heading.primitive.color.as_deref(), Some("#88c0d0"));
    /// ```
    pub fn glamour(&self) -> StyleConfig {
        StyleConfig::from_theme(self)
    }

    /// Checks that the theme is complete enough to be used.
    ///
    /// Every color of a [`Theme`] is typed, so the only way to build an
    /// incomplete theme is to leave the name empty. The Go `Validate` also
    /// reports missing colors; in Rust that case cannot happen.
    ///
    /// # Errors
    ///
    /// Returns [`ThemeError::MissingName`] when the name is empty or blank.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_themes::{Theme, ThemeError, themes};
    ///
    /// assert!(themes::nord().validate().is_ok());
    ///
    /// let mut theme = themes::nord();
    /// theme.name = "  ".to_owned();
    /// assert_eq!(theme.validate(), Err(ThemeError::MissingName));
    /// ```
    pub fn validate(&self) -> Result<(), ThemeError> {
        if self.name.trim().is_empty() {
            return Err(ThemeError::MissingName);
        }
        Ok(())
    }
}

impl SyntaxColors {
    /// Checks the syntax palette, mirroring the Go `SyntaxColors.Validate`.
    ///
    /// Always succeeds: every syntax color is a typed [`Rgb`], so none can be
    /// missing. The method exists so that code written against the Go API
    /// keeps compiling.
    ///
    /// # Errors
    ///
    /// Never returns an error in this implementation.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_themes::themes;
    ///
    /// assert!(themes::nord().syntax.validate().is_ok());
    /// ```
    pub fn validate(&self) -> Result<(), ThemeError> {
        Ok(())
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
        // Ver `force_truecolor` em `lipgloss.rs`: sem TTY o perfil e NoColor
        // e os tokens de cor viram string vazia.
        lipgloss::renderer::set_color_profile(lipgloss::renderer::ColorProfileKind::TrueColor);

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
