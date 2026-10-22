//! Flex container: proportional distribution of children along a row or column.

use crate::util::{
    as_refs, join_vertical_with_spacer, join_with_spacer, spacer_h, spacer_v, to_i32, to_i32_dim,
    to_i64,
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
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    /// Children are distributed horizontally, from left to right.
    #[default]
    Row,
    /// Children are distributed vertically, from top to bottom.
    Column,
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
    /// The value is a ratio: `0.3` means 30% of the space left after the
    /// gaps have been removed. Ratios are *not* normalised — the Go original
    /// simply multiplies the ratio by the available space, so a set of ratios
    /// that does not add up to `1.0` leaves free space in the container.
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
    pub children: Vec<FlexChild>,
    /// Spacing, in characters, inserted between two consecutive children.
    pub gap: usize,
    /// Available width of the container.
    width: usize,
    /// Available height of the container.
    height: usize,
}

impl Flex {
    /// Default width used by a newly created container.
    pub const DEFAULT_WIDTH: usize = 80;
    /// Default height used by a newly created container.
    pub const DEFAULT_HEIGHT: usize = 24;

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
        }
    }

    /// Sets the available width and height of the container.
    ///
    /// Call this before [`render`](Self::render) to make the layout adapt to
    /// the current terminal size.
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
    pub fn set_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the spacing inserted between two consecutive children.
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
    pub fn set_gap(mut self, gap: usize) -> Self {
        self.gap = gap;
        self
    }

    /// Appends a child with the given ratio and content.
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
        self.children.push(FlexChild {
            ratio,
            content: content.into(),
        });
        self
    }

    /// Computes the layout and returns the composed blocks.
    ///
    /// For a [`Direction::Row`] container every child receives a width of
    /// `int(ratio * float64(width - total_gap))` and the full height of the
    /// container. For a [`Direction::Column`] container every child receives
    /// the full width of the container and a height of
    /// `int(ratio * float64(height - total_gap))`.
    ///
    /// The space is distributed by truncation, exactly like the Go original.
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

        let total_gap = self.gap.saturating_mul(self.children.len() - 1);
        let mut rendered: Vec<String> = Vec::with_capacity(self.children.len());

        for child in &self.children {
            let style = match self.direction {
                Direction::Row => {
                    let available = to_i64(self.width) - to_i64(total_gap);
                    let child_width = (available as f64 * child.ratio) as i64;
                    lipgloss::Style::new()
                        .width(to_i32_dim(child_width))
                        .height(to_i32(self.height))
                }
                Direction::Column => {
                    let available = to_i64(self.height) - to_i64(total_gap);
                    let child_height = (available as f64 * child.ratio) as i64;
                    lipgloss::Style::new()
                        .width(to_i32(self.width))
                        .height(to_i32_dim(child_height))
                }
            };

            rendered.push(style.render(&child.content));
        }

        match self.direction {
            Direction::Row => {
                if self.gap > 0 {
                    let spacer = spacer_h(self.gap);
                    join_with_spacer(&rendered, &spacer, lipgloss::TOP)
                } else {
                    lipgloss::join_horizontal(lipgloss::TOP, &as_refs(&rendered))
                }
            }
            Direction::Column => {
                if self.gap > 0 {
                    let spacer = spacer_v(self.gap);
                    join_vertical_with_spacer(&rendered, &spacer)
                } else {
                    lipgloss::join_vertical(lipgloss::LEFT, &as_refs(&rendered))
                }
            }
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
    fn row_truncates_the_ratio_instead_of_rounding() {
        // 10 * 0.29 = 2.9 -> truncated to 2
        let flex = Flex::new(Direction::Row)
            .set_size(10, 1)
            .add_child(0.29, "a");
        let out = strip_ansi(&flex.render());
        assert_eq!(out.lines().next().map(str::len), Some(2));
    }

    #[test]
    fn zero_ratio_produces_a_zero_width_child() {
        // A zero width does not truncate the content, it only stops the
        // child from being padded.
        let flex = Flex::new(Direction::Row)
            .set_size(20, 1)
            .add_child(0.0, "a")
            .add_child(1.0, "b");
        let out = strip_ansi(&flex.render());
        assert_eq!(out.trim_end(), "ab");
    }

    #[test]
    fn ratios_that_do_not_add_up_to_one_leave_free_space() {
        let flex = Flex::new(Direction::Row)
            .set_size(40, 1)
            .add_child(0.25, "a")
            .add_child(0.25, "b");
        let out = strip_ansi(&flex.render());
        assert_eq!(out.trim_end(), "a         b");
    }

    #[test]
    fn gap_is_inserted_between_children() {
        let flex = Flex::new(Direction::Row)
            .set_size(10, 1)
            .set_gap(2)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        // Each child is 4 wide (8 available / 2) and the gap adds 2 columns.
        assert_eq!(out.trim_end(), "a     b");
    }

    #[test]
    fn gap_larger_than_the_width_does_not_panic() {
        let flex = Flex::new(Direction::Row)
            .set_size(4, 1)
            .set_gap(5)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        assert!(out.contains('a'));
        assert!(out.contains('b'));
    }

    #[test]
    fn column_stacks_children_vertically() {
        let flex = Flex::new(Direction::Column)
            .set_size(10, 4)
            .add_child(0.5, "a")
            .add_child(0.5, "b");
        let out = strip_ansi(&flex.render());
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines.len() >= 4);
        assert_eq!(lines[0].trim_end(), "a");
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
        assert_eq!(lines[3].trim_end(), "b");
    }
}
