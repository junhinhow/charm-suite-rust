//! Observable state for terminal UIs: a typed store with subscriptions,
//! bounded undo/redo and pluggable snapshots.
//!
//! This is a port of the Go package `github.com/junhinhow/charm-store`.
//!
//! ```
//! use charm_store::Store;
//! use std::sync::{Arc, Mutex};
//!
//! let counter = Store::new(0);
//! let seen = Arc::new(Mutex::new(Vec::new()));
//! let sink = Arc::clone(&seen);
//! let unsubscribe = counter.subscribe(move |v: i32| sink.lock().unwrap().push(v));
//!
//! counter.set(1);
//! counter.set(1); // equal value: no notification and no history
//! counter.update(|v| v + 1);
//! counter.undo();
//!
//! assert_eq!(counter.get(), 1);
//! assert_eq!(*seen.lock().unwrap(), vec![1, 2, 1]);
//! unsubscribe();
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError, Weak};

use serde::{de::DeserializeOwned, Serialize};

/// Default number of previous states kept for undo.
const DEFAULT_HISTORY_LIMIT: usize = 100;

/// Boxed error returned by a [`Codec`].
pub type CodecError = Box<dyn std::error::Error + Send + Sync>;

/// Errors returned by [`Store::snapshot`] and [`Store::restore`].
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The codec failed while serializing the current value.
    #[error("store: failed to create snapshot: {0}")]
    Snapshot(#[source] CodecError),
    /// The codec failed while deserializing the snapshot. The store is unchanged.
    #[error("store: failed to restore snapshot: {0}")]
    Restore(#[source] CodecError),
}

/// Serializes and deserializes the value of a [`Store`].
///
/// Implement this trait to plug in other formats (YAML, binary, ...).
pub trait Codec {
    /// Converts `value` into bytes.
    ///
    /// # Errors
    ///
    /// Returns the codec's own error when serialization fails.
    fn marshal<V: Serialize + ?Sized>(&self, value: &V) -> Result<Vec<u8>, CodecError>;

    /// Reads `data` and builds a value from it.
    ///
    /// # Errors
    ///
    /// Returns the codec's own error when the data is invalid.
    fn unmarshal<V: DeserializeOwned>(&self, data: &[u8]) -> Result<V, CodecError>;
}

/// A [`Codec`] based on `serde_json`.
///
/// # Examples
///
/// ```
/// use charm_store::{Codec, JsonCodec};
///
/// let bytes = JsonCodec.marshal(&vec![1, 2])?;
/// assert_eq!(bytes, b"[1,2]");
/// # Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
/// ```
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct JsonCodec;

impl Codec for JsonCodec {
    fn marshal<V: Serialize + ?Sized>(&self, value: &V) -> Result<Vec<u8>, CodecError> {
        Ok(serde_json::to_vec(value)?)
    }

    fn unmarshal<V: DeserializeOwned>(&self, data: &[u8]) -> Result<V, CodecError> {
        Ok(serde_json::from_slice(data)?)
    }
}

type Callback<T> = Arc<dyn Fn(T) + Send + Sync>;

struct Subscriber<T> {
    id: u64,
    callback: Callback<T>,
}

struct Inner<T> {
    value: T,
    subs: Vec<Subscriber<T>>,
    next_id: u64,
    undo: Vec<T>,
    redo: Vec<T>,
    limit: usize,
    derived: bool,
}

impl<T> Inner<T> {
    fn record(&mut self, previous: T) {
        if self.derived {
            return;
        }
        self.undo.push(previous);
        trim_front(&mut self.undo, self.limit);
    }

    fn callbacks(&self) -> Vec<Callback<T>> {
        self.subs
            .iter()
            .map(|sub| Arc::clone(&sub.callback))
            .collect()
    }
}

/// An observable container with history for undo and redo.
///
/// Every method takes `&self` and is safe to call from several threads.
/// Subscribers run after the internal lock is released, so a subscriber may
/// call back into the store.
///
/// # Examples
///
/// ```
/// use charm_store::Store;
///
/// let store = Store::new("a".to_string());
/// store.set("b".to_string());
/// assert!(store.undo());
/// assert_eq!(store.get(), "a");
/// ```
pub struct Store<T> {
    inner: Arc<Mutex<Inner<T>>>,
}

impl<T> fmt::Debug for Store<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store").finish_non_exhaustive()
    }
}

