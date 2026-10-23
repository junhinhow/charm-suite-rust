//! Circular-buffer log viewer for [`charmed_bubbletea`] applications.
//!
//! This is a Rust port of the Go `charm-logs` package. It provides severity
//! and text filtering, stable navigation, follow mode, plain-text export, and
//! a Bubble Tea [`Model`](charmed_bubbletea::Model) implementation.
//!
//! ```
//! use charm_logs::{Entry, Level, Model};
//! use std::time::SystemTime;
//!
//! let mut logs = Model::new();
//! logs.append([Entry::new(Level::Info, "application started")]);
//! assert_eq!(logs.len(), 1);
//! assert_eq!(logs.entries()[0].message, "application started");
//! let _ = SystemTime::now();
//! ```
//!
//! `append` is safe to call from other threads: only the shared circular
//! buffer is synchronized. UI state must be changed on the Bubble Tea update
//! thread.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod buffer;
mod export;
mod level;
mod model;
mod view;

pub use export::{ExportError, ExportedMsg};
pub use level::{parse_level, Level, ParseLevelError};
pub use model::{AppendedMsg, Entry, Model, Option as ModelOption};

/// Configure the circular buffer's maximum number of entries.
///
/// Values below 10 are raised to 10; the default is 1000.
pub fn with_max_entries(n: usize) -> ModelOption {
    ModelOption::max_entries(n)
}

#[cfg(test)]
mod tests;
