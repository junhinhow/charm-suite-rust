//! Command palette overlay for Bubble Tea applications.
//!
//! The palette filters commands by accent-insensitive subsequence matching,
//! ranks the matches, and provides a keyboard-driven centered overlay.
//!
//! ```
//! use charm_command::{Command, Palette};
//!
//! let mut palette = Palette::new(vec![Command::new("save", "Save file")]);
//! palette.set_size(80, 24);
//! palette.open();
//! palette.set_query("sav");
//! assert_eq!(palette.selected().map(|command| command.id), Some("save".to_owned()));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]

use std::fmt;
use std::option::Option as StdOption;
use std::sync::{Arc, Mutex, MutexGuard};

use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use lipgloss::{Color, Style};
use unicode_width::UnicodeWidthStr;

const DEFAULT_WIDTH: usize = 80;
const DEFAULT_HEIGHT: usize = 24;
const DEFAULT_MAX_VISIBLE: usize = 8;
const MAX_BOX_WIDTH: usize = 64;

/// An action presented in the palette.
#[derive(Clone)]
pub struct Command {
    /// Stable command identifier. Non-empty duplicate IDs replace in place.
    pub id: String,
    /// Primary label shown and searched.
    pub title: String,
    /// Secondary description shown and searched.
    pub description: String,
    /// Optional section heading and searchable text.
    pub group: String,
    /// Shortcut labels displayed at the right of a row.
    pub keys: Vec<String>,
    /// Action invoked when Enter selects this command.
    pub run: StdOption<Arc<dyn Fn() -> StdOption<Cmd> + Send + Sync>>,
}

impl fmt::Debug for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Command")
            .field("id", &self.id)
            .field("title", &self.title)
            .field("description", &self.description)
            .field("group", &self.group)
            .field("keys", &self.keys)
            .field("has_run", &self.run.is_some())
            .finish()
    }
}

impl Command {
    /// Creates a command with an ID and title.
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            group: String::new(),
            keys: Vec::new(),
            run: None,
        }
    }

    /// Sets the explanatory description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Sets the group heading.
    #[must_use]
    pub fn with_group(mut self, group: impl Into<String>) -> Self {
        self.group = group.into();
        self
    }

    /// Sets the keyboard shortcuts.
    #[must_use]
    pub fn with_keys(mut self, keys: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.keys = keys.into_iter().map(Into::into).collect();
        self
    }

    /// Sets the action to invoke on Enter.
    #[must_use]
    pub fn with_run<F>(mut self, run: F) -> Self
    where
        F: Fn() -> StdOption<Cmd> + Send + Sync + 'static,
    {
        self.run = Some(Arc::new(run));
        self
    }
}

/// Lipgloss styles used by the palette renderer.
#[derive(Clone)]
pub struct Theme {
    /// Base background and foreground style.
    pub background: Style,
    /// Box border and optional padding style.
    pub border: Style,
    /// Palette title.
    pub title: Style,
    /// Current query and prompt.
    pub query: Style,
    /// Unselected command rows.
    pub item: Style,
    /// Selected command row.
    pub selected_item: Style,
    /// Descriptions and placeholder.
    pub description: Style,
    /// Group headings.
    pub group: Style,
    /// Keyboard shortcuts.
    pub shortcut: Style,
}

impl fmt::Debug for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Theme").finish_non_exhaustive()
    }
}

/// Returns the default dark theme.
pub fn default_theme() -> Theme {
    Theme {
        background: Style::new()
            .background(Color::from("#1E1E2E"))
            .foreground(Color::from("#CDD6F4")),
        border: Style::new()
            .border(lipgloss::rounded_border())
            .border_foreground(Color::from("#7D56F4"))
            .padding_2(0, 1),
        title: Style::new().bold(true).foreground(Color::from("#FAFAFA")),
        query: Style::new().foreground(Color::from("#89B4FA")),
        item: Style::new().foreground(Color::from("#CDD6F4")),
        selected_item: Style::new()
            .bold(true)
            .foreground(Color::from("#FAFAFA"))
            .background(Color::from("#7D56F4")),
        description: Style::new().foreground(Color::from("#6C7086")),
        group: Style::new().bold(true).foreground(Color::from("#F9E2AF")),
        shortcut: Style::new().foreground(Color::from("#94E2D5")),
    }
}

