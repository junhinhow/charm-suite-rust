//! Core diff algorithm: LCS computation, hunk building and rendering.
//!
//! The implementation mirrors the Go original (`charm-diff`) line by line so
//! that the produced output is byte-for-byte identical, including the order in
//! which operations are emitted.

use charm_core::{pad_or_truncate as core_pad_or_truncate, split_lines as core_split_lines};

use crate::styles::DiffStyle;
use crate::Options;

/// The kind of a single diff operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum OpKind {
    /// The line is unchanged.
    Equal,
    /// The line was added.
    Insert,
    /// The line was removed.
    Delete,
}

/// A single operation of the diff.
#[derive(Clone, Debug)]
pub(crate) struct DiffOp {
    /// What happened to the line.
    pub(crate) kind: OpKind,
    /// Zero-based line number in the old text (`-1` when not applicable).
    pub(crate) old_line: i64,
    /// Zero-based line number in the new text (`-1` when not applicable).
    pub(crate) new_line: i64,
    /// Content of the line.
    pub(crate) text: String,
}

/// A contiguous block of operations shown as a single hunk.
#[derive(Clone, Debug)]
pub(crate) struct Hunk {
    /// Zero-based first line of the hunk in the old text.
    pub(crate) old_start: i64,
    /// Number of old lines covered by the hunk.
    pub(crate) old_count: i64,
    /// Zero-based first line of the hunk in the new text.
    pub(crate) new_start: i64,
    /// Number of new lines covered by the hunk.
    pub(crate) new_count: i64,
    /// Operations belonging to this hunk.
    pub(crate) ops: Vec<DiffOp>,
}

/// A pair of lines displayed side by side.
#[derive(Clone, Debug)]
pub(crate) struct SidePair {
    /// Operation shown in the left column.
    pub(crate) left: Option<DiffOp>,
    /// Operation shown in the right column.
    pub(crate) right: Option<DiffOp>,
}

/// A contiguous range of operations that contains at least one change.
#[derive(Clone, Copy, Debug)]
struct ChangeRange {
    /// Index of the first operation of the range.
    start: usize,
    /// Index of the last operation of the range.
    end: usize,
}

/// Converts a zero-based index into the line number used by the diff
/// operations.
fn line_number(index: usize) -> i64 {
    i64::try_from(index).unwrap_or(i64::MAX)
}

/// Splits a text into lines, dropping the trailing empty line.
///
/// Equivalent to `splitLines` of the Go original.
pub(crate) fn split_lines(text: &str) -> Vec<&str> {
    core_split_lines(text)
}

/// Pads or truncates a string to a fixed **byte** width.
///
/// Equivalent to `padOrTruncate` of the Go original, which decides whether to
/// pad or truncate using `len(s)` (bytes) and may cut a multi-byte character
/// in half. The Rust port performs the very same decision in bytes but always
/// returns a valid UTF-8 string (invalid sequences become `U+FFFD`).
pub(crate) fn pad_or_truncate(text: &str, width: usize) -> String {
    core_pad_or_truncate(text, width)
}

