use crate::buffer::Buffer;
use crate::export::{write_entries, ExportError, ExportedMsg};
use crate::Level;
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model as TeaModel, WindowSizeMsg};
use std::option::Option as StdOption;
use std::sync::Arc;
use std::time::SystemTime;

/// One log line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// Event timestamp. `SystemTime::UNIX_EPOCH` is used for the zero value.
    pub time: SystemTime,
    /// Severity.
    pub level: Level,
    /// Optional event source.
    pub source: String,
    /// Message text.
    pub message: String,
}

impl Entry {
    /// Creates an entry stamped with the current time.
    pub fn new(level: Level, message: impl Into<String>) -> Self {
        Self {
            time: SystemTime::now(),
            level,
            source: String::new(),
            message: message.into(),
        }
    }
    /// Sets the entry source.
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }
    /// Replaces the timestamp.
    #[must_use]
    pub fn with_time(mut self, time: SystemTime) -> Self {
        self.time = time;
        self
    }
}

/// Configuration supplied to [`Model::new`].
#[derive(Debug, Clone, Copy)]
pub struct Option {
    max_entries: usize,
}
impl Option {
    /// Sets capacity. Values lower than 10 are clamped to 10.
    pub const fn max_entries(n: usize) -> Self {
        Self {
            max_entries: if n < 10 { 10 } else { n },
        }
    }
}
impl Default for Option {
    fn default() -> Self {
        Self { max_entries: 1000 }
    }
}

/// Message emitted by an append command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppendedMsg {
    /// Number of entries appended.
    pub count: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct Positioned {
    pub(crate) seq: u64,
    pub(crate) entry: Entry,
}

/// Log viewer implementing [`charmed_bubbletea::Model`].
///
/// Clones share a synchronized buffer while retaining independent UI state.
#[derive(Debug, Clone)]
pub struct Model {
    pub(crate) buffer: StdOption<Arc<Buffer>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) min_level: Level,
    pub(crate) follow: bool,
    pub(crate) filter: String,
    cursor_seq: u64,
    pub(crate) searching: bool,
    previous_filter: String,
}

