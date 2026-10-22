//! Terminal diff viewer for the [Charm](https://charm.sh) ecosystem.
//!
//! `charm-diff` renders two texts either as a **unified diff** (the classic
//! `@@ -a,b +c,d @@` format) or as a **side-by-side diff** (two columns), using
//! [Lip Gloss](https://github.com/charmbracelet/lipgloss) styles.
//!
//! The diff itself is computed with a dynamic-programming
//! [longest common subsequence](https://en.wikipedia.org/wiki/Longest_common_subsequence)
//! (LCS) algorithm, exactly like the Go original, so the produced output is
//! byte-for-byte identical to `github.com/junhinhow/charm-diff` by default. A
//! linear-space Myers algorithm is also available for large inputs.
//!
//! # Examples
//!
//! Unified diff with the default options (3 lines of context):
//!
//! ```
//! use charm_diff::unified_diff;
//!
//! let old = "line1\nline2\nline3\n";
//! let new = "line1\nline2 changed\nline3\n";
//!
//! let diff = unified_diff(old, new);
//! assert!(diff.contains("-line2"));
//! assert!(diff.contains("+line2 changed"));
//! ```
//!
//! Side-by-side diff with a custom width and style:
//!
//! ```
//! use charm_diff::{side_by_side_diff_with_options, Algorithm, DiffStyle, Options};
//!
//! let opts = Options {
//!     context_lines: 5,
//!     style: DiffStyle::github(),
//!     algorithm: Algorithm::Lcs,
//! };
//!
//! let diff = side_by_side_diff_with_options("hello\nworld\n", "hello\nearth\n", 80, &opts);
//! assert!(diff.contains("world"));
//! assert!(diff.contains("earth"));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod diff;
mod styles;

pub use styles::DiffStyle;

/// Algorithm used to compute the diff operations.
///
/// # Examples
///
/// ```
/// use charm_diff::{Algorithm, Options};
///
/// let options = Options { algorithm: Algorithm::Myers, ..Options::default() };
/// assert_eq!(options.algorithm, Algorithm::Myers);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Algorithm {
    /// Full LCS table. This is the default and matches the original Go output.
    #[default]
    Lcs,
    /// Myers' shortest-edit-script algorithm using linear auxiliary space.
    Myers,
}

/// Configuration used when rendering a diff.
///
/// The default algorithm is [`Algorithm::Lcs`], preserving byte-for-byte
/// parity with the Go implementation. [`Algorithm::Myers`] uses linear
/// auxiliary memory and may choose a different valid operation order when
/// multiple shortest diffs are possible.
///
/// # Examples
///
/// ```
/// use charm_diff::{Algorithm, Options};
///
/// let opts = Options::default();
/// assert_eq!(opts.context_lines, 3);
/// assert_eq!(opts.algorithm, Algorithm::Lcs);
/// ```
#[derive(Clone, Debug)]
pub struct Options {
    /// Number of context lines kept around every change (default: `3`).
    pub context_lines: usize,
    /// Visual style used while rendering.
    pub style: DiffStyle,
    /// Algorithm used to compute the diff (default: [`Algorithm::Lcs`]).
    pub algorithm: Algorithm,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            context_lines: 3,
            style: DiffStyle::default(),
            algorithm: Algorithm::default(),
        }
    }
}

/// Renders a unified diff between two texts using the default options.
///
/// Returns an empty string when both texts are equal.
///
/// # Examples
///
/// ```
/// use charm_diff::unified_diff;
///
/// assert_eq!(unified_diff("same\n", "same\n"), "");
/// ```
pub fn unified_diff(old: &str, new: &str) -> String {
    unified_diff_with_options(old, new, &Options::default())
}

/// Renders a unified diff between two texts using custom options.
///
/// # Examples
///
/// ```
/// use charm_diff::{unified_diff_with_options, Options};
///
/// let opts = Options { context_lines: 0, ..Options::default() };
/// let diff = unified_diff_with_options("a\nb\nc\n", "a\nX\nc\n", &opts);
/// assert!(!diff.contains(" b\n"));
/// ```
pub fn unified_diff_with_options(old: &str, new: &str, opts: &Options) -> String {
    diff::unified_diff(old, new, opts)
}

/// Renders a side-by-side diff between two texts using the default options.
///
/// The `width` is the total width of the rendered output, in columns.
///
/// # Examples
///
/// ```
/// use charm_diff::side_by_side_diff;
///
/// let diff = side_by_side_diff("a\n", "b\n", 80);
/// assert!(diff.contains('|'));
/// ```
pub fn side_by_side_diff(old: &str, new: &str, width: usize) -> String {
    side_by_side_diff_with_options(old, new, width, &Options::default())
}

/// Renders a side-by-side diff between two texts using custom options.
///
/// # Examples
///
/// ```
/// use charm_diff::{side_by_side_diff_with_options, Algorithm, DiffStyle, Options};
///
/// let opts = Options {
///     context_lines: 3,
///     style: DiffStyle::terminal(),
///     algorithm: Algorithm::Lcs,
/// };
/// let diff = side_by_side_diff_with_options("a\n", "b\n", 40, &opts);
/// assert!(diff.contains('|'));
/// ```
pub fn side_by_side_diff_with_options(
    old: &str,
    new: &str,
    width: usize,
    opts: &Options,
) -> String {
    diff::side_by_side_diff(old, new, width, opts)
}
