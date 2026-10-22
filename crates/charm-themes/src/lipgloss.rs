//! Pre-built [`lipgloss::Style`] values derived from a [`Theme`].

use crate::theme::Theme;

/// A set of [`lipgloss::Style`] values derived from a [`Theme`].
///
/// The styles cover the most common terminal UI needs: base text, emphasis,
/// semantic status messages and a few component styles (title, subtitle,
/// highlight, border and code).
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let styles = themes::gruvbox().lipgloss();
/// println!("{}", styles.title.render("Gruvbox"));
/// println!("{}", styles.border.render("Boxed content"));
/// ```
#[derive(Debug, Clone)]
pub struct LipglossStyles {
    /// Base text style, painted with the theme foreground color.
    pub base: lipgloss::Style,

    /// Bold text, painted with the theme foreground color.
    pub bold: lipgloss::Style,

    /// Italic text, painted with the theme foreground color.
    pub italic: lipgloss::Style,

    /// De-emphasized text, painted with the theme muted color.
    pub faint: lipgloss::Style,

    /// Success message style.
    pub success: lipgloss::Style,

    /// Warning message style.
    pub warning: lipgloss::Style,

    /// Error message style.
    pub error: lipgloss::Style,

    /// Informational message style.
    pub info: lipgloss::Style,

    /// Title style: primary color, bold.
    pub title: lipgloss::Style,

    /// Subtitle style: secondary color.
    pub subtitle: lipgloss::Style,

    /// Highlight style: accent color, bold.
    pub highlight: lipgloss::Style,

    /// Boxed content style: rounded border in the muted color with padding.
    pub border: lipgloss::Style,

    /// Inline code style: syntax string color over the theme background.
    pub code: lipgloss::Style,
}

impl LipglossStyles {
    /// Builds every style from the colors of `theme`.
    pub(crate) fn from_theme(theme: &Theme) -> Self {
        Self {
            base: lipgloss::Style::new().foreground(theme.foreground.to_lipgloss()),
            bold: lipgloss::Style::new()
                .foreground(theme.foreground.to_lipgloss())
                .bold(true),
            italic: lipgloss::Style::new()
                .foreground(theme.foreground.to_lipgloss())
                .italic(true),
            faint: lipgloss::Style::new().foreground(theme.muted.to_lipgloss()),

            success: lipgloss::Style::new().foreground(theme.success.to_lipgloss()),
            warning: lipgloss::Style::new().foreground(theme.warning.to_lipgloss()),
            error: lipgloss::Style::new().foreground(theme.error.to_lipgloss()),
            info: lipgloss::Style::new().foreground(theme.info.to_lipgloss()),

            title: lipgloss::Style::new()
                .foreground(theme.primary.to_lipgloss())
                .bold(true),
            subtitle: lipgloss::Style::new().foreground(theme.secondary.to_lipgloss()),
            highlight: lipgloss::Style::new()
                .foreground(theme.accent.to_lipgloss())
                .bold(true),
            border: lipgloss::Style::new()
                .border(lipgloss::rounded_border())
                .border_foreground(theme.muted.to_lipgloss())
                .padding(1, 2, 1, 2),
            code: lipgloss::Style::new()
                .foreground(theme.syntax.string.to_lipgloss())
                .background(theme.background.to_lipgloss()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::themes;

    #[test]
    fn base_styles_use_the_expected_colors() {
        let theme = themes::dracula();
        let styles = theme.lipgloss();

        assert_eq!(
            styles.base.get_foreground(),
            Some(theme.foreground.to_lipgloss())
        );
        assert_eq!(
            styles.bold.get_foreground(),
            Some(theme.foreground.to_lipgloss())
        );
        assert!(styles.bold.get_bold());
        assert_eq!(
            styles.italic.get_foreground(),
            Some(theme.foreground.to_lipgloss())
        );
        assert!(styles.italic.get_italic());
        assert_eq!(
            styles.faint.get_foreground(),
            Some(theme.muted.to_lipgloss())
        );
    }

    #[test]
    fn semantic_styles_use_the_expected_colors() {
        let theme = themes::tokyo_night();
        let styles = theme.lipgloss();

        assert_eq!(
            styles.success.get_foreground(),
            Some(theme.success.to_lipgloss())
        );
        assert_eq!(
            styles.warning.get_foreground(),
            Some(theme.warning.to_lipgloss())
        );
        assert_eq!(
            styles.error.get_foreground(),
            Some(theme.error.to_lipgloss())
        );
        assert_eq!(styles.info.get_foreground(), Some(theme.info.to_lipgloss()));
    }

    #[test]
    fn component_styles_use_the_expected_colors() {
        let theme = themes::nord();
        let styles = theme.lipgloss();

        assert_eq!(
            styles.title.get_foreground(),
            Some(theme.primary.to_lipgloss())
        );
        assert!(styles.title.get_bold());
        assert_eq!(
            styles.subtitle.get_foreground(),
            Some(theme.secondary.to_lipgloss())
        );
        assert_eq!(
            styles.highlight.get_foreground(),
            Some(theme.accent.to_lipgloss())
        );
        assert!(styles.highlight.get_bold());
        assert_eq!(
            styles.code.get_foreground(),
            Some(theme.syntax.string.to_lipgloss())
        );
        assert_eq!(
            styles.code.get_background(),
            Some(theme.background.to_lipgloss())
        );
    }

    #[test]
    fn border_style_uses_a_rounded_border_with_padding() {
        let theme = themes::gruvbox();
        let styles = theme.lipgloss();

        assert_eq!(styles.border.get_border_style(), lipgloss::rounded_border());
        assert_eq!(styles.border.get_padding(), (1, 2, 1, 2));
        assert!(styles.border.get_border_top());
        assert!(styles.border.get_border_right());
        assert!(styles.border.get_border_bottom());
        assert!(styles.border.get_border_left());
    }

    #[test]
    fn border_renders_a_box() {
        let theme = themes::gruvbox();
        let styles = theme.lipgloss();
        let rendered = styles.border.render("Boxed content");
        let plain = charm_core::strip_ansi(&rendered);

        assert!(plain.starts_with('╭'), "unexpected border: {plain}");
        assert!(plain.ends_with('╯'), "unexpected border: {plain}");
        assert!(plain.contains("Boxed content"));
    }
}
