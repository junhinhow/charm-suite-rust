//! Terminal spinner animations, ported from the Go `charm-spinner` package.
//!
//! This crate provides the 30 predefined spinner animations of
//! [`charm-spinner`](https://github.com/junhinhow/charm-spinner) together with
//! the [`Spinner`] type used to describe a custom animation.
//!
//! Every predefined spinner is exposed as a function ([`dots`], [`line()`],
//! [`rocket`], ...) and all of them are available at once through [`all`].
//!
//! ```
//! use charm_spinner::dots;
//!
//! let spinner = dots();
//! assert_eq!(spinner.frames.len(), 10);
//! assert_eq!(spinner.interval.as_millis(), 80);
//!
//! // The frame index cycles with wrap-around.
//! assert_eq!(spinner.render(0), spinner.render(spinner.frames.len()));
//! ```
//!
//! # Rendering
//!
//! [`Spinner::render`] returns the frame at the given index, wrapping around
//! when the index goes past the last frame. The frame is styled with
//! [`DEFAULT_STYLE`], the Rust counterpart of the Go package-level
//! `var Style = lipgloss.NewStyle()`.
//!
//! ```
//! use std::time::Duration;
//! use charm_spinner::Spinner;
//!
//! let spinner = Spinner::new(vec!["o".to_owned(), "O".to_owned()], Duration::from_millis(120));
//! assert_eq!(spinner.render(0), "o");
//! assert_eq!(spinner.render(1), "O");
//! assert_eq!(spinner.render(2), "o");
//! ```
//!
//! # Differences from the Go implementation
//!
//! - The Go package exposes the spinners as mutable package-level variables;
//!   Rust exposes them as functions returning an owned [`Spinner`].
//! - `Spinner::render` returns an empty string for a spinner without frames,
//!   while the Go version panics with an index out of range error.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod spinners;

use std::sync::LazyLock;
use std::time::Duration;

pub use spinners::*;

/// The global style applied to every rendered frame.
///
/// This is the Rust counterpart of the Go package-level
/// `var Style = lipgloss.NewStyle()`: an empty (unstyled) lipgloss style.
///
/// # Examples
///
/// ```
/// let style = &charm_spinner::DEFAULT_STYLE;
/// assert_eq!(style.render("x"), "x");
/// ```
pub static DEFAULT_STYLE: LazyLock<lipgloss::Style> = LazyLock::new(lipgloss::Style::new);

/// Returns a copy of the default (unstyled) lipgloss style.
///
/// # Examples
///
/// ```
/// let style = charm_spinner::default_style();
/// assert_eq!(style.render("x"), "x");
/// ```
pub fn default_style() -> lipgloss::Style {
    DEFAULT_STYLE.clone()
}

/// A terminal spinner animation: an ordered list of frames and the delay
/// between them.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use charm_spinner::Spinner;
///
/// let spinner = Spinner::new(vec!["-".to_owned(), "\\".to_owned()], Duration::from_millis(130));
/// assert_eq!(spinner.frames.len(), 2);
/// assert_eq!(spinner.interval, Duration::from_millis(130));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spinner {
    /// The animation frames, in order.
    pub frames: Vec<String>,
    /// The delay between two consecutive frames.
    pub interval: Duration,
}

impl Spinner {
    /// Creates a spinner from its frames and the interval between them.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use charm_spinner::Spinner;
    ///
    /// let spinner = Spinner::new(vec!["a".to_owned()], Duration::from_millis(50));
    /// assert_eq!(spinner.render(0), "a");
    /// ```
    pub fn new(frames: Vec<String>, interval: Duration) -> Self {
        Self { frames, interval }
    }

    /// Renders the frame at the given index, cycling with wrap-around.
    ///
    /// This mirrors the Go implementation, which computes
    /// `frames[frame % len(frames)]`. The selected frame is rendered with
    /// [`DEFAULT_STYLE`].
    ///
    /// A spinner without frames renders as an empty string; the Go version
    /// panics with an index out of range error in that case.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_spinner::line;
    ///
    /// let spinner = line();
    /// let last = spinner.frames.len() - 1;
    /// assert_eq!(spinner.render(last), spinner.frames[last]);
    /// assert_eq!(spinner.render(last + 1), spinner.frames[0]);
    /// ```
    pub fn render(&self, frame: usize) -> String {
        if self.frames.is_empty() {
            return String::new();
        }
        let selected = &self.frames[frame % self.frames.len()];
        DEFAULT_STYLE.render(selected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_the_thirty_spinners() {
        assert_eq!(all().len(), 30);
    }

    #[test]
    fn all_keys_match_the_go_names() {
        let expected = [
            "Arrow",
            "Balloon",
            "Bar",
            "Bounce",
            "Christmas",
            "Circle",
            "Clock",
            "DNA",
            "Dots",
            "Earth",
            "Fading",
            "Fire",
            "Flip",
            "Grow",
            "Hearts",
            "Layer",
            "Line",
            "Matrix",
            "Monkey",
            "Moon",
            "Noise",
            "Orbit",
            "Pulse",
            "Rocket",
            "Soccer",
            "Square",
            "Star",
            "Toggle",
            "Triangle",
            "Weather",
        ];
        let names: Vec<&str> = all().keys().copied().collect();
        assert_eq!(names, expected);
    }

    #[test]
    fn every_spinner_has_frames_and_a_positive_interval() {
        for (name, spinner) in all() {
            assert!(!spinner.frames.is_empty(), "{name} has no frames");
            assert!(
                spinner.interval > Duration::ZERO,
                "{name} has an invalid interval"
            );
        }
    }

    #[test]
    fn render_wraps_around() {
        let spinner = dots();
        let total = spinner.frames.len();
        for index in 0..(total * 3) {
            assert_eq!(spinner.render(index), spinner.render(index % total));
        }
        assert_eq!(spinner.render(0), spinner.render(total));
        assert_eq!(spinner.render(total), spinner.render(total * 7));
    }

    #[test]
    fn render_of_an_empty_spinner_is_empty() {
        let spinner = Spinner::new(Vec::new(), Duration::from_millis(10));
        assert_eq!(spinner.render(0), "");
        assert_eq!(spinner.render(42), "");
    }

    #[test]
    fn default_style_is_empty() {
        assert_eq!(default_style().render("x"), "x");
    }
}
