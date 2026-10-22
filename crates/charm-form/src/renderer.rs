//! Bubble Tea model and interactive terminal runner.

use std::collections::HashSet;
use std::fmt::{self, Write as _};
use std::io::{Read, Write};

use charmed_bubbletea::{
    quit, InterruptMsg, KeyMsg, KeyType, Message, Model as BubbleTeaModel, Program, WindowSizeMsg,
};

use crate::{
    validate_field, Answers, Error, FieldSpec, FormSpec, TYPE_CONFIRM, TYPE_INPUT,
    TYPE_MULTISELECT, TYPE_SELECT, TYPE_TEXT,
};

/// Interactive state machine for a [`FormSpec`].
#[derive(Debug)]
pub struct Model {
    spec: FormSpec,
    fields: Vec<FieldState>,
    current: usize,
    width: usize,
    height: usize,
    answers: Answers,
    cancelled: bool,
    done: bool,
}

#[derive(Debug)]
struct FieldState {
    spec: FieldSpec,
    text: TextBuffer,
    cursor: usize,
    selected: HashSet<usize>,
    confirm: bool,
    error: Option<String>,
}

impl Model {
    /// Creates a model for the form, normalizing recognized field types.
    #[must_use]
    #[allow(clippy::needless_pass_by_value)]
    pub fn new(spec: FormSpec) -> Self {
        let spec = spec.normalize();
        let fields = spec
            .fields
            .iter()
            .cloned()
            .map(|spec| FieldState {
                spec,
                text: TextBuffer::default(),
                cursor: 0,
                selected: HashSet::new(),
                confirm: false,
                error: None,
            })
            .collect();
        Self {
            spec,
            fields,
            current: 0,
            width: 0,
            height: 0,
            answers: Answers::new(),
            cancelled: false,
            done: false,
        }
    }

    /// Returns a copy of all answers collected so far.
    #[must_use]
    pub fn answers(&self) -> Answers {
        self.answers.clone()
    }

    /// Reports whether the user cancelled the form.
    #[must_use]
    pub const fn cancelled(&self) -> bool {
        self.cancelled
    }

    /// Reports whether every field was confirmed.
    #[must_use]
    pub const fn done(&self) -> bool {
        self.done
    }

    fn handle_key(&mut self, key: KeyMsg) -> Option<charmed_bubbletea::Cmd> {
        if key.key_type == KeyType::CtrlC || key.key_type == KeyType::Esc {
            self.cancelled = true;
            return Some(quit());
        }
        if self.fields.is_empty() || self.done {
            return None;
        }

        let current = self.current;
        match self.fields[current].spec.field_type.as_str() {
            TYPE_SELECT => self.handle_select_key(current, &key),
            TYPE_CONFIRM => self.handle_confirm_key(current, &key),
            TYPE_MULTISELECT => self.handle_multiselect_key(current, &key),
            _ => self.handle_text_key(current, key),
        }
    }

    fn handle_text_key(&mut self, index: usize, key: KeyMsg) -> Option<charmed_bubbletea::Cmd> {
        let multiline = self.fields[index].spec.field_type == TYPE_TEXT;
        let key_type = key.key_type;
        match key_type {
            KeyType::Enter if multiline => self.fields[index].text.newline(),
            KeyType::Enter | KeyType::CtrlS => return self.confirm(index),
            KeyType::Tab => self.next(),
            KeyType::ShiftTab => self.previous(),
            KeyType::Backspace => self.fields[index].text.backspace(),
            KeyType::Delete => self.fields[index].text.delete(),
            KeyType::Left => self.fields[index].text.left(),
            KeyType::Right => self.fields[index].text.right(),
            KeyType::Home => self.fields[index].text.home(),
            KeyType::End => self.fields[index].text.end(),
            KeyType::Space if !key.alt => self.fields[index].text.insert(" "),
            KeyType::Runes if !key.alt && !key.paste => {
                let text: String = key.runes.into_iter().collect();
                if multiline {
                    self.fields[index].text.insert(&text);
                } else {
                    self.fields[index].text.insert_single_line(&text);
                }
            }
            KeyType::Runes if key.paste => {
                let text: String = key.runes.into_iter().collect();
                if multiline {
                    self.fields[index].text.insert(&text);
                } else {
                    self.fields[index].text.insert_single_line(&text);
                }
            }
            _ => {}
        }
        self.fields[index].error = None;
        None
    }

