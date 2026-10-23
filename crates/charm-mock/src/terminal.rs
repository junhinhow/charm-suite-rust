//! Environment and stdin helpers.

use std::collections::BTreeMap;

/// Target that can register cleanup callbacks, mirroring `*testing.T`.
pub trait EnvTarget {
    /// Registers `f` to run during cleanup.
    fn cleanup(&mut self, f: Box<dyn FnOnce() + Send + 'static>);
}

/// Sets environment variables and restores them through `t`'s cleanup mechanism.
///
/// Variables are applied in alphabetical order. A `None` target leaves the
/// variables set without restoration.
pub fn env<T: EnvTarget + ?Sized>(t: Option<&mut T>, vars: &BTreeMap<String, String>) {
    if vars.is_empty() {
        return;
    }
    let mut keys: Vec<&String> = vars.keys().collect();
    keys.sort();

    let mut restorations: Vec<(String, Option<String>)> = Vec::new();
    for key in keys {
        let value = &vars[key];
        let previous = std::env::var(key).ok();
        std::env::set_var(key, value);
        restorations.push((key.clone(), previous));
    }

    if let Some(t) = t {
        t.cleanup(Box::new(move || {
            for (key, prev) in restorations.into_iter().rev() {
                if let Some(v) = prev {
                    std::env::set_var(&key, v);
                } else {
                    std::env::remove_var(&key);
                }
            }
        }));
    }
}

/// Reader that returns `input` immediately, then EOF.
pub fn stdin<T>(_t: Option<&mut T>, input: &str) -> std::io::Cursor<Vec<u8>> {
    std::io::Cursor::new(input.as_bytes().to_vec())
}
