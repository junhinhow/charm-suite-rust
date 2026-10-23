use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message};

use crate::common::{ascii_only, sanitize_text, suffix_to_width, truncate};

/// Masked text input component.
#[derive(Debug, Clone)]
pub struct Masked {
    value: Vec<char>,
    mask: char,
    reveal: bool,
    temp_reveal: bool,
    width: i32,
    height: i32,
    focus: bool,
}

impl Masked {
    /// Creates an empty masked field. Zero/control masks become '*'.
    #[must_use]
    pub fn new(mask: char) -> Self {
        let mask = if mask == '\0' || mask.is_control() {
            '*'
        } else {
            mask
        };
        Self {
            value: Vec::new(),
            mask,
            reveal: false,
            temp_reveal: false,
            width: 0,
            height: 0,
            focus: false,
        }
    }

    /// Sets render box.
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    /// Returns real unmasked text.
    #[must_use]
    pub fn value(&self) -> String {
        self.value.iter().collect()
    }

    /// Replaces value, removing control chars.
    pub fn set_value(&mut self, s: &str) {
        self.value = sanitize_text(s).chars().collect();
    }

    /// Permanently reveals/masks.
    pub fn reveal(&mut self, enabled: bool) {
        self.reveal = enabled;
        if !enabled {
            self.temp_reveal = false;
        }
    }

    /// Returns covered text.
    #[must_use]
    pub fn masked(&self) -> String {
        let mask = self.mask_char();
        std::iter::repeat_n(mask, self.value.len()).collect()
    }

    fn mask_char(&self) -> char {
        if self.mask == '\0' || self.mask.is_control() {
            '*'
        } else {
            self.mask
        }
    }

    /// Activates.
    pub fn focus(&mut self) {
        self.focus = true;
    }

    /// Deactivates.
    pub fn blur(&mut self) {
        self.focus = false;
        self.temp_reveal = false;
    }

    /// Focused?
    #[must_use]
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Init.
    #[must_use]
    pub fn init(&self) -> Option<Cmd> {
        None
    }

    /// Handles key.
    pub fn handle_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        if !self.focus {
            return None;
        }
        if key.key_type == KeyType::CtrlR {
            self.temp_reveal = !self.temp_reveal;
            return None;
        }
        // any other key ends temp reveal
        self.temp_reveal = false;

        match key.key_type {
            KeyType::Backspace => {
                if !self.value.is_empty() {
                    self.value.pop();
                }
                return None;
            }
            KeyType::CtrlU => {
                self.value.clear();
                return None;
            }
            _ => {}
        }

        if key.key_type == KeyType::Runes {
            for r in &key.runes {
                if r.is_control() {
                    continue;
                }
                self.value.push(*r);
            }
        }
        // space key
        if key.key_type == KeyType::Space {
            self.value.push(' ');
        }
        None
    }

    /// Handles message.
    pub fn handle_msg(&mut self, msg: &Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            self.handle_key(key)
        } else {
            None
        }
    }

    /// Update.
    pub fn update(&mut self, message: Message) -> Option<Cmd> {
        self.handle_msg(&message)
    }

    /// View.
    #[must_use]
    pub fn view(&self) -> String {
        if self.width <= 0 || self.height <= 0 {
            return String::new();
        }
        let visible: Vec<char> = if !self.reveal && !self.temp_reveal {
            self.masked().chars().collect()
        } else {
            self.value.clone()
        };

        if !self.focus {
            return suffix_to_width(&visible, self.width).into_iter().collect();
        }
        let cursor = if ascii_only() { "|" } else { "▏" };
        let cursor_width = crate::common::display_width(cursor) as i32;
        if cursor_width >= self.width {
            return truncate(cursor, self.width);
        }
        let text: String = suffix_to_width(&visible, self.width - cursor_width)
            .into_iter()
            .collect();
        truncate(&(text + cursor), self.width)
    }
}

impl charmed_bubbletea::Model for Masked {
    fn init(&self) -> Option<Cmd> {
        None
    }
    fn update(&mut self, message: Message) -> Option<Cmd> {
        self.update(message)
    }
    fn view(&self) -> String {
        self.view()
    }
}
