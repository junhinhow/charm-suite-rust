//! Port of `layout_test.go` from the Go implementation: `Flex` behaviour.

use charm_core::{display_width, strip_ansi};
use charm_layout::{Dimension, Direction, Error, Flex, FlexChild};

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
fn flex_exact_width() {
    let flex = Flex::new(Direction::Row)
        .set_size(40, 3)
        .add_child(0.3, "A")
        .add_child(0.7, "B");

    assert_eq!(block_width(&flex.render()), 40);
}

#[test]
fn ratio_does_not_need_to_add_up_to_one() {
    // Flex-grow semantics: 0.3 and 0.3 split the free space in half.
    let flex = Flex::new(Direction::Row)
        .set_size(80, 3)
        .add_child(0.3, "A")
        .add_child(0.3, "B");

    let line = first_line(&flex.render());

    assert_eq!(column_of(&line, "A"), Some(0));
    assert_eq!(column_of(&line, "B"), Some(40));
    assert_eq!(display_width(&line), 80);
}

#[test]
fn invalid_ratio_is_reported_and_tolerated() {
    let flex = Flex::new(Direction::Row)
        .set_size(20, 2)
        .add_child(-0.5, "A");

    assert!(matches!(
        flex.try_render(),
        Err(Error::InvalidRatio { index: 0, .. })
    ));

    // The tolerant variant still produces a valid geometry.
    assert_eq!(block_width(&flex.render()), 20);
}

#[test]
fn column_direction_is_used_for_height() {
    let flex = Flex::new(Direction::Column)
        .set_size(20, 10)
        .set_gap(2)
        .add_child(0.5, "A")
        .add_child(0.5, "B");

    let out = flex.render();
    assert_eq!(block_height(&out), 10);
    assert_eq!(block_width(&out), 20);
}

#[test]
fn gap_is_respected() {
    let flex = Flex::new(Direction::Row)
        .set_size(30, 2)
        .set_gap(3)
        .add_child(0.5, "A")
        .add_child(0.5, "B");

    let line = first_line(&flex.render());

    assert_eq!(column_of(&line, "B"), Some(17));
    assert_eq!(display_width(&line), 30);
}

#[test]
fn gap_larger_than_the_space_is_reduced() {
    let flex = Flex::new(Direction::Row)
        .set_size(10, 2)
        .set_gap(50)
        .add_child(0.5, "A")
        .add_child(0.5, "B");

    assert!(block_width(&flex.render()) <= 10);
}

#[test]
fn long_content_does_not_overflow() {
    let flex = Flex::new(Direction::Row)
        .set_size(12, 2)
        .add_child(0.5, "conteúdo bem maior que a célula");

    assert_eq!(block_width(&flex.render()), 12);
}

#[test]
fn multibyte_content_does_not_overflow() {
    let flex = Flex::new(Direction::Row)
        .set_size(11, 2)
        .add_child(0.5, "日本語日本語");

    assert_eq!(block_width(&flex.render()), 11);
}

#[test]
fn negative_size_is_reported() {
    let flex = Flex::new(Direction::Row)
        .set_size(-10, 5)
        .add_child(0.5, "A");

    assert_eq!(
        flex.try_render(),
        Err(Error::InvalidSize {
            dimension: Dimension::Width,
            value: -10,
        })
    );
}

#[test]
fn negative_gap_is_reported_and_ignored_when_rendering() {
    let flex = Flex::new(Direction::Row).set_gap(-1).add_child(0.5, "A");

    assert_eq!(flex.try_render(), Err(Error::InvalidGap(-1)));
    assert_eq!(block_width(&flex.render()), 80);
}

#[test]
fn zero_ratio_sum_is_reported_and_split_evenly() {
    let flex = Flex::new(Direction::Row)
        .set_size(20, 2)
        .add_child(0.0, "A")
        .add_child(0.0, "B");

    assert_eq!(flex.try_render(), Err(Error::ZeroRatioSum));

    // The tolerant variant divides the space evenly.
    let line = first_line(&flex.render());
    assert_eq!(column_of(&line, "B"), Some(10));
}

#[test]
fn direct_edits_to_the_public_fields_are_validated() {
    let mut flex = Flex::new(Direction::Row).add_child(0.5, "A");
    flex.gap = -3;

    assert_eq!(flex.validate(), Err(Error::InvalidGap(-3)));
}

#[test]
fn children_copy_does_not_alias_the_container() {
    let flex = Flex::new(Direction::Row).add_child(0.5, "A");

    let mut copy: Vec<FlexChild> = flex.children_copy();
    copy[0].content = "alterado".to_owned();

    assert_eq!(flex.children[0].content, "A");
}

#[test]
fn size_and_child_count() {
    let flex = Flex::new(Direction::Row)
        .set_size(33, 7)
        .add_child(0.5, "A")
        .add_child(0.5, "B");

    assert_eq!(flex.size(), (33, 7));
    assert_eq!(flex.child_count(), 2);
}

#[test]
fn empty_flex_renders_nothing() {
    let flex = Flex::new(Direction::Row).set_size(10, 2);
    assert_eq!(flex.render(), "");
}
