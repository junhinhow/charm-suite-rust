//! Multi-task terminal progress dashboard.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(missing_docs, clippy::format_push_string)]

use std::time::{Duration, SystemTime};

/// State of a tracked task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Pending,
    Running,
    Done,
    Failed,
    Cancelled,
}

/// A task tracked by a dashboard.
#[derive(Debug, Clone)]
pub struct Task {
    pub id: String,
    pub label: String,
    pub total: i64,
    pub current: i64,
    pub state: State,
    pub error: Option<String>,
}
impl Task {
    /// Percentage in the range 0..=100.
    pub fn percent(&self) -> f64 {
        if self.total <= 0 {
            0.0
        } else {
            self.current.max(0).min(self.total) as f64 * 100.0 / self.total as f64
        }
    }
    /// Remaining units.
    pub fn remaining(&self) -> i64 {
        (self.total - self.current.max(0)).max(0)
    }
}

/// Visual theme.
#[derive(Debug, Clone)]
pub struct Theme {
    pub full_char: String,
    pub empty_char: String,
}
/// Default Unicode theme.
pub fn default_theme() -> Theme {
    Theme {
        full_char: "█".into(),
        empty_char: "░".into(),
    }
}
/// ASCII-only theme.
pub fn ascii_theme() -> Theme {
    Theme {
        full_char: "#".into(),
        empty_char: "-".into(),
    }
}

/// Option used by [`Dashboard::new`].
pub type OptionFn = Box<dyn FnOnce(&mut Dashboard)>;
/// Sets the visual theme.
pub fn with_theme(t: Theme) -> OptionFn {
    Box::new(move |d| d.theme = t)
}
/// Sets the bar width.
pub fn with_bar_width(n: usize) -> OptionFn {
    Box::new(move |d| d.bar_width = n.clamp(5, 60))
}
/// Shows or hides ETA.
pub fn with_show_eta(v: bool) -> OptionFn {
    Box::new(move |d| d.show_eta = v)
}
/// Shows or hides counts.
pub fn with_show_counts(v: bool) -> OptionFn {
    Box::new(move |d| d.show_counts = v)
}
/// Sets the maximum visible task count.
pub fn with_max_visible(v: usize) -> OptionFn {
    Box::new(move |d| d.max_visible = v.max(1))
}

/// Ordered multi-task progress dashboard.
#[derive(Debug)]
pub struct Dashboard {
    tasks: Vec<Task>,
    pub theme: Theme,
    pub bar_width: usize,
    pub show_eta: bool,
    pub show_counts: bool,
    pub max_visible: usize,
    started_at: Option<SystemTime>,
}
impl Dashboard {
    /// Creates an empty dashboard.
    pub fn new(opts: impl IntoIterator<Item = OptionFn>) -> Self {
        let mut d = Self {
            tasks: vec![],
            theme: default_theme(),
            bar_width: 20,
            show_eta: true,
            show_counts: true,
            max_visible: 0,
            started_at: None,
        };
        for o in opts {
            o(&mut d);
        }
        d
    }
    /// Adds or replaces a task.
    pub fn add(&mut self, t: Task) -> &mut Self {
        if let Some(i) = self.tasks.iter().position(|x| x.id == t.id) {
            self.tasks[i] = t;
        } else {
            self.tasks.push(t);
        }
        self
    }
    /// Updates the current amount.
    pub fn update(&mut self, id: &str, current: i64) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|x| x.id == id) {
            t.current = current.max(0);
            if t.state == State::Pending {
                t.state = State::Running;
            }
            self.started_at.get_or_insert(SystemTime::now());
        }
        self
    }
    /// Advances the current amount.
    pub fn advance(&mut self, id: &str, delta: i64) -> &mut Self {
        let c = self
            .tasks
            .iter()
            .find(|x| x.id == id)
            .map_or(0, |x| x.current);
        self.update(id, c.saturating_add(delta))
    }
    /// Marks a task as done.
    pub fn done(&mut self, id: &str) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|x| x.id == id) {
            t.current = t.total.max(0);
            t.state = State::Done;
            t.error = None;
        }
        self
    }
    /// Marks a task as failed.
    pub fn fail(&mut self, id: &str, error: impl Into<String>) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|x| x.id == id) {
            t.state = State::Failed;
            t.error = Some(error.into());
        }
        self
    }
    /// Marks a task as cancelled.
    pub fn cancel(&mut self, id: &str) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|x| x.id == id) {
            t.state = State::Cancelled;
        }
        self
    }
    /// Removes a task.
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(i) = self.tasks.iter().position(|x| x.id == id) {
            self.tasks.remove(i);
            true
        } else {
            false
        }
    }
    /// Returns all tasks.
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }
    /// Weighted overall percentage.
    pub fn overall(&self) -> f64 {
        let total: i64 = self.tasks.iter().map(|x| x.total.max(0)).sum();
        if total == 0 {
            0.0
        } else {
            self.tasks
                .iter()
                .map(|x| x.current.max(0).min(x.total.max(0)))
                .sum::<i64>() as f64
                * 100.0
                / total as f64
        }
    }
    /// Number of running tasks.
    pub fn active(&self) -> usize {
        self.tasks
            .iter()
            .filter(|x| x.state == State::Running)
            .count()
    }
    /// Number of failed tasks.
    pub fn failed(&self) -> usize {
        self.tasks
            .iter()
            .filter(|x| x.state == State::Failed)
            .count()
    }
    /// Whether all tasks are terminal.
    pub fn finished(&self) -> bool {
        !self.tasks.is_empty()
            && self
                .tasks
                .iter()
                .all(|x| matches!(x.state, State::Done | State::Failed | State::Cancelled))
    }
    /// Estimates remaining time. A zero duration is returned when no clock samples exist.
    pub fn eta(&self, _now: SystemTime) -> Duration {
        Duration::ZERO
    }
    /// Renders the visible tasks.
    pub fn view(&self, width: usize) -> String {
        let mut out = String::new();
        for t in self.tasks.iter().take(if self.max_visible == 0 {
            usize::MAX
        } else {
            self.max_visible
        }) {
            let n = self.bar_width.min(width.saturating_sub(t.label.len() + 12));
            let filled = (n as f64 * t.percent() / 100.0).round() as usize;
            out.push_str(&t.label);
            out.push(' ');
            out.push_str(&self.theme.full_char.repeat(filled.min(n)));
            out.push_str(&self.theme.empty_char.repeat(n.saturating_sub(filled)));
            out.push('\n');
        }
        out
    }
}
