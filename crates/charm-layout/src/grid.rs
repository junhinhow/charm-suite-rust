//! Grid container: automatic distribution of children over equally wide columns.

use crate::error::{Dimension, Error};
use crate::geometry::{distribute, equal_weights, render_cell};
use crate::util::{
    as_refs, join_vertical_with_spacer, join_with_spacer, spacer_h, spacer_v, to_i64,
};

/// A grid that distributes its children automatically into rows of equally
/// wide columns.
///
/// The last row may be incomplete: the missing cells are filled with empty
/// blocks so the alignment of the grid is preserved.
///
/// Configuration errors (fewer than one column, negative sizes or gaps) are
/// recorded while the grid is built. [`render`](Self::render) always produces
/// a valid geometry; [`try_render`](Self::try_render) reports the error.
///
/// # Examples
///
/// ```
/// use charm_core::strip_ansi;
/// use charm_layout::Grid;
///
/// let grid = Grid::new(2)
///     .set_size(20, 4)
///     .add_children(["a", "b", "c"]);
///
/// let rendered = strip_ansi(&grid.render());
/// let first = rendered.lines().next().unwrap_or_default();
/// assert_eq!(first.trim_end(), "a         b");
/// ```
#[derive(Debug, Clone)]
pub struct Grid {
    /// Number of columns of the grid. Values below one are invalid; see
    /// [`validate`](Self::validate).
    pub columns: i64,
    /// Contents of the grid cells.
    pub children: Vec<String>,
    /// Spacing, in characters, inserted between two consecutive cells.
    /// Negative values are invalid; see [`validate`](Self::validate).
    pub gap: i64,
    /// Available width of the grid.
    width: i64,
    /// Available height of the grid.
    height: i64,
    /// First configuration error found by the builder methods.
    error: Option<Error>,
}

impl Grid {
    /// Default width used by a newly created grid.
    pub const DEFAULT_WIDTH: i64 = 80;
    /// Default height used by a newly created grid.
    pub const DEFAULT_HEIGHT: i64 = 24;

    /// Creates a new grid with the given number of columns.
    ///
    /// Column counts below one are recorded as an [`Error::InvalidColumns`] and
    /// clamped to one, exactly like the Go original. Use [`try_new`](Self::try_new)
    /// to receive the error instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// assert_eq!(Grid::new(3).columns, 3);
    /// assert_eq!(Grid::new(0).columns, 1);
    /// ```
    #[must_use]
    pub fn new(columns: i64) -> Self {
        let mut grid = Self {
            columns,
            children: Vec::new(),
            gap: 0,
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
            error: None,
        };
        if columns < 1 {
            grid.record(Error::InvalidColumns(columns));
            grid.columns = 1;
        }
        grid
    }

    /// Creates a new grid and reports an error when the column count is invalid.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidColumns`] when `columns` is below one.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Error, Grid};
    ///
    /// assert_eq!(Grid::try_new(0).unwrap_err(), Error::InvalidColumns(0));
    /// assert!(Grid::try_new(3).is_ok());
    /// ```
    pub fn try_new(columns: i64) -> Result<Self, Error> {
        let grid = Self::new(columns);
        grid.validate()?;
        Ok(grid)
    }

    /// Sets the available width and height of the grid.
    ///
    /// Negative values are recorded as an [`Error::InvalidSize`] and clamped to
    /// zero when rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// let grid = Grid::new(2).set_size(40, 10);
    /// assert_eq!(grid.render(), "");
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

    /// Sets the spacing inserted between two consecutive cells.
    ///
    /// A negative gap is recorded as an [`Error::InvalidGap`] and treated as
    /// zero when rendering.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// assert_eq!(Grid::new(2).set_gap(3).gap, 3);
    /// ```
    #[must_use]
    pub fn set_gap(mut self, gap: i64) -> Self {
        if gap < 0 {
            self.record(Error::InvalidGap(gap));
        }
        self.gap = gap;
        self
    }

