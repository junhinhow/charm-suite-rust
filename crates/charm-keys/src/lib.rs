//! Context-scoped keybindings for Bubble Tea TUIs.
//!
//! # Examples
//!
//! ```
//! use charm_keys::{Binding, Resolver};
//!
//! let mut r = Resolver::new()
//!     .global(&[Binding { action: "quit".into(), keys: vec!["q".into()], help: "quit".into() }])
//!     .define("list", &[Binding { action: "open".into(), keys: vec!["enter".into()], help: "open".into() }]);
//! let r = r.push("list");
//! assert_eq!(r.resolve_string("q"), ("quit".to_owned(), true));
//! assert_eq!(r.resolve_string("enter"), ("open".to_owned(), true));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::{HashMap, HashSet};

use charmed_bubbletea::KeyMsg;

mod normalize;

pub use normalize::{normalize_key, parse_key};

/// A binding associates keys to a named action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binding {
    /// Stable identifier of the action (e.g. "quit").
    pub action: String,
    /// Keys in any spelling accepted by [`normalize_key`].
    pub keys: Vec<String>,
    /// Text shown by [`Resolver::help`].
    pub help: String,
}

impl Binding {
    /// Reports whether a textual key belongs to this binding.
    pub fn matches(&self, key: &str) -> bool {
        let normalized = normalize_key(key);
        if normalized.is_empty() {
            return false;
        }
        self.keys
            .iter()
            .any(|candidate| normalize_key(candidate) == normalized)
    }
}

/// A named group of bindings.
#[derive(Debug, Clone)]
pub struct Context {
    /// Name of the context.
    pub name: String,
    /// Bindings in registration order.
    pub bindings: Vec<Binding>,
    index: HashMap<String, String>,
}

impl Context {
    fn new(name: String, bindings: &[Binding]) -> Self {
        let mut ctx = Context {
            name,
            bindings: Vec::new(),
            index: HashMap::new(),
        };
        for binding in bindings {
            if binding.action.trim().is_empty() {
                continue;
            }
            let mut keys = Vec::new();
            for raw in &binding.keys {
                let normalized = normalize_key(raw);
                if normalized.is_empty() {
                    continue;
                }
                keys.push(raw.clone());
                ctx.index
                    .entry(normalized)
                    .or_insert_with(|| binding.action.clone());
            }
            if keys.is_empty() {
                continue;
            }
            ctx.bindings.push(Binding {
                action: binding.action.clone(),
                keys,
                help: binding.help.clone(),
            });
        }
        ctx
    }
}

/// A conflict between two actions sharing a key in the same context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Conflict {
    /// Context where the conflict happens.
    pub context: String,
    /// Normalized key.
    pub key: String,
    /// First action.
    pub action_a: String,
    /// Second action.
    pub action_b: String,
}

