use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message};

use crate::common::clamp_int;

/// Time selector component.
#[derive(Debug, Clone)]
pub struct Time {
    hour: i32,
    minute: i32,
    second: i32,
    field: i32,
    second_visible: bool,
    h24: bool,
    width: i32,
    height: i32,
    focus: bool,
}

impl Time {
    /// Creates a clock; out-of-range fields are clamped.
    #[must_use]
    pub fn new(hour: i32, minute: i32, second: i32) -> Self {
        let mut t = Self {
            hour: 0,
            minute: 0,
            second: 0,
            field: 0,
            second_visible: true,
            h24: true,
            width: 0,
            height: 0,
            focus: false,
        };
        t.set_value(hour, minute, second);
        t
    }

    /// Sets the render box.
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    /// Returns the current values.
    #[must_use]
    pub fn value(&self) -> (i32, i32, i32) {
        (self.hour, self.minute, self.second)
    }

    /// Sets and clamps all fields.
    pub fn set_value(&mut self, hour: i32, minute: i32, second: i32) {
        self.hour = clamp_int(hour, 0, 23);
        self.minute = clamp_int(minute, 0, 59);
        self.second = clamp_int(second, 0, 59);
    }

    /// Shows or hides seconds.
    pub fn set_second_visible(&mut self, visible: bool) {
        self.second_visible = visible;
        if !visible && self.field > 1 {
            self.field = 1;
        }
    }

    /// Chooses 24h or 12h display.
    pub fn set_24h(&mut self, enabled: bool) {
        self.h24 = enabled;
    }

    /// String representation, always 24h.
    #[must_use]
    pub fn string(&self) -> String {
        if self.second_visible {
            format!("{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
        } else {
            format!("{:02}:{:02}", self.hour, self.minute)
        }
    }

    /// Activates key reception.
    pub fn focus(&mut self) {
        self.focus = true;
    }

    /// Deactivates key reception.
    pub fn blur(&mut self) {
        self.focus = false;
    }

    /// Reports focused.
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
        match key.key_type {
            KeyType::Left => {
                let count = self.field_count() as i32;
                self.field = (self.field + count - 1) % count;
            }
            KeyType::Right => {
                let count = self.field_count() as i32;
                self.field = (self.field + 1) % count;
            }
            KeyType::Up => self.bump(1),
            KeyType::Down => self.bump(-1),
            _ => {}
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
        let mut fields = vec![
            format!("{:02}", self.display_hour()),
            format!("{:02}", self.minute),
        ];
        if self.second_visible {
            fields.push(format!("{:02}", self.second));
        }
        if self.focus {
            let field = clamp_int(self.field, 0, fields.len() as i32 - 1) as usize;
            fields[field] = format!("[{}]", fields[field]);
        }
        let mut line = fields.join(":");
        if !self.h24 {
            line.push_str(&format!(" {}", self.period()));
        }
        crate::common::truncate(&line, self.width)
    }

    fn field_count(&self) -> usize {
        if self.second_visible {
            3
        } else {
            2
        }
    }

    fn display_hour(&self) -> i32 {
        if self.h24 {
            self.hour
        } else {
            let mut h = self.hour % 12;
            if h == 0 {
                h = 12;
            }
            h
        }
    }

    fn period(&self) -> &'static str {
        if self.hour < 12 {
            "AM"
        } else {
            "PM"
        }
    }

    fn bump(&mut self, delta: i32) {
        match self.field {
            0 => self.hour = (self.hour + delta + 24).rem_euclid(24),
            1 => self.minute = (self.minute + delta + 60).rem_euclid(60),
            2 => self.second = (self.second + delta + 60).rem_euclid(60),
            _ => {}
        }
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.string())
    }
}

impl charmed_bubbletea::Model for Time {
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
