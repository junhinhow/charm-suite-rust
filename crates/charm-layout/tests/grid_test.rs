//! Port of `grid_test.go` from the Go implementation: `Grid` behaviour.

use charm_core::{display_width, strip_ansi};
use charm_layout::{Dimension, Error, Grid};

/// Display column in which `text` starts in `line`, or `None` when absent.
fn column_of(line: &str, text: &str) -> Option<usize> {
    line.find(text).map(|idx| display_width(&line[..idx]))
}

/// First line of the rendered output, without ANSI codes.
fn first_line(out: &str) -> String {
    strip_ansi(out.split('\n').next().unwrap_or_default())
}

/// Widest line of a rendered block, like `lipgloss.Width`.
fn block_width(out: &str) -> usize {
    out.split('\n')
        .map(|line| display_width(&strip_ansi(line)))
        .max()
        .unwrap_or(0)
}

/// Number of lines of a rendered block, like `lipgloss.Height`.
fn block_height(out: &str) -> usize {
    out.split('\n').count()
}

#[test]
fn zero_columns_do_not_panic_after_mutation() {
    // Regression: `columns` is public and can be zeroed after construction,
    // which used to cause a division by zero while rendering.
    let mut grid = Grid::new(3).add_child("a");
    grid.columns = 0;

    assert_ne!(grid.render(), "");
    assert_eq!(grid.try_render(), Err(Error::InvalidColumns(0)));
}

#[test]
fn negative_columns_do_not_panic_after_mutation() {
    let mut grid = Grid::new(3).add_child("a");
    grid.columns = -5;

    assert_ne!(grid.render(), "");
    assert_eq!(grid.try_render(), Err(Error::InvalidColumns(-5)));
}

#[test]
fn exact_width() {
    let grid = Grid::new(3)
        .set_size(80, 10)
        .set_gap(1)
        .add_children(["A", "B", "C"]);

    assert_eq!(block_width(&grid.render()), 80);
}

#[test]
fn columns_have_equal_width() {
    let grid = Grid::new(3)
        .set_size(80, 6)
        .set_gap(1)
        .add_children(["A", "B", "C"]);

    let line = first_line(&grid.render());

    assert_eq!(column_of(&line, "A"), Some(0));
    assert_eq!(column_of(&line, "B"), Some(27));
    assert_eq!(column_of(&line, "C"), Some(54));
}

#[test]
fn gap_larger_than_the_space_does_not_overflow() {
    // Regression: with gap 10, 10 columns and width 80 the old layout rendered
    // 91 columns, overflowing an 80 column terminal.
    let grid = Grid::new(10)
        .set_size(80, 10)
        .set_gap(10)
        .add_children(["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]);

    assert!(block_width(&grid.render()) <= 80);
}

#[test]
fn every_column_is_distributed() {
    // Integer division alone would lose columns: 10 columns in 80 with gap 1.
    let grid = Grid::new(10)
        .set_size(80, 10)
        .set_gap(1)
        .add_children(["A", "B", "C", "D", "E", "F", "G", "H", "I", "J"]);

    assert_eq!(block_width(&grid.render()), 80);
}

#[test]
fn number_of_rows() {
    let grid = Grid::new(2)
        .set_size(20, 3)
        .add_children(["A", "B", "C", "D", "E"]);

    assert_eq!(grid.render().split('\n').count(), 3);
}

#[test]
fn exact_height() {
    let grid = Grid::new(2)
        .set_size(20, 6)
        .add_children(["A", "B", "C", "D"]);

    assert_eq!(block_height(&grid.render()), 6);
}

#[test]
fn long_content_does_not_overflow() {
    let grid = Grid::new(2)
        .set_size(10, 4)
        .add_children(["conteúdo bem maior que a célula", "B"]);

    assert_eq!(block_width(&grid.render()), 10);
}

#[test]
fn negative_size_is_reported() {
    let grid = Grid::new(2).set_size(-1, 5).add_child("A");

    assert_eq!(
        grid.try_render(),
        Err(Error::InvalidSize {
            dimension: Dimension::Width,
            value: -1,
        })
    );
}

#[test]
fn negative_gap_is_reported() {
    let grid = Grid::new(2).set_gap(-2).add_child("A");

    assert_eq!(grid.try_render(), Err(Error::InvalidGap(-2)));
}

#[test]
fn try_new_reports_invalid_columns() {
    assert_eq!(Grid::try_new(0).unwrap_err(), Error::InvalidColumns(0));
    assert!(Grid::try_new(3).is_ok());
}

#[test]
fn children_copy_does_not_alias_the_grid() {
    let grid = Grid::new(2).add_children(["A", "B"]);

    let mut copy = grid.children_copy();
    copy[0] = "alterado".to_owned();

    assert_eq!(grid.children[0], "A");
}

#[test]
fn empty_grid_renders_nothing() {
    let grid = Grid::new(2).set_size(10, 2);
    assert_eq!(grid.render(), "");
}
