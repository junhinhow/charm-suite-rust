//! Built-in color themes.
//!
//! This module is **generated** from the original Go implementation
//! (`reference/go/charm-themes/themes.go`) by `scripts/extract-themes.ps1`.
//! The color values are never typed by hand, so they are guaranteed to
//! match the Go original value by value.

use std::collections::BTreeMap;

use charm_core::Rgb;

use crate::theme::{SyntaxColors, Theme};

/// Returns the **Gruvbox** theme.
///
/// A warm, retro-inspired palette on a dark background.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let theme = themes::gruvbox();
/// assert_eq!(theme.primary.to_hex(), "#fabd2f");
/// ```
pub fn gruvbox() -> Theme {
    Theme {
        name: "gruvbox".to_owned(),
        background: Rgb::from_lipgloss_str("#282828"),
        foreground: Rgb::from_lipgloss_str("#ebdbb2"),
        primary: Rgb::from_lipgloss_str("#fabd2f"),
        secondary: Rgb::from_lipgloss_str("#b8bb26"),
        accent: Rgb::from_lipgloss_str("#83a598"),
        success: Rgb::from_lipgloss_str("#b8bb26"),
        warning: Rgb::from_lipgloss_str("#fe8019"),
        error: Rgb::from_lipgloss_str("#fb4934"),
        info: Rgb::from_lipgloss_str("#83a598"),
        muted: Rgb::from_lipgloss_str("#928374"),
        syntax: SyntaxColors {
            keyword: Rgb::from_lipgloss_str("#fb4934"),
            string: Rgb::from_lipgloss_str("#b8bb26"),
            number: Rgb::from_lipgloss_str("#d3869b"),
            comment: Rgb::from_lipgloss_str("#928374"),
            function: Rgb::from_lipgloss_str("#b8bb26"),
            r#type: Rgb::from_lipgloss_str("#fabd2f"),
            operator: Rgb::from_lipgloss_str("#fe8019"),
            constant: Rgb::from_lipgloss_str("#d3869b"),
        },
    }
}

/// Returns the **Dracula** theme.
///
/// A high-contrast palette built around purple and green accents.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let theme = themes::dracula();
/// assert_eq!(theme.primary.to_hex(), "#bd93f9");
/// ```
pub fn dracula() -> Theme {
    Theme {
        name: "dracula".to_owned(),
        background: Rgb::from_lipgloss_str("#282a36"),
        foreground: Rgb::from_lipgloss_str("#f8f8f2"),
        primary: Rgb::from_lipgloss_str("#bd93f9"),
        secondary: Rgb::from_lipgloss_str("#50fa7b"),
        accent: Rgb::from_lipgloss_str("#ff79c6"),
        success: Rgb::from_lipgloss_str("#50fa7b"),
        warning: Rgb::from_lipgloss_str("#ffb86c"),
        error: Rgb::from_lipgloss_str("#ff5555"),
        info: Rgb::from_lipgloss_str("#8be9fd"),
        muted: Rgb::from_lipgloss_str("#6272a4"),
        syntax: SyntaxColors {
            keyword: Rgb::from_lipgloss_str("#ff79c6"),
            string: Rgb::from_lipgloss_str("#f1fa8c"),
            number: Rgb::from_lipgloss_str("#bd93f9"),
            comment: Rgb::from_lipgloss_str("#6272a4"),
            function: Rgb::from_lipgloss_str("#50fa7b"),
            r#type: Rgb::from_lipgloss_str("#8be9fd"),
            operator: Rgb::from_lipgloss_str("#ff79c6"),
            constant: Rgb::from_lipgloss_str("#bd93f9"),
        },
    }
}

/// Returns the **Nord** theme.
///
/// A cool, arctic palette of muted blues and greens.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let theme = themes::nord();
/// assert_eq!(theme.primary.to_hex(), "#88c0d0");
/// ```
pub fn nord() -> Theme {
    Theme {
        name: "nord".to_owned(),
        background: Rgb::from_lipgloss_str("#2e3440"),
        foreground: Rgb::from_lipgloss_str("#d8dee9"),
        primary: Rgb::from_lipgloss_str("#88c0d0"),
        secondary: Rgb::from_lipgloss_str("#a3be8c"),
        accent: Rgb::from_lipgloss_str("#b48ead"),
        success: Rgb::from_lipgloss_str("#a3be8c"),
        warning: Rgb::from_lipgloss_str("#ebcb8b"),
        error: Rgb::from_lipgloss_str("#bf616a"),
        info: Rgb::from_lipgloss_str("#5e81ac"),
        muted: Rgb::from_lipgloss_str("#4c566a"),
        syntax: SyntaxColors {
            keyword: Rgb::from_lipgloss_str("#81a1c1"),
            string: Rgb::from_lipgloss_str("#a3be8c"),
            number: Rgb::from_lipgloss_str("#b48ead"),
            comment: Rgb::from_lipgloss_str("#616e88"),
            function: Rgb::from_lipgloss_str("#88c0d0"),
            r#type: Rgb::from_lipgloss_str("#8fbcbb"),
            operator: Rgb::from_lipgloss_str("#81a1c1"),
            constant: Rgb::from_lipgloss_str("#b48ead"),
        },
    }
}

