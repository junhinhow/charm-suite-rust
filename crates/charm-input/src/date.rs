use std::fmt::Write as _;

use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message};
use chrono::{Datelike, NaiveDate};

use crate::common::truncate;

/// Message emitted when the user confirms a date with Enter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateSelectedMsg {
    /// The confirmed date.
    pub value: NaiveDate,
}

/// Date selector component.
#[derive(Debug, Clone)]
pub struct Date {
    value: NaiveDate,
    min: NaiveDate,
    max: NaiveDate,
    today: NaiveDate,
    format: String,
    width: i32,
    height: i32,
    focus: bool,
}

impl Default for Date {
    fn default() -> Self {
        Self::new(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap_or_default())
    }
}

impl Date {
    /// Creates a selector positioned at `t`. The day is also used as "today".
    #[must_use]
    pub fn new(t: NaiveDate) -> Self {
        Self {
            value: t,
            min: NaiveDate::default(),
            max: NaiveDate::default(),
            today: t,
            format: String::new(),
            width: 0,
            height: 0,
            focus: false,
        }
    }

    /// Sets the render box.
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = width;
        self.height = height;
    }

    /// Returns the selected date.
    #[must_use]
    pub fn value(&self) -> NaiveDate {
        self.value
    }

    /// Selects a date, clamped to the bounds.
    pub fn set_value(&mut self, t: NaiveDate) {
        self.value = self.clamp(t);
    }

    /// Sets the compact layout (Go style). Empty restores default.
    pub fn set_format(&mut self, layout: &str) {
        if layout.is_empty() {
            self.format = String::new();
        } else {
            layout.clone_into(&mut self.format);
        }
    }

    /// Sets the inclusive lower bound. Zero removes it.
    pub fn set_min(&mut self, t: NaiveDate) {
        self.min = t;
        self.normalize_bounds();
        self.value = self.clamp(self.value);
    }

    /// Sets the inclusive upper bound. Zero removes it.
    pub fn set_max(&mut self, t: NaiveDate) {
        self.max = t;
        self.normalize_bounds();
        self.value = self.clamp(self.value);
    }

    /// Sets the date highlighted as today. Zero removes highlight.
    pub fn set_today(&mut self, t: NaiveDate) {
        self.today = t;
    }

    /// Activates key reception.
    pub fn focus(&mut self) {
        self.focus = true;
    }

    /// Deactivates key reception.
    pub fn blur(&mut self) {
        self.focus = false;
    }

    /// Reports whether focused.
    #[must_use]
    pub fn focused(&self) -> bool {
        self.focus
    }

    /// Bubble Tea init.
    #[must_use]
    pub fn init(&self) -> Option<Cmd> {
        None
    }

    /// Handles a key message.
    pub fn handle_key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        if !self.focus {
            return None;
        }
        match key.key_type {
            KeyType::Left => self.move_days(-1),
            KeyType::Right => self.move_days(1),
            KeyType::Up => self.move_days(-7),
            KeyType::Down => self.move_days(7),
            KeyType::CtrlLeft => self.move_months(-1),
            KeyType::CtrlRight => self.move_months(1),
            KeyType::PgUp => self.move_months(-12),
            KeyType::PgDown => self.move_months(12),
            KeyType::Enter => {
                let value = self.value;
                return Some(Cmd::new(move || Message::new(DateSelectedMsg { value })));
            }
            _ => {}
        }
        None
    }

    /// Handles a Bubble Tea message.
    pub fn handle_msg(&mut self, msg: &Message) -> Option<Cmd> {
        if let Some(key) = msg.downcast_ref::<KeyMsg>() {
            self.handle_key(key)
        } else {
            None
        }
    }

    /// Bubble Tea update.
    pub fn update(&mut self, message: Message) -> Option<Cmd> {
        self.handle_msg(&message)
    }

    /// Renders the component.
    #[must_use]
    pub fn view(&self) -> String {
        if self.width <= 0 || self.height <= 0 {
            return String::new();
        }
        if self.height < 8 {
            return truncate(
                &self.value.format(&go_to_chrono(&self.layout())).to_string(),
                self.width,
            );
        }
        let lines = self.calendar_lines();
        lines
            .into_iter()
            .map(|l| truncate(&l, self.width))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn layout(&self) -> String {
        if self.format.is_empty() {
            "02/01/2006".to_owned()
        } else {
            self.format.clone()
        }
    }

    fn normalize_bounds(&mut self) {
        if self.min == NaiveDate::default() || self.max == NaiveDate::default() {
            return;
        }
        if self.max < self.min {
            std::mem::swap(&mut self.min, &mut self.max);
        }
    }

    fn clamp(&self, t: NaiveDate) -> NaiveDate {
        if self.min != NaiveDate::default() && t < self.min {
            return self.min;
        }
        if self.max != NaiveDate::default() && t > self.max {
            return self.max;
        }
        t
    }

    fn move_days(&mut self, days: i32) {
        if let Some(d) = self
            .value
            .checked_add_signed(chrono::Duration::days(days as i64))
        {
            self.value = self.clamp(d);
        }
    }

    fn move_months(&mut self, months: i32) {
        self.value = self.clamp(add_months(self.value, months));
    }

    fn calendar_lines(&self) -> Vec<String> {
        let mut title = self.value.format("%B %Y").to_string();
        if self.today != NaiveDate::default()
            && self.today.year() == self.value.year()
            && self.today.month() == self.value.month()
        {
            title.push_str(" *");
        }
        let mut lines = vec![title, "Su Mo Tu We Th Fr Sa".to_owned()];

        let first =
            NaiveDate::from_ymd_opt(self.value.year(), self.value.month(), 1).unwrap_or(self.value);
        // start = first - weekday days
        let weekday = first.weekday().num_days_from_sunday() as i64;
        let start = first - chrono::Duration::days(weekday);
        for week in 0..6 {
            let mut row = String::new();
            for wd in 0..7 {
                let day = start + chrono::Duration::days((week * 7 + wd) as i64);
                if day.month() != self.value.month() || day.year() != self.value.year() {
                    row.push_str("    ");
                    continue;
                }
                if day == self.value {
                    let _ = write!(row, "[{:2}]", day.day());
                } else if self.today != NaiveDate::default() && day == self.today {
                    let _ = write!(row, "*{:2} ", day.day());
                } else {
                    let _ = write!(row, " {:2} ", day.day());
                }
            }
            lines.push(row.trim_end().to_owned());
        }
        lines
    }
}

