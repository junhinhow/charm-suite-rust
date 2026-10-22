//! Core diff algorithm: LCS computation, hunk building and rendering.
//!
//! The implementation mirrors the Go original (`charm-diff`) line by line so
//! that the produced output is byte-for-byte identical, including the order in
//! which operations are emitted.

use charm_core::{pad_or_truncate as core_pad_or_truncate, split_lines as core_split_lines};

use crate::styles::DiffStyle;
use crate::{Algorithm, Options};

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

fn compute_selected_diff(
    old_lines: &[&str],
    new_lines: &[&str],
    algorithm: Algorithm,
) -> Vec<DiffOp> {
    match algorithm {
        Algorithm::Lcs => compute_diff(old_lines, new_lines),
        Algorithm::Myers => compute_diff_myers(old_lines, new_lines),
    }
}

/// Computes a shortest edit script using Myers' bidirectional middle-snake
/// search. The frontier vectors are released before recursing, so auxiliary
/// memory remains linear in the input size.
pub(crate) fn compute_diff_myers(old_lines: &[&str], new_lines: &[&str]) -> Vec<DiffOp> {
    let mut ops = Vec::with_capacity(old_lines.len().saturating_add(new_lines.len()));
    compute_diff_myers_range(old_lines, new_lines, 0, 0, &mut ops);
    ops
}

/// Reconstructs one range of a Myers diff, appending operations in source
/// order and retaining absolute line indexes.
fn compute_diff_myers_range(
    old_lines: &[&str],
    new_lines: &[&str],
    old_offset: usize,
    new_offset: usize,
    ops: &mut Vec<DiffOp>,
) {
    let mut prefix_len = 0;
    while prefix_len < old_lines.len().min(new_lines.len())
        && old_lines[prefix_len] == new_lines[prefix_len]
    {
        ops.push(DiffOp {
            kind: OpKind::Equal,
            old_line: line_number(old_offset + prefix_len),
            new_line: line_number(new_offset + prefix_len),
            text: old_lines[prefix_len].to_owned(),
        });
        prefix_len += 1;
    }

    let old_middle = &old_lines[prefix_len..];
    let new_middle = &new_lines[prefix_len..];
    let mut suffix_len = 0;
    while suffix_len < old_middle.len().min(new_middle.len())
        && old_middle[old_middle.len() - suffix_len - 1]
            == new_middle[new_middle.len() - suffix_len - 1]
    {
        suffix_len += 1;
    }

    let old_core = &old_middle[..old_middle.len() - suffix_len];
    let new_core = &new_middle[..new_middle.len() - suffix_len];
    let old_core_offset = old_offset + prefix_len;
    let new_core_offset = new_offset + prefix_len;

    match (old_core.len(), new_core.len()) {
        (0, 0) => {}
        (0, _) => push_inserts(new_core, new_core_offset, ops),
        (_, 0) => push_deletes(old_core, old_core_offset, ops),
        (1, _) => diff_single_old_line(old_core, new_core, old_core_offset, new_core_offset, ops),
        (_, 1) => diff_single_new_line(old_core, new_core, old_core_offset, new_core_offset, ops),
        _ => {
            if let Some((old_split, new_split)) = find_middle_split(old_core, new_core) {
                if (old_split, new_split) != (0, 0)
                    && (old_split, new_split) != (old_core.len(), new_core.len())
                {
                    compute_diff_myers_range(
                        &old_core[..old_split],
                        &new_core[..new_split],
                        old_core_offset,
                        new_core_offset,
                        ops,
                    );
                    compute_diff_myers_range(
                        &old_core[old_split..],
                        &new_core[new_split..],
                        old_core_offset + old_split,
                        new_core_offset + new_split,
                        ops,
                    );
                } else {
                    // A boundary-only overlap is resolved by an exact linear-
                    // space LCS reconstruction rather than risking recursion
                    // without progress.
                    compute_diff_hirschberg(
                        old_core,
                        new_core,
                        old_core_offset,
                        new_core_offset,
                        ops,
                    );
                }
            } else {
                compute_diff_hirschberg(old_core, new_core, old_core_offset, new_core_offset, ops);
            }
        }
    }

    for suffix_index in 0..suffix_len {
        let old_index = old_lines.len() - suffix_len + suffix_index;
        let new_index = new_lines.len() - suffix_len + suffix_index;
        ops.push(DiffOp {
            kind: OpKind::Equal,
            old_line: line_number(old_offset + old_index),
            new_line: line_number(new_offset + new_index),
            text: old_lines[old_index].to_owned(),
        });
    }
}

