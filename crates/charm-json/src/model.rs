use crate::{row, Node, Row};
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model as TeaModel, WindowSizeMsg};
use std::fmt;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// Mensagem emitida pela tecla `y` com o caminho selecionado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YankedMsg {
    /// JSON Pointer do nó selecionado.
    pub path: String,
}
/// Tipo de opção para construir [`Model`].
pub type ModelOption = Box<dyn FnOnce(&mut Model) + Send>;
/// Limita a profundidade visível (`0`/negativo = ilimitada).
pub fn with_max_depth(depth: isize) -> ModelOption {
    Box::new(move |m| m.max_depth = depth.max(0) as usize)
}
/// Liga ou desliga as cores de sintaxe.
pub fn with_syntax_colors(enabled: bool) -> ModelOption {
    Box::new(move |m| m.colors = enabled)
}

/// Explorador de árvore compatível com o modelo do Bubble Tea.
#[derive(Debug, Clone)]
pub struct Model {
    root: Option<Node>,
    width: usize,
    height: usize,
    cursor: usize,
    filter: String,
    searching: bool,
    max_depth: usize,
    colors: bool,
}
impl Model {
    /// Cria um modelo com raiz opcional e opções.
    pub fn new(
        root: std::option::Option<Node>,
        opts: impl IntoIterator<Item = ModelOption>,
    ) -> Self {
        let mut m = Self {
            root,
            width: 80,
            height: 24,
            cursor: 0,
            filter: String::new(),
            searching: false,
            max_depth: 0,
            colors: true,
        };
        for option in opts {
            option(&mut m);
        }
        m
    }
    /// Define as dimensões da área de renderização.
    pub fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }
    /// Renderiza a área configurada como texto.
    pub fn view(&self) -> String {
        self.render()
    }
    /// Define o filtro e reposiciona a seleção no início.
    pub fn set_filter(&mut self, filter: impl Into<String>) {
        self.filter = filter.into();
        self.searching = false;
        self.cursor = 0;
    }
    /// Retorna o filtro atual.
    pub fn filter(&self) -> &str {
        &self.filter
    }
    /// Retorna as linhas visíveis após filtro, colapso e limite de profundidade.
    pub fn rows(&self) -> Vec<Row> {
        self.root
            .as_ref()
            .map_or_else(Vec::new, |r| row::flatten(r, &self.filter, self.max_depth))
    }
    /// Caminho selecionado ou vazio se não houver seleção.
    pub fn selected_path(&self) -> String {
        self.rows()
            .get(self.cursor)
            .map_or_else(String::new, |r| r.node.path.clone())
    }
    /// Valor do escalar selecionado.
    pub fn selected_value(&self) -> std::option::Option<String> {
        self.rows()
            .get(self.cursor)
            .filter(|r| !r.node.is_container())
            .map(|r| r.node.value.clone())
    }
    fn clamp(&mut self) {
        let n = self.rows().len();
        self.cursor = if n == 0 { 0 } else { self.cursor.min(n - 1) }
    }
    fn move_by(&mut self, n: isize) {
        let len = self.rows().len();
        if len > 0 {
            self.cursor = (self.cursor as isize + n).clamp(0, len as isize - 1) as usize;
        }
    }
    fn toggle(&mut self, action: i8) {
        if let Some(row) = self.rows().get(self.cursor) {
            if let Some(root) = &mut self.root {
                if let Some(node) = root.find_mut(&row.node.path) {
                    if node.is_container() {
                        node.collapsed = match action {
                            -1 => true,
                            1 => false,
                            _ => !node.collapsed,
                        }
                    }
                }
            }
        }
        self.clamp();
    }
    fn update_key(&mut self, key: KeyMsg) -> Option<Cmd> {
        if self.searching {
            match key.key_type {
                KeyType::Esc => {
                    self.filter.clear();
                    self.searching = false;
                    self.cursor = 0;
                }
                KeyType::Enter => self.searching = false,
                KeyType::Backspace => {
                    self.filter.pop();
                    self.cursor = 0;
                }
                KeyType::Space => self.filter.push(' '),
                KeyType::Runes if !key.alt => self.filter.extend(key.runes),
                _ => {}
            }
            return None;
        }
        match key.key_type {
            KeyType::Up => self.move_by(-1),
            KeyType::Down => self.move_by(1),
            KeyType::Enter | KeyType::Space => self.toggle(0),
            KeyType::Left => self.toggle(-1),
            KeyType::Right => self.toggle(1),
            KeyType::Esc => {
                self.filter.clear();
                self.cursor = 0;
            }
            KeyType::Runes if !key.alt => {
                let text: String = key.runes.iter().collect();
                match text.as_str() {
                    "j" => self.move_by(1),
                    "k" => self.move_by(-1),
                    "h" => self.toggle(-1),
                    "l" => self.toggle(1),
                    "g" => self.cursor = 0,
                    "G" => self.cursor = self.rows().len().saturating_sub(1),
                    "/" => {
                        self.filter.clear();
                        self.searching = true;
                        self.cursor = 0;
                    }
                    "y" => {
                        let path = self.selected_path();
                        if !self.rows().is_empty() {
                            return Some(Cmd::new(move || Message::new(YankedMsg { path })));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        None
    }
    fn render(&self) -> String {
        if self.width == 0 || self.height == 0 {
            return String::new();
        }
        let rows = self.rows();
        let prompt = self.searching || !self.filter.is_empty();
        let list_h = self.height.saturating_sub(usize::from(prompt));
        let offset = self.cursor.saturating_add(1).saturating_sub(list_h);
        let mut lines = Vec::new();
        if list_h > 0 {
            for (i, row) in rows.iter().enumerate().skip(offset).take(list_h) {
                lines.push(self.render_row(row, i == self.cursor));
            }
        }
        if prompt {
            let p = format!("/{}{}", self.filter, if self.searching { "█" } else { "" });
            lines.push(truncate(&p, self.width));
        }
        lines.join("\n")
    }
    fn render_row(&self, row: &Row, selected: bool) -> String {
        let marker = if selected { "> " } else { "  " };
        let plain = format!(
            "{marker}{depth}{prefix}{display}",
            depth = "  ".repeat(row.depth),
            prefix = row.prefix,
            display = row.display
        );
        let styled = if self.colors {
            let (key, value) = if row.key_text.is_empty() {
                (
                    String::new(),
                    color_text(&row.val_text, kind_color(row.node.kind)),
                )
            } else {
                (
                    format!("\x1b[1;36m{}\x1b[0m: ", row.key_text),
                    color_text(&row.val_text, kind_color(row.node.kind)),
                )
            };
            format!(
                "{}{}\x1b[36m{}\x1b[0m{}{}",
                if selected { "\x1b[7m> \x1b[0m" } else { "  " },
                "  ".repeat(row.depth),
                row.prefix,
                key,
                value
            )
        } else {
            plain
        };
        truncate(&styled, self.width)
    }
}
impl TeaModel for Model {
    fn init(&self) -> Option<Cmd> {
        None
    }
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        match msg.try_downcast::<WindowSizeMsg>() {
            Ok(size) => {
                self.set_size(size.width as usize, size.height as usize);
                None
            }
            Err(msg) => match msg.try_downcast::<KeyMsg>() {
                Ok(key) => self.update_key(key),
                Err(_) => None,
            },
        }
    }
    fn view(&self) -> String {
        self.render()
    }
}
impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}
fn kind_color(kind: crate::Kind) -> &'static str {
    match kind {
        crate::Kind::String => "32",
        crate::Kind::Number => "33",
        crate::Kind::Bool => "35",
        _ => "2",
    }
}
fn color_text(text: &str, code: &str) -> String {
    format!("\x1b[{code}m{text}\x1b[0m")
}
fn truncate(s: &str, width: usize) -> String {
    let stripped = lipgloss::utils::strip_ansi(s);
    if UnicodeWidthStr::width(stripped.as_str()) <= width {
        return s.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let target = width.saturating_sub(1);
    let mut out = String::new();
    let mut cells = 0;
    for c in stripped.chars() {
        let w = UnicodeWidthChar::width(c).unwrap_or(0);
        if cells + w > target {
            break;
        }
        out.push(c);
        cells += w;
    }
    if width > 1 {
        out.push('…');
    }
    out
}
