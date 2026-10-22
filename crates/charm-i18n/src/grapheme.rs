//! Grapheme cluster splitting shared by the bidirectional reordering.

use unicode_segmentation::UnicodeSegmentation;

/// Splits `text` into extended grapheme clusters (UAX #29), keeping each
/// cluster together with its byte offset in `text`.
pub(crate) fn grapheme_indices(text: &str) -> Vec<(usize, &str)> {
    text.grapheme_indices(true).collect()
}

#[cfg(test)]
mod tests {
    use super::grapheme_indices;

    fn count(text: &str) -> usize {
        grapheme_indices(text).len()
    }

    #[test]
    fn counts_clusters() {
        assert_eq!(count("abc"), 3);
        assert_eq!(count("caf\u{e9}"), 4);
        assert_eq!(count("\u{5e9}\u{5dc}\u{5d5}\u{5dd}"), 4);
        assert_eq!(count(""), 0);
    }

    #[test]
    fn keeps_zwj_sequences_together() {
        assert_eq!(count("\u{1f468}\u{200d}\u{1f469}\u{200d}\u{1f467}"), 1);
    }

    #[test]
    fn keeps_combining_marks_with_their_base() {
        let clusters = grapheme_indices("e\u{301}x");
        assert_eq!(clusters, vec![(0, "e\u{301}"), (3, "x")]);
    }

    #[test]
    fn keeps_regional_indicator_pairs_together() {
        assert_eq!(count("\u{1f1e7}\u{1f1f7}"), 1);
    }
}