fn push_inserts(lines: &[&str], offset: usize, ops: &mut Vec<DiffOp>) {
    for (index, text) in lines.iter().enumerate() {
        ops.push(DiffOp {
            kind: OpKind::Insert,
            old_line: -1,
            new_line: line_number(offset + index),
            text: (*text).to_owned(),
        });
    }
}

fn push_deletes(lines: &[&str], offset: usize, ops: &mut Vec<DiffOp>) {
    for (index, text) in lines.iter().enumerate() {
        ops.push(DiffOp {
            kind: OpKind::Delete,
            old_line: line_number(offset + index),
            new_line: -1,
            text: (*text).to_owned(),
        });
    }
}

fn diff_single_old_line(
    old_lines: &[&str],
    new_lines: &[&str],
    old_offset: usize,
    new_offset: usize,
    ops: &mut Vec<DiffOp>,
) {
    if let Some(equal_index) = new_lines.iter().position(|line| *line == old_lines[0]) {
        push_inserts(&new_lines[..equal_index], new_offset, ops);
        ops.push(DiffOp {
            kind: OpKind::Equal,
            old_line: line_number(old_offset),
            new_line: line_number(new_offset + equal_index),
            text: old_lines[0].to_owned(),
        });
        push_inserts(
            &new_lines[equal_index + 1..],
            new_offset + equal_index + 1,
            ops,
        );
    } else {
        push_deletes(old_lines, old_offset, ops);
        push_inserts(new_lines, new_offset, ops);
    }
}

fn diff_single_new_line(
    old_lines: &[&str],
    new_lines: &[&str],
    old_offset: usize,
    new_offset: usize,
    ops: &mut Vec<DiffOp>,
) {
    if let Some(equal_index) = old_lines.iter().position(|line| *line == new_lines[0]) {
        push_deletes(&old_lines[..equal_index], old_offset, ops);
        ops.push(DiffOp {
            kind: OpKind::Equal,
            old_line: line_number(old_offset + equal_index),
            new_line: line_number(new_offset),
            text: new_lines[0].to_owned(),
        });
        push_deletes(
            &old_lines[equal_index + 1..],
            old_offset + equal_index + 1,
            ops,
        );
    } else {
        push_deletes(old_lines, old_offset, ops);
        push_inserts(new_lines, new_offset, ops);
    }
}

/// Finds an overlap of the forward and reverse Myers frontiers.
fn find_middle_split(old_lines: &[&str], new_lines: &[&str]) -> Option<(usize, usize)> {
    let old_len = isize::try_from(old_lines.len()).ok()?;
    let new_len = isize::try_from(new_lines.len()).ok()?;
    let max_distance = (old_len + new_len + 1) / 2;
    let offset = max_distance;
    let vector_len = usize::try_from(2 * max_distance + 3).ok()?;
    let vector_offset = usize::try_from(offset + 1).ok()?;
    let mut forward = vec![-1isize; vector_len];
    let mut reverse = vec![-1isize; vector_len];
    forward[vector_offset + 1] = 0;
    reverse[vector_offset + 1] = 0;
    let delta = old_len - new_len;
    let odd_delta = delta % 2 != 0;

    for distance in 0..=max_distance {
        let mut diagonal = -distance;
        while diagonal <= distance {
            let index = usize::try_from(offset + diagonal + 1).ok()?;
            let mut x = if diagonal == -distance
                || (diagonal != distance && forward[index - 1] < forward[index + 1])
            {
                forward[index + 1]
            } else {
                forward[index - 1] + 1
            };
            let mut y = x - diagonal;
            while x < old_len
                && y < new_len
                && old_lines[usize::try_from(x).ok()?] == new_lines[usize::try_from(y).ok()?]
            {
                x += 1;
                y += 1;
            }
            forward[index] = x;

            if odd_delta {
                let reverse_diagonal = delta - diagonal;
                let reverse_index = offset + reverse_diagonal + 1;
                if reverse_index >= 0
                    && usize::try_from(reverse_index).ok()? < reverse.len()
                    && reverse[usize::try_from(reverse_index).ok()?] != -1
                {
                    let reverse_x = old_len - reverse[usize::try_from(reverse_index).ok()?];
                    if x >= reverse_x {
                        return Some((usize::try_from(x).ok()?, usize::try_from(y).ok()?));
                    }
                }
            }
            diagonal += 2;
        }

        let mut diagonal = -distance;
        while diagonal <= distance {
            let index = usize::try_from(offset + diagonal + 1).ok()?;
            let mut x = if diagonal == -distance
                || (diagonal != distance && reverse[index - 1] < reverse[index + 1])
            {
                reverse[index + 1]
            } else {
                reverse[index - 1] + 1
            };
            let mut y = x - diagonal;
            while x < old_len
                && y < new_len
                && old_lines[usize::try_from(old_len - x - 1).ok()?]
                    == new_lines[usize::try_from(new_len - y - 1).ok()?]
            {
                x += 1;
                y += 1;
            }
            reverse[index] = x;

            if !odd_delta {
                let forward_diagonal = delta - diagonal;
                let forward_index = offset + forward_diagonal + 1;
                if forward_index >= 0
                    && usize::try_from(forward_index).ok()? < forward.len()
                    && forward[usize::try_from(forward_index).ok()?] != -1
                {
                    let forward_x = forward[usize::try_from(forward_index).ok()?];
                    let reverse_x = old_len - x;
                    if forward_x >= reverse_x {
                        let split_x = usize::try_from(forward_x).ok()?;
                        let split_y = usize::try_from(forward_x - forward_diagonal).ok()?;
                        return Some((split_x, split_y));
                    }
                }
            }
            diagonal += 2;
        }
    }

    None
}

