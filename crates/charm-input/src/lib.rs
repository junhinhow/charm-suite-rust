//! Date, time, range, and masked-text inputs for Bubble Tea.
//!
//! This crate is a Rust port of the Go package
//! [`github.com/junhinhow/charm-input`](https://github.com/junhinhow/charm-input).
//!
//! Each component is a plain value that also implements
//! [`charmed_bubbletea::Model`], so it can be driven by the Bubble Tea runtime.
//!
//! ```
//! use charm_input::Date;
//! use chrono::NaiveDate;
//!
//! let mut date = Date::new(NaiveDate::from_ymd_opt(2026, 10, 22).expect("valid date"));
//! date.set_size(40, 8);
//! date.focus();
//! assert!(date.view().contains("October 2026"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::cast_possible_wrap,
    clippy::format_push_string,
    clippy::needless_pass_by_value,
    clippy::assigning_clones
)]

mod common;
mod date;
mod masked;
mod range;
mod time;

pub use common::{
    ascii_only, clamp_float, clamp_int, display_width, finite_float, sanitize_text,
    suffix_to_width, truncate,
};
pub use date::{Date, DateSelectedMsg};
pub use masked::Masked;
pub use range::Range;
pub use time::Time;

pub use charmed_bubbletea;
