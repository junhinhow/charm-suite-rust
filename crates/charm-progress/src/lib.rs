//! Multi-task terminal progress dashboard.
#![forbid(unsafe_code)]
#![allow(missing_docs, clippy::format_push_string)]

use lipgloss::{Color, Style};
use std::{
    fmt,
    time::{Duration, SystemTime},
};
use unicode_width::UnicodeWidthStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum State {
    #[default]
    Pending,
    Running,
    Done,
    Failed,
    Cancelled,
}

impl State {
    fn terminal(self) -> bool {
        matches!(self, Self::Done | Self::Failed | Self::Cancelled)
    }
}

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
    pub fn percent(&self) -> f64 {
        if self.total <= 0 {
            0.0
        } else {
            self.current.clamp(0, self.total) as f64 / self.total as f64 * 100.0
        }
    }
    pub fn remaining(&self) -> i64 {
        if self.total <= 0 {
            0
        } else {
            self.total.saturating_sub(self.current.max(0)).max(0)
        }
    }
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub bar_full: Style,
    pub bar_empty: Style,
    pub pending: Style,
    pub running: Style,
    pub done: Style,
    pub failed: Style,
    pub cancelled: Style,
    pub label: Style,
    pub muted: Style,
    pub full_char: String,
    pub empty_char: String,
    pub pending_symbol: String,
    pub running_symbol: String,
    pub done_symbol: String,
    pub failed_symbol: String,
    pub cancelled_symbol: String,
    pub ellipsis: String,
}
impl Theme {
    fn normalized(mut self) -> Self {
        let d = default_theme();
        macro_rules! fill {
            ($field:ident) => {
                if self.$field.is_empty() {
                    self.$field = d.$field.clone();
                }
            };
        }
        fill!(full_char);
        fill!(empty_char);
        fill!(pending_symbol);
        fill!(running_symbol);
        fill!(done_symbol);
        fill!(failed_symbol);
        fill!(cancelled_symbol);
        fill!(ellipsis);
        self
    }
    fn symbol(&self, s: State) -> &str {
        match s {
            State::Running => &self.running_symbol,
            State::Done => &self.done_symbol,
            State::Failed => &self.failed_symbol,
            State::Cancelled => &self.cancelled_symbol,
            State::Pending => &self.pending_symbol,
        }
    }
    fn state_style(&self, s: State) -> &Style {
        match s {
            State::Running => &self.running,
            State::Done => &self.done,
            State::Failed => &self.failed,
            State::Cancelled => &self.cancelled,
            State::Pending => &self.pending,
        }
    }
}
pub fn default_theme() -> Theme {
    Theme {
        bar_full: Style::new().foreground(Color::from("#7D56F4")),
        bar_empty: Style::new().foreground(Color::from("240")),
        pending: Style::new().foreground(Color::from("240")),
        running: Style::new().foreground(Color::from("39")).bold(true),
        done: Style::new().foreground(Color::from("42")),
        failed: Style::new().foreground(Color::from("196")).bold(true),
        cancelled: Style::new().foreground(Color::from("244")),
        label: Style::new(),
        muted: Style::new().foreground(Color::from("244")),
        full_char: "█".into(),
        empty_char: "░".into(),
        pending_symbol: "○".into(),
        running_symbol: "◐".into(),
        done_symbol: "✔".into(),
        failed_symbol: "✖".into(),
        cancelled_symbol: "⊘".into(),
        ellipsis: "…".into(),
    }
}
pub fn ascii_theme() -> Theme {
    Theme {
        bar_full: Style::new(),
        bar_empty: Style::new(),
        pending: Style::new(),
        running: Style::new(),
        done: Style::new(),
        failed: Style::new(),
        cancelled: Style::new(),
        label: Style::new(),
        muted: Style::new(),
        full_char: "#".into(),
        empty_char: "-".into(),
        pending_symbol: ".".into(),
        running_symbol: ">".into(),
        done_symbol: "+".into(),
        failed_symbol: "x".into(),
        cancelled_symbol: "-".into(),
        ellipsis: "...".into(),
    }
}

