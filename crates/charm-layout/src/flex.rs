//! Flex container: proportional distribution of children along a row or column.

use std::fmt;

use crate::error::{Dimension, Error};
use crate::geometry::{distribute, limit_gap, ratio_is_valid, render_cell, safe_ratio};
use crate::util::{
    as_refs, join_vertical_with_spacer, join_with_spacer, spacer_h, spacer_v, to_i64,
};

/// Direction in which a [`Flex`] container distributes its children.
///
/// This is the Rust counterpart of the `layout.Direction` enum of the Go
/// original, where `Row` is the zero value.
///
/// # Examples
///
/// ```
/// use charm_layout::Direction;
///
/// assert_eq!(Direction::default(), Direction::Row);
/// assert_eq!(Direction::Column.to_string(), "Column");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    /// Children are distributed horizontally, from left to right.
    #[default]
    Row,
    /// Children are distributed vertically, from top to bottom.
    Column,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Row => f.write_str("Row"),
            Self::Column => f.write_str("Column"),
        }
    }
}

/// A child of a [`Flex`] container.
///
/// # Examples
///
/// ```
/// use charm_layout::FlexChild;
///
/// let child = FlexChild {
///     ratio: 0.3,
///     content: "Sidebar".to_string(),
/// };
/// assert_eq!(child.ratio, 0.3);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct FlexChild {
    /// Share of the available space occupied by this child.
    ///
    /// Ratios work like CSS `flex-grow`: the free space is split in proportion
    /// to the ratios, which do not need to add up to `1.0`. Two children with
    /// a ratio of `0.3` each split the space in half. Negative, NaN and
    /// infinite ratios are invalid; see [`Flex::validate`].
    pub ratio: f64,
    /// Textual content rendered inside the child.
    pub content: String,
}

/// A flexible container that distributes its children proportionally.
///
/// The layout engine is inspired by CSS Flexbox: every child receives a share
/// of the available space defined by its [`ratio`](FlexChild::ratio), and the
/// remaining space is distributed according to the container
/// [`direction`](Flex::direction).
///
/// Configuration errors (negative sizes or gaps, invalid ratios) are recorded
/// while the container is built. [`render`](Self::render) always produces a
/// valid geometry; [`try_render`](Self::try_render) reports the error instead.
///
/// # Examples
///
/// ```
/// use charm_core::strip_ansi;
/// use charm_layout::{Direction, Flex};
///
/// let flex = Flex::new(Direction::Row)
///     .set_size(20, 2)
///     .add_child(0.25, "a")
///     .add_child(0.75, "b");
///
/// let rendered = strip_ansi(&flex.render());
/// let first = rendered.lines().next().unwrap_or_default();
/// assert_eq!(first.trim_end(), "a    b");
/// ```
#[derive(Debug, Clone)]
pub struct Flex {
    /// Direction in which the children are distributed.
    pub direction: Direction,
    /// Children of the container.
    ///
    /// The field is public, so it can be changed without going through the
    /// builder methods. Use [`validate`](Self::validate) to check the current
    /// configuration.
    pub children: Vec<FlexChild>,
    /// Spacing, in characters, inserted between two consecutive children.
    /// Negative values are invalid; see [`validate`](Self::validate).
    pub gap: i64,
    /// Available width of the container.
    width: i64,
    /// Available height of the container.
    height: i64,
    /// First configuration error found by the builder methods.
    error: Option<Error>,
}

impl Flex {
    /// Default width used by a newly created container.
    pub const DEFAULT_WIDTH: i64 = 80;
    /// Default height used by a newly created container.
    pub const DEFAULT_HEIGHT: i64 = 24;

