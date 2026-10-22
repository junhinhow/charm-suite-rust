//! Visual reordering of mixed left-to-right and right-to-left text.
//!
//! The embedding levels come from the UAX #9 implementation of
//! `unicode-bidi`. The reordering works on grapheme clusters, as the Go
//! original does, so combining marks and emoji sequences are never split.

use unicode_bidi::{BidiInfo, Level};

use crate::grapheme::grapheme_indices;

/// Returns `text` in visual order (left to right) for a terminal that does
/// not perform bidirectional reordering itself.
///
/// Text without right-to-left characters is returned unchanged.
pub(crate) fn reverse_visual(text: &str) -> String {
    let clusters = grapheme_indices(text);
    if clusters.len() <= 1 {
        return text.to_owned();
    }

    let info = BidiInfo::new(text, None);
    let Some(paragraph) = info.paragraphs.first() else {
        return text.to_owned();
    };
    let paragraph_level = paragraph.level.number();

    let mut levels: Vec<u8> = clusters
        .iter()
        .map(|(start, _)| {
            info.levels
                .get(*start)
                .map_or(paragraph_level, Level::number)
        })
        .collect();

    // L1: separators reset to the paragraph level, and so does whitespace
    // that closes the line.
    for (level, (_, cluster)) in levels.iter_mut().zip(&clusters) {
        if cluster.chars().next().is_some_and(is_separator) {
            *level = paragraph_level;
        }
    }
    for (level, (_, cluster)) in levels.iter_mut().zip(&clusters).rev() {
        match cluster.chars().next() {
            Some(c) if is_space(c) => *level = paragraph_level,
            _ => break,
        }
    }

    // L2: reverse every run at or above each level, from the highest one down
    // to the lowest odd level.
    let max_level = levels.iter().copied().max().unwrap_or(0);
    let Some(min_odd) = levels.iter().copied().filter(|l| l % 2 == 1).min() else {
        return text.to_owned();
    };

    let mut ordered: Vec<&str> = clusters.iter().map(|(_, cluster)| *cluster).collect();
    for level in (min_odd..=max_level).rev() {
        let mut i = 0;
        while i < ordered.len() {
            if levels[i] < level {
                i += 1;
                continue;
            }
            let mut j = i;
            while j < ordered.len() && levels[j] >= level {
                j += 1;
            }
            ordered[i..j].reverse();
            levels[i..j].reverse();
            i = j;
        }
    }

    ordered.concat()
}

/// Reports whether `c` is a paragraph or segment separator (UAX #9 classes
/// B and S).
fn is_separator(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r' | '\u{1f}' | '\u{2029}')
}

/// Reports whether `c` is whitespace that is not a separator.
fn is_space(c: char) -> bool {
    c.is_whitespace() && !is_separator(c)
}

#[cfg(test)]
mod tests {
    use super::reverse_visual;

    const SHALOM: &str = "\u{5e9}\u{5dc}\u{5d5}\u{5dd}";
    const SHALOM_VISUAL: &str = "\u{5dd}\u{5d5}\u{5dc}\u{5e9}";

    #[test]
    fn left_to_right_text_is_unchanged() {
        assert_eq!(reverse_visual("Hello"), "Hello");
        assert_eq!(reverse_visual("abc def"), "abc def");
    }

    #[test]
    fn right_to_left_word_is_reversed() {
        assert_eq!(reverse_visual(SHALOM), SHALOM_VISUAL);
    }

    #[test]
    fn mixed_text_keeps_latin_runs_in_order() {
        let input = format!("Hello {SHALOM}");
        assert_eq!(reverse_visual(&input), format!("Hello {SHALOM_VISUAL}"));
    }

    #[test]
    fn numbers_stay_in_logical_order() {
        let input = format!("{SHALOM} 12");
        assert_eq!(reverse_visual(&input), format!("12 {SHALOM_VISUAL}"));
    }

    #[test]
    fn combining_marks_are_not_split() {
        assert_eq!(reverse_visual("caf\u{e9}"), "caf\u{e9}");
        assert_eq!(reverse_visual("e\u{301}x"), "e\u{301}x");
    }

    #[test]
    fn empty_and_single_cluster_inputs() {
        assert_eq!(reverse_visual(""), "");
        assert_eq!(reverse_visual("a"), "a");
    }
}
