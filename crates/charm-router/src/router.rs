//! Router implementation.

use std::sync::{Arc, Mutex};

use charmed_bubbletea::{Cmd, Message, Model, WindowSizeMsg};
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::screen::{Screen, ScreenUpdate};

/// A guard that may block `pop`.
pub type Guard = Box<dyn Fn() -> (bool, Option<Cmd>) + Send>;

#[allow(missing_debug_implementations)]
/// A navigation request recorded by a screen.
pub enum NavRequest {
    /// Push a new screen.
    Push(Box<dyn Screen>, Option<Value>),
    /// Pop the top screen.
    Pop,
    /// Replace the top screen.
    Replace(Box<dyn Screen>),
    /// Reset to the base screen.
    Reset,
}

struct Entry {
    screen: Option<Box<dyn Screen>>,
    params: Option<Value>,
    guard: Option<Guard>,
}

struct RouterInner {
    stack: Vec<Entry>,
    width: i32,
    height: i32,
    has_size: bool,
    gen: u64,
    nav: crate::screen::NavHandle,
}

#[allow(missing_debug_implementations)]
/// The router: a stack of screens plus terminal size.
pub struct Router {
    inner: Arc<Mutex<RouterInner>>,
}

impl Router {
    /// Creates a router with `root` as the base screen.
    pub fn new(root: Option<Box<dyn Screen>>) -> Self {
        let nav = crate::screen::NavHandle::default();
        let mut stack = Vec::new();
        if let Some(s) = root {
            stack.push(Entry {
                screen: Some(s),
                params: None,
                guard: None,
            });
        }
        let inner = RouterInner {
            stack,
            width: 0,
            height: 0,
            has_size: false,
            gen: 0,
            nav: nav.clone(),
        };
        Router {
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// Handle used by screens to navigate.
    pub fn nav_handle(&self) -> crate::screen::NavHandle {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .nav
            .clone()
    }

    /// Pushes a screen and returns the new screen's init command.
    pub fn push(&self, screen: Box<dyn Screen>, params: Option<Value>) -> Option<Cmd> {
        let cmd = screen.init();
        let mut inner = self.lock();
        let has_size = inner.has_size;
        let (w, h) = (inner.width, inner.height);
        inner.stack.push(Entry {
            screen: Some(screen),
            params,
            guard: None,
        });
        inner.gen += 1;
        if has_size {
            let last = inner.stack.len() - 1;
            if let Some(s) = inner.stack[last].screen.as_mut() {
                s.set_size(w, h);
            }
        }
        cmd
    }

    /// Pops the top screen. Returns the guard's command when it blocks.
    pub fn pop(&self) -> Option<Cmd> {
        let mut inner = self.lock();
        if inner.stack.len() <= 1 {
            return None;
        }
        if let Some(guard) = inner.stack[inner.stack.len() - 1].guard.as_ref() {
            let (ok, guard_cmd) = guard();
            if !ok {
                return guard_cmd;
            }
            // allowed: fall through, possibly with guard_cmd
            let last = inner.stack.len() - 1;
            inner.stack[last] = Entry {
                screen: None,
                params: None,
                guard: None,
            };
            inner.stack.truncate(last);
            inner.gen += 1;
            return guard_cmd;
        }
        let last = inner.stack.len() - 1;
        inner.stack[last] = Entry {
            screen: None,
            params: None,
            guard: None,
        };
        inner.stack.truncate(last);
        inner.gen += 1;
        None
    }

    /// Replaces the top screen.
    pub fn replace(&self, screen: Box<dyn Screen>) -> Option<Cmd> {
        let cmd = screen.init();
        let mut inner = self.lock();
        let has_size = inner.has_size;
        let (w, h) = (inner.width, inner.height);
        let params = if inner.stack.is_empty() {
            None
        } else {
            inner.stack[inner.stack.len() - 1].params.clone()
        };
        if inner.stack.is_empty() {
            inner.stack.push(Entry {
                screen: Some(screen),
                params,
                guard: None,
            });
        } else {
            let last = inner.stack.len() - 1;
            inner.stack[last] = Entry {
                screen: Some(screen),
                params,
                guard: None,
            };
        }
        inner.gen += 1;
        if has_size {
            let last = inner.stack.len() - 1;
            if let Some(s) = inner.stack[last].screen.as_mut() {
                s.set_size(w, h);
            }
        }
        cmd
    }

    /// Resets the stack to the base screen.
    pub fn reset(&self) -> Option<Cmd> {
        let mut inner = self.lock();
        if inner.stack.len() <= 1 {
            return None;
        }
        for i in 1..inner.stack.len() {
            inner.stack[i] = Entry {
                screen: None,
                params: None,
                guard: None,
            };
        }
        inner.stack.truncate(1);
        inner.gen += 1;
        None
    }

    /// Current top screen's title, if any.
    pub fn current_title(&self) -> Option<String> {
        let inner = self.lock();
        inner
            .stack
            .last()
            .and_then(|e| e.screen.as_ref().map(|s| s.title()))
    }

    /// Number of screens on the stack.
    pub fn depth(&self) -> usize {
        self.lock().stack.len()
    }

    /// Last known terminal dimensions, or `None` before the first resize.
    pub fn size(&self) -> Option<(i32, i32)> {
        let inner = self.lock();
        inner.has_size.then_some((inner.width, inner.height))
    }

    /// Titles from base to top.
    pub fn titles(&self) -> Vec<String> {
        let inner = self.lock();
        inner
            .stack
            .iter()
            .map(|e| e.screen.as_ref().map(|s| s.title()).unwrap_or_default())
            .collect()
    }

    /// Renders screen titles as a width-limited breadcrumb (`A › … › Z`).
    /// Titles are normalized to one line. Styling is left to the caller so the
    /// returned string remains suitable for terminals and tests alike.
    pub fn breadcrumb(&self, width: usize) -> String {
        use unicode_width::UnicodeWidthStr;
        if width == 0 {
            return String::new();
        }
        let titles: Vec<String> = self
            .titles()
            .into_iter()
            .map(|title| title.replace(['\n', '\r', '\t'], " "))
            .collect();
        if titles.is_empty() {
            return String::new();
        }
        let full = titles.join(" › ");
        if UnicodeWidthStr::width(full.as_str()) <= width {
            return full;
        }
        let last = titles.last().expect("non-empty titles");
        let truncate = |text: &str, max: usize| -> String {
            let mut out = String::new();
            let mut used = 0;
            let reserve_ellipsis = UnicodeWidthStr::width(text) > max && max > 0;
            let limit = if reserve_ellipsis { max - 1 } else { max };
            for ch in text.chars() {
                let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                if used + cw > limit {
                    break;
                }
                out.push(ch);
                used += cw;
            }
            if out != text && max > 0 {
                out.push('…');
            }
            out
        };
        if titles.len() == 1 {
            return truncate(last, width);
        }
        let prefix = format!("{} › … › ", titles[0]);
        if UnicodeWidthStr::width(prefix.as_str()) < width {
            let candidate = format!(
                "{prefix}{}",
                truncate(last, width - UnicodeWidthStr::width(prefix.as_str()))
            );
            if UnicodeWidthStr::width(candidate.as_str()) <= width {
                return candidate;
            }
        }
        let short = format!("… › {last}");
        if UnicodeWidthStr::width(short.as_str()) <= width {
            return short;
        }
        truncate(last, width)
    }

    /// Sets a guard on the top screen.
    pub fn set_guard(&self, guard: Guard) {
        let mut inner = self.lock();
        if inner.stack.is_empty() {
            return;
        }
        let last = inner.stack.len() - 1;
        inner.stack[last].guard = Some(guard);
    }

    /// Clears the guard on the top screen.
    pub fn clear_guard(&self) {
        let mut inner = self.lock();
        if inner.stack.is_empty() {
            return;
        }
        let last = inner.stack.len() - 1;
        inner.stack[last].guard = None;
    }

    /// Parameters of the current screen.
    pub fn params(&self) -> Option<Value> {
        let inner = self.lock();
        inner.stack.last()?.params.clone()
    }

    /// Parameters of the current screen, converted to `T`.
    pub fn params_as<T>(&self) -> (Option<T>, bool)
    where
        T: DeserializeOwned + 'static,
    {
        let Some(v) = self.params() else {
            return (None, false);
        };
        match serde_json::from_value::<T>(v) {
            Ok(t) => (Some(t), true),
            Err(_) => (None, false),
        }
    }

    /// Applies a size to every screen on the stack.
    pub fn set_size(&self, width: i32, height: i32) {
        let mut inner = self.lock();
        let w = width.max(0);
        let h = height.max(0);
        inner.width = w;
        inner.height = h;
        inner.has_size = true;
        for e in &mut inner.stack {
            if let Some(s) = e.screen.as_mut() {
                s.set_size(w, h);
            }
        }
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.lock().stack.is_empty()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, RouterInner> {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    // --- Model trait helpers (called by the impl below) ---

    pub(crate) fn model_init(&self) -> Option<Cmd> {
        let inner = self.lock();
        inner
            .stack
            .last()
            .and_then(|e| e.screen.as_ref().map(|s| s.init()))
            .flatten()
    }

    pub(crate) fn model_update(&self, msg: Message) -> Option<Cmd> {
        // WindowSizeMsg
        if let Some(size) = msg.downcast_ref::<WindowSizeMsg>().copied() {
            self.set_size(size.width as i32, size.height as i32);
        }
        // get current screen arc
        let screen_present = {
            let inner = self.lock();
            !inner.stack.is_empty() && inner.stack[inner.stack.len() - 1].screen.is_some()
        };
        if !screen_present {
            return None;
        }
        let generation = self.lock().gen;
        // call update on the screen
        let (update, cmd) = {
            let mut inner = self.lock();
            let last = inner.stack.len() - 1;
            let screen = inner.stack[last].screen.as_mut().unwrap();
            screen.update(msg)
        };
        if let ScreenUpdate::Replace(next) = update {
            let mut inner = self.lock();
            // Do not overwrite a screen installed by navigation inside update.
            if inner.gen == generation {
                let last = inner.stack.len() - 1;
                inner.stack[last].screen = Some(next);
            }
        }
        // process pending navigation
        let pending = {
            let inner = self.lock();
            inner
                .stack
                .last()
                .and_then(|e| e.screen.as_ref())
                .and_then(|s| s.nav_handle())
        };
        let mut result = cmd;
        if let Some(handle) = pending {
            if let Some(req) = handle.take_pending() {
                match req {
                    NavRequest::Push(s, p) => {
                        let c = self.push(s, p);
                        if c.is_some() {
                            result = c;
                        }
                    }
                    NavRequest::Pop => {
                        let c = self.pop();
                        if c.is_some() {
                            result = c;
                        }
                    }
                    NavRequest::Replace(s) => {
                        let c = self.replace(s);
                        if c.is_some() {
                            result = c;
                        }
                    }
                    NavRequest::Reset => {
                        let c = self.reset();
                        if c.is_some() {
                            result = c;
                        }
                    }
                }
            }
        }
        result
    }

    pub(crate) fn model_view(&self) -> String {
        let inner = self.lock();
        inner
            .stack
            .last()
            .and_then(|e| e.screen.as_ref().map(|s| s.view()))
            .unwrap_or_default()
    }
}

impl Model for Router {
    fn init(&self) -> Option<Cmd> {
        self.model_init()
    }

    fn update(&mut self, msg: Message) -> Option<Cmd> {
        self.model_update(msg)
    }

    fn view(&self) -> String {
        self.model_view()
    }
}