    /// Creates a new flex container with the given direction.
    ///
    /// The container starts with the default size of 80x24 characters and no
    /// gap. Use [`set_size`](Self::set_size) to make it responsive.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Column);
    /// assert_eq!(flex.direction, Direction::Column);
    /// assert_eq!(flex.children.len(), 0);
    /// ```
    #[must_use]
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            children: Vec::new(),
            gap: 0,
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
            error: None,
        }
    }

    /// Sets the available width and height of the container.
    ///
    /// Call this before [`render`](Self::render) to make the layout adapt to
    /// the current terminal size. Negative values are recorded as an
    /// [`Error::InvalidSize`] and clamped to zero when rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Row).set_size(40, 10);
    /// let flex = flex.add_child(1.0, "content");
    /// assert_ne!(flex.render(), "");
    /// ```
    #[must_use]
    pub fn set_size(mut self, width: i64, height: i64) -> Self {
        if width < 0 {
            self.record(Error::InvalidSize {
                dimension: Dimension::Width,
                value: width,
            });
        }
        if height < 0 {
            self.record(Error::InvalidSize {
                dimension: Dimension::Height,
                value: height,
            });
        }
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the spacing inserted between two consecutive children.
    ///
    /// A negative gap is recorded as an [`Error::InvalidGap`] and treated as
    /// zero when rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Row).set_gap(2);
    /// assert_eq!(flex.gap, 2);
    /// ```
    #[must_use]
    pub fn set_gap(mut self, gap: i64) -> Self {
        if gap < 0 {
            self.record(Error::InvalidGap(gap));
        }
        self.gap = gap;
        self
    }

    /// Appends a child with the given ratio and content.
    ///
    /// A negative, NaN or infinite ratio is recorded as an
    /// [`Error::InvalidRatio`] and counts as zero when rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Row)
    ///     .add_child(0.3, "Sidebar")
    ///     .add_child(0.7, "Main");
    /// assert_eq!(flex.children.len(), 2);
    /// assert_eq!(flex.children[0].content, "Sidebar");
    /// ```
    #[must_use]
    pub fn add_child(mut self, ratio: f64, content: impl Into<String>) -> Self {
        if !ratio_is_valid(ratio) {
            self.record(Error::InvalidRatio {
                index: self.children.len(),
                ratio,
            });
        }
        self.children.push(FlexChild {
            ratio,
            content: content.into(),
        });
        self
    }

    /// Returns the available width and height of the container.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Row).set_size(33, 7);
    /// assert_eq!(flex.size(), (33, 7));
    /// ```
    #[must_use]
    pub fn size(&self) -> (i64, i64) {
        (self.width, self.height)
    }

    /// Returns the number of children.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Row).add_child(0.5, "a").add_child(0.5, "b");
    /// assert_eq!(flex.child_count(), 2);
    /// ```
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns a copy of the children. Changing the copy does not affect the
    /// container.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Row).add_child(0.5, "A");
    /// let mut copy = flex.children_copy();
    /// copy[0].content = "changed".to_string();
    /// assert_eq!(flex.children[0].content, "A");
    /// ```
    #[must_use]
    pub fn children_copy(&self) -> Vec<FlexChild> {
        self.children.clone()
    }

    /// Checks the current configuration and returns the first error found.
    ///
    /// Errors recorded by the builder methods are reported first, then the
    /// direct edits to the public fields are checked.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidGap`] for a negative gap,
    /// [`Error::InvalidSize`] for a negative width or height,
    /// [`Error::InvalidRatio`] for an invalid ratio and
    /// [`Error::ZeroRatioSum`] when the ratios of the children add up to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Error, Flex};
    ///
    /// let flex = Flex::new(Direction::Row).add_child(0.0, "a");
    /// assert_eq!(flex.validate(), Err(Error::ZeroRatioSum));
    /// ```
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        if self.gap < 0 {
            return Err(Error::InvalidGap(self.gap));
        }
        if self.width < 0 {
            return Err(Error::InvalidSize {
                dimension: Dimension::Width,
                value: self.width,
            });
        }
        if self.height < 0 {
            return Err(Error::InvalidSize {
                dimension: Dimension::Height,
                value: self.height,
            });
        }

        let mut sum = 0.0;
        for (index, child) in self.children.iter().enumerate() {
            if !ratio_is_valid(child.ratio) {
                return Err(Error::InvalidRatio {
                    index,
                    ratio: child.ratio,
                });
            }
            sum += child.ratio;
        }
        if !self.children.is_empty() && sum <= 0.0 {
            return Err(Error::ZeroRatioSum);
        }
        Ok(())
    }

    /// Computes the layout and returns the composed blocks.
    ///
    /// For a [`Direction::Row`] container the children share the width (minus
    /// the gaps) and each one gets the full height. For a
    /// [`Direction::Column`] container the children share the height (minus the
    /// gaps) and each one gets the full width. Each child is exactly as large
    /// as its share: content that does not fit is truncated.
    ///
    /// This variant never fails: invalid configuration is clamped to a valid
    /// geometry. Use [`try_render`](Self::try_render) to detect it.
    ///
    /// An empty container renders as an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_core::strip_ansi;
    /// use charm_layout::{Direction, Flex};
    ///
    /// let flex = Flex::new(Direction::Row)
    ///     .set_size(20, 1)
    ///     .add_child(0.5, "a")
    ///     .add_child(0.5, "b");
    ///
    /// assert_eq!(strip_ansi(&flex.render()).trim_end(), "a         b");
    /// ```
    pub fn render(&self) -> String {
        if self.children.is_empty() {
            return String::new();
        }

        let gap = self.gap.max(0);
        let width = self.width.max(0);
        let height = self.height.max(0);
        let weights: Vec<f64> = self
            .children
            .iter()
            .map(|child| safe_ratio(child.ratio))
            .collect();

        match self.direction {
            Direction::Row => self.render_row(gap, width, height, &weights),
            Direction::Column => self.render_column(gap, width, height, &weights),
        }
    }

    /// Computes the layout like [`render`](Self::render), but reports the
    /// configuration error instead of clamping it.
    ///
    /// # Errors
    ///
    /// Returns the first error reported by [`validate`](Self::validate).
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Direction, Error, Flex};
    ///
    /// let flex = Flex::new(Direction::Row).set_gap(-1).add_child(1.0, "a");
    /// assert_eq!(flex.try_render(), Err(Error::InvalidGap(-1)));
    /// ```
    pub fn try_render(&self) -> Result<String, Error> {
        self.validate()?;
        Ok(self.render())
    }

    /// Stores the first configuration error found while building the container.
    fn record(&mut self, error: Error) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn render_row(&self, gap: i64, width: i64, height: i64, weights: &[f64]) -> String {
        let count = self.children.len();
        let effective_gap = limit_gap(gap, width, count);
        let available = width - effective_gap * (to_i64(count) - 1);

        let widths = distribute(available, weights);
        let block_height = height.max(1);

        let cells: Vec<String> = self
            .children
            .iter()
            .zip(&widths)
            .map(|(child, &cell_width)| render_cell(&child.content, cell_width, block_height))
            .collect();

        if effective_gap > 0 {
            join_with_spacer(&cells, &spacer_h(effective_gap as usize), lipgloss::TOP)
        } else {
            lipgloss::join_horizontal(lipgloss::TOP, &as_refs(&cells))
        }
    }

    fn render_column(&self, gap: i64, width: i64, height: i64, weights: &[f64]) -> String {
        let count = self.children.len();
        let effective_gap = limit_gap(gap, height, count);
        let available = height - effective_gap * (to_i64(count) - 1);

        // Each child takes at least one line.
        let heights: Vec<i64> = distribute(available, weights)
            .into_iter()
            .map(|cell_height| cell_height.max(1))
            .collect();

        let cells: Vec<String> = self
            .children
            .iter()
            .zip(&heights)
            .map(|(child, &cell_height)| render_cell(&child.content, width, cell_height))
            .collect();

        if effective_gap > 0 {
            // The separator block already occupies one line, so `gap - 1`
            // newlines leave exactly `gap` blank lines between two children.
            join_vertical_with_spacer(&cells, &spacer_v((effective_gap - 1) as usize))
        } else {
            lipgloss::join_vertical(lipgloss::LEFT, &as_refs(&cells))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use charm_core::strip_ansi;

    #[test]
    fn new_uses_the_default_size() {
        let flex = Flex::new(Direction::Row);
        assert_eq!(flex.size(), (80, 24));
        assert_eq!(flex.gap, 0);
        assert_eq!(flex.children.len(), 0);
    }

    #[test]
    fn empty_container_renders_empty_string() {
        assert_eq!(Flex::new(Direction::Row).render(), "");
        assert_eq!(Flex::new(Direction::Column).render(), "");
    }

    #[test]
    fn row_splits_the_width_between_children() {
        let flex = Flex::new(Direction::Row)
            .set_size(40, 1)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        assert_eq!(out.trim_end(), "a                   b");
    }

    #[test]
    fn single_child_takes_the_whole_row() {
        // 10 * 0.29 / 0.29 = 10: a single child takes the whole row.
        let flex = Flex::new(Direction::Row)
            .set_size(10, 1)
            .add_child(0.29, "a");
        let out = strip_ansi(&flex.render());
        assert_eq!(out.lines().next().map(str::len), Some(10));
    }

    #[test]
    fn zero_ratio_produces_an_empty_child() {
        let flex = Flex::new(Direction::Row)
            .set_size(20, 1)
            .add_child(0.0, "a")
            .add_child(1.0, "b");
        let out = strip_ansi(&flex.render());
        assert_eq!(out.trim_end(), "b");
    }

    #[test]
    fn ratios_do_not_need_to_add_up_to_one() {
        let flex = Flex::new(Direction::Row)
            .set_size(40, 1)
            .add_child(0.25, "a")
            .add_child(0.25, "b");
        let out = strip_ansi(&flex.render());
        assert_eq!(out.trim_end(), "a                   b");
    }

    #[test]
    fn gap_is_inserted_between_children() {
        let flex = Flex::new(Direction::Row)
            .set_size(10, 1)
            .set_gap(2)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        // 8 columns are left after the gap: 4 for each child.
        assert_eq!(out.trim_end(), "a     b");
    }

    #[test]
    fn gap_larger_than_the_width_is_reduced() {
        let flex = Flex::new(Direction::Row)
            .set_size(4, 1)
            .set_gap(5)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        assert!(out.lines().all(|line| line.chars().count() <= 4));
        assert_eq!(out.trim(), "");
    }

    #[test]
    fn column_stacks_children_vertically() {
        let flex = Flex::new(Direction::Column)
            .set_size(10, 4)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].trim_end(), "a");
        assert_eq!(lines[2].trim_end(), "b");
    }

    #[test]
    fn column_gap_adds_blank_lines() {
        let flex = Flex::new(Direction::Column)
            .set_size(10, 2)
            .set_gap(1)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0].trim_end(), "a");
        assert_eq!(lines[2].trim_end(), "b");
    }

    #[test]
    fn negative_size_is_reported_by_validation() {
        let flex = Flex::new(Direction::Row)
            .set_size(-1, 2)
            .add_child(1.0, "a");
        assert_eq!(
            flex.validate(),
            Err(Error::InvalidSize {
                dimension: Dimension::Width,
                value: -1,
            })
        );
    }
}