/// Returns the **Catppuccin** theme.
///
/// A pastel palette of soft pinks, blues and greens.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let theme = themes::catppuccin();
/// assert_eq!(theme.primary.to_hex(), "#cba6f7");
/// ```
pub fn catppuccin() -> Theme {
    Theme {
        name: "catppuccin".to_owned(),
        background: Rgb::from_lipgloss_str("#1e1e2e"),
        foreground: Rgb::from_lipgloss_str("#cdd6f4"),
        primary: Rgb::from_lipgloss_str("#cba6f7"),
        secondary: Rgb::from_lipgloss_str("#a6e3a1"),
        accent: Rgb::from_lipgloss_str("#f5c2e7"),
        success: Rgb::from_lipgloss_str("#a6e3a1"),
        warning: Rgb::from_lipgloss_str("#f9e2af"),
        error: Rgb::from_lipgloss_str("#f38ba8"),
        info: Rgb::from_lipgloss_str("#89b4fa"),
        muted: Rgb::from_lipgloss_str("#585b70"),
        syntax: SyntaxColors {
            keyword: Rgb::from_lipgloss_str("#cba6f7"),
            string: Rgb::from_lipgloss_str("#a6e3a1"),
            number: Rgb::from_lipgloss_str("#fab387"),
            comment: Rgb::from_lipgloss_str("#585b70"),
            function: Rgb::from_lipgloss_str("#89b4fa"),
            r#type: Rgb::from_lipgloss_str("#f9e2af"),
            operator: Rgb::from_lipgloss_str("#89dceb"),
            constant: Rgb::from_lipgloss_str("#fab387"),
        },
    }
}

/// Returns the **Tokyo Night** theme.
///
/// A neon-inspired dark palette built for night-time UIs.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let theme = themes::tokyo_night();
/// assert_eq!(theme.primary.to_hex(), "#7aa2f7");
/// ```
pub fn tokyo_night() -> Theme {
    Theme {
        name: "tokyo-night".to_owned(),
        background: Rgb::from_lipgloss_str("#1a1b26"),
        foreground: Rgb::from_lipgloss_str("#a9b1d6"),
        primary: Rgb::from_lipgloss_str("#7aa2f7"),
        secondary: Rgb::from_lipgloss_str("#9ece6a"),
        accent: Rgb::from_lipgloss_str("#bb9af7"),
        success: Rgb::from_lipgloss_str("#9ece6a"),
        warning: Rgb::from_lipgloss_str("#e0af68"),
        error: Rgb::from_lipgloss_str("#f7768e"),
        info: Rgb::from_lipgloss_str("#7dcfff"),
        muted: Rgb::from_lipgloss_str("#565f89"),
        syntax: SyntaxColors {
            keyword: Rgb::from_lipgloss_str("#bb9af7"),
            string: Rgb::from_lipgloss_str("#9ece6a"),
            number: Rgb::from_lipgloss_str("#ff9e64"),
            comment: Rgb::from_lipgloss_str("#565f89"),
            function: Rgb::from_lipgloss_str("#7aa2f7"),
            r#type: Rgb::from_lipgloss_str("#2ac3de"),
            operator: Rgb::from_lipgloss_str("#89ddff"),
            constant: Rgb::from_lipgloss_str("#ff9e64"),
        },
    }
}

/// Returns all built-in themes indexed by name.
///
/// The map is keyed by the theme name as used by the Go original:
/// `"gruvbox"`, `"dracula"`, `"nord"`, `"catppuccin"` and `"tokyo-night"`.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let all = themes::themes();
/// assert_eq!(all.len(), 5);
/// assert!(all.contains_key("tokyo-night"));
/// ```
pub fn themes() -> BTreeMap<&'static str, Theme> {
    let mut map = BTreeMap::new();
    map.insert("gruvbox", gruvbox());
    map.insert("dracula", dracula());
    map.insert("nord", nord());
    map.insert("catppuccin", catppuccin());
    map.insert("tokyo-night", tokyo_night());
    map
}

/// Returns a copy of the built-in theme named `name`, or `None` when there is none.
///
/// The lookup ignores letter case and surrounding whitespace, like the Go `Get`.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// assert_eq!(themes::get("  NORD ").map(|t| t.name), Some("nord".to_owned()));
/// assert!(themes::get("missing").is_none());
/// ```
pub fn get(name: &str) -> Option<Theme> {
    match name.trim().to_lowercase().as_str() {
        "gruvbox" => Some(gruvbox()),
        "dracula" => Some(dracula()),
        "nord" => Some(nord()),
        "catppuccin" => Some(catppuccin()),
        "tokyo-night" => Some(tokyo_night()),
        _ => None,
    }
}

/// Returns the names of the built-in themes in alphabetical order.
///
/// Unlike the keys of [`themes`], the order is deterministic by contract.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// assert_eq!(themes::theme_names().first(), Some(&"catppuccin"));
/// ```
pub fn theme_names() -> Vec<&'static str> {
    vec!["catppuccin", "dracula", "gruvbox", "nord", "tokyo-night"]
}
