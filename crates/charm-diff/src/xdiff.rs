//! Git-compatible xdiff engine: linear-space Myers, record cleanup and change compaction.

use std::collections::HashMap;

use crate::diff::{line_number, DiffOp, OpKind};

const MAX_EQ_LIMIT: usize = 1024;
const SIM_SCAN_WINDOW: usize = 100;
const KPDIS_RUN: usize = 4;
const SNAKE_COUNT: usize = 20;
const K_HEURISTIC: isize = 4;
const MAX_COST_MIN: usize = 256;
const HEURISTIC_MIN_COST: usize = 256;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Change {
    pub(crate) i1: usize,
    pub(crate) i2: usize,
    pub(crate) chg1: usize,
    pub(crate) chg2: usize,
}

#[derive(Debug)]
pub(crate) struct Result {
    pub(crate) ops: Vec<DiffOp>,
    pub(crate) script: Vec<Change>,
    pub(crate) change_op_index: Vec<usize>,
    pub(crate) old_len: usize,
    pub(crate) new_len: usize,
}

struct File<'a> {
    lines: &'a [&'a str],
    ids: Vec<usize>,
    changed: Vec<bool>,
    references: Vec<usize>,
    nreff: usize,
    dstart: isize,
    dend: isize,
}

impl<'a> File<'a> {
    fn new(lines: &'a [&'a str]) -> Self {
        Self {
            lines,
            ids: vec![0; lines.len()],
            changed: vec![false; lines.len() + 2],
            references: Vec::new(),
            nreff: 0,
            dstart: 0,
            dend: -1,
        }
    }

    fn changed_at(&self, index: isize) -> bool {
        index >= 0
            && usize::try_from(index).is_ok_and(|i| i < self.lines.len() && self.changed[i + 1])
    }

    fn set_changed(&mut self, index: usize, changed: bool) {
        self.changed[index + 1] = changed;
    }
}

#[derive(Default)]
struct Classifier {
    ids: HashMap<String, usize>,
    counts_old: Vec<usize>,
    counts_new: Vec<usize>,
}

