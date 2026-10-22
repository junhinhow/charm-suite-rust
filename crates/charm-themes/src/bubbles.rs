//! Styles for the `bubbles` components, derived from a [`Theme`].
//!
//! The workspace has no port of the `bubbles` widgets, so these types only
//! carry the style values. They mirror the `Styles` structs of the Go
//! components, with the same field meanings and defaults.

use std::time::Duration;

use charm_core::Rgb;
use lipgloss::Style;

use crate::theme::Theme;

/// Shape of the terminal cursor, mirroring `tea.CursorShape` of Bubble Tea.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorShape {
    /// A filled block covering the character cell (the default).
    #[default]
    Block,
    /// A line under the character cell.
    Underline,
    /// A thin vertical bar before the character cell.
    Bar,
}

/// Cursor appearance of the text input and text area components.
#[derive(Debug, Clone, PartialEq)]
pub struct CursorStyle {
    /// Color of the cursor.
    pub color: Rgb,
    /// Shape of the cursor.
    pub shape: CursorShape,
    /// Whether the cursor blinks.
    pub blink: bool,
    /// Time between two blink phases.
    pub blink_speed: Duration,
}

/// Styles of a text input in one focus state.
#[derive(Debug, Clone)]
pub struct TextInputStateStyles {
    /// Style of the typed text.
    pub text: Style,
    /// Style of the placeholder shown while the input is empty.
    pub placeholder: Style,
    /// Style of the completion suggestion.
    pub suggestion: Style,
    /// Style of the prompt printed before the input.
    pub prompt: Style,
}

/// Styles of the `textinput` component.
#[derive(Debug, Clone)]
pub struct TextInputStyles {
    /// Styles used while the input has focus.
    pub focused: TextInputStateStyles,
    /// Styles used while the input has no focus.
    pub blurred: TextInputStateStyles,
    /// Cursor appearance.
    pub cursor: CursorStyle,
}

/// Styles of a text area in one focus state.
#[derive(Debug, Clone)]
pub struct TextAreaStateStyles {
    /// Style of the whole component.
    pub base: Style,
    /// Style of the text.
    pub text: Style,
    /// Style of the line numbers.
    pub line_number: Style,
    /// Style of the line number of the cursor line.
    pub cursor_line_number: Style,
    /// Style of the cursor line.
    pub cursor_line: Style,
    /// Style of the rows past the end of the text.
    pub end_of_buffer: Style,
    /// Style of the placeholder shown while the text area is empty.
    pub placeholder: Style,
    /// Style of the prompt printed before each line.
    pub prompt: Style,
}

/// Styles of the `textarea` component.
#[derive(Debug, Clone)]
pub struct TextAreaStyles {
    /// Styles used while the text area has focus.
    pub focused: TextAreaStateStyles,
    /// Styles used while the text area has no focus.
    pub blurred: TextAreaStateStyles,
    /// Cursor appearance.
    pub cursor: CursorStyle,
}

/// Styles of the `table` component.
#[derive(Debug, Clone)]
pub struct TableStyles {
    /// Style of the header row.
    pub header: Style,
    /// Style of the regular cells.
    pub cell: Style,
    /// Style of the selected row.
    pub selected: Style,
}

/// Styles of the `help` component.
#[derive(Debug, Clone)]
pub struct HelpStyles {
    /// Style of the ellipsis shown when help is truncated.
    pub ellipsis: Style,
    /// Style of a key in the short help.
    pub short_key: Style,
    /// Style of a description in the short help.
    pub short_desc: Style,
    /// Style of the separator in the short help.
    pub short_separator: Style,
    /// Style of a key in the full help.
    pub full_key: Style,
    /// Style of a description in the full help.
    pub full_desc: Style,
    /// Style of the separator in the full help.
    pub full_separator: Style,
}

/// Styles of the `list` component.
#[derive(Debug, Clone)]
pub struct ListStyles {
    /// Style of the title bar container.
    pub title_bar: Style,
    /// Style of the title.
    pub title: Style,
    /// Style of the loading spinner.
    pub spinner: Style,
    /// Styles of the filter input.
    pub filter: TextInputStyles,
    /// Style of the characters matched by the filter.
    pub default_filter_character_match: Style,
    /// Style of the status bar.
    pub status_bar: Style,
    /// Style of the status message when the list is empty.
    pub status_empty: Style,
    /// Style of the status bar while the filter is active.
    pub status_bar_active_filter: Style,
    /// Style of the item count in the status bar.
    pub status_bar_filter_count: Style,
    /// Style of the message shown when there are no items.
    pub no_items: Style,
    /// Style of the pagination container.
    pub pagination_style: Style,
    /// Style of the help container.
    pub help_style: Style,
    /// Style of the dot marking the current page.
    pub active_pagination_dot: Style,
    /// Style of the dots marking the other pages.
    pub inactive_pagination_dot: Style,
    /// Style of the "page x of y" text.
    pub arabic_pagination: Style,
    /// Style of the separator between help items.
    pub divider_dot: Style,
}