/// Computes the sequence of operations that transforms `old_lines` into
/// `new_lines`.
///
/// The algorithm is the classic dynamic-programming LCS: the full
/// `(old_len + 1) x (new_len + 1)` table is filled and then walked backwards
/// to rebuild the operations, which are finally reversed. The tie-breaking
/// rules are identical to the Go original, so the order of the operations is
/// preserved exactly.
pub(crate) fn compute_diff(old_lines: &[&str], new_lines: &[&str]) -> Vec<DiffOp> {
    let old_len = old_lines.len();
    let new_len = new_lines.len();

    if old_len == 0 && new_len == 0 {
        return Vec::new();
    }

    // LCS table.
    let mut lcs = vec![vec![0i64; new_len + 1]; old_len + 1];
    for i in 1..=old_len {
        for j in 1..=new_len {
            if old_lines[i - 1] == new_lines[j - 1] {
                lcs[i][j] = lcs[i - 1][j - 1] + 1;
            } else if lcs[i - 1][j] >= lcs[i][j - 1] {
                lcs[i][j] = lcs[i - 1][j];
            } else {
                lcs[i][j] = lcs[i][j - 1];
            }
        }
    }

    // Rebuild the operations walking the table backwards.
    let mut ops: Vec<DiffOp> = Vec::new();
    let mut i = old_len;
    let mut j = new_len;
    while i > 0 || j > 0 {
        if i > 0 && j > 0 && old_lines[i - 1] == new_lines[j - 1] {
            ops.push(DiffOp {
                kind: OpKind::Equal,
                old_line: line_number(i - 1),
                new_line: line_number(j - 1),
                text: old_lines[i - 1].to_owned(),
            });
            i -= 1;
            j -= 1;
        } else if j > 0 && (i == 0 || lcs[i][j - 1] >= lcs[i - 1][j]) {
            ops.push(DiffOp {
                kind: OpKind::Insert,
                old_line: -1,
                new_line: line_number(j - 1),
                text: new_lines[j - 1].to_owned(),
            });
            j -= 1;
        } else if i > 0 {
            ops.push(DiffOp {
                kind: OpKind::Delete,
                old_line: line_number(i - 1),
                new_line: -1,
                text: old_lines[i - 1].to_owned(),
            });
            i -= 1;
        }
    }

    // It was built from the end, so reverse it.
    ops.reverse();
    ops
}

/// Groups the operations into hunks, keeping `context_lines` lines of context
/// around every change.
///
/// Hunks that would overlap are merged, exactly like the Go original does.
pub(crate) fn build_hunks(ops: &[DiffOp], context_lines: usize) -> Vec<Hunk> {
    if ops.is_empty() {
        return Vec::new();
    }

    // Identify the ranges of changes.
    let mut changes: Vec<ChangeRange> = Vec::new();
    for (i, op) in ops.iter().enumerate() {
        if op.kind != OpKind::Equal {
            match changes.last_mut() {
                Some(last) if i <= last.end + 1 => last.end = i,
                _ => changes.push(ChangeRange { start: i, end: i }),
            }
        }
    }

    if changes.is_empty() {
        return Vec::new();
    }

    // Group nearby changes into hunks.
    let mut hunks: Vec<Hunk> = Vec::new();
    for change in &changes {
        let start = change.start.saturating_sub(context_lines);
        let mut end = change.end.saturating_add(context_lines).saturating_add(1);
        if end > ops.len() {
            end = ops.len();
        }

        // Merge with the previous hunk when they overlap.
        let mut merged = false;
        if let Some(last) = hunks.last_mut() {
            let mut last_end: i64 = 0;
            for op in &last.ops {
                if op.old_line >= 0 {
                    last_end = op.old_line;
                }
            }
            let first_op = &ops[start];
            if first_op.old_line >= 0 && first_op.old_line <= last_end + 1 {
                // Extend the previous hunk.
                let mut i = last.ops.len();
                loop {
                    let current_len = last.ops.len();
                    if start + i - current_len >= end {
                        break;
                    }
                    let idx = start + i - current_len;
                    if idx >= start && idx < end {
                        // Avoid duplicates by checking whether the operation is
                        // already part of the hunk.
                        let candidate = &ops[idx];
                        let found = last.ops.iter().any(|existing| {
                            existing.old_line == candidate.old_line
                                && existing.new_line == candidate.new_line
                        });
                        if !found {
                            last.ops.push(candidate.clone());
                        }
                    }
                    i += 1;
                }
                recalculate_hunk(last);
                merged = true;
            }
        }
        if merged {
            continue;
        }

        let mut hunk = Hunk {
            old_start: 0,
            old_count: 0,
            new_start: 0,
            new_count: 0,
            ops: ops[start..end].to_vec(),
        };
        recalculate_hunk(&mut hunk);
        hunks.push(hunk);
    }

    hunks
}

