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

use std::collections::HashMap;

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
        let key = normalize_key(&msg.to_string());
        if key.is_empty() {
            return (String::new(), false);
        }
        self.resolve_normalized(&key)
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
        self.help_width(0)
    }

    /// Effective help laid out in columns of the given width.
    pub fn help_width(&self, width: usize) -> String {
        // Collect effective bindings: top-down, first definition wins.
        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut entries: Vec<(String, String)> = Vec::new(); // (keys, help)
                                                             // walk from top context down to global
        let mut contexts: Vec<&Context> = Vec::new();
        for name in self.stack.iter().rev() {
            if let Some(ctx) = self.contexts.get(name) {
                contexts.push(ctx);
            }
        }
        if let Some(g) = &self.global {
            contexts.push(g);
        }
        for ctx in contexts {
            for binding in &ctx.bindings {
                for key in &binding.keys {
                    let nk = normalize_key(key);
                    if nk.is_empty() {
                        continue;
                    }
                    if !seen.insert(nk.clone()) {
                        continue;
                    }
                    let keys = binding.keys.join("/");
                    let help = if binding.help.is_empty() {
                        binding.action.clone()
                    } else {
                        binding.help.clone()
                    };
                    entries.push((keys, help));
                }
            }
        }
        if entries.is_empty() {
            return String::new();
        }
        if width == 0 {
            let mut out = String::new();
            for (keys, help) in &entries {
                out.push_str(keys);
                out.push_str(": ");
                out.push_str(help);
                out.push('\n');
            }
            return out;
        }
        // Column layout: best-effort, one entry per line for simplicity.
        let mut out = String::new();
        for (keys, help) in &entries {
            out.push_str(keys);
            out.push_str(": ");
            out.push_str(help);
            out.push('\n');
        }
        out
    }
}

fn collect_conflicts(ctx: &Context, out: &mut Vec<Conflict>) {
    // For each normalized key, find distinct actions.
    let mut by_key: HashMap<String, Vec<String>> = HashMap::new();
    for binding in &ctx.bindings {
        for key in &binding.keys {
            let nk = normalize_key(key);
            if nk.is_empty() {
                continue;
            }
            by_key.entry(nk).or_default().push(binding.action.clone());
        }
    }
    for (key, actions) in by_key {
        // distinct
        let mut distinct: Vec<String> = Vec::new();
        for a in actions {
            if !distinct.contains(&a) {
                distinct.push(a);
            }
        }
        if distinct.len() >= 2 {
            out.push(Conflict {
                context: ctx.name.clone(),
                key,
                action_a: distinct[0].clone(),
                action_b: distinct[1].clone(),
            });
        }
    }
}