/// Styles of every `bubbles` component derived from a [`Theme`].
///
/// Apply the parts to the components, for example with
/// `list.Styles = theme.Bubbles().List` in Go.
///
/// # Examples
///
/// ```
/// use charm_themes::themes;
///
/// let styles = themes::catppuccin().bubbles();
/// let title = styles.list.title.render("Items");
/// assert!(charm_core::strip_ansi(&title).contains("Items"));
/// ```
#[derive(Debug, Clone)]
pub struct BubblesStyles {
    /// Styles of the `list` component.
    pub list: ListStyles,
    /// Styles of the `table` component.
    pub table: TableStyles,
    /// Styles of the `textinput` component.
    pub text_input: TextInputStyles,
    /// Styles of the `textarea` component.
    pub text_area: TextAreaStyles,
    /// Styles of the `help` component.
    pub help: HelpStyles,
}

fn fg(color: Rgb) -> Style {
    Style::new().foreground(color.to_lipgloss())
}

impl BubblesStyles {
    /// Builds every component style from the colors of `theme`.
    pub(crate) fn from_theme(theme: &Theme) -> Self {
        Self {
            list: list_styles(theme),
            table: table_styles(theme),
            text_input: text_input_styles(theme),
            text_area: text_area_styles(theme),
            help: help_styles(theme),
        }
    }
}

fn list_styles(t: &Theme) -> ListStyles {
    ListStyles {
        title_bar: Style::new().padding(0, 0, 1, 2),
        title: Style::new()
            .background(t.primary.to_lipgloss())
            .foreground(t.background.to_lipgloss())
            .bold(true)
            .padding_2(0, 1),
        spinner: fg(t.muted),
        filter: text_input_styles(t),
        default_filter_character_match: fg(t.accent).underline(true),
        status_bar: fg(t.muted).padding(0, 0, 1, 2),
        status_empty: fg(t.muted),
        status_bar_active_filter: fg(t.foreground),
        status_bar_filter_count: fg(t.muted),
        no_items: fg(t.muted),
        pagination_style: Style::new().padding_left(2),
        help_style: Style::new().padding(1, 0, 0, 2),
        active_pagination_dot: fg(t.accent).set_string("•"),
        inactive_pagination_dot: fg(t.muted).set_string("•"),
        arabic_pagination: fg(t.muted),
        divider_dot: fg(t.muted).set_string(" • "),
    }
}

fn table_styles(t: &Theme) -> TableStyles {
    TableStyles {
        header: fg(t.primary).bold(true).padding_2(0, 1),
        cell: fg(t.foreground).padding_2(0, 1),
        selected: fg(t.background)
            .background(t.accent.to_lipgloss())
            .bold(true),
    }
}

fn text_input_styles(t: &Theme) -> TextInputStyles {
    TextInputStyles {
        focused: TextInputStateStyles {
            text: fg(t.foreground),
            placeholder: fg(t.muted),
            suggestion: fg(t.muted),
            prompt: fg(t.primary),
        },
        blurred: TextInputStateStyles {
            text: fg(t.muted),
            placeholder: fg(t.muted),
            suggestion: fg(t.muted),
            prompt: fg(t.muted),
        },
        cursor: cursor(t),
    }
}

fn text_area_styles(t: &Theme) -> TextAreaStyles {
    TextAreaStyles {
        focused: TextAreaStateStyles {
            base: fg(t.foreground),
            text: fg(t.foreground),
            line_number: fg(t.muted),
            cursor_line_number: fg(t.primary),
            cursor_line: fg(t.foreground),
            end_of_buffer: fg(t.muted),
            placeholder: fg(t.muted),
            prompt: fg(t.primary),
        },
        blurred: TextAreaStateStyles {
            base: fg(t.muted),
            text: fg(t.muted),
            line_number: fg(t.muted),
            cursor_line_number: fg(t.muted),
            cursor_line: fg(t.muted),
            end_of_buffer: fg(t.muted),
            placeholder: fg(t.muted),
            prompt: fg(t.muted),
        },
        cursor: cursor(t),
    }
}

fn cursor(t: &Theme) -> CursorStyle {
    CursorStyle {
        color: t.foreground,
        shape: CursorShape::Block,
        blink: true,
        blink_speed: Duration::from_millis(500),
    }
}

fn help_styles(t: &Theme) -> HelpStyles {
    HelpStyles {
        ellipsis: fg(t.muted),
        short_key: fg(t.accent),
        short_desc: fg(t.muted),
        short_separator: fg(t.muted),
        full_key: fg(t.accent),
        full_desc: fg(t.muted),
        full_separator: fg(t.muted),
    }
}
