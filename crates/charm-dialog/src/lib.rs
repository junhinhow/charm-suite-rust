//! Modal dialog boxes for terminal user interfaces.
//!
//! This crate is a Rust port of the Go library
//! [`github.com/junhinhow/charm-dialog`](https://github.com/junhinhow/charm-dialog).
//! A [`Dialog`] is a plain value: it holds the title, message, buttons and
//! selection state, and renders itself to a string with [`lipgloss`] styles.
//! Key presses are routed to it with [`Dialog::set_key_msg`] or
//! [`Dialog::handle_msg`].
//!
//! ```
//! use charm_dialog::Dialog;
//!
//! let mut dialog = Dialog::new_confirm("Confirmar", "Deseja continuar?");
//! assert_eq!(dialog.selected_button(), "Yes");
//!
//! dialog.next_button();
//! assert_eq!(dialog.selected_button(), "No");
//!
//! let boxed = dialog.render(40);
//! assert!(charm_core::strip_ansi(&boxed).contains("Deseja continuar?"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use lipgloss::{Color, Style};

mod keys;
#[cfg(test)]
mod tests;

pub use keys::Action;

/// The kind of a [`Dialog`].
///
/// Equivalent to the Go `DialogType` constants `TypeConfirm`, `TypeAlert`,
/// `TypeInput` and `TypeCustom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogType {
    /// Confirmation dialog with `Yes` and `No` buttons.
    Confirm,
    /// Alert dialog with a single `OK` button.
    Alert,
    /// Dialog with a text input field.
    Input,
    /// Dialog with caller-provided buttons.
    Custom,
}

/// Styles used to render a [`Dialog`].
///
/// Equivalent to the Go `Styles` struct. [`Styles::default`] returns the same
/// values as the Go `DefaultStyles()` function.
#[derive(Debug, Clone)]
pub struct Styles {
    /// Style of the dialog title.
    pub title_style: Style,
    /// Style of the message (or of the input prompt).
    pub message_style: Style,
    /// Style of inactive buttons.
    pub button_style: Style,
    /// Style of the selected button.
    pub active_button_style: Style,
    /// Style of the border around the dialog content.
    pub border_style: Style,
}

impl Default for Styles {
    fn default() -> Self {
        Self {
            title_style: Style::new()
                .bold(true)
                .foreground(color("#FAFAFA"))
                .background(color("#7D56F4"))
                .padding_2(0, 1),
            message_style: Style::new().foreground(color("#DDDDDD")).padding_2(1, 0),
            button_style: Style::new()
                .foreground(color("#888888"))
                .background(color("#333333"))
                .padding_2(0, 2)
                .margin(0, 1, 0, 1),
            active_button_style: Style::new()
                .foreground(color("#FAFAFA"))
                .background(color("#7D56F4"))
                .padding_2(0, 2)
                .margin(0, 1, 0, 1),
            border_style: Style::new()
                .border(lipgloss::rounded_border())
                .border_foreground(color("#7D56F4"))
                .padding_2(1, 2),
        }
    }
}

/// A modal dialog box.
///
/// Create one with [`Dialog::new_confirm`], [`Dialog::new_alert`],
/// [`Dialog::new_input`] or [`Dialog::new_custom`], move the selection with
/// [`Dialog::next_button`] and [`Dialog::prev_button`], and render it with
/// [`Dialog::render`] or [`Dialog::render_overlay`].
///
/// ```
/// use charm_dialog::{Dialog, DialogType};
///
/// let dialog = Dialog::new_alert("Aviso", "Operacao concluida");
/// assert_eq!(dialog.r#type, DialogType::Alert);
/// assert_eq!(dialog.buttons, ["OK"]);
/// ```
#[derive(Debug, Clone)]
pub struct Dialog {
    /// Title shown at the top of the dialog.
    pub title: String,
    /// Main message of the dialog.
    pub message: String,
    /// Button labels, in display order.
    pub buttons: Vec<String>,
    /// Index of the currently selected button.
    pub selected: usize,
    /// Kind of the dialog.
    pub r#type: DialogType,
    /// Current value of the input field (only used by [`DialogType::Input`]).
    pub input_value: String,
    /// Prompt shown before the input field (only used by [`DialogType::Input`]).
    pub input_prompt: String,
    /// Styles used by [`Dialog::render`].
    pub styles: Styles,
}

impl Dialog {
    /// Creates a confirmation dialog with `Yes` and `No` buttons.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let dialog = Dialog::new_confirm("Confirmar", "Tem certeza?");
    /// assert_eq!(dialog.buttons, ["Yes", "No"]);
    /// ```
    pub fn new_confirm(title: &str, message: &str) -> Self {
        Self::build(DialogType::Confirm, title, message, &["Yes", "No"])
    }

