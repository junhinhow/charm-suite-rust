//! Accessibility profiles and graceful degradation for Charm terminal
//! applications, ported from the Go `charm-a11y` package.
//!
//! The entry point is [`Profile::detect`] (or [`Profile::default_profile`],
//! which detects once per process). The resulting [`Profile`] removes colors,
//! swaps Unicode glyphs for ASCII equivalents, disables animations, simplifies
//! borders and produces linear text for screen readers.
//!
//! Detection is conservative and based only on environment variables. The
//! terminal is never queried, so the result is predictable and testable.
//!
//! ```
//! use charm_a11y::Profile;
//!
//! let profile = Profile::detect_from(|key| match key {
//!     "TERM" => Some("dumb".to_owned()),
//!     _ => None,
//! });
//! assert!(!profile.unicode);
//! assert_eq!(profile.symbol("✓", "ok"), "ok");
//! ```
//!
//! # Differences from the Go implementation
//!
//! See `KNOWN_DIFFERENCES.md` at the workspace root. The most visible ones:
//!
//! - `Profile::default_profile` replaces Go's `Default()`; the Rust
//!   [`Default`] trait gives the zero value, as in Go.
//! - `DetectFrom`'s `getenv` returns `Option<String>`, so a `nil` getter in Go
//!   becomes a closure that returns `None`.
//! - [`Profile::announce`] receives an already formatted message. Use
//!   `format!` where Go passes a format string and arguments.
//! - `Profile::width` is a `usize`; `0` means unknown, as in Go.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod ascii;
mod ascii_table;
mod profile;
mod strip;
mod style;

pub use ascii::transliterate;
pub use profile::Profile;
pub use strip::strip_ansi;
