//! Terminal spinner animations, ported from the Go `charm-spinner` package.
//!
//! This crate provides the 42 predefined spinner animations of
//! [`charm-spinner`](https://github.com/junhinhow/charm-spinner) together with
//! the [`Spinner`] type used to describe a custom animation.
//!
//! Every predefined spinner is exposed as a function ([`dots`], [`line()`],
//! [`rocket`], ...) and all of them are available at once through [`all`] and
//! [`names`].
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
//! when the index goes past the last frame. The frame is styled with the
//! spinner's own style when it has one, or with the package default style
//! otherwise (see [`default_style`] and [`set_default_style`]).
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
//! # Styles
//!
//! ```
//! use charm_spinner::{dots, Spinner};
//! use lipgloss::Style;
//! use std::time::Duration;
//!
//! let bold = Style::new().bold(true);
//! let custom = Spinner::new(vec!["a".to_owned()], Duration::from_millis(100))
//!     .with_style(bold);
//! assert!(custom.render(0).contains("\u{1b}[1m"));
//! ```
//!
//! # Differences from the Go implementation
//!
//! - The Go package exposes the spinners as package-level variables; Rust
//!   exposes them as functions returning an owned [`Spinner`].
//! - `frame` is a `usize`, so negative indexes (which count from the end in
//!   Go) do not exist. Use `frames.len() - n` to address the end of the cycle.
//! - `Spinner::render` returns an empty string for a spinner without frames,
//!   while the Go version panics with an index out of range error.
//! - [`all`] returns an owned map on every call, which is the equivalent of the
//!   Go `Clone` function. The Go `All` returns a shared map that must not be
//!   modified.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod spinners;

use std::sync::{LazyLock, PoisonError, RwLock};
use std::time::Duration;

pub use lipgloss::Style;
pub use spinners::*;

/// The built-in default style applied to frames of spinners that do not set
/// their own style.
///
/// This is the Rust counterpart of the Go package-level
/// `var Style = lipgloss.NewStyle()`: an empty (unstyled) lipgloss style. It
/// is the initial value of the package default, which can be changed at run
/// time with [`set_default_style`].
///
/// # Examples
///
/// ```
/// let style = &charm_spinner::DEFAULT_STYLE;
/// assert_eq!(style.render("x"), "x");
/// ```
pub static DEFAULT_STYLE: LazyLock<Style> = LazyLock::new(Style::new);

/// The current package default style, shared by every thread.
static PACKAGE_STYLE: LazyLock<RwLock<Style>> =
    LazyLock::new(|| RwLock::new(DEFAULT_STYLE.clone()));

/// Returns a copy of the current package default style.
///
/// Initially this is the unstyled [`DEFAULT_STYLE`]; [`set_default_style`]
/// replaces it.
///
/// # Examples
///
/// ```
/// let style = charm_spinner::default_style();
/// assert_eq!(style.render("x"), "x");
/// ```
pub fn default_style() -> Style {
    PACKAGE_STYLE
        .read()
        .unwrap_or_else(PoisonError::into_inner)
        .clone()
}

/// Replaces the package default style used by spinners without a style of
/// their own.
///
/// This is safe to call while other threads render spinners: the change is
/// atomic with respect to each render.
///
/// # Examples
///
/// ```
/// use charm_spinner::{default_style, set_default_style};
/// use lipgloss::Style;
///
/// let previous = default_style();
/// set_default_style(Style::new().bold(true));
/// assert!(charm_spinner::dots().render(0).contains("\u{1b}[1m"));
/// set_default_style(previous);
/// ```
pub fn set_default_style(style: Style) {
    *PACKAGE_STYLE
        .write()
        .unwrap_or_else(PoisonError::into_inner) = style;
}

/// A terminal spinner animation: an ordered list of frames, the delay between
/// them and an optional style.
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
/// assert!(spinner.style.is_none());
/// ```
#[derive(Debug, Clone)]
pub struct Spinner {
    /// The animation frames, in order.
    pub frames: Vec<String>,
    /// The delay between two consecutive frames.
    pub interval: Duration,
    /// The style applied to the frames.
    ///
    /// When it is `None`, or an empty style, the package default style
    /// ([`default_style`]) is used instead.
    pub style: Option<Style>,
}