fn add_months(t: NaiveDate, months: i32) -> NaiveDate {
    let y = t.year();
    let m = t.month().cast_signed();
    let d = t.day().cast_signed();
    let total = (y * 12 + (m - 1)) + months;
    let ny = total.div_euclid(12);
    let nm = (total.rem_euclid(12) + 1) as u32;
    let last = days_in_month(ny, nm);
    let nd = d.min(last);
    NaiveDate::from_ymd_opt(ny, nm, nd as u32).unwrap_or(t)
}

fn days_in_month(year: i32, month: u32) -> i32 {
    // first day next month minus one
    let (ny, nm) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_next = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap_or_default();
    (first_next - chrono::Duration::days(1)).day().cast_signed()
}

fn go_to_chrono(layout: &str) -> String {
    // Convert common Go time layout tokens to chrono format.
    let mut s = layout.to_owned();
    // Order: longest first.
    let pairs = [
        ("Monday", "%A"),
        ("January", "%B"),
        ("2006", "%Y"),
        ("Mon", "%a"),
        ("Jan", "%b"),
        ("06", "%y"),
        ("01", "%m"),
        ("02", "%d"),
        ("1", "%-m"),
        ("2", "%-d"),
    ];
    for (go, chrono) in pairs {
        s = s.replace(go, chrono);
    }
    s
}

// Implement BubbleTeaModel
impl charmed_bubbletea::Model for Date {
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
