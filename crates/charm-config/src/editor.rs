use std::{fs, io, path::PathBuf};

use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model, WindowSizeMsg};
use lipgloss::Style;
use serde_json::Value;
use unicode_width::UnicodeWidthChar;

use crate::{
    loader::{encode, lookup, set_path, Loader},
    schema::{Field, Schema},
    values::{format_value, is_list, normalize, to_bool, to_duration, to_int},
};

/// Error returned when saving without a file layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrNoWritableFile;
impl std::fmt::Display for ErrNoWritableFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("config: nenhum arquivo gravável; adicione uma camada com Loader::file")
    }
}
impl std::error::Error for ErrNoWritableFile {}

/// Errors produced while serializing or writing editor values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveError {
    /// The loader has no writable file layer.
    NoWritableFile(ErrNoWritableFile),
    /// Serialization, write, or permission failure.
    Other(String),
}
impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoWritableFile(error) => error.fmt(f),
            Self::Other(error) => f.write_str(error),
        }
    }
}
impl std::error::Error for SaveError {}
impl From<String> for SaveError {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Values,
    Docs,
}

/// Bubble Tea model for inspecting and editing a configuration snapshot.
#[derive(Debug, Clone)]
pub struct Editor {
    loader: Loader,
    schema: Schema,
    values: Value,
    initial: Value,
    cursor: usize,
    mode: Mode,
    editing: bool,
    buffer: Vec<char>,
    position: usize,
    width: usize,
    height: usize,
    status: String,
    status_error: bool,
    confirm_quit: bool,
}

impl Editor {
    /// Creates an editor from the loader's current merged values.
    /// Missing field defaults are inserted into this editor snapshot only.
    #[must_use]
    pub fn new(loader: Loader, schema: Schema) -> Self {
        let mut values = loader.all();
        for field in &schema.fields {
            if lookup(&values, &field.key).is_none() {
                if let Some(default) = &field.default {
                    set_path(&mut values, &field.key, default.clone());
                }
            }
            if let Some(value) = lookup(&values, &field.key).cloned() {
                if let Some(normalized) = normalize(&field.field_type, &value) {
                    set_path(&mut values, &field.key, normalized);
                }
            }
        }
        Self {
            loader,
            schema,
            initial: values.clone(),
            values,
            cursor: 0,
            mode: Mode::Values,
            editing: false,
            buffer: Vec::new(),
            position: 0,
            width: 80,
            height: 24,
            status: String::new(),
            status_error: false,
            confirm_quit: false,
        }
    }

    /// Sets the render area. Zero dimensions produce an empty view.
    pub fn set_size(&mut self, width: i32, height: i32) {
        self.width = usize::try_from(width.max(0)).unwrap_or(0);
        self.height = usize::try_from(height.max(0)).unwrap_or(0);
    }
    /// Returns an independent copy of current values.
    #[must_use]
    pub fn values(&self) -> Value {
        self.values.clone()
    }
    /// Whether current values differ from the last saved snapshot.
    #[must_use]
    pub fn dirty(&self) -> bool {
        self.values != self.initial
    }

    /// Saves effective values to the highest-priority file layer (mode 0600).
    pub fn save(&mut self) -> Result<(), SaveError> {
        let result = self.save_inner();
        if let Err(error) = &result {
            self.status = error.to_string();
            self.status_error = true;
        }
        result
    }

    fn save_inner(&mut self) -> Result<(), SaveError> {
        let Some(source) = self.loader.writable_source() else {
            return Err(SaveError::NoWritableFile(ErrNoWritableFile));
        };
        let (Some(path), Some(format)) = (&source.path, source.format) else {
            return Err(SaveError::NoWritableFile(ErrNoWritableFile));
        };
        let path = path.clone();
        let values = self.save_values();
        let bytes = encode(&values, format).map_err(SaveError::Other)?;
        fs::write(&path, bytes).map_err(|e| {
            SaveError::Other(format!(
                "config: não foi possível gravar '{}': {e}",
                path.display()
            ))
        })?;
        set_private_permissions(&path).map_err(|e| {
            SaveError::Other(format!(
                "config: não foi possível ajustar as permissões de '{}' para 0600: {e}",
                path.display()
            ))
        })?;
        self.initial.clone_from(&self.values);
        self.status = format!("salvo em {}", path.display());
        self.status_error = false;
        Ok(())
    }

    fn save_values(&self) -> Value {
        let mut out = self.values.clone();
        for field in &self.schema.fields {
            let Some(value) = lookup(&out, &field.key).cloned() else {
                continue;
            };
            let kind = field.field_type.trim().to_ascii_lowercase();
            let serialized = match kind.as_str() {
                "int" => to_int(&value).map(|v| Value::Number(v.into())),
                "bool" => to_bool(&value).map(Value::Bool),
                "duration" => to_duration(&value).map(|v| Value::String(v.to_string())),
                "string" if value.is_string() => Some(value),
                "list" if is_list(&value) => Some(value),
                _ => None,
            };
            if let Some(value) = serialized {
                set_path(&mut out, &field.key, value);
            }
        }
        out
    }