    fn handle_select_key(&mut self, index: usize, key: &KeyMsg) -> Option<charmed_bubbletea::Cmd> {
        let count = self.fields[index].spec.options.as_ref().map_or(0, Vec::len);
        if count == 0 {
            return None;
        }
        match key.key_type {
            KeyType::Up => self.move_cursor(index, count, false),
            KeyType::Down => self.move_cursor(index, count, true),
            KeyType::Runes if key.runes == ['k'] => self.move_cursor(index, count, false),
            KeyType::Runes if key.runes == ['j'] => self.move_cursor(index, count, true),
            KeyType::Enter => return self.confirm(index),
            KeyType::Tab => self.next(),
            KeyType::ShiftTab => self.previous(),
            _ => {}
        }
        self.fields[index].error = None;
        None
    }

    fn handle_multiselect_key(
        &mut self,
        index: usize,
        key: &KeyMsg,
    ) -> Option<charmed_bubbletea::Cmd> {
        let count = self.fields[index].spec.options.as_ref().map_or(0, Vec::len);
        if count == 0 {
            return None;
        }
        match key.key_type {
            KeyType::Up => self.move_cursor(index, count, false),
            KeyType::Down => self.move_cursor(index, count, true),
            KeyType::Runes if key.runes == ['k'] => self.move_cursor(index, count, false),
            KeyType::Runes if key.runes == ['j'] => self.move_cursor(index, count, true),
            KeyType::Space => {
                let cursor = self.fields[index].cursor;
                if self.fields[index].selected.contains(&cursor) {
                    self.fields[index].selected.remove(&cursor);
                } else {
                    self.fields[index].selected.insert(cursor);
                }
            }
            KeyType::Enter => return self.confirm(index),
            KeyType::Tab => self.next(),
            KeyType::ShiftTab => self.previous(),
            _ => {}
        }
        self.fields[index].error = None;
        None
    }

    fn handle_confirm_key(&mut self, index: usize, key: &KeyMsg) -> Option<charmed_bubbletea::Cmd> {
        match key.key_type {
            KeyType::Left | KeyType::Right => {
                self.fields[index].confirm = !self.fields[index].confirm;
            }
            KeyType::Runes if key.runes == ['y'] || key.runes == ['Y'] => {
                self.fields[index].confirm = !self.fields[index].confirm;
            }
            KeyType::Runes if key.runes == ['n'] || key.runes == ['N'] => {
                self.fields[index].confirm = false;
            }
            KeyType::Enter => return self.confirm(index),
            KeyType::Tab => self.next(),
            KeyType::ShiftTab => self.previous(),
            _ => {}
        }
        self.fields[index].error = None;
        None
    }

    fn move_cursor(&mut self, index: usize, count: usize, forward: bool) {
        if count == 0 {
            return;
        }
        self.fields[index].cursor = if forward {
            (self.fields[index].cursor + 1) % count
        } else {
            (self.fields[index].cursor + count - 1) % count
        };
    }

    fn next(&mut self) {
        if self.current + 1 < self.fields.len() {
            self.current += 1;
        }
    }