    /// Appends a single cell to the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// let grid = Grid::new(2).add_child("a");
    /// assert_eq!(grid.children, vec!["a".to_string()]);
    /// ```
    #[must_use]
    pub fn add_child(mut self, content: impl Into<String>) -> Self {
        self.children.push(content.into());
        self
    }

    /// Appends several cells to the grid at once.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// let grid = Grid::new(3).add_children(["a", "b", "c"]);
    /// assert_eq!(grid.children.len(), 3);
    /// ```
    #[must_use]
    pub fn add_children(mut self, contents: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.children.extend(contents.into_iter().map(Into::into));
        self
    }

    /// Returns the available width and height of the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// assert_eq!(Grid::new(2).set_size(20, 5).size(), (20, 5));
    /// ```
    #[must_use]
    pub fn size(&self) -> (i64, i64) {
        (self.width, self.height)
    }

    /// Returns the number of cells in the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// assert_eq!(Grid::new(2).add_children(["a", "b"]).child_count(), 2);
    /// ```
    #[must_use]
    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    /// Returns a copy of the cells. Changing the copy does not affect the grid.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// let grid = Grid::new(2).add_children(["A", "B"]);
    /// let mut copy = grid.children_copy();
    /// copy[0] = "changed".to_string();
    /// assert_eq!(grid.children[0], "A");
    /// ```
    #[must_use]
    pub fn children_copy(&self) -> Vec<String> {
        self.children.clone()
    }

