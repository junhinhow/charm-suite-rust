//! In-memory clipboard.

use std::sync::{Arc, Mutex};

use crate::lock;

struct ClipboardInner {
    contents: String,
    history: Vec<String>,
}

/// An in-memory clipboard, isolated per instance.
#[derive(Clone)]
pub struct Clipboard {
    inner: Arc<Mutex<ClipboardInner>>,
}

impl std::fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard").finish_non_exhaustive()
    }
}

impl Default for Clipboard {
    fn default() -> Self {
        Self::new()
    }
}

impl Clipboard {
    /// Creates an empty clipboard.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ClipboardInner {
                contents: String::new(),
                history: Vec::new(),
            })),
        }
    }

    /// Returns the current contents.
    pub fn read(&self) -> std::io::Result<String> {
        Ok(lock(&self.inner).contents.clone())
    }

    /// Sets the current contents and appends `s` to the history.
    pub fn write(&self, s: &str) -> std::io::Result<()> {
        let mut inner = lock(&self.inner);
        inner.contents.clear();
        inner.contents.push_str(s);
        inner.history.push(s.to_owned());
        Ok(())
    }

    /// Returns the current contents without an error.
    pub fn contents(&self) -> String {
        lock(&self.inner).contents.clone()
    }

    /// Returns a copy of the history, oldest first.
    pub fn history(&self) -> Vec<String> {
        lock(&self.inner).history.clone()
    }
}