    fn previous(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    fn confirm(&mut self, index: usize) -> Option<charmed_bubbletea::Cmd> {
        let field = &self.fields[index];
        let value = match field.spec.field_type.as_str() {
            TYPE_CONFIRM => serde_json::Value::Bool(field.confirm),
            TYPE_SELECT => {
                let selected = field
                    .spec
                    .options
                    .as_ref()
                    .and_then(|options| options.get(field.cursor));
                serde_json::Value::String(selected.cloned().unwrap_or_default())
            }
            TYPE_MULTISELECT => {
                let values = field
                    .spec
                    .options
                    .as_ref()
                    .into_iter()
                    .flatten()
                    .enumerate()
                    .filter(|(option_index, _)| field.selected.contains(option_index))
                    .map(|(_, option)| serde_json::Value::String(option.clone()))
                    .collect();
                serde_json::Value::Array(values)
            }
            _ => serde_json::Value::String(field.text.value()),
        };

        if let Err(error) = validate_field(&field.spec, &value) {
            self.fields[index].error = Some(error.to_string());
            return None;
        }
        let name = self.fields[index].spec.name.clone();
        self.answers.insert(name, value);
        self.fields[index].error = None;
        if index + 1 == self.fields.len() {
            self.done = true;
            Some(quit())
        } else {
            self.current += 1;
            None
        }
    }

    fn insert_paste(&mut self, text: &str) {
        if self.fields.is_empty() {
            return;
        }
        let field = &mut self.fields[self.current];
        match field.spec.field_type.as_str() {
            TYPE_INPUT => field.text.insert_single_line(text),
            TYPE_TEXT => field.text.insert(text),
            _ => return,
        }
        field.error = None;
    }

    fn render(&self) -> String {
        let mut output = String::new();
        output.push_str(&self.spec.title);
        output.push('\n');
        if !self.spec.description.trim().is_empty() {
            output.push_str(&self.spec.description);
            output.push('\n');
        }
        let _ = writeln!(
            output,
            "field {} of {}\n",
            self.current + 1,
            self.fields.len()
        );
        for (index, field) in self.fields.iter().enumerate() {
            if index > 0 {
                output.push('\n');
            }
            let active = index == self.current;
            output.push_str(if active { "  > " } else { "    " });
            output.push_str(&field.spec.label_text());
            if field.spec.required {
                output.push_str(" *");
            }
            output.push('\n');
            match field.spec.field_type.as_str() {
                TYPE_SELECT => Self::render_select(field, active, &mut output),
                TYPE_MULTISELECT => Self::render_multiselect(field, active, &mut output),
                TYPE_CONFIRM => {
                    let (yes, no) = if field.confirm {
                        ("(●)", "( )")
                    } else {
                        ("( )", "(●)")
                    };
                    let _ = writeln!(output, "      {yes} Yes    {no} No");
                }
                _ => Self::render_text_field(field, active, &mut output),
            }
            if let Some(error) = &field.error {
                output.push_str("  ! ");
                output.push_str(error);
                output.push('\n');
            }
        }
        output.push('\n');
        output.push_str(self.help());
        if self.width > 0 {
            output = output
                .lines()
                .map(|line| truncate_chars(line, self.width))
                .collect::<Vec<_>>()
                .join("\n");
        }
        output
    }

    fn render_text_field(field: &FieldState, active: bool, output: &mut String) {
        let text = field.text.value();
        if text.is_empty() {
            if let Some(placeholder) = field
                .spec
                .placeholder
                .as_deref()
                .filter(|placeholder| !placeholder.is_empty())
            {
                output.push_str("      ");
                if active {
                    output.push(' ');
                }
                output.push_str(placeholder);
                output.push('\n');
                return;
            }
        }
        for line in text.split('\n') {
            output.push_str("      ");
            output.push_str(line);
            output.push('\n');
        }
    }

    fn render_select(field: &FieldState, active: bool, output: &mut String) {
        for (index, option) in field.spec.options.iter().flatten().enumerate() {
            let marker = if active && index == field.cursor {
                "> "
            } else {
                "  "
            };
            let _ = writeln!(output, "      {marker}{option}");
        }
    }

    fn render_multiselect(field: &FieldState, active: bool, output: &mut String) {
        for (index, option) in field.spec.options.iter().flatten().enumerate() {
            let marker = if active && index == field.cursor {
                "> "
            } else {
                "  "
            };
            let checked = if field.selected.contains(&index) {
                'x'
            } else {
                ' '
            };
            let _ = writeln!(output, "      {marker}[{checked}] {option}");
        }
    }

    fn help(&self) -> &'static str {
        let Some(field) = self.fields.get(self.current) else {
            return "esc cancel";
        };
        match field.spec.field_type.as_str() {
            TYPE_TEXT => "ctrl+s confirm  •  enter newline  •  tab next  •  shift+tab previous  •  esc cancel",
            TYPE_SELECT => "up/down choose  •  enter confirm  •  tab next  •  shift+tab previous  •  esc cancel",
            TYPE_CONFIRM => "y/n toggle  •  enter confirm  •  tab next  •  shift+tab previous  •  esc cancel",
            TYPE_MULTISELECT => "space toggle  •  up/down move  •  enter confirm  •  tab next  •  shift+tab previous  •  esc cancel",
            _ => "enter confirm  •  tab next  •  shift+tab previous  •  esc cancel",
        }
    }
}

impl BubbleTeaModel for Model {
    fn init(&self) -> Option<charmed_bubbletea::Cmd> {
        None
    }