/// Recomputes the counters and the start positions of a hunk from its
/// operations.
fn recalculate_hunk(hunk: &mut Hunk) {
    hunk.old_count = 0;
    hunk.new_count = 0;
    hunk.old_start = -1;
    hunk.new_start = -1;

    for op in &hunk.ops {
        match op.kind {
            OpKind::Equal => {
                hunk.old_count += 1;
                hunk.new_count += 1;
                if hunk.old_start < 0 && op.old_line >= 0 {
                    hunk.old_start = op.old_line;
                }
                if hunk.new_start < 0 && op.new_line >= 0 {
                    hunk.new_start = op.new_line;
                }
            }
            OpKind::Delete => {
                hunk.old_count += 1;
                if hunk.old_start < 0 && op.old_line >= 0 {
                    hunk.old_start = op.old_line;
                }
            }
            OpKind::Insert => {
                hunk.new_count += 1;
                if hunk.new_start < 0 && op.new_line >= 0 {
                    hunk.new_start = op.new_line;
                }
            }
        }
    }

    if hunk.old_start < 0 {
        hunk.old_start = 0;
    }
    if hunk.new_start < 0 {
        hunk.new_start = 0;
    }
}

/// Groups the operations into pairs for the side-by-side layout.
///
/// A deletion directly followed by an insertion is reported as a single
/// "modified line" pair.
pub(crate) fn build_side_by_side_pairs(ops: &[DiffOp]) -> Vec<SidePair> {
    let mut pairs: Vec<SidePair> = Vec::new();
    let mut i = 0;
    while i < ops.len() {
        let op = &ops[i];
        match op.kind {
            OpKind::Equal => {
                pairs.push(SidePair {
                    left: Some(op.clone()),
                    right: Some(op.clone()),
                });
                i += 1;
            }
            OpKind::Delete => {
                // Check whether the next operation is an insertion
                // (a modified line).
                if i + 1 < ops.len() && ops[i + 1].kind == OpKind::Insert {
                    pairs.push(SidePair {
                        left: Some(op.clone()),
                        right: Some(ops[i + 1].clone()),
                    });
                    i += 2;
                } else {
                    pairs.push(SidePair {
                        left: Some(op.clone()),
                        right: None,
                    });
                    i += 1;
                }
            }
            OpKind::Insert => {
                pairs.push(SidePair {
                    left: None,
                    right: Some(op.clone()),
                });
                i += 1;
            }
        }
    }
    pairs
}

/// Formats the line numbers shown in front of every operation.
///
/// Returns the styled `"<old> <new> "` prefix, where each number is padded to
/// four columns and replaced by four spaces when it does not apply.
fn format_line_numbers(op: &DiffOp, style: &DiffStyle) -> String {
    let old = if op.old_line >= 0 {
        format!("{:>4}", op.old_line + 1)
    } else {
        "    ".to_owned()
    };
    let new = if op.new_line >= 0 {
        format!("{:>4}", op.new_line + 1)
    } else {
        "    ".to_owned()
    };
    format!("{} ", style.line_number.render(&format!("{old} {new}")))
}

/// Renders a unified diff.
pub(crate) fn unified_diff(old: &str, new: &str, opts: &Options) -> String {
    let old_lines = split_lines(old);
    let new_lines = split_lines(new);
    let ops = compute_diff(&old_lines, &new_lines);
    let hunks = build_hunks(&ops, opts.context_lines);

    if hunks.is_empty() {
        return String::new();
    }

    let mut buf = String::new();
    let style = &opts.style;

    for hunk in &hunks {
        // Hunk header.
        let header = format!(
            "@@ -{},{} +{},{} @@",
            hunk.old_start + 1,
            hunk.old_count,
            hunk.new_start + 1,
            hunk.new_count
        );
        buf.push_str(&style.hunk_header.render(&header));
        buf.push('\n');

        for op in &hunk.ops {
            let line_num = format_line_numbers(op, style);
            match op.kind {
                OpKind::Equal => {
                    buf.push_str(&line_num);
                    buf.push_str(&style.context.render(&format!(" {}", op.text)));
                    buf.push('\n');
                }
                OpKind::Delete => {
                    buf.push_str(&line_num);
                    buf.push_str(&style.deletion.render(&format!("-{}", op.text)));
                    buf.push('\n');
                }
                OpKind::Insert => {
                    buf.push_str(&line_num);
                    buf.push_str(&style.addition.render(&format!("+{}", op.text)));
                    buf.push('\n');
                }
            }
        }
    }

    buf
}

