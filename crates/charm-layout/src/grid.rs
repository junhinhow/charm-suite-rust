//! Grid container: automatic distribution of children over equally wide columns.

use crate::util::{
    as_refs, join_vertical_with_spacer, join_with_spacer, spacer_h, spacer_v, to_i32_dim, to_i64,
};

/// A grid that distributes its children automatically into rows of equally
/// wide columns.
///
/// The last row may be incomplete: the missing cells are filled with empty
/// blocks so the alignment of the grid is preserved.
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
    /// Number of columns of the grid.
    pub columns: usize,
    /// Contents of the grid cells.
    pub children: Vec<String>,
    /// Spacing, in characters, inserted between two consecutive cells.
    pub gap: usize,
    /// Available width of the grid.
    width: usize,
    /// Available height of the grid.
    height: usize,
}

impl Grid {
    /// Default width used by a newly created grid.
    pub const DEFAULT_WIDTH: usize = 80;
    /// Default height used by a newly created grid.
    pub const DEFAULT_HEIGHT: usize = 24;

    /// Creates a new grid with the given number of columns.
    ///
    /// Column counts below one are clamped to one, exactly like the Go
    /// original.
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
    pub fn new(columns: usize) -> Self {
        Self {
            columns: columns.max(1),
            children: Vec::new(),
            gap: 0,
            width: Self::DEFAULT_WIDTH,
            height: Self::DEFAULT_HEIGHT,
        }
    }

    /// Sets the available width and height of the grid.
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
    pub fn set_size(mut self, width: usize, height: usize) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the spacing inserted between two consecutive cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_layout::Grid;
    ///
    /// assert_eq!(Grid::new(2).set_gap(3).gap, 3);
    /// ```
    #[must_use]
    pub fn set_gap(mut self, gap: usize) -> Self {
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

    /// Computes the layout and returns the composed blocks.
    ///
    /// The width of a column is `(width - gap * (columns - 1)) / columns` and
    /// the height of a row is `(height - gap * (rows - 1)) / rows`, both using
    /// integer division. The last row is padded with empty cells so every row
    /// keeps the same width.
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

        // The Go original divides by `g.Columns`; a grid whose column count
        // was set to zero would panic there. Clamping keeps the port total.
        let columns = self.columns.max(1);

        let total_h_gap = self.gap.saturating_mul(columns - 1);
        let col_width = (to_i64(self.width) - to_i64(total_h_gap)) / to_i64(columns);

        let row_count = self.children.len().div_ceil(columns);
        let total_v_gap = self.gap.saturating_mul(row_count - 1);
        let row_height = if row_count > 0 {
            (to_i64(self.height) - to_i64(total_v_gap)) / to_i64(row_count)
        } else {
            0
        };

        let cell_style = lipgloss::Style::new()
            .width(to_i32_dim(col_width))
            .height(to_i32_dim(row_height));

        let mut rows: Vec<String> = Vec::with_capacity(row_count);

        let mut index = 0usize;
        while index < self.children.len() {
            let end = (index + columns).min(self.children.len());

            let mut cells: Vec<String> = self.children[index..end]
                .iter()
                .map(|content| cell_style.render(content))
                .collect();

            // Fill the missing cells of the last row to keep the alignment.
            while cells.len() < columns {
                cells.push(cell_style.render(""));
            }

            let row = if self.gap > 0 {
                join_with_spacer(&cells, &spacer_h(self.gap), lipgloss::TOP)
            } else {
                lipgloss::join_horizontal(lipgloss::TOP, &as_refs(&cells))
            };

            rows.push(row);
            index += columns;
        }

        if self.gap > 0 {
            join_vertical_with_spacer(&rows, &spacer_v(self.gap))
        } else {
            lipgloss::join_vertical(lipgloss::LEFT, &as_refs(&rows))
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
        // Each cell is 9 wide ((20 - 1) / 2) and the gap adds one column.
        assert_eq!(lines[0].trim_end(), "a         b");
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