/// Resolves keys considering a stack of contexts.
#[derive(Debug, Clone)]
pub struct Resolver {
    global: Option<Context>,
    contexts: HashMap<String, Context>,
    order: Vec<String>,
    stack: Vec<String>,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Resolver {
    /// Creates an empty resolver.
    pub fn new() -> Self {
        Self {
            global: None,
            contexts: HashMap::new(),
            order: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// Defines the always-active base context.
    #[must_use]
    pub fn global(mut self, bindings: &[Binding]) -> Self {
        self.global = Some(Context::new("global".to_owned(), bindings));
        self
    }

    /// Registers or replaces a named context.
    #[must_use]
    pub fn define(mut self, name: &str, bindings: &[Binding]) -> Self {
        let name = name.trim();
        if name.is_empty() {
            return self;
        }
        if name == "global" {
            return self.global(bindings);
        }
        if !self.contexts.contains_key(name) {
            self.order.push(name.to_owned());
        }
        self.contexts
            .insert(name.to_owned(), Context::new(name.to_owned(), bindings));
        self
    }

    /// Activates a context with the highest priority (idempotent).
    #[must_use]
    pub fn push(mut self, name: &str) -> Self {
        let name = name.trim();
        if name.is_empty() || name == "global" {
            return self;
        }
        if self.stack.iter().any(|a| a == name) {
            return self;
        }
        self.stack.push(name.to_owned());
        self
    }

    /// Deactivates the top context. Returns its name, or `""` when empty.
    pub fn pop(&mut self) -> String {
        if self.stack.is_empty() {
            return String::new();
        }
        let top = self.stack[self.stack.len() - 1].clone();
        self.stack.pop();
        top
    }

    /// Active contexts, lowest to highest priority.
    pub fn active(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.global.is_some() {
            out.push("global".to_owned());
        }
        out.extend(self.stack.clone());
        out
    }

    /// Resolves a key message to an action.
    pub fn resolve(&self, msg: &KeyMsg) -> (String, bool) {
        let text = msg.to_string();
        let (action, found) = self.resolve_string(&text);
        if found {
            return (action, true);
        }
        let key = normalize_key(&text);
        if key.starts_with("ctrl+") || key.starts_with("alt+") {
            return (String::new(), false);
        }
        let key = match msg.key_type {
            charmed_bubbletea::KeyType::CtrlC => "ctrl+c".to_owned(),
            charmed_bubbletea::KeyType::CtrlD => "ctrl+d".to_owned(),
            charmed_bubbletea::KeyType::CtrlZ => "ctrl+z".to_owned(),
            charmed_bubbletea::KeyType::Runes if msg.runes.len() == 1 => {
                let mut mods = Vec::new();
                if msg.alt {
                    mods.push("alt");
                }
                if mods.is_empty() {
                    msg.runes[0].to_string()
                } else {
                    format!("{}+{}", mods.join("+"), msg.runes[0])
                }
            }
            _ => key,
        };
        self.resolve_normalized(&normalize_key(&key))
    }

    /// Resolves a textual key such as `"ctrl+c"`.
    pub fn resolve_string(&self, key: &str) -> (String, bool) {
        let key = normalize_key(key);
        if key.is_empty() {
            return (String::new(), false);
        }
        self.resolve_normalized(&key)
    }

    fn resolve_normalized(&self, key: &str) -> (String, bool) {
        if key.is_empty() {
            return (String::new(), false);
        }
        for i in (0..self.stack.len()).rev() {
            if let Some(ctx) = self.contexts.get(&self.stack[i]) {
                if let Some(action) = ctx.index.get(key) {
                    return (action.clone(), true);
                }
            }
        }
        if let Some(global) = &self.global {
            if let Some(action) = global.index.get(key) {
                return (action.clone(), true);
            }
        }
        (String::new(), false)
    }

    /// Reports conflicts: actions fighting over the same key in the same context.
    pub fn conflicts(&self) -> Vec<Conflict> {
        let mut out = Vec::new();
        // global
        if let Some(g) = &self.global {
            collect_conflicts(g, &mut out);
        }
        for name in &self.order {
            if let Some(ctx) = self.contexts.get(name) {
                collect_conflicts(ctx, &mut out);
            }
        }
        // also contexts in stack not in order? order covers all defined.
        out
    }

    /// Effective help, one binding per line.
    pub fn help(&self) -> String {
        self.help_entries()
            .iter()
            .map(|(keys, desc)| format!("{keys}  {desc}"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Effective help laid out in columns of the given width.
    pub fn help_width(&self, width: usize) -> String {
        let entries = self.help_entries();
        if entries.is_empty() {
            return String::new();
        }
        if width == 0 {
            return String::new();
        }
        let labels: Vec<String> = entries.iter().map(|(k, d)| format!("{k}  {d}")).collect();
        let max_width = labels.iter().map(|s| display_width(s)).max().unwrap_or(0);
        let col_width = (max_width + 2).min(width);
        let cols = (width / col_width).max(1).min(labels.len());
        let rows = labels.len().div_ceil(cols);
        let content_width = if cols > 1 {
            col_width.saturating_sub(2)
        } else {
            width
        };
        let mut lines = Vec::new();
        for row in 0..rows {
            let mut line = String::new();
            for col in 0..cols {
                let i = col * rows + row;
                if i >= labels.len() {
                    break;
                }
                let cell = truncate(&labels[i], content_width);
                line.push_str(&cell);
                if i + rows < labels.len() {
                    line.push_str(&" ".repeat(col_width.saturating_sub(display_width(&cell))));
                }
            }
            lines.push(line.trim_end().to_owned());
        }
        lines.join("\n")
    }

    fn help_entries(&self) -> Vec<(String, String)> {
        let contexts = self.priority_contexts();
        let mut effective = HashMap::<String, String>::new();
        for ctx in &contexts {
            for (key, action) in &ctx.index {
                effective
                    .entry(key.clone())
                    .or_insert_with(|| action.clone());
            }
        }
        let mut actions = HashSet::new();
        let mut entries = Vec::new();
        for ctx in contexts {
            for binding in &ctx.bindings {
                if actions.contains(&binding.action) {
                    continue;
                }
                let mut seen = HashSet::new();
                let keys: Vec<_> = binding
                    .keys
                    .iter()
                    .filter(|key| {
                        let normalized = normalize_key(key);
                        !normalized.is_empty()
                            && effective.get(&normalized) == Some(&binding.action)
                            && seen.insert(normalized)
                    })
                    .cloned()
                    .collect();
                if keys.is_empty() {
                    continue;
                }
                let desc = if binding.help.trim().is_empty() {
                    binding.action.clone()
                } else {
                    binding.help.clone()
                };
                actions.insert(binding.action.clone());
                entries.push((keys.join(", "), desc));
            }
        }
        entries
    }

    fn priority_contexts(&self) -> Vec<&Context> {
        let mut contexts = Vec::new();
        for name in self.stack.iter().rev() {
            if let Some(ctx) = self.contexts.get(name) {
                contexts.push(ctx);
            }
        }
        if let Some(global) = &self.global {
            contexts.push(global);
        }
        contexts
    }
}

fn collect_conflicts(ctx: &Context, out: &mut Vec<Conflict>) {
    let mut seen = HashMap::<String, String>::new();
    for binding in &ctx.bindings {
        for key in &binding.keys {
            let nk = normalize_key(key);
            if nk.is_empty() {
                continue;
            }
            if let Some(first) = seen.get(&nk) {
                if first != &binding.action {
                    out.push(Conflict {
                        context: ctx.name.clone(),
                        key: nk,
                        action_a: first.clone(),
                        action_b: binding.action.clone(),
                    });
                }
            } else {
                seen.insert(nk, binding.action.clone());
            }
        }
    }
}

fn display_width(s: &str) -> usize {
    s.chars()
        .map(|c| if c as u32 >= 0x1100 { 2 } else { 1 })
        .sum()
}
fn truncate(s: &str, width: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for c in s.chars() {
        let w = if c as u32 >= 0x1100 { 2 } else { 1 };
        if used + w > width {
            break;
        }
        out.push(c);
        used += w;
    }
    out
}
