//! Flexbox-inspired layout engine for terminal user interfaces.
//!
//! `charm-layout` provides two layout primitives built on top of
//! [`lipgloss`]:
//!
//! - [`Flex`]: distributes children along a row or a column, giving each one a
//!   proportional share of the available space (like CSS `flex-grow`).
//! - [`Grid`]: distributes children over a fixed number of equally wide
//!   columns, wrapping into new rows as needed.
//!
//! Both containers are *responsive*: call [`Flex::set_size`] (or
//! [`Grid::set_size`]) with the current terminal dimensions before rendering.
//!
//! ```
//! use charm_core::strip_ansi;
//! use charm_layout::{Direction, Flex};
//!
//! let flex = Flex::new(Direction::Row)
//!     .set_size(20, 3)
//!     .add_child(0.5, "left")
//!     .add_child(0.5, "right");
//!
//! let rendered = strip_ansi(&flex.render());
//! assert!(rendered.starts_with("left      right"));
//! ```
//!
//! # Parity with the Go original
//!
//! This crate is a faithful port of `github.com/junhinhow/charm-layout`.
//! The space assigned to a child is computed as
//! `int(ratio * float64(available))` — a *truncation*, not a rounding — so
//! the output matches the Go implementation byte for byte.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod util;

pub mod flex;
pub mod grid;

pub use flex::{Direction, Flex, FlexChild};
pub use grid::Grid;