    fn current_field(&self) -> Option<&Field> {
        self.schema
            .fields
            .get(self.cursor.min(self.schema.fields.len().saturating_sub(1)))
    }
    fn move_cursor(&mut self, delta: isize) {
        if self.schema.fields.is_empty() {
            self.cursor = 0;
            return;
        }
        self.cursor = self
            .cursor
            .saturating_add_signed(delta)
            .min(self.schema.fields.len() - 1);
    }
    fn start_edit(&mut self) {
        let Some(field) = self.current_field() else {
            return;
        };
        let text = edit_value(field, lookup(&self.values, &field.key));
        self.buffer = text.chars().collect();
        self.position = self.buffer.len();
        self.editing = true;
        self.status.clear();
        self.status_error = false;
    }
    fn commit_edit(&mut self) {
        let Some(field) = self.current_field().cloned() else {
            self.cancel_edit();
            return;
        };
        match parse_value(&field, &self.buffer.iter().collect::<String>()) {
            Ok(value) => {
                set_path(&mut self.values, &field.key, value);
                self.editing = false;
                self.buffer.clear();
                self.position = 0;
                self.status.clear();
                self.status_error = false;
            }
            Err(error) => {
                self.status = error;
                self.status_error = true;
            }
        }
    }
    fn cancel_edit(&mut self) {
        self.editing = false;
        self.buffer.clear();
        self.position = 0;
        self.status.clear();
        self.status_error = false;
    }

    fn key(&mut self, key: &KeyMsg) -> Option<Cmd> {
        if self.confirm_quit {
            match key_char(key).as_str() {
                "q" | "y" | "enter" | "ctrl+c" => {
                    return Some(Cmd::new(|| Message::new(charmed_bubbletea::QuitMsg)))
                }
                "n" | "esc" => {
                    self.confirm_quit = false;
                    self.status.clear();
                    self.status_error = false;
                }
                _ => {}
            }
            return None;
        }
        if self.editing {
            self.editing_key(key);
            return None;
        }
        match key.key_type {
            KeyType::Up => self.move_cursor(-1),
            KeyType::Down => self.move_cursor(1),
            KeyType::Tab => {
                self.mode = if self.mode == Mode::Values {
                    Mode::Docs
                } else {
                    Mode::Values
                }
            }
            KeyType::Enter => self.start_edit(),
            KeyType::CtrlC => return self.quit_or_confirm(),
            KeyType::Runes => match key.runes.iter().collect::<String>().as_str() {
                "k" => self.move_cursor(-1),
                "j" => self.move_cursor(1),
                "s" => {
                    if let Err(e) = self.save() {
                        self.status = e.to_string();
                        self.status_error = true;
                    }
                }
                "q" => return self.quit_or_confirm(),
                _ => {}
            },
            _ => {}
        }
        None
    }
    fn quit_or_confirm(&mut self) -> Option<Cmd> {
        if self.dirty() {
            self.confirm_quit = true;
            self.status.clear();
            self.status_error = false;
            None
        } else {
            Some(Cmd::new(|| Message::new(charmed_bubbletea::QuitMsg)))
        }
    }
    fn editing_key(&mut self, key: &KeyMsg) {
        match key.key_type {
            KeyType::Esc => self.cancel_edit(),
            KeyType::Enter => self.commit_edit(),
            KeyType::Backspace | KeyType::CtrlH => {
                if self.position > 0 {
                    self.position -= 1;
                    self.buffer.remove(self.position);
                }
            }
            KeyType::Delete => {
                if self.position < self.buffer.len() {
                    self.buffer.remove(self.position);
                }
            }
            KeyType::Left => self.position = self.position.saturating_sub(1),
            KeyType::Right => self.position = (self.position + 1).min(self.buffer.len()),
            KeyType::Home => self.position = 0,
            KeyType::End => self.position = self.buffer.len(),
            KeyType::Runes => {
                for c in &key.runes {
                    if !c.is_control() {
                        self.buffer.insert(self.position, *c);
                        self.position += 1;
                    }
                }
            }
            _ => {}
        }
    }
    fn render(&self) -> String {
        if self.width == 0 || self.height == 0 {
            return String::new();
        }
        let title = Style::new().bold(true).render(&truncate(
            &format!(
                "charm-config  {}",
                if self.mode == Mode::Values {
                    "[valores] docs"
                } else {
                    "valores [docs]"
                }
            ),
            self.width,
        ));
        let mut lines = vec![title];
        let list_height = self.height.saturating_sub(2);
        if list_height > 0 {
            if self.schema.fields.is_empty() {
                lines.push(truncate("(nenhum campo)", self.width));
            } else {
                let start = self.cursor.saturating_sub(list_height - 1);
                let end = (start + list_height).min(self.schema.fields.len());
                for i in start..end {
                    lines.push(self.render_row(i));
                }
            }
        }
        if self.height >= 2 {
            let footer = if self.confirm_quit {
                "alterações não salvas: q ou y saem, n ou esc cancelam"
            } else if !self.status.is_empty() {
                &self.status
            } else if self.mode == Mode::Docs {
                "↑/↓ navegar · enter editar · tab valores · s salvar · q sair"
            } else {
                "↑/↓ navegar · enter editar · tab docs · s salvar · q sair"
            };
            let style = if self.confirm_quit || self.status_error {
                Style::new().bold(true)
            } else {
                Style::new().faint(true)
            };
            lines.push(style.render(&truncate(&one_line(footer), self.width)));
        }
        lines.truncate(self.height);
        lines.join("\n")
    }
    fn render_row(&self, index: usize) -> String {
        let field = &self.schema.fields[index];
        let selected = index == self.cursor;
        let marker = if selected { "> " } else { "  " };
        let row = if self.editing && selected {
            let mut chars = self.buffer.clone();
            chars.insert(self.position.min(chars.len()), '█');
            format!(
                "{marker}{} = {}",
                field.key,
                chars.iter().collect::<String>()
            )
        } else if self.mode == Mode::Docs {
            let doc = if field.doc.is_empty() {
                "—".to_owned()
            } else {
                one_line(&field.doc)
            };
            format!("{marker}{}  {doc}", field.key)
        } else {
            let value = lookup(&self.values, &field.key)
                .map_or_else(|| "—".into(), |v| display_value(field, v));
            let mut row = format!("{marker}{} = {value}", field.key);
            let doc = one_line(&field.doc);
            let used = lipgloss::utils::width_visible(&row);
            if !doc.is_empty() && self.width.saturating_sub(used) >= 14 {
                row.push_str("  ");
                row.push_str(&truncate(&doc, self.width.saturating_sub(used + 2)));
            }
            row
        };
        let row = truncate(&row, self.width);
        if selected {
            Style::new().reverse(true).render(&row)
        } else {
            row
        }
    }
}