/// Configuration applied when creating a [`Palette`].
#[derive(Clone)]
pub enum Option {
    /// Maximum number of commands displayed at once (minimum one).
    MaxVisible(usize),
    /// Title shown above the query; empty hides it.
    Title(String),
    /// Replaces the renderer theme.
    Theme(Box<Theme>),
}

impl fmt::Debug for Option {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MaxVisible(count) => f.debug_tuple("MaxVisible").field(count).finish(),
            Self::Title(title) => f.debug_tuple("Title").field(title).finish(),
            Self::Theme(_) => f.write_str("Theme(..)"),
        }
    }
}

/// Sets the maximum visible number of command rows.
pub fn with_max_visible(n: usize) -> Option {
    Option::MaxVisible(n.max(1))
}

/// Sets the palette title; an empty string hides it.
pub fn with_title(title: impl Into<String>) -> Option {
    Option::Title(title.into())
}

/// Sets a custom renderer theme.
pub fn with_theme(theme: Theme) -> Option {
    Option::Theme(Box::new(theme))
}

#[derive(Clone)]
struct State {
    commands: Vec<Command>,
    query: String,
    cursor: usize,
    open: bool,
    width: usize,
    height: usize,
    title: String,
    placeholder: String,
    max_visible: usize,
    theme: Theme,
}

impl Default for State {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            query: String::new(),
            cursor: 0,
            open: false,
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            title: "Commands".to_owned(),
            placeholder: "Type a command…".to_owned(),
            max_visible: DEFAULT_MAX_VISIBLE,
            theme: default_theme(),
        }
    }
}

/// Bubble Tea command palette model.
///
/// Clones share their state, so a model returned by [`Model::update`] and the
/// original handle always observe the same selection and query.
#[derive(Clone)]
pub struct Palette {
    state: Arc<Mutex<State>>,
}

impl fmt::Debug for Palette {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.lock();
        f.debug_struct("Palette")
            .field("commands", &state.commands)
            .field("query", &state.query)
            .field("cursor", &state.cursor)
            .field("open", &state.open)
            .finish_non_exhaustive()
    }
}

impl Default for Palette {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Palette {
    /// Creates a palette with the supplied commands and optional settings.
    ///
    /// Duplicate non-empty IDs are replaced by their last occurrence without
    /// changing the first occurrence's position. Empty IDs are retained.
    pub fn new(commands: Vec<Command>) -> Self {
        Self::with_options(commands, &[])
    }