    /// Creates an alert dialog with a single `OK` button.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let dialog = Dialog::new_alert("Aviso", "Operacao concluida");
    /// assert_eq!(dialog.selected_button(), "OK");
    /// ```
    pub fn new_alert(title: &str, message: &str) -> Self {
        Self::build(DialogType::Alert, title, message, &["OK"])
    }

    /// Creates a dialog with a text input field.
    ///
    /// `prompt` is shown before the input field. The dialog has `Submit` and
    /// `Cancel` buttons, and the input value starts empty.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let mut dialog = Dialog::new_input("Entrada", "Digite seu nome:");
    /// dialog.input_value.push_str("Ana");
    /// assert_eq!(dialog.input_prompt, "Digite seu nome:");
    /// assert_eq!(dialog.buttons, ["Submit", "Cancel"]);
    /// ```
    pub fn new_input(title: &str, prompt: &str) -> Self {
        let mut dialog = Self::build(DialogType::Input, title, "", &["Submit", "Cancel"]);
        prompt.clone_into(&mut dialog.input_prompt);
        dialog
    }

    /// Creates a dialog with caller-provided buttons.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let dialog = Dialog::new_custom("Opcoes", "Escolha", &["Salvar", "Cancelar"]);
    /// assert_eq!(dialog.buttons, ["Salvar", "Cancelar"]);
    /// ```
    pub fn new_custom(title: &str, message: &str, buttons: &[&str]) -> Self {
        Self::build(DialogType::Custom, title, message, buttons)
    }

    /// Replaces the styles of the dialog and returns it.
    ///
    /// Use [`Dialog::set_styles`] to change the styles of an existing value
    /// in place, as the Go `WithStyles` method does.
    ///
    /// ```
    /// use charm_dialog::{Dialog, Styles};
    ///
    /// let dialog = Dialog::new_alert("Aviso", "Oi").with_styles(Styles::default());
    /// assert_eq!(dialog.buttons, ["OK"]);
    /// ```
    #[must_use]
    pub fn with_styles(mut self, styles: Styles) -> Self {
        self.styles = styles;
        self
    }

    /// Replaces the styles of the dialog in place.
    ///
    /// ```
    /// use charm_dialog::{Dialog, Styles};
    ///
    /// let mut dialog = Dialog::new_alert("Aviso", "Oi");
    /// dialog.set_styles(Styles::default());
    /// assert_eq!(dialog.buttons, ["OK"]);
    /// ```
    pub fn set_styles(&mut self, styles: Styles) {
        self.styles = styles;
    }

    /// Moves the selection to the next button.
    ///
    /// The selection stops at the last button; it does not wrap around.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let mut dialog = Dialog::new_confirm("Teste", "nav");
    /// dialog.next_button();
    /// dialog.next_button();
    /// assert_eq!(dialog.selected, 1);
    /// ```
    pub fn next_button(&mut self) {
        if self.selected + 1 < self.buttons.len() {
            self.selected += 1;
        }
    }