impl PartialEq for Spinner {
    fn eq(&self, other: &Self) -> bool {
        self.frames == other.frames
            && self.interval == other.interval
            && match (&self.style, &other.style) {
                (None, None) => true,
                (Some(a), Some(b)) => a.is_equivalent(b),
                _ => false,
            }
    }
}

impl Eq for Spinner {}

impl Spinner {
    /// Creates a spinner from its frames and the interval between them.
    ///
    /// The spinner has no style of its own, so it renders with the package
    /// default style.
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
        Self {
            frames,
            interval,
            style: None,
        }
    }

    /// Sets the style applied to the frames, replacing the package default.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use charm_spinner::{Spinner, Style};
    ///
    /// let spinner = Spinner::new(vec!["a".to_owned()], Duration::from_millis(50))
    ///     .with_style(Style::new().italic(true));
    /// assert!(spinner.render(0).contains("\u{1b}[3m"));
    /// ```
    #[must_use]
    pub fn with_style(mut self, style: Style) -> Self {
        self.style = Some(style);
        self
    }

    /// Renders the frame at the given index, cycling with wrap-around.
    ///
    /// This mirrors the Go implementation, which computes
    /// `frames[frame % len(frames)]`. The selected frame is styled with the
    /// spinner's own style, or with the package default style when it has
    /// none.
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
        self.effective_style().render(selected)
    }

    /// The style used to render this spinner: its own, when it defines one,
    /// or the package default otherwise.
    fn effective_style(&self) -> Style {
        match &self.style {
            Some(style) if !style.is_equivalent(&Style::new()) => style.clone(),
            _ => default_style(),
        }
    }
}

/// Returns the names of every predefined spinner, in alphabetical order.
///
/// The names are the keys of [`all`], which the Go package returns sorted by
/// `Names`.
///
/// # Examples
///
/// ```
/// let names = charm_spinner::names();
/// assert_eq!(names.len(), 42);
/// assert_eq!(names.first(), Some(&"Arrow"));
/// ```
pub fn names() -> Vec<&'static str> {
    all().into_keys().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_contains_the_forty_two_spinners() {
        assert_eq!(all().len(), 42);
    }

    #[test]
    fn all_keys_match_the_go_names() {
        let expected = [
            "Arrow",
            "Balloon",
            "Bar",
            "Beach",
            "Bounce",
            "BouncingBar",
            "Cat",
            "Christmas",
            "Circle",
            "Clock",
            "DNA",
            "Diamond",
            "Digits",
            "Dots",
            "Earth",
            "Ellipsis",
            "Fading",
            "Fire",
            "Flip",
            "Grow",
            "Hearts",
            "Hexagon",
            "Layer",
            "Line",
            "Matrix",
            "Money",
            "Monkey",
            "Moon",
            "Music",
            "Noise",
            "Orbit",
            "Pulse",
            "Quotes",
            "Rectangle",
            "Rocket",
            "Soccer",
            "Square",
            "Star",
            "Target",
            "Toggle",
            "Triangle",
            "Weather",
        ];
        let keys: Vec<&str> = all().keys().copied().collect();
        assert_eq!(keys, expected);
    }

    #[test]
    fn names_are_sorted_and_match_all_keys() {
        let names = names();
        assert_eq!(names.len(), all().len());
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(names.iter().all(|name| all().contains_key(name)));
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

    #[test]
    fn spinner_style_is_applied() {
        let spinner = Spinner::new(vec!["a".to_owned()], Duration::from_millis(100))
            .with_style(Style::new().bold(true));
        assert!(spinner.render(0).contains("\u{1b}[1m"));
    }

    #[test]
    fn empty_spinner_style_falls_back_to_the_default() {
        let spinner =
            Spinner::new(vec!["a".to_owned()], Duration::from_millis(100)).with_style(Style::new());
        assert_eq!(spinner.render(0), "a");
    }

    #[test]
    fn equality_considers_frames_interval_and_style() {
        let base = Spinner::new(vec!["a".to_owned()], Duration::from_millis(100));
        assert_eq!(base, base.clone());
        assert_ne!(base, base.clone().with_style(Style::new().bold(true)));
        assert_ne!(
            base,
            Spinner::new(vec!["b".to_owned()], Duration::from_millis(100))
        );
        assert_ne!(
            base,
            Spinner::new(vec!["a".to_owned()], Duration::from_millis(200))
        );
    }
}
