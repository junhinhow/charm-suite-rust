//! Keyboard handling for [`Dialog`], mirroring the Go `keys.go` file.

use std::fmt;

use charmed_bubbletea::{KeyMsg, KeyType, Message};

use crate::{Dialog, DialogType};

/// Action triggered by the user inside a [`Dialog`].
///
/// Equivalent to the Go `Action` type. Its [`fmt::Display`] output matches the
/// Go `String()` method: `none`, `submit` or `cancel`.
///
/// ```
/// use charm_dialog::Action;
///
/// assert_eq!(Action::Submit.to_string(), "submit");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// No action was triggered.
    None,
    /// The user confirmed the dialog (Enter).
    Submit,
    /// The user cancelled the dialog (Esc).
    Cancel,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::None => "none",
            Self::Submit => "submit",
            Self::Cancel => "cancel",
        })
    }
}

impl Dialog {
    /// Handles a key press and reports the action it triggered.
    ///
    /// The first value is the [`Action`] triggered by the key. The second value
    /// tells whether the dialog consumed the key. When it is `false`, the
    /// application should continue with its normal key processing.
    ///
    /// Handled keys:
    ///
    /// | Key | Effect |
    /// |-----|--------|
    /// | `left` | previous button |
    /// | `right` | next button |
    /// | `tab` | next button |
    /// | `shift+tab` | previous button |
    /// | `enter` | [`Action::Submit`] |
    /// | `esc` | [`Action::Cancel`] |
    ///
    /// In [`DialogType::Input`] dialogs, printable characters are appended to
    /// `input_value`, `backspace` deletes the last character and `ctrl+u` clears
    /// the field. The cursor does not move: text is always typed at the end.
    ///
    /// ```
    /// use charm_dialog::{Action, Dialog};
    /// use charmed_bubbletea::{KeyMsg, KeyType};
    ///
    /// let mut dialog = Dialog::new_input("Rename", "Enter new name:");
    /// for c in "docs".chars() {
    ///     dialog.set_key_msg(&KeyMsg::from_char(c));
    /// }
    /// assert_eq!(dialog.input_value, "docs");
    ///
    /// let (action, consumed) = dialog.set_key_msg(&KeyMsg::from_type(KeyType::Enter));
    /// assert_eq!(action, Action::Submit);
    /// assert!(consumed);
    /// ```
    pub fn set_key_msg(&mut self, key: &KeyMsg) -> (Action, bool) {
        let is_input = self.r#type == DialogType::Input;
        match key.key_type {
            KeyType::Esc => (Action::Cancel, true),
            KeyType::Enter => (Action::Submit, true),
            KeyType::Tab | KeyType::Right => {
                self.next_button();
                (Action::None, true)
            }
            KeyType::ShiftTab | KeyType::Left => {
                self.prev_button();
                (Action::None, true)
            }
            KeyType::Backspace if is_input => {
                self.input_value.pop();
                (Action::None, true)
            }
            KeyType::CtrlU if is_input => {
                self.input_value.clear();
                (Action::None, true)
            }
            KeyType::Runes if is_input && !key.alt && !key.runes.is_empty() => {
                self.input_value.extend(key.runes.iter());
                (Action::None, true)
            }
            KeyType::Space if is_input && !key.alt => {
                self.input_value.push(' ');
                (Action::None, true)
            }
            _ => (Action::None, false),
        }
    }

    /// Handles a Bubble Tea message.
    ///
    /// Key messages (a [`KeyMsg`] wrapped in a [`Message`]) are delegated to
    /// [`Dialog::set_key_msg`]. Other messages are ignored and return
    /// `(Action::None, false)`.
    ///
    /// ```
    /// use charm_dialog::{Action, Dialog};
    /// use charmed_bubbletea::{KeyMsg, KeyType, Message};
    ///
    /// let mut dialog = Dialog::new_confirm("Delete File", "Are you sure?");
    /// dialog.handle_msg(&Message::new(KeyMsg::from_type(KeyType::Tab)));
    /// let (action, _) = dialog.handle_msg(&Message::new(KeyMsg::from_type(KeyType::Enter)));
    ///
    /// assert_eq!(dialog.selected_button(), "No");
    /// assert_eq!(action, Action::Submit);
    /// ```
    pub fn handle_msg(&mut self, msg: &Message) -> (Action, bool) {
        msg.downcast_ref::<KeyMsg>()
            .map_or((Action::None, false), |key| self.set_key_msg(key))
    }
}