    /// Moves the selection to the previous button.
    ///
    /// The selection stops at the first button; it does not wrap around.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let mut dialog = Dialog::new_confirm("Teste", "nav");
    /// dialog.prev_button();
    /// assert_eq!(dialog.selected, 0);
    /// ```
    pub fn prev_button(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    /// Returns the label of the selected button, or `""` when there are no
    /// buttons.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let dialog = Dialog::new_custom("T", "M", &[]);
    /// assert_eq!(dialog.selected_button(), "");
    /// ```
    pub fn selected_button(&self) -> &str {
        self.buttons.get(self.selected).map_or("", String::as_str)
    }

    /// Renders the dialog as a box, centered in `width` columns.
    ///
    /// When `width` is greater than zero, the content is wrapped so the output
    /// never exceeds `width` columns. A short dialog keeps its natural width
    /// and is then centered. A `width` of `0` skips the centering and applies
    /// no limit.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let dialog = Dialog::new_confirm("Confirmar", "Deseja continuar?");
    /// let out = charm_core::strip_ansi(&dialog.render(80));
    /// assert!(out.contains("Confirmar"));
    /// assert!(out.contains("Yes"));
    /// ```
    pub fn render(&self, width: usize) -> String {
        let boxed = self.render_box(width);
        if width > 0 {
            lipgloss::place_horizontal(to_i32(width), lipgloss::CENTER, &boxed, &[])
        } else {
            boxed
        }
    }

    /// Renders the dialog centered both vertically and horizontally inside a
    /// `width` x `height` area, so it can be drawn over an existing screen.
    ///
    /// The result has exactly `height` lines, joined by `\n` with no trailing
    /// newline, and no line is wider than `width`. When the dialog is taller
    /// than the area, it is truncated. A `height` of `0` returns an empty
    /// string.
    ///
    /// ```
    /// use charm_dialog::Dialog;
    ///
    /// let dialog = Dialog::new_alert("Info", "Mensagem de teste");
    /// let out = dialog.render_overlay(80, 24);
    /// assert_eq!(out.matches('\n').count(), 23);
    /// assert!(charm_core::strip_ansi(&out).contains("Mensagem de teste"));
    /// ```
    pub fn render_overlay(&self, width: usize, height: usize) -> String {
        if height == 0 {
            return String::new();
        }

        let boxed = self.render_box(width);
        let mut lines: Vec<&str> = boxed.split('\n').collect();
        lines.truncate(height);

        let top_pad = if height > lines.len() {
            (height - lines.len()) / 2
        } else {
            0
        };
        let bottom_pad = height - lines.len() - top_pad;

        let dialog_width = block_width(&lines.join("\n"));
        let left_pad = if width > dialog_width {
            " ".repeat((width - dialog_width) / 2)
        } else {
            String::new()
        };
        let blank_line = " ".repeat(width);

        let mut out: Vec<String> = Vec::with_capacity(height);
        out.extend((0..top_pad).map(|_| blank_line.clone()));
        out.extend(lines.iter().map(|line| format!("{left_pad}{line}")));
        out.extend((0..bottom_pad).map(|_| blank_line.clone()));
        out.join("\n")
    }

    /// Renders the bordered box, limiting its total width to `max_width` when
    /// that is greater than zero. The limit is applied to the inner blocks, so
    /// long messages wrap instead of overflowing.
    fn render_box(&self, max_width: usize) -> String {
        let frame =
            usize::try_from(self.styles.border_style.get_horizontal_frame_size()).unwrap_or(0);

        let mut content = self.render_content(0);
        if max_width > 0 && block_width(&content) + frame > max_width {
            let inner = max_width.saturating_sub(frame).max(1);
            content = self.render_content(inner);
        }

        self.styles.border_style.render(&content)
    }

    /// Builds the inner content (title, body, buttons) without the border.
    ///
    /// When `inner` is greater than zero, each block is limited to that width.
    fn render_content(&self, inner: usize) -> String {
        let styles = &self.styles;

        let mut title_style = styles.title_style.clone();
        let mut message_style = styles.message_style.clone();
        if inner > 0 {
            let width = to_i32(inner);
            title_style = title_style.width(width);
            message_style = message_style.width(width);
        }

        let title = title_style.render(&self.title);

        let body = if self.r#type == DialogType::Input {
            let input_line = Style::new()
                .foreground(color("#FAFAFA"))
                .underline(true)
                .render(&format!("{}_", self.input_value));
            pad_block(&message_style.render(&format!("{}\n{input_line}", self.input_prompt)))
        } else {
            pad_block(&message_style.render(&self.message))
        };

        let buttons: Vec<String> = self
            .buttons
            .iter()
            .enumerate()
            .map(|(index, label)| {
                if index == self.selected {
                    styles.active_button_style.render(label)
                } else {
                    styles.button_style.render(label)
                }
            })
            .collect();
        let button_refs: Vec<&str> = buttons.iter().map(String::as_str).collect();
        let mut buttons_row = lipgloss::join_horizontal(lipgloss::CENTER, &button_refs);
        if inner > 0 {
            buttons_row = pad_block(&Style::new().width(to_i32(inner)).render(&buttons_row));
        }

        lipgloss::join_vertical(lipgloss::CENTER, &[&title, &body, "", &buttons_row])
    }

    fn build(kind: DialogType, title: &str, message: &str, buttons: &[&str]) -> Self {
        Self {
            title: title.to_owned(),
            message: message.to_owned(),
            buttons: buttons.iter().map(|&label| label.to_owned()).collect(),
            selected: 0,
            r#type: kind,
            input_value: String::new(),
            input_prompt: String::new(),
            styles: Styles::default(),
        }
    }
}

/// Widest visible line of a multi-line block, like Go's `lipgloss.Width`.
fn block_width(text: &str) -> usize {
    text.split('\n')
        .map(lipgloss::utils::width_visible)
        .max()
        .unwrap_or(0)
}

/// Pads every line of a multi-line block with spaces up to the block width, so
/// that a later centering moves the block as a whole. Go's `lipgloss` does this
/// inside `Render`; the Rust port does not.
fn pad_block(text: &str) -> String {
    let width = block_width(text);
    text.split('\n')
        .map(|line| {
            let padding = width.saturating_sub(lipgloss::utils::width_visible(line));
            format!("{line}{}", " ".repeat(padding))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Converts a width to the `i32` used by `lipgloss`, saturating at `i32::MAX`.
fn to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

/// Builds a [`lipgloss::Color`] from a hex string such as `"#7D56F4"`.
fn color(hex: &str) -> Color {
    Color(hex.to_owned())
}