/// Linear-space LCS fallback for a degenerate middle-snake split. It computes
/// the midpoint from forward and reverse score rows, then reconstructs both
/// halves recursively.
fn compute_diff_hirschberg(
    old_lines: &[&str],
    new_lines: &[&str],
    old_offset: usize,
    new_offset: usize,
    ops: &mut Vec<DiffOp>,
) {
    if old_lines.is_empty() {
        push_inserts(new_lines, new_offset, ops);
        return;
    }
    if new_lines.is_empty() {
        push_deletes(old_lines, old_offset, ops);
        return;
    }
    if old_lines.len() == 1 {
        diff_single_old_line(old_lines, new_lines, old_offset, new_offset, ops);
        return;
    }
    if new_lines.len() == 1 {
        diff_single_new_line(old_lines, new_lines, old_offset, new_offset, ops);
        return;
    }

    let old_mid = old_lines.len() / 2;
    let forward = lcs_score_row(&old_lines[..old_mid], new_lines);
    let reversed_old = old_lines[old_mid..]
        .iter()
        .rev()
        .copied()
        .collect::<Vec<_>>();
    let reversed_new = new_lines.iter().rev().copied().collect::<Vec<_>>();
    let reverse = lcs_score_row(&reversed_old, &reversed_new);
    let split = (0..=new_lines.len())
        .max_by_key(|&index| forward[index] + reverse[new_lines.len() - index])
        .unwrap_or(0);
    drop(forward);
    drop(reverse);
    drop(reversed_old);
    drop(reversed_new);
    compute_diff_hirschberg(
        &old_lines[..old_mid],
        &new_lines[..split],
        old_offset,
        new_offset,
        ops,
    );
    compute_diff_hirschberg(
        &old_lines[old_mid..],
        &new_lines[split..],
        old_offset + old_mid,
        new_offset + split,
        ops,
    );
}