fn lock<T>(inner: &Mutex<Inner<T>>) -> MutexGuard<'_, Inner<T>> {
    inner.lock().unwrap_or_else(PoisonError::into_inner)
}

fn dispatch<T: Clone>(callbacks: Vec<Callback<T>>, value: T) {
    for callback in callbacks {
        callback(value.clone());
    }
}

fn trim_front<T>(items: &mut Vec<T>, max: usize) {
    if items.len() > max {
        let excess = items.len() - max;
        items.drain(..excess);
    }
}

impl<T> Store<T> {
    /// Creates a store with `initial` as its value, no history and a limit of
    /// 100 states for undo.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    ///
    /// let store = Store::new(7);
    /// assert_eq!(store.get(), 7);
    /// assert_eq!(store.history(), 0);
    /// ```
    pub fn new(initial: T) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                value: initial,
                subs: Vec::new(),
                next_id: 0,
                undo: Vec::new(),
                redo: Vec::new(),
                limit: DEFAULT_HISTORY_LIMIT,
                derived: false,
            })),
        }
    }

    /// Returns a copy of the current value.
    pub fn get(&self) -> T
    where
        T: Clone,
    {
        lock(&self.inner).value.clone()
    }

    /// Replaces the current value.
    ///
    /// If `value` equals the current one, nothing happens: no history entry
    /// and no notification. Otherwise the previous value goes to the undo
    /// history, the redo history is cleared and subscribers are notified.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    ///
    /// let store = Store::new(1);
    /// store.set(1);
    /// assert_eq!(store.history(), 0);
    /// store.set(2);
    /// assert_eq!(store.history(), 1);
    /// ```
    pub fn set(&self, value: T)
    where
        T: Clone + PartialEq,
    {
        let mut inner = lock(&self.inner);
        if inner.value == value {
            return;
        }
        let previous = std::mem::replace(&mut inner.value, value.clone());
        inner.record(previous);
        inner.redo.clear();
        let callbacks = inner.callbacks();
        drop(inner);
        dispatch(callbacks, value);
    }

    /// Computes the next value with `f` and applies it as [`Store::set`].
    ///
    /// `f` runs outside the lock, so concurrent updates follow last-write-wins
    /// rather than an atomic read-modify-write.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    ///
    /// let store = Store::new(2);
    /// store.update(|v| v * 10);
    /// assert_eq!(store.get(), 20);
    /// ```
    pub fn update<F>(&self, f: F)
    where
        T: Clone + PartialEq,
        F: FnOnce(T) -> T,
    {
        self.set(f(self.get()));
    }

    /// Registers `f` to receive the value on every change and on every
    /// [`Store::notify`]. Observers run in registration order.
    ///
    /// The returned function cancels the subscription. It is idempotent, so
    /// calling it more than once is harmless. If an observer cancels another
    /// one during a notification, the cancelled observer still receives that
    /// round, but not the following ones.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let store = Store::new(0);
    /// let seen = Arc::new(Mutex::new(Vec::new()));
    /// let sink = Arc::clone(&seen);
    /// let cancel = store.subscribe(move |v: i32| sink.lock().unwrap().push(v));
    /// store.set(3);
    /// cancel();
    /// store.set(4);
    /// assert_eq!(*seen.lock().unwrap(), vec![3]);
    /// ```
    pub fn subscribe<F>(&self, f: F) -> impl Fn() + Send + Sync + 'static
    where
        T: Send + 'static,
        F: Fn(T) + Send + Sync + 'static,
    {
        let id = {
            let mut inner = lock(&self.inner);
            inner.next_id += 1;
            let id = inner.next_id;
            inner.subs.push(Subscriber {
                id,
                callback: Arc::new(f),
            });
            id
        };
        let weak: Weak<Mutex<Inner<T>>> = Arc::downgrade(&self.inner);
        move || {
            if let Some(inner) = weak.upgrade() {
                lock(&inner).subs.retain(|sub| sub.id != id);
            }
        }
    }

    /// Calls every subscriber with the current value, even if it has not
    /// changed.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    /// use std::sync::{Arc, Mutex};
    ///
    /// let store = Store::new(5);
    /// let seen = Arc::new(Mutex::new(Vec::new()));
    /// let sink = Arc::clone(&seen);
    /// let _cancel = store.subscribe(move |v: i32| sink.lock().unwrap().push(v));
    /// store.notify();
    /// assert_eq!(*seen.lock().unwrap(), vec![5]);
    /// ```
    pub fn notify(&self)
    where
        T: Clone,
    {
        let (value, callbacks) = {
            let inner = lock(&self.inner);
            (inner.value.clone(), inner.callbacks())
        };
        dispatch(callbacks, value);
    }

    /// Goes back one state and returns `true`.
    ///
    /// Returns `false` and changes nothing when there is no earlier state or
    /// when the store was created by [`select`]. The current value moves to
    /// the redo history and subscribers receive the restored value.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    ///
    /// let store = Store::new(0);
    /// assert!(!store.undo());
    /// store.set(1);
    /// assert!(store.undo());
    /// assert_eq!(store.get(), 0);
    /// ```
    pub fn undo(&self) -> bool
    where
        T: Clone,
    {
        let mut guard = lock(&self.inner);
        let state = &mut *guard;
        if state.derived {
            return false;
        }
        let Some(previous) = state.undo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut state.value, previous.clone());
        state.redo.push(current);
        trim_front(&mut state.redo, state.limit);
        let callbacks = state.callbacks();
        drop(guard);
        dispatch(callbacks, previous);
        true
    }

    /// Re-applies the last undone change and returns `true`.
    ///
    /// Returns `false` and changes nothing when there is nothing to redo or
    /// when the store was created by [`select`]. The current value moves to
    /// the undo history and subscribers receive the restored value.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    ///
    /// let store = Store::new(0);
    /// store.set(1);
    /// store.undo();
    /// assert!(store.redo());
    /// assert_eq!(store.get(), 1);
    /// ```
    pub fn redo(&self) -> bool
    where
        T: Clone,
    {
        let mut guard = lock(&self.inner);
        let state = &mut *guard;
        if state.derived {
            return false;
        }
        let Some(next) = state.redo.pop() else {
            return false;
        };
        let current = std::mem::replace(&mut state.value, next.clone());
        state.undo.push(current);
        trim_front(&mut state.undo, state.limit);
        let callbacks = state.callbacks();
        drop(guard);
        dispatch(callbacks, next);
        true
    }

    /// Returns `true` when [`Store::undo`] would change the value.
    pub fn can_undo(&self) -> bool {
        let inner = lock(&self.inner);
        !inner.derived && !inner.undo.is_empty()
    }

    /// Returns `true` when [`Store::redo`] would change the value.
    pub fn can_redo(&self) -> bool {
        let inner = lock(&self.inner);
        !inner.derived && !inner.redo.is_empty()
    }

    /// Returns how many earlier states are available for undo. Redo entries
    /// are not counted. Stores created by [`select`] always return `0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    ///
    /// let store = Store::new(0);
    /// store.set(1);
    /// store.set(2);
    /// assert_eq!(store.history(), 2);
    /// ```
    pub fn history(&self) -> usize {
        let inner = lock(&self.inner);
        if inner.derived {
            0
        } else {
            inner.undo.len()
        }
    }

    /// Sets how many states are kept for undo and redo. A limit below `1` is
    /// raised to `1`. When the limit shrinks, the oldest states are dropped.
    /// Subscribers are not notified.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::Store;
    ///
    /// let store = Store::new(0);
    /// store.set(1);
    /// store.set(2);
    /// store.set_history_limit(1);
    /// assert_eq!(store.history(), 1);
    /// ```
    pub fn set_history_limit(&self, n: usize) {
        let n = n.max(1);
        let mut inner = lock(&self.inner);
        inner.limit = n;
        trim_front(&mut inner.undo, n);
        trim_front(&mut inner.redo, n);
    }

    /// Serializes the current value with `codec`.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Snapshot`] when the codec fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::{JsonCodec, Store};
    ///
    /// let store = Store::new(vec!["a".to_string()]);
    /// let bytes = store.snapshot(&JsonCodec)?;
    /// assert_eq!(bytes, br#"["a"]"#);
    /// # Ok::<(), charm_store::Error>(())
    /// ```
    pub fn snapshot<C: Codec>(&self, codec: &C) -> Result<Vec<u8>, Error>
    where
        T: Clone + Serialize,
    {
        let value = self.get();
        codec.marshal(&value).map_err(Error::Snapshot)
    }

    /// Replaces the value with the one stored in `data`.
    ///
    /// On success the undo and redo histories are cleared and subscribers are
    /// notified, even when the restored value equals the current one. On
    /// failure the store is left unchanged.
    ///
    /// Restoring a store created by [`select`] is allowed, but the value may
    /// be overwritten by the next change of its source.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Restore`] when the codec cannot read `data`.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_store::{JsonCodec, Store};
    ///
    /// let store = Store::new(1);
    /// store.restore(&JsonCodec, b"42")?;
    /// assert_eq!(store.get(), 42);
    /// assert_eq!(store.history(), 0);
    /// # Ok::<(), charm_store::Error>(())
    /// ```
    pub fn restore<C: Codec>(&self, codec: &C, data: &[u8]) -> Result<(), Error>
    where
        T: Clone + DeserializeOwned,
    {
        let value: T = codec.unmarshal(data).map_err(Error::Restore)?;
        let callbacks = {
            let mut inner = lock(&self.inner);
            inner.value = value.clone();
            inner.undo.clear();
            inner.redo.clear();
            inner.callbacks()
        };
        dispatch(callbacks, value);
        Ok(())
    }
}