    fn update(&mut self, message: Message) -> Option<charmed_bubbletea::Cmd> {
        if let Some(size) = message.downcast_ref::<WindowSizeMsg>() {
            self.width = usize::from(size.width);
            self.height = usize::from(size.height);
            return None;
        }
        if message.is::<InterruptMsg>() {
            self.cancelled = true;
            return Some(quit());
        }
        if let Some(key) = message.downcast::<KeyMsg>() {
            if key.paste {
                let text: String = key.runes.into_iter().collect();
                self.insert_paste(&text);
                None
            } else {
                self.handle_key(key)
            }
        } else {
            None
        }
    }

    fn view(&self) -> String {
        self.render()
    }
}

/// Options for an interactive form run.
#[derive(Default)]
pub struct Options {
    /// Optional input reader. When absent, the runtime reads the terminal.
    pub input: Option<Box<dyn Read + Send>>,
    /// Optional output writer. When absent, the runtime writes to stdout.
    pub output: Option<Box<dyn Write + Send>>,
    /// Initial terminal width; useful with scripted input.
    pub width: usize,
    /// Initial terminal height; useful with scripted input.
    pub height: usize,
}

impl fmt::Debug for Options {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Options")
            .field("input", &self.input.as_ref().map(|_| "<reader>"))
            .field("output", &self.output.as_ref().map(|_| "<writer>"))
            .field("width", &self.width)
            .field("height", &self.height)
            .finish()
    }
}

/// Runs a form interactively using standard terminal input and output.
///
/// # Errors
/// Returns [`Error::Runner`] if Bubble Tea fails, or [`Error::Cancelled`] if
/// the user cancels with Escape or Ctrl+C.
pub fn run(spec: FormSpec) -> Result<Answers, Error> {
    run_with_options(spec, Options::default())
}

/// Runs a form with custom input/output streams and initial terminal size.
///
/// # Errors
/// Returns [`Error::Runner`] if Bubble Tea fails, or [`Error::Cancelled`] if
/// the user cancels with Escape or Ctrl+C.
pub fn run_with_options(spec: FormSpec, options: Options) -> Result<Answers, Error> {
    let mut model = Model::new(spec);
    if options.width > 0 && options.height > 0 {
        model.width = options.width;
        model.height = options.height;
    }
    let mut program = Program::new(model);
    if let Some(input) = options.input {
        program = program.with_input(input);
    }
    if let Some(output) = options.output {
        program = program.with_output(output);
    }
    let final_model = program.run().map_err(Error::Runner)?;
    // Bubble Tea 0.2.4 consumes its InterruptMsg before calling Model::update.
    // An unfinished returned model therefore also represents Ctrl+C.
    if final_model.cancelled || !final_model.done {
        Err(Error::Cancelled)
    } else {
        Ok(final_model.answers())
    }
}

#[derive(Debug, Default)]
struct TextBuffer {
    value: Vec<char>,
    cursor: usize,
}

impl TextBuffer {
    fn value(&self) -> String {
        self.value.iter().collect()
    }

    fn insert(&mut self, text: &str) {
        let inserted: Vec<char> = text.chars().collect();
        if inserted.is_empty() {
            return;
        }
        self.value
            .splice(self.cursor..self.cursor, inserted.iter().copied());
        self.cursor += inserted.len();
    }

    fn insert_single_line(&mut self, text: &str) {
        self.insert(&text.replace('\n', " "));
    }

    fn newline(&mut self) {
        self.insert("\n");
    }

    fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.value.remove(self.cursor);
        }
    }

    fn delete(&mut self) {
        if self.cursor < self.value.len() {
            self.value.remove(self.cursor);
        }
    }

    fn left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
    }

    fn right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.value.len());
    }

    fn home(&mut self) {
        self.cursor = 0;
    }

    fn end(&mut self) {
        self.cursor = self.value.len();
    }
}

fn truncate_chars(text: &str, width: usize) -> String {
    text.chars().take(width).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_buffer_editing_and_lines_match_go() {
        let mut buffer = TextBuffer::default();
        buffer.insert("abc");
        buffer.home();
        buffer.insert("x");
        assert_eq!(buffer.value(), "xabc");
        buffer.end();
        buffer.backspace();
        assert_eq!(buffer.value(), "xab");
        buffer.home();
        buffer.delete();
        assert_eq!(buffer.value(), "ab");
        buffer.left();
        buffer.backspace();
        assert_eq!(buffer.value(), "ab");
        buffer.insert_single_line("l1\nl2");
        assert!(!buffer.value().contains('\n'));
    }
}