    /// Checks the current configuration and returns the first error found.
    ///
    /// Errors recorded by the builder methods are reported first, then the
    /// direct edits to the public fields are checked.
    ///
    /// # Errors
    ///
    /// Returns [`Error::InvalidColumns`] when there are fewer than one column,
    /// [`Error::InvalidGap`] for a negative gap and [`Error::InvalidSize`] for a
    /// negative width or height.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::{Error, Grid};
    ///
    /// let mut grid = Grid::new(3);
    /// grid.columns = 0;
    /// assert_eq!(grid.validate(), Err(Error::InvalidColumns(0)));
    /// ```
    pub fn validate(&self) -> Result<(), Error> {
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        if self.columns < 1 {
            return Err(Error::InvalidColumns(self.columns));
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
        Ok(())
    }

    /// Computes the layout of the grid and returns it as a string.
    ///
    /// The children are placed in rows of equally wide columns. The gap shrinks
    /// when it does not fit the available width or height, so the grid never
    /// grows beyond its container.
    ///
    /// This variant never fails: invalid configuration is clamped to a valid
    /// geometry. Use [`try_render`](Self::try_render) to detect it.
    ///
    /// An empty grid renders as an empty string.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_core::strip_ansi;
    /// use charm_layout::Grid;
    ///
    /// let grid = Grid::new(2)
    ///     .set_size(20, 2)
    ///     .add_children(["a", "b"]);
    ///
    /// let rendered = strip_ansi(&grid.render());
    /// assert_eq!(rendered.lines().next().unwrap_or_default().trim_end(), "a         b");
    /// ```
    pub fn render(&self) -> String {
        if self.children.is_empty() {
            return String::new();
        }

        let columns = self.columns.max(1);
        let column_count = usize::try_from(columns).unwrap_or(usize::MAX);
        let row_count = self.children.len().div_ceil(column_count);
        let width = self.width.max(0);
        let height = self.height.max(0);

        // The gap is reduced when it fits neither the width nor the height.
        let mut effective_gap = self.gap.max(0);
        if columns > 1 {
            effective_gap = effective_gap.min(width / (columns - 1));
        }
        if row_count > 1 {
            effective_gap = effective_gap.min(height / (to_i64(row_count) - 1));
        }

        let available_width = width - effective_gap * (columns - 1);
        let available_height = height - effective_gap * (to_i64(row_count) - 1);

        let widths = distribute(available_width, &equal_weights(column_count));
        let heights: Vec<i64> = distribute(available_height, &equal_weights(row_count))
            .into_iter()
            .map(|row_height| row_height.max(1))
            .collect();

        let width_of = |column: usize| widths.get(column).copied().unwrap_or(0);
        let mut rows: Vec<String> = Vec::with_capacity(row_count);

        for (row_index, chunk) in self.children.chunks(column_count).enumerate() {
            let row_height = heights.get(row_index).copied().unwrap_or(1);

            let mut cells: Vec<String> = chunk
                .iter()
                .enumerate()
                .map(|(column, content)| render_cell(content, width_of(column), row_height))
                .collect();

            // Fill the missing cells of the last row to keep the alignment.
            for column in chunk.len()..column_count {
                cells.push(render_cell("", width_of(column), row_height));
            }

            rows.push(if effective_gap > 0 {
                join_with_spacer(&cells, &spacer_h(effective_gap as usize), lipgloss::TOP)
            } else {
                lipgloss::join_horizontal(lipgloss::TOP, &as_refs(&cells))
            });
        }

        if effective_gap > 0 {
            // The separator block already occupies one line, so `gap - 1`
            // newlines leave exactly `gap` blank lines between two rows.
            join_vertical_with_spacer(&rows, &spacer_v((effective_gap - 1) as usize))
        } else {
            lipgloss::join_vertical(lipgloss::LEFT, &as_refs(&rows))
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
    /// use charm_layout::{Error, Grid};
    ///
    /// let grid = Grid::new(2).set_gap(-2).add_child("A");
    /// assert_eq!(grid.try_render(), Err(Error::InvalidGap(-2)));
    /// ```
    pub fn try_render(&self) -> Result<String, Error> {
        self.validate()?;
        Ok(self.render())
    }

    /// Stores the first configuration error found while building the grid.
    fn record(&mut self, error: Error) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use charm_core::strip_ansi;

    #[test]
    fn new_clamps_the_column_count_to_one() {
        assert_eq!(Grid::new(0).columns, 1);
        assert_eq!(Grid::new(1).columns, 1);
        assert_eq!(Grid::new(5).columns, 5);
        assert_eq!(Grid::new(0).validate(), Err(Error::InvalidColumns(0)));
    }

    #[test]
    fn try_new_rejects_invalid_columns() {
        assert_eq!(Grid::try_new(-5).unwrap_err(), Error::InvalidColumns(-5));
        assert!(Grid::try_new(3).is_ok());
    }

    #[test]
    fn empty_grid_renders_empty_string() {
        assert_eq!(Grid::new(3).render(), "");
    }

    #[test]
    fn children_are_split_into_rows() {
        let grid = Grid::new(2).set_size(20, 4).add_children(["a", "b", "c"]);
        let out = strip_ansi(&grid.render());
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].trim_end(), "a         b");
        assert_eq!(lines[2].trim_end(), "c");
    }

    #[test]
    fn odd_number_of_children_leaves_an_incomplete_last_row() {
        let grid = Grid::new(3)
            .set_size(30, 2)
            .add_children(["a", "b", "c", "d"]);
        let out = strip_ansi(&grid.render());
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines[0].trim_end(), "a         b         c");
        assert_eq!(lines[1].trim_end(), "d");
    }

    #[test]
    fn gap_is_inserted_between_cells_and_rows() {
        let grid = Grid::new(2)
            .set_size(20, 4)
            .set_gap(1)
            .add_children(["a", "b", "c", "d"]);
        let out = strip_ansi(&grid.render());
        let lines: Vec<&str> = out.lines().collect();
        // 19 columns are left after the gap: the first cell takes the extra one.
        assert_eq!(lines[0].trim_end(), "a          b");
        assert!(lines.len() >= 4);
    }

    #[test]
    fn zero_columns_does_not_panic() {
        let mut grid = Grid::new(3);
        grid.columns = 0;
        let out = grid.add_child("a").render();
        assert_ne!(out, "");
    }

    #[test]
    fn add_child_appends_to_the_children() {
        let grid = Grid::new(2).add_child("a").add_child("b");
        assert_eq!(grid.children, vec!["a".to_string(), "b".to_string()]);
    }
}