impl Model for Editor {
    fn init(&self) -> Option<Cmd> {
        None
    }
    fn update(&mut self, message: Message) -> Option<Cmd> {
        match message.try_downcast::<WindowSizeMsg>() {
            Ok(size) => {
                self.set_size(i32::from(size.width), i32::from(size.height));
                None
            }
            Err(message) => match message.try_downcast::<KeyMsg>() {
                Ok(key) => self.key(&key),
                Err(_) => None,
            },
        }
    }
    fn view(&self) -> String {
        self.render()
    }
}

fn set_private_permissions(path: &PathBuf) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

fn key_char(key: &KeyMsg) -> String {
    match key.key_type {
        KeyType::Enter => "enter".into(),
        KeyType::Esc => "esc".into(),
        KeyType::CtrlC => "ctrl+c".into(),
        KeyType::Runes => key.runes.iter().collect(),
        _ => key.to_string(),
    }
}
fn edit_value(field: &Field, value: Option<&Value>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    match field.field_type.trim().to_ascii_lowercase().as_str() {
        "string" => value.as_str().unwrap_or("").to_owned(),
        "int" => to_int(value).map_or_else(|| format_value(value), |n| n.to_string()),
        "bool" => to_bool(value).map_or_else(|| format_value(value), |b| b.to_string()),
        "duration" => to_duration(value).map_or_else(|| format_value(value), |d| d.to_string()),
        "list" => value.as_array().map_or_else(
            || format_value(value),
            |items| {
                items
                    .iter()
                    .map(format_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            },
        ),
        _ => format_value(value),
    }
}
fn display_value(field: &Field, value: &Value) -> String {
    edit_value(field, Some(value))
}
fn parse_value(field: &Field, text: &str) -> Result<Value, String> {
    let kind = field.field_type.trim().to_ascii_lowercase();
    match kind.as_str() {
        "int" => to_int(&Value::String(text.trim().to_owned()))
            .map(|v| Value::Number(v.into()))
            .ok_or_else(|| format!("campo {:?}: valor inteiro inválido {:?}", field.key, text)),
        "bool" => to_bool(&Value::String(text.trim().to_owned()))
            .map(Value::Bool)
            .ok_or_else(|| format!("campo {:?}: valor booleano inválido {:?}", field.key, text)),
        "duration" => to_duration(&Value::String(text.trim().to_owned()))
            .map(|d| Value::String(d.to_string()))
            .ok_or_else(|| {
                format!(
                    "campo {:?}: duração inválida {:?} (exemplos: 250ms, 5s, 1m30s)",
                    field.key, text
                )
            }),
        "list" => Ok(Value::Array(if text.trim().is_empty() {
            Vec::new()
        } else {
            text.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| Value::String(s.to_owned()))
                .collect()
        })),
        _ => Ok(Value::String(text.to_owned())),
    }
}
fn truncate(text: &str, width: usize) -> String {
    if lipgloss::utils::width_visible(text) <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let target = width.saturating_sub(1);
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > target {
            break;
        }
        out.push(ch);
        used += w;
    }
    if width > 0 {
        out.push('…');
    }
    out
}
fn one_line(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
