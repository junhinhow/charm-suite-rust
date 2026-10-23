//! Errors returned by the `try_*` renderers.

use thiserror::Error;

/// Error returned by the `try_*` renderers when their input is invalid.
///
/// The Go original also has an `ErrNilColor` value. It has no counterpart
/// here, because a [`charm_core::Rgb`] value cannot be nil.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum Error {
    /// No color was provided.
    #[error("no colors provided")]
    NoColors,

    /// No theme was provided.
    #[error("no themes provided")]
    NoThemes,

    /// The display width is zero.
    #[error("invalid display width: {0}")]
    InvalidWidth(usize),

    /// The gradient dimensions include a zero.
    #[error("invalid gradient size: {width}x{height}")]
    InvalidDimensions {
        /// The requested width.
        width: usize,
        /// The requested height.
        height: usize,
    },
}
