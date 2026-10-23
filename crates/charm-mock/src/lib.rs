//! Deterministic fakes for the outside world of a TUI: clock, filesystem, HTTP,
//! clipboard and environment.
//!
//! This crate is a Rust port of the Go library
//! [`github.com/junhinhow/charm-mock`](https://github.com/junhinhow/charm-mock).
//!
//! # Example
//!
//! ```
//! use std::time::{Duration, SystemTime, UNIX_EPOCH};
//!
//! use charm_mock::{Clock, FakeClock};
//!
//! let clock = FakeClock::new(UNIX_EPOCH);
//! assert_eq!(clock.now(), UNIX_EPOCH);
//! clock.advance(Duration::from_secs(2));
//! assert_eq!(clock.since(UNIX_EPOCH), Duration::from_secs(2));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::io;
use std::sync::Mutex;

mod clipboard;
mod clock;
mod fs;
mod http;
mod terminal;

pub use clipboard::Clipboard;
pub use clock::{Clock, FakeClock};
pub use fs::{
    DirEntry, FileInfo, FileSystem, MemFS, OP_READ, OP_READDIR, OP_REMOVE, OP_STAT, OP_WRITE,
};
pub use http::{HttpClient, HttpMock, HttpResponse, Request, ResponseWriter};
pub use terminal::{env, stdin, EnvTarget};

// Silence unused import warnings for modules that use these.
#[allow(unused)]
fn _assert_send_sync() {
    fn is_send_sync<T: Send + Sync>() {}
    is_send_sync::<FakeClock>();
    is_send_sync::<Clipboard>();
    is_send_sync::<MemFS>();
    is_send_sync::<HttpMock>();
}

// Helper for modules.
pub(crate) fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

// Provide io re-export.
pub use io::Error as IoError;