fn lcs_score_row(old_lines: &[&str], new_lines: &[&str]) -> Vec<usize> {
    let mut scores = vec![0; new_lines.len() + 1];
    for old_line in old_lines {
        let mut diagonal = 0;
        for (index, new_line) in new_lines.iter().enumerate() {
            let previous = scores[index + 1];
            scores[index + 1] = if *old_line == *new_line {
                diagonal + 1
            } else {
                scores[index + 1].max(scores[index])
            };
            diagonal = previous;
        }
    }
    scores
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
    let ops = compute_selected_diff(&old_lines, &new_lines, opts.algorithm);
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
    let ops = compute_selected_diff(&old_lines, &new_lines, opts.algorithm);

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

    #[test]
    fn myers_identical_inputs_are_all_equal() {
        let ops = compute_diff_myers(&["alpha", "beta"], &["alpha", "beta"]);
        assert_eq!(ops.len(), 2);
        assert!(ops.iter().all(|op| op.kind == OpKind::Equal));
    }

    #[test]
    fn myers_handles_insert_delete_and_modification() {
        let old = ["before", "removed", "changed", "after"];
        let new = ["before", "inserted", "changed!", "after"];
        let ops = compute_diff_myers(&old, &new);
        assert!(ops.iter().any(|op| op.kind == OpKind::Insert));
        assert!(ops.iter().any(|op| op.kind == OpKind::Delete));
        assert_eq!(apply_ops(&old, &new, &ops), new);
    }

    #[test]
    fn myers_handles_empty_disjoint_and_repeated_inputs() {
        for (old, new) in [
            (&[][..], &[][..]),
            (&[][..], &["a"][..]),
            (&["a"][..], &[][..]),
            (&["a", "b"][..], &["x", "y"][..]),
            (&["x", "a", "x", "a", "x"][..], &["a", "x", "a"][..]),
        ] {
            let ops = compute_diff_myers(old, new);
            assert_eq!(apply_ops(old, new, &ops), new);
        }
    }

    #[test]
    fn myers_matches_go_example_with_minimal_edits() {
        // Go's TestSideBySideDiff_Basic example: the exact tie order can differ
        // from LCS for ambiguous repeated lines, but both scripts stay minimal.
        let old = ["hello", "world"];
        let new = ["hello", "earth"];
        let lcs = compute_diff(&old, &new);
        let myers = compute_diff_myers(&old, &new);
        assert_eq!(apply_ops(&old, &new, &myers), new);
        assert_eq!(edit_count(&myers), edit_count(&lcs));

        let ambiguous_old = ["a", "b"];
        let ambiguous_new = ["b", "a"];
        let lcs = compute_diff(&ambiguous_old, &ambiguous_new);
        let myers = compute_diff_myers(&ambiguous_old, &ambiguous_new);
        assert_eq!(edit_count(&myers), edit_count(&lcs));
        assert_eq!(
            apply_ops(&ambiguous_old, &ambiguous_new, &myers),
            ambiguous_new
        );
    }

    #[test]
    fn myers_handles_ten_thousand_lines_with_one_change() {
        let old = (0..10_000)
            .map(|line| format!("line {line}"))
            .collect::<Vec<_>>();
        let mut new = old.clone();
        new[5_000] = "changed line".to_owned();
        let old_refs = old.iter().map(String::as_str).collect::<Vec<_>>();
        let new_refs = new.iter().map(String::as_str).collect::<Vec<_>>();

        let start = std::time::Instant::now();
        let ops = compute_diff_myers(&old_refs, &new_refs);
        let elapsed = start.elapsed();

        assert_eq!(ops.len(), 10_001);
        assert!(
            elapsed < std::time::Duration::from_secs(2),
            "took {elapsed:?}"
        );
        assert_eq!(apply_ops(&old_refs, &new_refs, &ops), new_refs);
    }

    #[test]
    fn myers_property_random_sequences_are_valid_and_minimal() {
        let alphabet = ["a", "b", "c", "d"];
        let mut random = Xorshift64(0x9e37_79b9_7f4a_7c15);

        for _ in 0..1_000 {
            let old_len = (random.next() % 16) as usize;
            let new_len = (random.next() % 16) as usize;
            let old = (0..old_len)
                .map(|_| alphabet[(random.next() % alphabet.len() as u64) as usize])
                .collect::<Vec<_>>();
            let new = (0..new_len)
                .map(|_| alphabet[(random.next() % alphabet.len() as u64) as usize])
                .collect::<Vec<_>>();

            let myers = compute_diff_myers(&old, &new);
            let lcs = compute_diff(&old, &new);
            assert_eq!(apply_ops(&old, &new, &myers), new);
            assert!(
                edit_count(&myers) <= edit_count(&lcs),
                "old={old:?}, new={new:?}"
            );
        }
    }

    fn edit_count(ops: &[DiffOp]) -> usize {
        ops.iter().filter(|op| op.kind != OpKind::Equal).count()
    }

    fn apply_ops<'a>(old: &[&'a str], new: &[&'a str], ops: &[DiffOp]) -> Vec<&'a str> {
        let mut old_index = 0;
        let mut new_index = 0;
        let mut output = Vec::new();
        let mut old_equal_lines = Vec::new();
        let mut new_equal_lines = Vec::new();

        for op in ops {
            match op.kind {
                OpKind::Equal => {
                    assert_eq!(old.get(old_index), Some(&op.text.as_str()));
                    assert_eq!(new.get(new_index), Some(&op.text.as_str()));
                    assert_eq!(op.old_line, line_number(old_index));
                    assert_eq!(op.new_line, line_number(new_index));
                    output.push(old[old_index]);
                    old_equal_lines.push(old[old_index]);
                    new_equal_lines.push(new[new_index]);
                    old_index += 1;
                    new_index += 1;
                }
                OpKind::Delete => {
                    assert_eq!(old.get(old_index), Some(&op.text.as_str()));
                    assert_eq!(op.old_line, line_number(old_index));
                    assert_eq!(op.new_line, -1);
                    old_index += 1;
                }
                OpKind::Insert => {
                    assert_eq!(new.get(new_index), Some(&op.text.as_str()));
                    assert_eq!(op.old_line, -1);
                    assert_eq!(op.new_line, line_number(new_index));
                    output.push(new[new_index]);
                    new_index += 1;
                }
            }
        }

        assert_eq!(old_index, old.len());
        assert_eq!(new_index, new.len());
        assert_eq!(old_equal_lines, new_equal_lines);
        output
    }

    struct Xorshift64(u64);

    impl Xorshift64 {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }
    }
}
