//! Toast notifications with auto-dismiss for terminal user interfaces.
//!
//! This crate is a Rust port of the Go library
//! [`github.com/junhinhow/charm-toast`](https://github.com/junhinhow/charm-toast).
//! A [`Toast`] is a single status message with a lifetime, and a
//! [`ToastManager`] stacks several of them and drops the expired ones.
//!
//! ```
//! use std::time::Duration;
//!
//! use charm_toast::{Toast, ToastManager, ToastType};
//!
//! let mut manager = ToastManager::new();
//! manager.add_success("Arquivo salvo", Duration::from_secs(3));
//! manager.add(Toast::new("Falha de rede", Duration::from_secs(5)).with_type(ToastType::Error));
//!
//! assert_eq!(manager.count(), 2);
//! let rendered = manager.render(60);
//! assert!(charm_core::strip_ansi(&rendered).contains("Arquivo salvo"));
//! ```
//!
//! # Time handling
//!
//! The Go original reads the wall clock internally. The methods without a
//! suffix do the same here, using [`Instant::now`]. Every time-dependent method
//! also has an `_at` variant that takes the current [`Instant`] explicitly
//! (for example [`Toast::is_expired_at`] or [`ToastManager::render_at`]), so
//! expiry can be tested without sleeping.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;
use std::time::{Duration, Instant};

use charm_core::Rgb;
use lipgloss::{Color, Style};

#[cfg(test)]
mod tests;

/// The severity of a [`Toast`].
///
/// Equivalent to the Go `ToastType` constants `Info`, `Success`, `Warning` and
/// `Error`. The [`fmt::Display`] implementation matches the Go `String()` method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToastType {
    /// Informational notification.
    Info,
    /// Success notification.
    Success,
    /// Warning notification.
    Warning,
    /// Error notification.
    Error,
}

impl ToastType {
    /// Returns the lowercase name of the type (`"info"`, `"success"`, ...).
    ///
    /// ```
    /// use charm_toast::ToastType;
    ///
    /// assert_eq!(ToastType::Warning.label(), "warning");
    /// ```
    pub const fn label(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }

    /// Returns the single-character icon shown inside the toast.
    ///
    /// ```
    /// use charm_toast::ToastType;
    ///
    /// assert_eq!(ToastType::Error.icon(), "x");
    /// ```
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Info => "i",
            Self::Success => "v",
            Self::Warning => "!",
            Self::Error => "x",
        }
    }

    /// Returns the accent color of the type.
    ///
    /// ```
    /// use charm_toast::ToastType;
    ///
    /// assert_eq!(ToastType::Info.color().to_hex(), "#61afef");
    /// ```
    pub const fn color(self) -> Rgb {
        match self {
            Self::Info => Rgb::rgb(0x61, 0xAF, 0xEF),
            Self::Success => Rgb::rgb(0x98, 0xC3, 0x79),
            Self::Warning => Rgb::rgb(0xE5, 0xC0, 0x7B),
            Self::Error => Rgb::rgb(0xE0, 0x6C, 0x75),
        }
    }
}

impl fmt::Display for ToastType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// A single toast notification.
///
/// A toast expires once `Duration` has elapsed since `created_at`.
///
/// ```
/// use std::time::{Duration, Instant};
///
/// use charm_toast::{Toast, ToastType};
///
/// let start = Instant::now();
/// let toast = Toast::new("Salvo", Duration::from_secs(2))
///     .with_type(ToastType::Success)
///     .with_created_at(start);
///
/// assert!(!toast.is_expired_at(start + Duration::from_secs(1)));
/// assert!(toast.is_expired_at(start + Duration::from_secs(2)));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Toast {
    /// Text shown in the notification.
    pub message: String,
    /// How long the toast stays visible.
    pub duration: Duration,
    /// Severity of the toast.
    pub r#type: ToastType,
    /// Moment the toast was created.
    pub created_at: Instant,
}

impl Toast {
    /// Creates an [`ToastType::Info`] toast that expires after `duration`.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::{Toast, ToastType};
    ///
    /// let toast = Toast::new("Teste", Duration::from_secs(3));
    /// assert_eq!(toast.r#type, ToastType::Info);
    /// ```
    pub fn new(message: &str, duration: Duration) -> Self {
        Self {
            message: message.to_owned(),
            duration,
            r#type: ToastType::Info,
            created_at: Instant::now(),
        }
    }