    /// Creates a palette and applies construction options.
    pub fn with_options(commands: Vec<Command>, options: &[Option]) -> Self {
        let mut state = State {
            commands: normalize(commands),
            ..State::default()
        };
        for option in options {
            match option {
                Option::MaxVisible(n) => state.max_visible = (*n).max(1),
                Option::Title(title) => state.title.clone_from(title),
                Option::Theme(theme) => state.theme.clone_from(theme),
            }
        }
        Self {
            state: Arc::new(Mutex::new(state)),
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// Replaces the command list and resets the selection.
    pub fn set_commands(&mut self, commands: Vec<Command>) {
        let mut state = self.lock();
        state.commands = normalize(commands);
        state.cursor = 0;
    }

    /// Returns a defensive copy in insertion order.
    #[must_use]
    pub fn commands(&self) -> Vec<Command> {
        self.lock().commands.clone()
    }

    /// Adds commands, replacing matching non-empty IDs in place.
    pub fn add(&mut self, commands: impl IntoIterator<Item = Command>) {
        let mut state = self.lock();
        for command in commands {
            if !command.id.is_empty() {
                if let Some(existing) = state.commands.iter_mut().find(|c| c.id == command.id) {
                    *existing = command;
                    continue;
                }
            }
            state.commands.push(command);
        }
        clamp_cursor(&mut state);
    }

    /// Removes the first command with `id` and reports whether one existed.
    pub fn remove(&mut self, id: &str) -> bool {
        let mut state = self.lock();
        let Some(index) = state.commands.iter().position(|command| command.id == id) else {
            return false;
        };
        state.commands.remove(index);
        clamp_cursor(&mut state);
        true
    }

    /// Opens the palette. The Rust Bubble Tea runtime has no nil command, so
    /// this operation returns `None`.
    pub fn open(&mut self) -> StdOption<Cmd> {
        self.lock().open = true;
        None
    }

    /// Closes the palette without changing the query or selection.
    pub fn close(&mut self) {
        self.lock().open = false;
    }

    /// Toggles visibility and returns no command.
    pub fn toggle(&mut self) -> StdOption<Cmd> {
        let mut state = self.lock();
        state.open = !state.open;
        None
    }

    /// Reports whether the palette is open.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.lock().open
    }

    /// Sets the search query and resets selection when it changes.
    pub fn set_query(&mut self, query: impl Into<String>) {
        let query = query.into();
        let mut state = self.lock();
        if state.query != query {
            state.query = query;
            state.cursor = 0;
        }
    }

    /// Returns the current search query.
    #[must_use]
    pub fn query(&self) -> String {
        self.lock().query.clone()
    }

    /// Returns commands matching the current query, sorted by score then title.
    #[must_use]
    pub fn filtered(&self) -> Vec<Command> {
        let state = self.lock();
        filter_commands(&state.commands, &state.query)
    }

    /// Returns the selected command, if the filtered list is non-empty.
    #[must_use]
    pub fn selected(&self) -> StdOption<Command> {
        let state = self.lock();
        filter_commands(&state.commands, &state.query)
            .get(state.cursor)
            .cloned()
    }

    /// Sets the viewport dimensions. Negative signed values clamp to zero.
    pub fn set_size(&mut self, width: i32, height: i32) {
        let mut state = self.lock();
        state.width = width.max(0) as usize;
        state.height = height.max(0) as usize;
    }

    /// Sets the placeholder rendered when the query is empty.
    pub fn set_placeholder(&mut self, placeholder: impl Into<String>) {
        self.lock().placeholder = placeholder.into();
    }

    /// Sets the maximum number of visible commands, clamped to at least one.
    pub fn set_max_visible(&mut self, count: usize) {
        self.lock().max_visible = count.max(1);
    }

    /// Replaces the rendering theme.
    pub fn set_theme(&mut self, theme: Theme) {
        self.lock().theme = theme;
    }

    /// Renders the centered overlay. Closed or zero-sized palettes are empty.
    #[must_use]
    pub fn render(&self) -> String {
        let state = self.lock();
        render(&state)
    }

    fn handle_key(&mut self, key: KeyMsg) -> StdOption<Cmd> {
        let mut state = self.lock();
        if !state.open {
            return None;
        }
        match key.key_type {
            KeyType::Esc => {
                state.open = false;
                state.query.clear();
                state.cursor = 0;
            }
            KeyType::Enter => {
                let selected = filter_commands(&state.commands, &state.query)
                    .get(state.cursor)
                    .and_then(|command| command.run.clone());
                state.open = false;
                drop(state);
                return selected.and_then(|run| run());
            }
            KeyType::Up | KeyType::CtrlP => move_cursor(&mut state, -1),
            KeyType::Down | KeyType::CtrlN => move_cursor(&mut state, 1),
            KeyType::PgUp => page(&mut state, -1),
            KeyType::PgDown => page(&mut state, 1),
            KeyType::Home => state.cursor = 0,
            KeyType::End => {
                let count = filter_commands(&state.commands, &state.query).len();
                state.cursor = count.saturating_sub(1);
            }
            KeyType::Backspace | KeyType::CtrlH => {
                state.query.pop();
                state.cursor = 0;
            }
            KeyType::CtrlU => {
                state.query.clear();
                state.cursor = 0;
            }
            KeyType::Runes if !key.alt && !key.paste => {
                if !key.runes.is_empty() {
                    state.query.extend(key.runes);
                    state.cursor = 0;
                }
            }
            KeyType::Space if !key.alt && !key.paste => {
                state.query.push(' ');
                state.cursor = 0;
            }
            _ => {}
        }
        None
    }
}

impl Model for Palette {
    fn init(&self) -> StdOption<Cmd> {
        None
    }