/// Renders a side-by-side diff of the given total width.
pub(crate) fn side_by_side_diff(old: &str, new: &str, width: usize, opts: &Options) -> String {
    let old_lines = split_lines(old);
    let new_lines = split_lines(new);
    let ops = compute_diff(&old_lines, &new_lines);

    if ops.is_empty() {
        return String::new();
    }

    let style = &opts.style;

    // Width of each column: half of the space left after the separator and
    // the line numbers.
    let line_num_width: i64 = 4;
    let sep_width: i64 = 3;
    let total_width = i64::try_from(width).unwrap_or(i64::MAX);
    let mut col_width = (total_width - sep_width - 2 * line_num_width) / 2;
    if col_width < 10 {
        col_width = 10;
    }
    let col_width = col_width as usize;

    let separator = style.separator.render(" | ");

    // Group the operations into pairs for the side-by-side display.
    let pairs = build_side_by_side_pairs(&ops);

    let mut buf = String::new();
    for pair in &pairs {
        let mut left_num = "    ".to_owned();
        let mut left_text = String::new();
        let mut right_num = "    ".to_owned();
        let mut right_text = String::new();

        if let Some(left) = &pair.left {
            left_num = style
                .line_number
                .render(&format!("{:>4}", left.old_line + 1));
            left_text.clone_from(&left.text);
        }
        if let Some(right) = &pair.right {
            right_num = style
                .line_number
                .render(&format!("{:>4}", right.new_line + 1));
            right_text.clone_from(&right.text);
        }

        // Truncate or pad to a fixed width.
        left_text = pad_or_truncate(&left_text, col_width);
        right_text = pad_or_truncate(&right_text, col_width);

        // Apply the style based on the type of the operation.
        let (left_text, right_text) = match (&pair.left, &pair.right) {
            (Some(left), Some(right))
                if left.kind == OpKind::Delete && right.kind == OpKind::Insert =>
            {
                // Modified line.
                (
                    style.deletion.render(&left_text),
                    style.addition.render(&right_text),
                )
            }
            (Some(left), _) if left.kind == OpKind::Delete => (
                style.deletion.render(&left_text),
                style.context.render(&right_text),
            ),
            (_, Some(right)) if right.kind == OpKind::Insert => (
                style.context.render(&left_text),
                style.addition.render(&right_text),
            ),
            _ => (
                style.context.render(&left_text),
                style.context.render(&right_text),
            ),
        };

        buf.push_str(&left_num);
        buf.push(' ');
        buf.push_str(&left_text);
        buf.push_str(&separator);
        buf.push_str(&right_num);
        buf.push(' ');
        buf.push_str(&right_text);
        buf.push('\n');
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Port of `TestUnifiedDiff_NoChanges` from `diff_test.go`.
    #[test]
    fn unified_diff_no_changes() {
        let text = "line1\nline2\nline3\n";
        let result = crate::unified_diff(text, text);
        assert!(result.is_empty(), "expected an empty diff, got {result:?}");
    }

    /// Port of `TestUnifiedDiff_Addition` from `diff_test.go`.
    #[test]
    fn unified_diff_addition() {
        let old = "line1\nline3\n";
        let new = "line1\nline2\nline3\n";
        let result = crate::unified_diff(old, new);
        assert!(result.contains("+line2"), "expected '+line2' in {result:?}");
    }

    /// Port of `TestUnifiedDiff_Deletion` from `diff_test.go`.
    #[test]
    fn unified_diff_deletion() {
        let old = "line1\nline2\nline3\n";
        let new = "line1\nline3\n";
        let result = crate::unified_diff(old, new);
        assert!(result.contains("-line2"), "expected '-line2' in {result:?}");
    }

    /// Port of `TestSideBySideDiff_Basic` from `diff_test.go`.
    #[test]
    fn side_by_side_diff_basic() {
        let old = "hello\nworld\n";
        let new = "hello\nearth\n";
        let result = crate::side_by_side_diff(old, new, 80);
        assert!(!result.is_empty(), "expected a non-empty result");
        assert!(
            result.contains("world") && result.contains("earth"),
            "expected 'world' and 'earth' in {result:?}"
        );
    }

    /// Port of `TestSideBySideDiff_NoChanges` from `diff_test.go`.
    #[test]
    fn side_by_side_diff_no_changes() {
        let text = "same\n";
        let result = crate::side_by_side_diff(text, text, 80);
        assert!(result.contains("same"), "expected 'same' in {result:?}");
    }

    /// Port of `TestComputeDiff_Empty` from `diff_test.go`.
    #[test]
    fn compute_diff_empty() {
        let ops = compute_diff(&[], &[]);
        assert_eq!(ops.len(), 0);
    }

    /// Port of `TestSplitLines` from `diff_test.go`.
    #[test]
    fn split_lines_drops_trailing_empty_line() {
        let lines = split_lines("a\nb\nc\n");
        assert_eq!(lines, vec!["a", "b", "c"]);
    }

    /// Port of `TestPadOrTruncate` from `diff_test.go`.
    #[test]
    fn pad_or_truncate_pads_and_truncates() {
        let result = pad_or_truncate("hello", 10);
        assert_eq!(result.len(), 10);

        let result = pad_or_truncate("hello world this is long", 10);
        assert_eq!(result.len(), 10);
    }

    #[test]
    fn compute_diff_marks_insertions() {
        let ops = compute_diff(&["a"], &["a", "b"]);
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].kind, OpKind::Equal);
        assert_eq!(ops[1].kind, OpKind::Insert);
        assert_eq!(ops[1].old_line, -1);
        assert_eq!(ops[1].new_line, 1);
    }

    #[test]
    fn compute_diff_marks_deletions() {
        let ops = compute_diff(&["a", "b"], &["a"]);
        assert_eq!(ops.len(), 2);
        assert_eq!(ops[0].kind, OpKind::Equal);
        assert_eq!(ops[1].kind, OpKind::Delete);
        assert_eq!(ops[1].old_line, 1);
        assert_eq!(ops[1].new_line, -1);
    }

    #[test]
    fn compute_diff_empty_inputs() {
        assert!(compute_diff(&[], &[]).is_empty());
        let ops = compute_diff(&[], &["a"]);
        assert_eq!(ops.len(), 1);
        assert_eq!(ops[0].kind, OpKind::Insert);
    }

    #[test]
    fn build_hunks_uses_context_lines() {
        let ops = compute_diff(
            &["1", "2", "3", "4", "5", "6", "7", "8", "9", "10"],
            &["1", "2", "3", "4", "X", "6", "7", "8", "9", "10"],
        );
        let hunks = build_hunks(&ops, 1);
        assert_eq!(hunks.len(), 1);
        // The deleted and the inserted line are adjacent, so the hunk holds
        // one context line before and one after the change.
        assert_eq!(hunks[0].ops.len(), 4);
        assert_eq!(hunks[0].old_start, 3);
        assert_eq!(hunks[0].new_start, 3);
        assert_eq!(hunks[0].old_count, 3);
        assert_eq!(hunks[0].new_count, 3);
    }

    #[test]
    fn build_hunks_splits_distant_changes() {
        let old = [
            "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
        ];
        let new = ["1", "2", "3", "4", "X", "6", "7", "8", "9", "10", "Y", "12"];
        let ops = compute_diff(&old, &new);
        let hunks = build_hunks(&ops, 1);
        assert_eq!(hunks.len(), 2);
    }

    #[test]
    fn side_by_side_pairs_group_deletions_and_insertions() {
        let ops = compute_diff(&["a"], &["b"]);
        let pairs = build_side_by_side_pairs(&ops);
        assert_eq!(pairs.len(), 1);
        assert_eq!(
            pairs[0].left.as_ref().map(|op| op.kind),
            Some(OpKind::Delete)
        );
        assert_eq!(
            pairs[0].right.as_ref().map(|op| op.kind),
            Some(OpKind::Insert)
        );
    }

    #[test]
    fn format_line_numbers_uses_placeholders() {
        let style = DiffStyle::default();
        let op = DiffOp {
            kind: OpKind::Insert,
            old_line: -1,
            new_line: 4,
            text: "x".to_owned(),
        };
        let rendered = format_line_numbers(&op, &style);
        let plain = charm_core::strip_ansi(&rendered);
        assert_eq!(plain, "        5 ");
    }
}