    /// Returns the toast with its type replaced.
    #[must_use]
    pub fn with_type(mut self, t: ToastType) -> Self {
        self.r#type = t;
        self
    }

    /// Returns the toast with its message replaced.
    #[must_use]
    pub fn with_message(mut self, m: &str) -> Self {
        m.clone_into(&mut self.message);
        self
    }

    /// Returns the toast with its duration replaced.
    #[must_use]
    pub fn with_duration(mut self, d: Duration) -> Self {
        self.duration = d;
        self
    }

    /// Returns the toast with its creation instant replaced.
    ///
    /// Useful to build toasts with a known age, for example in tests.
    #[must_use]
    pub fn with_created_at(mut self, at: Instant) -> Self {
        self.created_at = at;
        self
    }

    /// Reports whether the toast has expired, using the current time.
    ///
    /// A toast expires when at least `duration` has elapsed since it was created.
    pub fn is_expired(&self) -> bool {
        self.is_expired_at(Instant::now())
    }

    /// Reports whether the toast has expired at `now`.
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use charm_toast::Toast;
    ///
    /// let start = Instant::now();
    /// let toast = Toast::new("x", Duration::from_millis(50)).with_created_at(start);
    /// assert!(!toast.is_expired_at(start));
    /// assert!(toast.is_expired_at(start + Duration::from_millis(60)));
    /// ```
    pub fn is_expired_at(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.created_at) >= self.duration
    }

    /// Returns the time left before the toast expires, using the current time.
    ///
    /// Never negative: returns [`Duration::ZERO`] once the toast has expired.
    pub fn remaining(&self) -> Duration {
        self.remaining_at(Instant::now())
    }

    /// Returns the time left before the toast expires, as seen at `now`.
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use charm_toast::Toast;
    ///
    /// let start = Instant::now();
    /// let toast = Toast::new("x", Duration::from_secs(1)).with_created_at(start);
    /// assert_eq!(toast.remaining_at(start), Duration::from_secs(1));
    /// assert_eq!(toast.remaining_at(start + Duration::from_secs(5)), Duration::ZERO);
    /// ```
    pub fn remaining_at(&self, now: Instant) -> Duration {
        self.duration
            .saturating_sub(now.saturating_duration_since(self.created_at))
    }

    /// Renders the toast as a rounded box with a colored icon.
    ///
    /// A `width` greater than zero right-aligns the box inside that many
    /// columns; `0` returns the box alone.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::{Toast, ToastType};
    ///
    /// let toast = Toast::new("Operacao concluida", Duration::from_secs(1))
    ///     .with_type(ToastType::Success);
    /// let out = charm_core::strip_ansi(&toast.render(60));
    /// assert!(out.contains("[v]"));
    /// assert!(out.contains("Operacao concluida"));
    /// ```
    pub fn render(&self, width: usize) -> String {
        let color = self.r#type.color().to_lipgloss();

        let icon_style = Style::new()
            .foreground(Color("#1E1E1E".to_owned()))
            .background(color.clone())
            .bold(true)
            .padding_2(0, 1);

        let message_style = Style::new()
            .foreground(Color("#FAFAFA".to_owned()))
            .background(Color("#2D2D2D".to_owned()))
            .padding_2(0, 1);

        let border_style = Style::new()
            .border(lipgloss::rounded_border())
            .border_foreground(color);

        let icon = icon_style.render(&format!("[{}]", self.r#type.icon()));
        let message = message_style.render(&self.message);

        let content = lipgloss::join_horizontal(lipgloss::CENTER, &[&icon, &message]);
        let boxed = border_style.render(&content);

        if width > 0 {
            lipgloss::place_horizontal(
                i32::try_from(width).unwrap_or(i32::MAX),
                lipgloss::RIGHT,
                &boxed,
                &[],
            )
        } else {
            boxed
        }
    }
}

/// Manages several [`Toast`]s and renders the visible ones as a stack.
///
/// Toasts are stored in insertion order. [`ToastManager::render`] shows at
/// most [`ToastManager::max_visible`] of the most recent active toasts, oldest
/// first, one below the other.
///
/// ```
/// use std::time::Duration;
///
/// use charm_toast::ToastManager;
///
/// let mut manager = ToastManager::new();
/// manager.add_info("Um", Duration::from_secs(60));
/// manager.add_error("Dois", Duration::from_secs(60));
/// assert_eq!(manager.count(), 2);
///
/// manager.clear();
/// assert_eq!(manager.render(40), "");
/// ```
#[derive(Debug, Clone)]
pub struct ToastManager {
    toasts: Vec<Toast>,
    /// Maximum number of toasts rendered at the same time (default `5`).
    pub max_visible: usize,
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ToastManager {
    /// Creates an empty manager that shows up to 5 toasts.
    ///
    /// ```
    /// use charm_toast::ToastManager;
    ///
    /// let manager = ToastManager::new();
    /// assert_eq!(manager.max_visible, 5);
    /// assert_eq!(manager.count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_visible: 5,
        }
    }

    /// Adds a toast, restarting its clock: `created_at` is set to now.
    ///
    /// This mirrors the Go `Add`, which overwrites `CreatedAt`.
    pub fn add(&mut self, t: Toast) {
        self.add_at(t, Instant::now());
    }

    /// Adds a toast as if it were created at `now`.
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use charm_toast::{Toast, ToastManager};
    ///
    /// let mut manager = ToastManager::new();
    /// let start = Instant::now();
    /// manager.add_at(Toast::new("x", Duration::from_secs(1)), start);
    /// assert_eq!(manager.count_at(start + Duration::from_secs(2)), 0);
    /// ```
    pub fn add_at(&mut self, mut t: Toast, now: Instant) {
        t.created_at = now;
        self.toasts.push(t);
    }

    /// Adds an [`ToastType::Info`] toast.
    pub fn add_info(&mut self, message: &str, duration: Duration) {
        self.add(Toast::new(message, duration));
    }

    /// Adds a [`ToastType::Success`] toast.
    pub fn add_success(&mut self, message: &str, duration: Duration) {
        self.add(Toast::new(message, duration).with_type(ToastType::Success));
    }

    /// Adds a [`ToastType::Warning`] toast.
    pub fn add_warning(&mut self, message: &str, duration: Duration) {
        self.add(Toast::new(message, duration).with_type(ToastType::Warning));
    }

    /// Adds a [`ToastType::Error`] toast.
    pub fn add_error(&mut self, message: &str, duration: Duration) {
        self.add(Toast::new(message, duration).with_type(ToastType::Error));
    }

    /// Removes the expired toasts, using the current time.
    pub fn cleanup(&mut self) {
        self.cleanup_at(Instant::now());
    }

    /// Removes the toasts that have expired at `now`.
    pub fn cleanup_at(&mut self, now: Instant) {
        self.toasts.retain(|t| !t.is_expired_at(now));
    }

    /// Returns the number of active (not expired) toasts, using the current time.
    pub fn count(&self) -> usize {
        self.count_at(Instant::now())
    }

    /// Returns the number of toasts still active at `now`.
    pub fn count_at(&self, now: Instant) -> usize {
        self.toasts.iter().filter(|t| !t.is_expired_at(now)).count()
    }

    /// Removes every toast, expired or not.
    pub fn clear(&mut self) {
        self.toasts.clear();
    }

    /// Renders the visible toasts stacked vertically, using the current time.
    ///
    /// Returns an empty string when no toast is active.
    pub fn render(&self, width: usize) -> String {
        self.render_at(width, Instant::now())
    }

    /// Renders the toasts that are active at `now`, stacked vertically.
    ///
    /// At most [`ToastManager::max_visible`] toasts are shown: the most recent
    /// ones, in insertion order.
    pub fn render_at(&self, width: usize, now: Instant) -> String {
        let mut lines: Vec<String> = self
            .toasts
            .iter()
            .rev()
            .filter(|t| !t.is_expired_at(now))
            .take(self.max_visible)
            .map(|t| t.render(width))
            .collect();
        lines.reverse();
        lines.join("\n")
    }
}