impl Default for Model {
    fn default() -> Self {
        Self::new()
    }
}
impl Model {
    /// Creates an empty viewer (1000 entries, Debug, follow enabled, 80x24).
    pub fn new() -> Self {
        Self::with_options(Option::default())
    }
    /// Creates a viewer with options.
    pub fn with_options(option: Option) -> Self {
        Self {
            buffer: Some(Arc::new(Buffer::new(option.max_entries))),
            width: 80,
            height: 24,
            min_level: Level::Debug,
            follow: true,
            filter: String::new(),
            cursor_seq: 0,
            searching: false,
            previous_filter: String::new(),
        }
    }
    fn ensure_buffer(&mut self) -> Arc<Buffer> {
        Arc::clone(
            self.buffer
                .get_or_insert_with(|| Arc::new(Buffer::new(1000))),
        )
    }
    /// Appends entries and returns a command yielding [`AppendedMsg`].
    pub fn append<I: IntoIterator<Item = Entry>>(&mut self, entries: I) -> StdOption<Cmd> {
        let entries: Vec<_> = entries.into_iter().collect();
        if entries.is_empty() {
            return None;
        }
        let b = self.ensure_buffer();
        for entry in entries.iter().cloned() {
            b.append(entry);
        }
        let count = entries.len();
        Some(Cmd::new(move || Message::new(AppendedMsg { count })))
    }
    /// Appends a single entry.
    pub fn append_one(&mut self, entry: Entry) -> StdOption<Cmd> {
        self.append([entry])
    }
    /// Sets the viewport size; negative-like values cannot occur with `usize`.
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
    /// Sets the least severe level shown.
    pub fn set_min_level(&mut self, level: Level) {
        self.min_level = level;
    }
    /// Enables or disables tail-following and selects the newest visible row.
    pub fn set_follow(&mut self, follow: bool) {
        self.follow = follow;
        self.cursor_seq = self.visible().last().map_or(0, |e| e.seq);
    }
    /// Whether the viewer follows the newest entry.
    pub const fn follow(&self) -> bool {
        self.follow
    }
    /// Sets the case-insensitive substring filter for source and message.
    pub fn set_filter(&mut self, filter: impl AsRef<str>) {
        self.filter = sanitize_text(filter.as_ref());
        self.searching = false;
        self.previous_filter.clear();
    }
    /// Current text filter.
    pub fn filter(&self) -> &str {
        &self.filter
    }
    /// Returns a copy of visible entries in insertion order.
    pub fn entries(&self) -> Vec<Entry> {
        self.visible().into_iter().map(|p| p.entry).collect()
    }
    /// Number of buffered entries, including filtered-out entries.
    pub fn len(&self) -> usize {
        self.buffer.as_ref().map_or(0, |b| b.len())
    }
    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// Clears entries while preserving filter, level, and follow state.
    pub fn clear(&mut self) {
        if let Some(b) = &self.buffer {
            b.clear();
        }
        self.cursor_seq = 0;
        self.searching = false;
        self.previous_filter.clear();
    }
    /// Exports visible entries as plain text.
    pub fn export(&self, writer: &mut dyn std::io::Write) -> Result<(), ExportError> {
        write_entries(writer, &self.entries())
    }
    pub(crate) fn visible(&self) -> Vec<Positioned> {
        let Some(buffer) = &self.buffer else {
            return Vec::new();
        };
        let (all, _) = buffer.snapshot();
        let query = self.filter.to_lowercase();
        all.into_iter()
            .filter(|(_, e)| {
                e.level >= self.min_level
                    && (query.is_empty()
                        || e.message.to_lowercase().contains(&query)
                        || e.source.to_lowercase().contains(&query))
            })
            .map(|(seq, entry)| Positioned { seq, entry })
            .collect()
    }
    pub(crate) fn selected(&self, rows: &[Positioned]) -> usize {
        if rows.is_empty() {
            return 0;
        }
        if self.follow || self.cursor_seq == 0 {
            return rows.len() - 1;
        }
        rows.iter()
            .position(|r| r.seq >= self.cursor_seq)
            .unwrap_or(rows.len() - 1)
    }
    fn key(&mut self, key: &KeyMsg) -> StdOption<Cmd> {
        if self.searching {
            return self.search_key(key);
        }
        let ch = if key.key_type == KeyType::Runes {
            key.runes.iter().collect::<String>()
        } else {
            String::new()
        };
        match key.key_type {
            KeyType::Up => self.move_up(),
            KeyType::Down => self.move_down(),
            KeyType::Runes => match ch.as_str() {
                "k" => self.move_up(),
                "j" => self.move_down(),
                "g" => self.go_top(),
                "G" => self.set_follow(true),
                "/" => {
                    self.previous_filter = self.filter.clone();
                    self.filter.clear();
                    self.searching = true;
                }
                "l" => {
                    self.min_level = match self.min_level {
                        Level::Debug => Level::Info,
                        Level::Info => Level::Warn,
                        Level::Warn => Level::Error,
                        Level::Error => Level::Debug,
                    }
                }
                "f" => self.set_follow(!self.follow),
                "c" => self.clear(),
                "e" => return Some(self.export_command()),
                _ => {}
            },
            _ => {}
        }
        None
    }
    fn search_key(&mut self, key: &KeyMsg) -> StdOption<Cmd> {
        match key.key_type {
            KeyType::Esc => {
                self.filter.clone_from(&self.previous_filter);
                self.previous_filter.clear();
                self.searching = false;
            }
            KeyType::Enter => {
                self.previous_filter.clear();
                self.searching = false;
            }
            KeyType::Backspace | KeyType::CtrlH => {
                self.filter.pop();
            }
            KeyType::Runes => self
                .filter
                .push_str(&sanitize_text(&key.runes.iter().collect::<String>())),
            _ => {}
        }
        None
    }
    fn move_up(&mut self) {
        let rows = self.visible();
        if rows.is_empty() {
            self.follow = false;
            self.cursor_seq = 0;
            return;
        }
        let mut i = if self.follow {
            rows.len() - 1
        } else {
            self.selected(&rows)
        };
        self.follow = false;
        i = i.saturating_sub(1);
        self.cursor_seq = rows[i].seq;
    }
    fn move_down(&mut self) {
        if self.follow {
            return;
        }
        let rows = self.visible();
        if rows.is_empty() {
            self.cursor_seq = 0;
            return;
        }
        let i = (self.selected(&rows) + 1).min(rows.len() - 1);
        self.cursor_seq = rows[i].seq;
    }
    fn go_top(&mut self) {
        self.follow = false;
        self.cursor_seq = self.visible().first().map_or(0, |e| e.seq);
    }
    fn export_command(&self) -> Cmd {
        let rows = self.entries();
        Cmd::new(move || {
            let result = std::fs::File::create("logs.txt")
                .map_err(ExportError::from)
                .and_then(|mut f| write_entries(&mut f, &rows));
            Message::new(ExportedMsg {
                path: "logs.txt".into(),
                error: result.err().map(|e| e.to_string()),
            })
        })
    }
}

impl TeaModel for Model {
    fn init(&self) -> StdOption<Cmd> {
        None
    }
    fn update(&mut self, msg: Message) -> StdOption<Cmd> {
        match msg.try_downcast::<WindowSizeMsg>() {
            Ok(size) => {
                self.set_size(size.width.into(), size.height.into());
                None
            }
            Err(msg) => match msg.try_downcast::<KeyMsg>() {
                Ok(key) => self.key(&key),
                Err(_) => None,
            },
        }
    }
    fn view(&self) -> String {
        crate::view::render(self)
    }
}

pub(crate) fn sanitize_text(s: &str) -> String {
    charm_core::strip_ansi(s)
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
}
