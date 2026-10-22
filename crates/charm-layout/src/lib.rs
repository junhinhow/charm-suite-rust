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
//! # Errors
//!
//! Invalid configuration (negative sizes or gaps, invalid ratios, fewer than
//! one column) is recorded while the containers are built. [`Flex::render`] and
//! [`Grid::render`] clamp it to a valid geometry, while
//! [`Flex::try_render`] and [`Grid::try_render`] return the [`Error`].
//!
//! # Parity with the Go original
//!
//! This crate is a port of `github.com/junhinhow/charm-layout`. The space is
//! split with the largest remainder method: each child receives the integer
//! part of its exact share, and the leftover columns go to the children with
//! the largest fractions. The layout therefore always fills the container
//! exactly, and the output matches the Go implementation.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod geometry;
mod util;

pub mod flex;
pub mod grid;

pub use error::{Dimension, Error};
pub use flex::{Direction, Flex, FlexChild};
pub use grid::Grid;