pub type OptionFn = Box<dyn FnOnce(&mut Dashboard)>;
pub fn with_theme(t: Theme) -> OptionFn {
    Box::new(move |d| d.theme = t.normalized())
}
pub fn with_bar_width(n: usize) -> OptionFn {
    Box::new(move |d| d.bar_width = n.clamp(5, 60))
}
pub fn with_show_eta(v: bool) -> OptionFn {
    Box::new(move |d| d.show_eta = v)
}
pub fn with_show_counts(v: bool) -> OptionFn {
    Box::new(move |d| d.show_counts = v)
}
pub fn with_max_visible(v: usize) -> OptionFn {
    Box::new(move |d| d.max_visible = v.max(1))
}

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
    pub fn add(&mut self, mut t: Task) -> &mut Self {
        t.current = t.current.max(0);
        if let Some(i) = self.tasks.iter().position(|x| x.id == t.id) {
            self.tasks[i] = t;
        } else {
            self.tasks.push(t);
        }
        self
    }
    pub fn update(&mut self, id: &str, current: i64) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.current = current.max(0);
            if t.state == State::Pending {
                t.state = State::Running;
            }
            self.started_at.get_or_insert_with(SystemTime::now);
        }
        self
    }
    pub fn advance(&mut self, id: &str, delta: i64) -> &mut Self {
        if let Some(t) = self.tasks.iter().find(|t| t.id == id) {
            self.update(id, t.current.saturating_add(delta).max(0));
        }
        self
    }
    pub fn done(&mut self, id: &str) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            if t.total > 0 {
                t.current = t.total;
            }
            t.state = State::Done;
            t.error = None;
        }
        self
    }
    pub fn fail(&mut self, id: &str, error: impl fmt::Display) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.state = State::Failed;
            t.error = Some(error.to_string());
        }
        self
    }
    pub fn cancel(&mut self, id: &str) -> &mut Self {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.id == id) {
            t.state = State::Cancelled;
            t.error = None;
        }
        self
    }
    pub fn remove(&mut self, id: &str) -> bool {
        if let Some(i) = self.tasks.iter().position(|t| t.id == id) {
            self.tasks.remove(i);
            true
        } else {
            false
        }
    }
    pub fn task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }
    pub fn tasks(&self) -> &[Task] {
        &self.tasks
    }
    pub fn overall(&self) -> f64 {
        if self.tasks.is_empty() {
            return 0.0;
        }
        let weight: f64 = self
            .tasks
            .iter()
            .filter(|t| t.total > 0)
            .map(|t| t.total as f64)
            .sum();
        if weight > 0.0 {
            self.tasks
                .iter()
                .filter(|t| t.total > 0)
                .map(|t| t.current.clamp(0, t.total) as f64)
                .sum::<f64>()
                / weight
                * 100.0
        } else {
            self.tasks.iter().filter(|t| t.state == State::Done).count() as f64
                / self.tasks.len() as f64
                * 100.0
        }
    }
    pub fn eta(&self, now: SystemTime) -> Duration {
        let Some(start) = self.started_at else {
            return Duration::ZERO;
        };
        let Some(elapsed) = now.duration_since(start).ok().filter(|d| !d.is_zero()) else {
            return Duration::ZERO;
        };
        let mut p = 0i64;
        let mut r = 0i64;
        let mut n = 0;
        for t in &self.tasks {
            if t.state == State::Running && t.total > 0 {
                n += 1;
                p = p.saturating_add(t.current.clamp(0, t.total));
                r = r.saturating_add(t.remaining());
            }
        }
        if n == 0 || p <= 0 || r <= 0 {
            return Duration::ZERO;
        }
        Duration::from_secs_f64((r as f64 * elapsed.as_secs_f64() / p as f64).min(u64::MAX as f64))
    }
    pub fn active(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.state == State::Running)
            .count()
    }
    pub fn failed(&self) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.state == State::Failed)
            .count()
    }
    pub fn finished(&self) -> bool {
        !self.tasks.is_empty() && self.tasks.iter().all(|t| t.state.terminal())
    }
    pub fn view(&self, width: usize) -> String {
        if width == 0 || self.tasks.is_empty() {
            return String::new();
        }
        let count = if self.max_visible == 0 {
            self.tasks.len()
        } else {
            self.max_visible.min(self.tasks.len())
        };
        let visible = &self.tasks[..count];
        let mut lines: Vec<String> = visible.iter().map(|t| self.view_task(t, width)).collect();
        if count < self.tasks.len() {
            let hidden = self.tasks.len() - count;
            lines.push(truncate(
                &self.theme.muted.render(&format!(
                    "{} +{} {}",
                    self.theme.ellipsis,
                    hidden,
                    if hidden == 1 { "tarefa" } else { "tarefas" }
                )),
                width,
                "",
            ));
        }
        lines.push(self.summary(width));
        lines.join("\n")
    }
    pub fn view_task(&self, t: &Task, width: usize) -> String {
        if width == 0 {
            return String::new();
        }
        let label = sanitize(if t.label.is_empty() { &t.id } else { &t.label });
        let label = if label.is_empty() {
            "tarefa".into()
        } else {
            label
        };
        let prefix = format!(
            "{} ",
            self.theme
                .state_style(t.state)
                .render(self.theme.symbol(t.state))
        );
        let counts = self.show_counts;
        let count_text = if counts {
            format!("{}  ", format_counts(t))
        } else {
            String::new()
        };
        let suffix = self
            .theme
            .muted
            .render(&format!("{count_text}{:3.0}%", t.percent()));
        let pw = display_width(&prefix);
        let sw = self
            .tasks
            .iter()
            .map(|task| {
                let count_text = if counts {
                    format!("{}  ", format_counts(task))
                } else {
                    String::new()
                };
                display_width(
                    &self
                        .theme
                        .muted
                        .render(&format!("{count_text}{:3.0}%", task.percent())),
                )
            })
            .max()
            .unwrap_or_else(|| display_width(&suffix));
        let avail = width.saturating_sub(pw + sw + 1);
        let bw = if avail >= 17 {
            self.bar_width.min(avail - 13)
        } else {
            0
        };
        let lw = avail.saturating_sub(if bw > 0 { bw + 1 } else { 0 });
        let mut text = prefix;
        if lw == 0 {
            return truncate(&format!("{}{:3.0}%", text, t.percent()), width, "");
        }
        let l = truncate(&label, lw, &self.theme.ellipsis);
        text.push_str(&l);
        text.push_str(&" ".repeat(lw.saturating_sub(display_width(&l))));
        text.push(' ');
        if bw > 0 {
            text.push_str(&self.bar(t.percent(), bw));
            text.push(' ');
        }
        text.push_str(&" ".repeat(sw.saturating_sub(display_width(&suffix))));
        text.push_str(&suffix);
        truncate(&text, width, "")
    }
    fn bar(&self, p: f64, n: usize) -> String {
        let filled = ((n as f64 * p.clamp(0.0, 100.0) / 100.0).floor() as usize).min(n);
        format!(
            "{}{}",
            self.theme
                .bar_full
                .render(&repeat_width(&self.theme.full_char, filled)),
            self.theme
                .bar_empty
                .render(&repeat_width(&self.theme.empty_char, n - filled))
        )
    }
    fn summary(&self, width: usize) -> String {
        let p = self.overall();
        let bar_width = if width >= 32 {
            self.bar_width.min(width.saturating_sub(25))
        } else {
            0
        };
        let mut line = "total".to_owned();
        if bar_width >= 5 {
            line.push_str(&format!(" {}", self.bar(p, bar_width)));
        }
        line.push_str(&format!(" {p:3.0}%"));
        if self.show_counts {
            let done = self.tasks.iter().filter(|t| t.state == State::Done).count();
            line.push_str(&format!(
                "  {}/{} {}",
                done,
                self.tasks.len(),
                if self.tasks.len() == 1 {
                    "tarefa"
                } else {
                    "tarefas"
                }
            ));
        }
        if self.show_eta {
            if let Some(now) = self
                .started_at
                .and_then(|s| s.elapsed().ok().map(|e| s + e))
            {
                let eta = self.eta(now);
                if !eta.is_zero() {
                    line.push_str(&format!("  ETA {}", format_duration(eta)));
                }
            }
        }
        truncate(&self.theme.label.render(&line), width, "")
    }
}
fn display_width(s: &str) -> usize {
    let plain = strip_ansi_escapes::strip(s.as_bytes());
    UnicodeWidthStr::width(std::str::from_utf8(&plain).unwrap_or(s))
}
fn truncate(s: &str, width: usize, tail: &str) -> String {
    if display_width(s) <= width {
        return s.to_owned();
    }
    let plain = strip_ansi_escapes::strip(s.as_bytes());
    let plain = String::from_utf8_lossy(&plain);
    let mut out = String::new();
    let mut w = 0;
    for c in plain.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if w + cw > width.saturating_sub(display_width(tail)) {
            break;
        }
        out.push(c);
        w += cw;
    }
    out.push_str(tail);
    out
}
fn repeat_width(ch: &str, n: usize) -> String {
    if n == 0 {
        return String::new();
    }
    let w = UnicodeWidthStr::width(ch);
    if w == 0 {
        return " ".repeat(n);
    }
    let mut s = ch.repeat(n / w);
    s.push_str(&" ".repeat(n % w));
    s
}
fn sanitize(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect::<String>()
        .trim()
        .to_owned()
}
fn format_counts(t: &Task) -> String {
    if t.total <= 0 {
        t.current.max(0).to_string()
    } else {
        format!("{}/{}", t.current.max(0), t.total)
    }
}
fn format_duration(d: Duration) -> String {
    let s = d.as_secs();
    if d.is_zero() {
        "0s".into()
    } else if s == 0 {
        "<1s".into()
    } else if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m{:02}s", s / 60, s % 60)
    } else if s < 86400 {
        format!("{}h{:02}m", s / 3600, (s % 3600) / 60)
    } else {
        format!("{}d{:02}h", s / 86400, (s % 86400) / 3600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(id: &str, label: &str, total: i64) -> Task {
        Task {
            id: id.into(),
            label: label.into(),
            total,
            current: 0,
            state: State::Pending,
            error: None,
        }
    }

    #[test]
    fn task_progress_is_clamped_and_safe() {
        assert_eq!(task("a", "", 0).percent(), 0.0);
        assert_eq!(task("a", "", 100).remaining(), 100);
        let mut t = task("a", "", 100);
        t.current = 150;
        assert_eq!(t.percent(), 100.0);
        assert_eq!(t.remaining(), 0);
    }

    #[test]
    fn dashboard_transitions_and_aggregate_match_contract() {
        let mut d = Dashboard::new([]);
        d.add(task("a", "one", 100)).add(task("b", "two", 300));
        d.update("a", 50).update("b", 150);
        assert_eq!(d.overall(), 50.0);
        assert_eq!(d.active(), 2);
        d.fail("a", "network");
        assert_eq!(d.failed(), 1);
        assert_eq!(d.task("a").unwrap().error.as_deref(), Some("network"));
        d.done("b");
        assert!(d.finished());
        d.cancel("a");
        assert!(d.finished());
    }

    #[test]
    fn duplicate_add_keeps_order_and_max_visible_adds_notice() {
        let mut d = Dashboard::new([
            with_theme(ascii_theme()),
            with_show_eta(false),
            with_max_visible(1),
        ]);
        d.add(task("a", "first", 10)).add(task("b", "second", 10));
        d.add(Task {
            label: "replaced".into(),
            ..task("a", "", 10)
        });
        assert_eq!(d.tasks()[0].label, "replaced");
        let out = d.view(60);
        assert!(out.contains("... +1 tarefa"));
        assert!(!out.contains("second"));
    }

    #[test]
    fn ascii_render_golden_example() {
        let mut d = Dashboard::new([
            with_theme(ascii_theme()),
            with_bar_width(10),
            with_show_eta(false),
        ]);
        d.add(task("dl-1", "baixar-1", 100))
            .add(task("dl-2", "baixar-2", 100));
        d.update("dl-1", 60).done("dl-2");
        assert_eq!(d.view(46), "> baixar-1            ######----  60/100   60%\n+ baixar-2            ########## 100/100  100%\ntotal ########--  80%  1/2 tarefas");
    }

    #[test]
    fn labels_are_sanitized_and_lines_respect_width() {
        let mut d = Dashboard::new([with_theme(ascii_theme()), with_show_eta(false)]);
        d.add(Task {
            label: "first\nsecond\tline".into(),
            ..task("id", "", 10)
        });
        let out = d.view(28);
        assert_eq!(out.lines().count(), 2);
        assert!(!out.contains('\t'));
        assert!(out.lines().all(|l| display_width(l) <= 28));
    }

    #[test]
    fn eta_uses_running_tasks_and_elapsed_rate() {
        let start = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let mut d = Dashboard::new([]);
        d.started_at = Some(start);
        d.add(Task {
            state: State::Running,
            current: 20,
            ..task("a", "a", 100)
        });
        d.add(Task {
            state: State::Running,
            current: 10,
            ..task("b", "b", 50)
        });
        assert_eq!(
            d.eta(start + Duration::from_secs(10)),
            Duration::from_secs(40)
        );
        assert_eq!(d.eta(start - Duration::from_secs(1)), Duration::ZERO);
    }

    #[test]
    fn remove_and_update_preserve_terminal_state() {
        let mut d = Dashboard::new([]);
        d.add(task("a", "a", 10)).add(task("b", "b", 10));
        d.update("a", 4).done("a").update("a", 2);
        assert_eq!(d.task("a").unwrap().state, State::Done);
        assert_eq!(d.task("a").unwrap().current, 2);
        assert!(d.remove("a"));
        assert!(!d.remove("a"));
        assert!(d.task("b").is_some());
    }

    #[test]
    fn ascii_theme_and_count_option_remain_ascii_and_respect_width() {
        let mut d = Dashboard::new([
            with_theme(ascii_theme()),
            with_show_counts(false),
            with_show_eta(false),
        ]);
        d.add(Task {
            current: 5,
            state: State::Running,
            ..task("a", "long task label", 10)
        });
        let out = d.view(17);
        assert!(out.lines().all(|line| display_width(line) <= 17));
        assert!(!out.contains("5/10"));
        assert!(out.chars().all(|ch| ch.is_ascii() || ch.is_control()));
    }
}