impl Classifier {
    fn classify(&mut self, file: &mut File<'_>, old: bool) {
        for (i, line) in file.lines.iter().enumerate() {
            let id = if let Some(id) = self.ids.get(*line) {
                *id
            } else {
                let id = self.ids.len();
                self.ids.insert((*line).to_owned(), id);
                self.counts_old.push(0);
                self.counts_new.push(0);
                id
            };
            if old {
                self.counts_old[id] += 1;
            } else {
                self.counts_new[id] += 1;
            }
            file.ids[i] = id;
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Split {
    i1: usize,
    i2: usize,
    min_lo: bool,
    min_hi: bool,
}

struct Environment {
    max_cost: usize,
    snake_count: usize,
    heuristic_min: usize,
    offset: isize,
}

/// Computes the same operation ordering and line alignment as Git's xdiff.
pub(crate) fn compute(old_lines: &[&str], new_lines: &[&str]) -> Result {
    let mut old = File::new(old_lines);
    let mut new = File::new(new_lines);
    let mut result = Result {
        ops: Vec::new(),
        script: Vec::new(),
        change_op_index: Vec::new(),
        old_len: old_lines.len(),
        new_len: new_lines.len(),
    };
    if old_lines.is_empty() && new_lines.is_empty() {
        return result;
    }

    let mut classifier = Classifier::default();
    classifier.classify(&mut old, true);
    classifier.classify(&mut new, false);
    trim_ends(&mut old, &mut new);
    cleanup_records(&classifier, &mut old, &mut new);

    let diagonals = old.nreff.saturating_add(new.nreff).saturating_add(3);
    let size = old.nreff.saturating_add(new.nreff).saturating_add(5);
    let mut forward = vec![-1isize; size];
    let mut backward = vec![isize::MAX; size];
    let mut max_cost = bogo_sqrt(diagonals).max(MAX_COST_MIN);
    if max_cost > isize::MAX as usize / 4 {
        max_cost = isize::MAX as usize / 4;
    }
    let env = Environment {
        max_cost,
        snake_count: SNAKE_COUNT,
        heuristic_min: HEURISTIC_MIN_COST,
        offset: isize::try_from(new.nreff).unwrap_or(isize::MAX / 4) + 2,
    };
    let old_nreff = old.nreff;
    let new_nreff = new.nreff;
    recs_cmp(
        &mut old,
        0,
        old_nreff,
        &mut new,
        0,
        new_nreff,
        &mut forward,
        &mut backward,
        false,
        &env,
    );

    compact_changes(&mut old, &new);
    compact_changes(&mut new, &old);
    result.script = build_script(&old, &new);
    (result.ops, result.change_op_index) = build_ops(&old, &new, &result.script);
    result
}

fn bogo_sqrt(mut n: usize) -> usize {
    let mut i = 1usize;
    while n > 0 && i <= usize::MAX / 2 {
        n >>= 2;
        i <<= 1;
    }
    i
}

fn trim_ends(old: &mut File<'_>, new: &mut File<'_>) {
    let limit = old.lines.len().min(new.lines.len());
    let mut i = 0;
    while i < limit && old.ids[i] == new.ids[i] {
        i += 1;
    }
    old.dstart = i.cast_signed();
    new.dstart = i.cast_signed();
    let mut suffix = 0;
    while suffix < limit - i
        && old.ids[old.lines.len() - suffix - 1] == new.ids[new.lines.len() - suffix - 1]
    {
        suffix += 1;
    }
    old.dend = old.lines.len().cast_signed() - suffix.cast_signed() - 1;
    new.dend = new.lines.len().cast_signed() - suffix.cast_signed() - 1;
}

fn cleanup_records(classifier: &Classifier, old: &mut File<'_>, new: &mut File<'_>) {
    let offset = usize::try_from(old.dstart).unwrap_or(0);
    let len_old = usize::try_from(old.dend - old.dstart + 1).unwrap_or(0);
    let len_new = usize::try_from(new.dend - new.dstart + 1).unwrap_or(0);
    let limit_old = bogo_sqrt(old.lines.len()).min(MAX_EQ_LIMIT);
    let limit_new = bogo_sqrt(new.lines.len()).min(MAX_EQ_LIMIT);
    let actions_old = (0..len_old)
        .map(|i| {
            let count = classifier.counts_new[old.ids[offset + i]];
            if count == 0 {
                0
            } else if count < limit_old {
                1
            } else {
                2
            }
        })
        .collect::<Vec<_>>();
    let actions_new = (0..len_new)
        .map(|i| {
            let count = classifier.counts_old[new.ids[offset + i]];
            if count == 0 {
                0
            } else if count < limit_new {
                1
            } else {
                2
            }
        })
        .collect::<Vec<_>>();

    for (i, initial) in actions_old.iter().copied().enumerate() {
        let keep = if initial == 2 {
            !clean_mmatch(&actions_old, i)
        } else {
            initial == 1
        };
        if keep {
            old.references.push(offset + i);
            old.nreff += 1;
        } else {
            old.set_changed(offset + i, true);
        }
    }
    for (i, initial) in actions_new.iter().copied().enumerate() {
        let keep = if initial == 2 {
            !clean_mmatch(&actions_new, i)
        } else {
            initial == 1
        };
        if keep {
            new.references.push(offset + i);
            new.nreff += 1;
        } else {
            new.set_changed(offset + i, true);
        }
    }
}

fn clean_mmatch(actions: &[u8], index: usize) -> bool {
    let start = index.saturating_sub(SIM_SCAN_WINDOW);
    let end = index
        .saturating_add(SIM_SCAN_WINDOW)
        .min(actions.len().saturating_sub(1));
    let mut discarded_before = 0usize;
    let mut potential_before = 1usize;
    for action in actions[start..index].iter().rev() {
        match action {
            0 => discarded_before += 1,
            2 => potential_before += 1,
            _ => break,
        }
    }
    if discarded_before == 0 {
        return false;
    }
    let mut discarded_after = 0usize;
    let mut potential_after = 1usize;
    for action in actions.iter().take(end + 1).skip(index + 1) {
        match action {
            0 => discarded_after += 1,
            2 => potential_after += 1,
            _ => break,
        }
    }
    if discarded_after == 0 {
        return false;
    }
    discarded_after += discarded_before;
    potential_after += potential_before;
    potential_after.saturating_mul(KPDIS_RUN) < potential_after.saturating_add(discarded_after)
}

#[allow(clippy::too_many_arguments)] // Mirrors xdiff's recursive range-and-frontier parameters.
fn recs_cmp(
    old: &mut File<'_>,
    mut off_old: usize,
    mut lim_old: usize,
    new: &mut File<'_>,
    mut off_new: usize,
    mut lim_new: usize,
    forward: &mut [isize],
    backward: &mut [isize],
    need_min: bool,
    env: &Environment,
) {
    while off_old < lim_old
        && off_new < lim_new
        && old.ids[old.references[off_old]] == new.ids[new.references[off_new]]
    {
        off_old += 1;
        off_new += 1;
    }
    while off_old < lim_old
        && off_new < lim_new
        && old.ids[old.references[lim_old - 1]] == new.ids[new.references[lim_new - 1]]
    {
        lim_old -= 1;
        lim_new -= 1;
    }
    if off_old == lim_old {
        for index in off_new..lim_new {
            new.set_changed(new.references[index], true);
        }
        return;
    }
    if off_new == lim_new {
        for index in off_old..lim_old {
            old.set_changed(old.references[index], true);
        }
        return;
    }
    let mut split = Split::default();
    split_search(
        old, off_old, lim_old, new, off_new, lim_new, forward, backward, need_min, &mut split, env,
    );
    recs_cmp(
        old,
        off_old,
        split.i1,
        new,
        off_new,
        split.i2,
        forward,
        backward,
        split.min_lo,
        env,
    );
    recs_cmp(
        old,
        split.i1,
        lim_old,
        new,
        split.i2,
        lim_new,
        forward,
        backward,
        split.min_hi,
        env,
    );
}

#[allow(clippy::too_many_arguments, clippy::too_many_lines)] // Direct port of Git xdiff's middle-snake search.
fn split_search(
    old: &File<'_>,
    off_old: usize,
    lim_old: usize,
    new: &File<'_>,
    off_new: usize,
    lim_new: usize,
    forward: &mut [isize],
    backward: &mut [isize],
    need_min: bool,
    split: &mut Split,
    env: &Environment,
) {
    let dmin = off_old.cast_signed() - lim_new.cast_signed();
    let dmax = lim_old.cast_signed() - off_new.cast_signed();
    let fmid = off_old.cast_signed() - off_new.cast_signed();
    let bmid = lim_old.cast_signed() - lim_new.cast_signed();
    let odd = (fmid - bmid) & 1;
    let (mut fmin, mut fmax) = (fmid, fmid);
    let (mut bmin, mut bmax) = (bmid, bmid);
    let offset = env.offset;
    forward[(fmid + offset) as usize] = off_old.cast_signed();
    backward[(bmid + offset) as usize] = lim_old.cast_signed();

    for cost in 1.. {
        let mut got_snake = false;
        if fmin > dmin {
            fmin -= 1;
            forward[(fmin - 1 + offset) as usize] = -1;
        } else {
            fmin += 1;
        }
        if fmax < dmax {
            fmax += 1;
            forward[(fmax + 1 + offset) as usize] = -1;
        } else {
            fmax -= 1;
        }
        let mut d = fmax;
        while d >= fmin {
            let i = (d + offset) as usize;
            let mut x = if forward[i - 1] >= forward[i + 1] {
                forward[i - 1] + 1
            } else {
                forward[i + 1]
            };
            let previous_x = x;
            let mut y = x - d;
            while x < lim_old.cast_signed()
                && y < lim_new.cast_signed()
                && old.ids[old.references[x as usize]] == new.ids[new.references[y as usize]]
            {
                x += 1;
                y += 1;
            }
            got_snake |= x - previous_x > env.snake_count.cast_signed();
            forward[i] = x;
            if odd != 0 && bmin <= d && d <= bmax && backward[i] <= x {
                split.i1 = x as usize;
                split.i2 = y as usize;
                split.min_lo = true;
                split.min_hi = true;
                return;
            }
            d -= 2;
        }

        if bmin > dmin {
            bmin -= 1;
            backward[(bmin - 1 + offset) as usize] = isize::MAX;
        } else {
            bmin += 1;
        }
        if bmax < dmax {
            bmax += 1;
            backward[(bmax + 1 + offset) as usize] = isize::MAX;
        } else {
            bmax -= 1;
        }
        let mut d = bmax;
        while d >= bmin {
            let i = (d + offset) as usize;
            let mut x = if backward[i - 1] < backward[i + 1] {
                backward[i - 1]
            } else {
                backward[i + 1] - 1
            };
            let previous_x = x;
            let mut y = x - d;
            while x > off_old.cast_signed()
                && y > off_new.cast_signed()
                && old.ids[old.references[(x - 1) as usize]]
                    == new.ids[new.references[(y - 1) as usize]]
            {
                x -= 1;
                y -= 1;
            }
            got_snake |= previous_x - x > env.snake_count.cast_signed();
            backward[i] = x;
            if odd == 0 && fmin <= d && d <= fmax && x <= forward[i] {
                split.i1 = x as usize;
                split.i2 = y as usize;
                split.min_lo = true;
                split.min_hi = true;
                return;
            }
            d -= 2;
        }

        if need_min {
            continue;
        }
        if got_snake && cost > env.heuristic_min {
            let mut best = 0isize;
            let mut d = fmax;
            while d >= fmin {
                let delta = (d - fmid).abs();
                let x = forward[(d + offset) as usize];
                let y = x - d;
                let score = (x - off_old.cast_signed()) + (y - off_new.cast_signed()) - delta;
                if score > K_HEURISTIC * cost.cast_signed()
                    && score > best
                    && x >= off_old.cast_signed() + env.snake_count.cast_signed()
                    && x < lim_old.cast_signed()
                    && y >= off_new.cast_signed() + env.snake_count.cast_signed()
                    && y < lim_new.cast_signed()
                    && (1..=env.snake_count).all(|k| {
                        old.ids[old.references[(x - k.cast_signed()) as usize]]
                            == new.ids[new.references[(y - k.cast_signed()) as usize]]
                    })
                {
                    best = score;
                    split.i1 = x as usize;
                    split.i2 = y as usize;
                }
                d -= 2;
            }
            if best > 0 {
                split.min_lo = true;
                split.min_hi = false;
                return;
            }

            best = 0;
            let mut d = bmax;
            while d >= bmin {
                let delta = (d - bmid).abs();
                let x = backward[(d + offset) as usize];
                let y = x - d;
                let score = (lim_old.cast_signed() - x) + (lim_new.cast_signed() - y) - delta;
                if score > K_HEURISTIC * cost.cast_signed()
                    && score > best
                    && x > off_old.cast_signed()
                    && x <= lim_old.cast_signed() - env.snake_count.cast_signed()
                    && y > off_new.cast_signed()
                    && y <= lim_new.cast_signed() - env.snake_count.cast_signed()
                    && (0..env.snake_count).all(|k| {
                        old.ids[old.references[(x + k.cast_signed()) as usize]]
                            == new.ids[new.references[(y + k.cast_signed()) as usize]]
                    })
                {
                    best = score;
                    split.i1 = x as usize;
                    split.i2 = y as usize;
                }
                d -= 2;
            }
            if best > 0 {
                split.min_lo = false;
                split.min_hi = true;
                return;
            }
        }
        if cost >= env.max_cost {
            let mut fbest = isize::MIN;
            let mut fbest_x = 0;
            let mut d = fmax;
            while d >= fmin {
                let mut x = forward[(d + offset) as usize].min(lim_old.cast_signed());
                let mut y = x - d;
                if lim_new.cast_signed() < y {
                    x = lim_new.cast_signed() + d;
                    y = lim_new.cast_signed();
                }
                if fbest < x + y {
                    fbest = x + y;
                    fbest_x = x;
                }
                d -= 2;
            }
            let mut bbest = isize::MAX;
            let mut bbest_x = isize::MAX;
            let mut d = bmax;
            while d >= bmin {
                let mut x = backward[(d + offset) as usize].max(off_old.cast_signed());
                let mut y = x - d;
                if y < off_new.cast_signed() {
                    x = off_new.cast_signed() + d;
                    y = off_new.cast_signed();
                }
                if x + y < bbest {
                    bbest = x + y;
                    bbest_x = x;
                }
                d -= 2;
            }
            if (lim_old.cast_signed() + lim_new.cast_signed()) - bbest
                < fbest - (off_old.cast_signed() + off_new.cast_signed())
            {
                split.i1 = fbest_x as usize;
                split.i2 = (fbest - fbest_x) as usize;
                split.min_lo = true;
                split.min_hi = false;
            } else {
                split.i1 = bbest_x as usize;
                split.i2 = (bbest - bbest_x) as usize;
                split.min_lo = false;
                split.min_hi = true;
            }
            return;
        }
    }
}

#[derive(Clone, Copy)]
struct Group {
    start: usize,
    end: usize,
}

fn group_init(file: &File<'_>) -> Group {
    let mut group = Group { start: 0, end: 0 };
    while file.changed_at(group.end.cast_signed()) {
        group.end += 1;
    }
    group
}

fn group_next(file: &File<'_>, group: &mut Group) -> bool {
    if group.end == file.lines.len() {
        return false;
    }
    group.start = group.end + 1;
    group.end = group.start;
    while file.changed_at(group.end.cast_signed()) {
        group.end += 1;
    }
    true
}

fn group_previous(file: &File<'_>, group: &mut Group) -> bool {
    if group.start == 0 {
        return false;
    }
    group.end = group.start - 1;
    group.start = group.end;
    while group.start > 0 && file.changed_at(group.start.cast_signed() - 1) {
        group.start -= 1;
    }
    true
}

fn group_slide_down(file: &mut File<'_>, group: &mut Group) -> bool {
    if group.end < file.lines.len() && file.lines[group.start] == file.lines[group.end] {
        file.set_changed(group.start, false);
        group.start += 1;
        file.set_changed(group.end, true);
        group.end += 1;
        while file.changed_at(group.end.cast_signed()) {
            group.end += 1;
        }
        true
    } else {
        false
    }
}

fn group_slide_up(file: &mut File<'_>, group: &mut Group) -> bool {
    if group.start > 0 && file.lines[group.start - 1] == file.lines[group.end - 1] {
        group.start -= 1;
        file.set_changed(group.start, true);
        group.end -= 1;
        file.set_changed(group.end, false);
        while group.start > 0 && file.changed_at(group.start.cast_signed() - 1) {
            group.start -= 1;
        }
        true
    } else {
        false
    }
}

fn compact_changes(file: &mut File<'_>, other: &File<'_>) {
    let mut group = group_init(file);
    let mut other_group = group_init(other);
    loop {
        if group.end != group.start {
            let mut group_size;
            let mut earliest_end;
            let mut end_matching_other;
            loop {
                group_size = group.end - group.start;
                end_matching_other = None;
                while group_slide_up(file, &mut group) {
                    if !group_previous(other, &mut other_group) {
                        break;
                    }
                }
                earliest_end = group.end;
                if other_group.end > other_group.start {
                    end_matching_other = Some(group.end);
                }
                loop {
                    if !group_slide_down(file, &mut group) {
                        break;
                    }
                    if !group_next(other, &mut other_group) {
                        break;
                    }
                    if other_group.end > other_group.start {
                        end_matching_other = Some(group.end);
                    }
                }
                if group_size == group.end - group.start {
                    break;
                }
            }
            if group.end != earliest_end && end_matching_other.is_some() {
                while other_group.end == other_group.start {
                    if !group_slide_up(file, &mut group) {
                        break;
                    }
                    if !group_previous(other, &mut other_group) {
                        break;
                    }
                }
            }
        }
        if !group_next(file, &mut group) || !group_next(other, &mut other_group) {
            break;
        }
    }
}

fn build_script(old: &File<'_>, new: &File<'_>) -> Vec<Change> {
    let mut script = Vec::new();
    let mut i1 = old.lines.len().cast_signed();
    let mut i2 = new.lines.len().cast_signed();
    while i1 >= 0 || i2 >= 0 {
        if old.changed_at(i1 - 1) || new.changed_at(i2 - 1) {
            let end_old = i1;
            while old.changed_at(i1 - 1) {
                i1 -= 1;
            }
            let end_new = i2;
            while new.changed_at(i2 - 1) {
                i2 -= 1;
            }
            script.push(Change {
                i1: usize::try_from(i1).unwrap_or(0),
                i2: usize::try_from(i2).unwrap_or(0),
                chg1: usize::try_from(end_old - i1).unwrap_or(0),
                chg2: usize::try_from(end_new - i2).unwrap_or(0),
            });
        }
        i1 -= 1;
        i2 -= 1;
    }
    script.reverse();
    script
}

fn build_ops(old: &File<'_>, new: &File<'_>, script: &[Change]) -> (Vec<DiffOp>, Vec<usize>) {
    let mut ops = Vec::with_capacity(old.lines.len().saturating_add(new.lines.len()));
    let mut change_indices = Vec::with_capacity(script.len());
    let (mut i1, mut i2) = (0, 0);
    for change in script {
        while i1 < change.i1 && i2 < change.i2 {
            ops.push(DiffOp {
                kind: OpKind::Equal,
                old_line: line_number(i1),
                new_line: line_number(i2),
                text: new.lines[i2].to_owned(),
            });
            i1 += 1;
            i2 += 1;
        }
        change_indices.push(ops.len());
        for k in change.i1..change.i1 + change.chg1 {
            ops.push(DiffOp {
                kind: OpKind::Delete,
                old_line: line_number(k),
                new_line: -1,
                text: old.lines[k].to_owned(),
            });
        }
        for k in change.i2..change.i2 + change.chg2 {
            ops.push(DiffOp {
                kind: OpKind::Insert,
                old_line: -1,
                new_line: line_number(k),
                text: new.lines[k].to_owned(),
            });
        }
        i1 = change.i1 + change.chg1;
        i2 = change.i2 + change.chg2;
    }
    while i1 < old.lines.len() && i2 < new.lines.len() {
        ops.push(DiffOp {
            kind: OpKind::Equal,
            old_line: line_number(i1),
            new_line: line_number(i2),
            text: new.lines[i2].to_owned(),
        });
        i1 += 1;
        i2 += 1;
    }
    (ops, change_indices)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xdiff_reconstructs_inputs_and_uses_linear_sized_frontier() {
        let old = ["head", "same", "gone", "tail"];
        let new = ["head", "same", "added", "tail"];
        let result = compute(&old, &new);
        assert!(result.ops.iter().any(|op| op.kind == OpKind::Delete));
        assert!(result.ops.iter().any(|op| op.kind == OpKind::Insert));
        assert_eq!(result.old_len, old.len());
        assert_eq!(result.new_len, new.len());
    }
}
