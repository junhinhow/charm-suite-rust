//! Configuration errors reported by [`Flex`](crate::Flex) and [`Grid`](crate::Grid).

use std::fmt;

/// Dimension of a container that received an invalid size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dimension {
    /// The width of the container.
    Width,
    /// The height of the container.
    Height,
}

impl fmt::Display for Dimension {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Width => f.write_str("width"),
            Self::Height => f.write_str("height"),
        }
    }
}

/// Configuration error found by the validation of a [`Flex`](crate::Flex) or
/// a [`Grid`](crate::Grid).
///
/// The variants mirror the sentinel errors of the Go original. Go reports an
/// invalid direction as well; that case cannot be represented here because
/// [`Direction`](crate::Direction) is a closed enum.
///
/// # Examples
///
/// ```
/// use charm_layout::{Error, Flex, Direction};
///
/// let flex = Flex::new(Direction::Row).add_child(-1.0, "a");
/// assert!(matches!(flex.validate(), Err(Error::InvalidRatio { index: 0, .. })));
/// ```
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The gap between children or cells is negative.
    #[error("invalid gap {0}: the gap must not be negative")]
    InvalidGap(i64),
    /// The width or the height of the container is negative.
    #[error("invalid {dimension} {value}: the size must not be negative")]
    InvalidSize {
        /// Which dimension was invalid.
        dimension: Dimension,
        /// The rejected value.
        value: i64,
    },
    /// A ratio is negative, NaN or infinite.
    #[error("invalid ratio {ratio} for child {index}: ratios must be finite and not negative")]
    InvalidRatio {
        /// Index of the child that owns the ratio.
        index: usize,
        /// The rejected ratio.
        ratio: f64,
    },
    /// The ratios of all children add up to zero.
    #[error("the ratios of all children add up to zero")]
    ZeroRatioSum,
    /// A grid has fewer than one column.
    #[error("invalid column count {0}: a grid needs at least one column")]
    InvalidColumns(i64),
}
