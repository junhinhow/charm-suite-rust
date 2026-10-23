use std::sync::Arc;

use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message};

use crate::common::{ascii_only, clamp_float, clamp_int, finite_float, truncate};

/// Range selector component.
#[derive(Clone)]
pub struct Range {
    min: f64,
    max: f64,
    value: f64,
    step: f64,
    format: Option<Arc<dyn Fn(f64) -> String + Send + Sync>>,
    width: i32,
    height: i32,
    focus: bool,
}

impl std::fmt::Debug for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Range")
            .field("min", &self.min)
            .field("max", &self.max)
            .field("value", &self.value)
            .field("step", &self.step)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("focus", &self.focus)
            .finish_non_exhaustive()
    }
}

impl Range {
    /// Creates a range; swapped bounds and out-of-range values normalized.
    #[must_use]
    pub fn new(minimum: f64, maximum: f64, value: f64) -> Self {
        let mut minimum = finite_float(minimum);
        let mut maximum = finite_float(maximum);
        if minimum > maximum {
            std::mem::swap(&mut minimum, &mut maximum);
        }
        let mut r = Self {
            min: minimum,
            max: maximum,
            value: 0.0,
            step: 1.0,
            format: None,
            width: 0,
            height: 0,
            focus: false,
        };
        r.set_value(value);
        r
    }

    /// Sets render box.
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    /// Returns current value.
    #[must_use]
    pub fn value(&self) -> f64 {
        self.value
    }

    /// Sets value clamped.
    pub fn set_value(&mut self, v: f64) {
        self.value = clamp_float(v, self.min, self.max);
    }

    /// Sets step.
    pub fn set_step(&mut self, step: f64) {
        if step <= 0.0 || step.is_nan() || step.is_infinite() {
            self.step = 1.0;
        } else {
            self.step = step;
        }
    }

    /// Sets format function.
    pub fn set_format<F>(&mut self, f: Option<F>)
    where
        F: Fn(f64) -> String + Send + Sync + 'static,
    {
        self.format = f.map(|func| Arc::new(func) as Arc<dyn Fn(f64) -> String + Send + Sync>);
    }

    /// Percent 0..100.
    #[must_use]
    pub fn percent(&self) -> f64 {
        if self.max <= self.min {
            return 0.0;
        }
        clamp_float(
            (self.value - self.min) / (self.max - self.min) * 100.0,
            0.0,
            100.0,
        )
    }

    /// Activates.
    pub fn focus(&mut self) {
        self.focus = true;
    }

    /// Deactivates.
    pub fn blur(&mut self) {
        self.focus = false;
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
        match key.key_type {
            KeyType::Left => self.set_value(self.value - self.step),
            KeyType::Right => self.set_value(self.value + self.step),
            KeyType::Home => self.set_value(self.min),
            KeyType::End => self.set_value(self.max),
            KeyType::PgUp => self.set_value(self.value + (self.max - self.min) / 10.0),
            KeyType::PgDown => self.set_value(self.value - (self.max - self.min) / 10.0),
            KeyType::Runes => {
                if key.runes == ['h'] {
                    self.set_value(self.value - self.step);
                } else if key.runes == ['l'] {
                    self.set_value(self.value + self.step);
                }
            }
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
        let value = truncate(&self.format_value(), self.width);
        let value_width = crate::common::display_width(&value);
        if value_width >= self.width as usize {
            return value;
        }
        let bar_width = self.width as usize - value_width - 1;
        if bar_width == 0 {
            return value;
        }
        let filled = ((bar_width as f64) * self.percent() / 100.0).round() as i32;
        let filled = clamp_int(filled, 0, bar_width as i32) as usize;
        let (full, empty) = if ascii_only() {
            ("#", "-")
        } else {
            ("█", "░")
        };
        let bar = full.repeat(filled) + &empty.repeat(bar_width - filled);
        format!("{bar} {value}")
    }

    fn format_value(&self) -> String {
        if let Some(f) = &self.format {
            f(self.value)
        } else {
            // Go strconv FormatFloat 'f' -1
            format!("{}", self.value)
        }
    }
}

impl charmed_bubbletea::Model for Range {
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