/// Creates a derived store that holds `fn(source value)` and follows `source`.
///
/// The derived store only notifies its subscribers when the mapped value
/// actually changes. It keeps no history: `undo` and `redo` return `false`,
/// `can_undo` and `can_redo` return `false` and `history` returns `0`. The
/// subscription to `source` lives as long as `source` does.
///
/// # Examples
///
/// ```
/// use charm_store::{select, Store};
/// use std::sync::{Arc, Mutex};
///
/// let source = Store::new(vec![1, 2, 3]);
/// let size = select(&source, |v: Vec<i32>| v.len());
/// let seen = Arc::new(Mutex::new(Vec::new()));
/// let sink = Arc::clone(&seen);
/// let _cancel = size.subscribe(move |n: usize| sink.lock().unwrap().push(n));
///
/// source.set(vec![1, 2]);
/// source.set(vec![1, 2, 3, 4]);
/// assert_eq!(*seen.lock().unwrap(), vec![2, 4]);
/// ```
pub fn select<T, V, F>(source: &Store<T>, f: F) -> Store<V>
where
    T: Clone + Send + 'static,
    V: Clone + PartialEq + Send + 'static,
    F: Fn(T) -> V + Send + Sync + 'static,
{
    let derived = Store::new(f(source.get()));
    lock(&derived.inner).derived = true;
    let target = Store {
        inner: Arc::clone(&derived.inner),
    };
    let _cancel = source.subscribe(move |value: T| target.set(f(value)));
    derived
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_front_keeps_newest_items() {
        let mut items = vec![1, 2, 3, 4];
        trim_front(&mut items, 2);
        assert_eq!(items, vec![3, 4]);
    }
}