    fn update(&mut self, message: Message) -> StdOption<Cmd> {
        match message.downcast::<KeyMsg>() {
            Some(key) => self.handle_key(key),
            None => None,
        }
    }

    fn view(&self) -> String {
        self.render()
    }
}

#[derive(Clone)]
struct Candidate {
    command: Command,
    score: i32,
    title_key: String,
    order: usize,
}

fn normalize(commands: Vec<Command>) -> Vec<Command> {
    let mut output: Vec<Command> = Vec::with_capacity(commands.len());
    for command in commands {
        if !command.id.is_empty() {
            if let Some(index) = output.iter().position(|c| c.id == command.id) {
                output[index] = command;
                continue;
            }
        }
        output.push(command);
    }
    output
}

fn fold(text: &str) -> String {
    text.chars().flat_map(fold_char).collect()
}

fn fold_char(ch: char) -> Vec<char> {
    let lower = ch.to_lowercase().next().unwrap_or(ch);
    let folded = match lower {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'ā' | 'ă' | 'ą' | 'æ' => 'a',
        'ç' | 'ć' | 'ĉ' | 'ċ' | 'č' => 'c',
        'ď' | 'đ' | 'ð' => 'd',
        'è' | 'é' | 'ê' | 'ë' | 'ē' | 'ĕ' | 'ė' | 'ę' | 'ě' => 'e',
        'ĝ' | 'ğ' | 'ġ' | 'ģ' => 'g',
        'ĥ' | 'ħ' => 'h',
        'ì' | 'í' | 'î' | 'ï' | 'ĩ' | 'ī' | 'ĭ' | 'į' | 'ı' => 'i',
        'ĵ' => 'j',
        'ķ' => 'k',
        'ĺ' | 'ļ' | 'ľ' | 'ł' => 'l',
        'ñ' | 'ń' | 'ņ' | 'ň' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' | 'ō' | 'ŏ' | 'ő' | 'œ' => 'o',
        'ŕ' | 'ŗ' | 'ř' => 'r',
        'ś' | 'ŝ' | 'ş' | 'š' | 'ß' => 's',
        'ţ' | 'ť' | 'ŧ' | 'þ' => 't',
        'ù' | 'ú' | 'û' | 'ü' | 'ũ' | 'ū' | 'ŭ' | 'ů' | 'ű' | 'ų' => 'u',
        'ŵ' => 'w',
        'ý' | 'ÿ' | 'ŷ' => 'y',
        'ź' | 'ż' | 'ž' => 'z',
        _ => lower,
    };
    vec![folded]
}

fn filter_commands(commands: &[Command], query: &str) -> Vec<Command> {
    let pattern = fold(query.trim());
    let mut found = Vec::with_capacity(commands.len());
    for (order, command) in commands.iter().enumerate() {
        let score = if pattern.is_empty() {
            0
        } else {
            let (score, matched) = command_score(&pattern, command);
            if !matched {
                continue;
            }
            score
        };
        found.push(Candidate {
            command: command.clone(),
            score,
            title_key: fold(&command.title),
            order,
        });
    }
    found.sort_by(|a, b| {
        b.score
            .cmp(&a.score)
            .then_with(|| a.title_key.cmp(&b.title_key))
            .then_with(|| a.order.cmp(&b.order))
    });
    found
        .into_iter()
        .map(|candidate| candidate.command)
        .collect()
}

fn command_score(pattern: &str, command: &Command) -> (i32, bool) {
    let (title, title_ok) = match_score(pattern, &command.title);
    let (group, group_ok) = match_score(pattern, &command.group);
    let (description, description_ok) = match_score(pattern, &command.description);
    let score = title * 3
        + group * 2
        + description
        + if title_ok && (group_ok || description_ok) {
            15
        } else {
            0
        };
    (score, title_ok || group_ok || description_ok)
}

fn match_score(pattern: &str, text: &str) -> (i32, bool) {
    if pattern.is_empty() {
        return (0, true);
    }
    let pattern: Vec<char> = pattern.chars().collect();
    let text: Vec<char> = fold(text).chars().collect();
    if pattern.len() > text.len() {
        return (0, false);
    }
    let mut positions = Vec::with_capacity(pattern.len());
    let mut cursor = 0;
    for wanted in &pattern {
        let Some(relative) = text[cursor..].iter().position(|ch| ch == wanted) else {
            return (0, false);
        };
        cursor += relative;
        positions.push(cursor);
        cursor += 1;
    }
    let mut score = 100;
    if pattern == text {
        score += 700;
    } else if text.starts_with(&pattern) {
        score += 400;
    } else if boundary(&text, positions[0]) {
        score += 200;
    }
    if positions[0] < 40 {
        score += 40 - i32::try_from(positions[0]).unwrap_or(40);
    }
    score -= i32::try_from(
        (positions[positions.len() - 1] - positions[0] - (positions.len() - 1)).min(30),
    )
    .unwrap_or(30);
    for pair in positions.windows(2) {
        if pair[1] == pair[0] + 1 {
            score += 15;
        } else if boundary(&text, pair[1]) {
            score += 8;
        }
    }
    (score, true)
}

fn boundary(text: &[char], index: usize) -> bool {
    index == 0
        || matches!(
            text[index - 1],
            ' ' | '\t' | '-' | '_' | '.' | '/' | ':' | '\\' | '(' | '['
        )
}

fn clamp_cursor(state: &mut State) {
    state.cursor = state.cursor.min(
        filter_commands(&state.commands, &state.query)
            .len()
            .saturating_sub(1),
    );
}

fn move_cursor(state: &mut State, delta: isize) {
    let count = filter_commands(&state.commands, &state.query).len();
    state.cursor = state
        .cursor
        .saturating_add_signed(delta)
        .min(count.saturating_sub(1));
}

fn page(state: &mut State, direction: isize) {
    let step = isize::try_from(state.max_visible).unwrap_or(isize::MAX);
    move_cursor(state, direction * step);
}

fn render(state: &State) -> String {
    if !state.open || state.width == 0 || state.height == 0 {
        return String::new();
    }
    let width = state.width.min(MAX_BOX_WIDTH);
    let border_enabled = width >= 3 && state.height >= 3;
    let horizontal_border = usize::from(border_enabled) * 2;
    let padding = usize::from(border_enabled) * 2;
    let inner_width = width.saturating_sub(horizontal_border + padding).max(1);
    let vertical_border = usize::from(border_enabled) * 2;
    let inner_height = state.height.saturating_sub(vertical_border).max(1);
    let mut lines = content_lines(state, inner_width, inner_height);
    if border_enabled {
        for line in &mut lines {
            let plain_width = lipgloss::width(line);
            let padding = inner_width.saturating_sub(plain_width);
            *line = format!("│ {}{} │", line, " ".repeat(padding));
        }
        let border = state.theme.border.get_border_style();
        lines.insert(
            0,
            format!(
                "{}{}{}",
                border.top_left,
                border.top.repeat(width.saturating_sub(2)),
                border.top_right
            ),
        );
        lines.push(format!(
            "{}{}{}",
            border.bottom_left,
            border.bottom.repeat(width.saturating_sub(2)),
            border.bottom_right
        ));
    }
    lines.truncate(state.height);
    lines = lines
        .into_iter()
        .map(|line| state.theme.background.clone().render(&line))
        .collect();
    let left = state.width.saturating_sub(width) / 2;
    let top = state.height.saturating_sub(lines.len()) / 2;
    let blank = " ".repeat(state.width);
    let mut output = Vec::with_capacity(state.height);
    output.extend((0..top).map(|_| blank.clone()));
    for line in lines {
        let clipped = truncate_plain(&line, width);
        let pad = width.saturating_sub(lipgloss::width(&clipped));
        output.push(format!(
            "{}{}{}{}",
            " ".repeat(left),
            clipped,
            " ".repeat(pad),
            " ".repeat(state.width.saturating_sub(left + width))
        ));
    }
    output.extend((output.len()..state.height).map(|_| blank.clone()));
    output.join("\n")
}

fn content_lines(state: &State, width: usize, height: usize) -> Vec<String> {
    let filtered = filter_commands(&state.commands, &state.query);
    let show_title = !state.title.is_empty() && width >= 10 && height >= 5;
    let show_separator = height >= 4;
    let capacity = height
        .saturating_sub(1 + usize::from(show_title) + usize::from(show_separator))
        .min(state.max_visible);
    let mut lines = Vec::new();
    if show_title {
        lines.push(fit(
            &state
                .theme
                .title
                .clone()
                .render(&truncate(&state.title, width)),
            width,
        ));
    }
    let prompt = if width >= 3 { "> " } else { "" };
    let query = if state.query.is_empty() {
        truncate(&state.placeholder, width.saturating_sub(prompt.len()))
    } else {
        truncate(&state.query, width.saturating_sub(prompt.len()))
    };
    let qstyle = if state.query.is_empty() {
        &state.theme.description
    } else {
        &state.theme.query
    };
    lines.push(fit(
        &format!(
            "{}{}",
            state.theme.query.clone().render(prompt),
            qstyle.clone().render(&query)
        ),
        width,
    ));
    if show_separator {
        lines.push("─".repeat(width));
    }
    if capacity > 0 {
        if filtered.is_empty() {
            lines.push(fit(
                &state
                    .theme
                    .description
                    .clone()
                    .render(&truncate("No commands found.", width)),
                width,
            ));
        } else {
            let rows = rows(&filtered);
            let selected_row = rows
                .iter()
                .position(|(_, index)| *index == Some(state.cursor))
                .unwrap_or(0);
            let start = selected_row
                .saturating_add(1)
                .saturating_sub(capacity)
                .min(rows.len().saturating_sub(capacity));
            for (group, index) in rows.iter().skip(start).take(capacity) {
                if let Some(group) = group {
                    lines.push(fit(
                        &state
                            .theme
                            .group
                            .clone()
                            .render(&truncate(&format!("  {group}"), width)),
                        width,
                    ));
                } else if let Some(index) = index {
                    lines.push(render_item(
                        state,
                        &filtered[*index],
                        *index == state.cursor,
                        width,
                    ));
                }
            }
        }
    }
    lines.truncate(height);
    lines
}

fn rows(commands: &[Command]) -> Vec<(StdOption<String>, StdOption<usize>)> {
    let mut rows = Vec::new();
    let mut last_group = String::new();
    for (index, command) in commands.iter().enumerate() {
        if !command.group.is_empty() && command.group != last_group {
            rows.push((Some(command.group.clone()), None));
        }
        rows.push((None, Some(index)));
        last_group.clone_from(&command.group);
    }
    rows
}

fn render_item(state: &State, command: &Command, selected: bool, width: usize) -> String {
    let prefix = if selected { "▸ " } else { "  " };
    let label = if command.title.is_empty() {
        &command.id
    } else {
        &command.title
    };
    let keys = command.keys.join(" ");
    let keys_width = lipgloss::width(&keys);
    let mut available = width.saturating_sub(2);
    let show_keys = !keys.is_empty() && width >= 30 && keys_width + 2 <= available / 2;
    if show_keys && keys_width + 2 < available {
        available -= keys_width + 2;
    }
    let title = truncate(label, available);
    let mut body = format!("{prefix}{title}");
    let remaining = available.saturating_sub(lipgloss::width(&title));
    if !selected && !command.description.is_empty() && width >= 48 && remaining >= 8 {
        body.push_str("  ");
        body.push_str(&truncate(&command.description, remaining.saturating_sub(2)));
    }
    if show_keys {
        let gap = width
            .saturating_sub(lipgloss::width(&body) + keys_width)
            .max(1);
        body.push_str(&" ".repeat(gap));
        body.push_str(&keys);
    }
    body = pad(&body, width);
    if selected {
        state.theme.selected_item.clone().render(&body)
    } else {
        let mut line = state.theme.item.clone().render(prefix);
        line.push_str(&state.theme.item.clone().render(&title));
        if !command.description.is_empty() && width >= 48 && remaining >= 8 {
            let description = format!(
                "  {}",
                truncate(&command.description, remaining.saturating_sub(2))
            );
            line.push_str(&state.theme.description.clone().render(&description));
        }
        if show_keys {
            let gap = width
                .saturating_sub(lipgloss::width(&line) + keys_width)
                .max(1);
            line.push_str(&" ".repeat(gap));
            line.push_str(&state.theme.shortcut.clone().render(&keys));
        }
        fit(&line, width)
    }
}

fn truncate(text: &str, width: usize) -> String {
    if lipgloss::width(text) <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    if width == 1 {
        return "…".to_owned();
    }
    let mut output = String::new();
    let mut used = 0;
    for character in text.chars() {
        let size = UnicodeWidthStr::width(character.encode_utf8(&mut [0; 4]));
        if used + size > width - 1 {
            break;
        }
        output.push(character);
        used += size;
    }
    output.push('…');
    output
}

fn truncate_plain(text: &str, width: usize) -> String {
    if lipgloss::width(text) <= width {
        return text.to_owned();
    }
    let plain = strip_ansi(text);
    truncate(&plain, width)
}

fn strip_ansi(text: &str) -> String {
    let mut output = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for code in chars.by_ref() {
                if ('@'..='~').contains(&code) {
                    break;
                }
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn fit(text: &str, width: usize) -> String {
    let current = lipgloss::width(text);
    if current >= width {
        truncate_plain(text, width)
    } else {
        format!("{text}{}", " ".repeat(width - current))
    }
}

fn pad(text: &str, width: usize) -> String {
    let current = lipgloss::width(text);
    if current >= width {
        truncate(text, width)
    } else {
        format!("{text}{}", " ".repeat(width - current))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cmd(id: &str, title: &str) -> Command {
        Command::new(id, title)
    }

    #[test]
    fn query_is_accent_insensitive_subsequence_and_ranked() {
        let palette = Palette::new(vec![
            cmd("open", "Abrir arquivo"),
            cmd("exact", "Ação"),
            cmd("prefix", "Acao rápida"),
        ]);
        let mut palette = palette;
        palette.set_query("ACAO");
        let filtered = palette.filtered();
        let ids: Vec<_> = filtered.iter().map(|item| item.id.as_str()).collect();
        assert_eq!(ids, ["exact", "prefix"]);
    }

    #[test]
    fn duplicates_replace_but_empty_ids_are_kept() {
        let palette = Palette::new(vec![
            cmd("a", "First"),
            cmd("", "No id"),
            cmd("a", "Last"),
            cmd("", "No id 2"),
        ]);
        let commands = palette.commands();
        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0].title, "Last");
    }

    #[test]
    fn navigation_clamps_and_enter_closes() {
        let mut palette = Palette::new(vec![cmd("a", "Alpha"), cmd("b", "Beta")]);
        palette.open();
        palette.update(Message::new(KeyMsg::from_type(KeyType::Down)));
        palette.update(Message::new(KeyMsg::from_type(KeyType::Down)));
        assert_eq!(
            palette.selected().as_ref().map(|c| c.id.as_str()),
            Some("b")
        );
        palette.update(Message::new(KeyMsg::from_type(KeyType::Esc)));
        assert!(!palette.is_open());
    }

    #[test]
    fn view_obeys_size_and_is_empty_when_closed() {
        let mut palette = Palette::new(vec![cmd("a", "Alpha")]);
        palette.set_size(40, 10);
        assert_eq!(palette.render(), "");
        palette.open();
        let view = palette.render();
        assert_eq!(view.lines().count(), 10);
        assert!(view.lines().all(|line| lipgloss::width(line) == 40));
    }

    #[test]
    fn options_and_callbacks_are_applied() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let invoked = Arc::new(AtomicBool::new(false));
        let marker = Arc::clone(&invoked);
        let command = cmd("run", "Run").with_run(move || {
            marker.store(true, Ordering::SeqCst);
            Some(Cmd::new(|| Message::new("done")))
        });
        let mut palette = Palette::with_options(
            vec![command],
            &[
                with_max_visible(0),
                with_title("Actions"),
                with_theme(default_theme()),
            ],
        );
        palette.set_size(40, 10);
        palette.open();
        assert!(palette.view().contains("Actions"));
        let command = palette.update(Message::new(KeyMsg::from_type(KeyType::Enter)));
        assert!(invoked.load(Ordering::SeqCst));
        assert_eq!(
            command
                .and_then(Cmd::execute)
                .and_then(Message::downcast::<&str>),
            Some("done")
        );
        assert!(!palette.is_open());
    }

    #[test]
    fn tiny_and_zero_viewports_never_exceed_dimensions() {
        let mut palette = Palette::new(vec![cmd("a", "A long title")]);
        palette.open();
        for (width, height) in [(0, 0), (1, 1), (2, 2), (3, 3), (10, 5)] {
            palette.set_size(width, height);
            let view = palette.view();
            if width == 0 || height == 0 {
                assert_eq!(view, "");
                continue;
            }
            assert_eq!(view.lines().count(), height as usize);
            assert!(view
                .lines()
                .all(|line| lipgloss::width(line) <= width as usize));
        }
    }

    #[test]
    fn query_editing_and_empty_results_follow_keyboard_lifecycle() {
        let mut palette = Palette::new(vec![cmd("one", "One"), cmd("two", "Two")]);
        palette.open();
        palette.update(Message::new(KeyMsg::from_runes(vec!['t', 'w', 'o'])));
        assert_eq!(palette.query(), "two");
        assert_eq!(
            palette.selected().map(|item| item.id),
            Some("two".to_owned())
        );
        palette.update(Message::new(KeyMsg::from_type(KeyType::Backspace)));
        assert_eq!(palette.query(), "tw");
        palette.update(Message::new(KeyMsg::from_type(KeyType::CtrlU)));
        assert_eq!(palette.query(), "");
        palette.set_query("missing");
        assert!(palette.selected().is_none());
        assert!(palette.view().contains("No commands found."));
    }

    #[test]
    fn list_mutations_preserve_ids_and_clamp_selection() {
        let mut palette = Palette::new(vec![cmd("a", "Alpha"), cmd("b", "Beta")]);
        palette.open();
        palette.update(Message::new(KeyMsg::from_type(KeyType::End)));
        palette.add([cmd("a", "Updated alpha"), cmd("c", "Gamma")]);
        assert_eq!(palette.commands().len(), 3);
        assert_eq!(palette.commands()[0].title, "Updated alpha");
        assert_eq!(palette.selected().map(|item| item.id), Some("c".to_owned()));
        assert!(palette.remove("b"));
        assert_eq!(palette.selected().map(|item| item.id), Some("a".to_owned()));
        assert!(!palette.remove("absent"));
        palette.set_commands(vec![cmd("new", "New")]);
        assert_eq!(
            palette.selected().map(|item| item.id),
            Some("new".to_owned())
        );
    }

    #[test]
    fn clones_share_state_and_empty_queries_sort_by_folded_title() {
        let mut palette = Palette::new(vec![cmd("z", "Zulu"), cmd("a", "Árvore")]);
        let mut cloned = palette.clone();
        cloned.open();
        cloned.set_query("z");
        assert!(palette.is_open());
        assert_eq!(palette.query(), "z");
        palette.set_query("");
        let ids: Vec<_> = palette
            .filtered()
            .into_iter()
            .map(|command| command.id)
            .collect();
        assert_eq!(ids, ["a", "z"]);
    }
}
