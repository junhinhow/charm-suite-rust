//! Toast notifications with auto-dismiss for terminal user interfaces.
//!
//! This crate is a Rust port of the Go library
//! [`github.com/junhinhow/charm-toast`](https://github.com/junhinhow/charm-toast).
//! A [`Toast`] is a single status message with a lifetime, and a
//! [`ToastManager`] stacks several of them, drops the expired ones and renders
//! the most recent ones at the top.
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
//!
//! # Bubble Tea integration
//!
//! The Go `Tick` returns a `tea.Cmd`. This crate has no runtime dependency, so
//! the same information is exposed as plain values: [`ToastManager::tick_delay`]
//! tells the caller when to wake up, and [`ToastManager::expired_msg`] builds
//! the [`ExpiredMsg`] to deliver at that moment.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use charm_core::Rgb;
use lipgloss::{Color, Style};

#[cfg(test)]
mod tests;

/// Default number of toasts rendered at the same time.
const DEFAULT_MAX_VISIBLE: usize = 5;

/// Grace period added to the tick delay, so the toast is already expired when
/// the [`ExpiredMsg`] arrives.
const EXPIRY_GRACE: Duration = Duration::from_millis(1);

/// Columns used by the left and right border.
const BORDER_WIDTH: usize = 2;

/// Columns used by the icon: `[x]` plus one column of padding on each side.
const ICON_WIDTH: usize = 5;

const ICON_FOREGROUND: &str = "#1E1E1E";
const MESSAGE_FOREGROUND: &str = "#FAFAFA";
const MESSAGE_BACKGROUND: &str = "#2D2D2D";

/// Counter behind the identifiers generated for new toasts, as in Go.
static TOAST_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Returns a new identifier in the `toast-N` form used by the Go original.
fn next_toast_id() -> String {
    let n = TOAST_COUNTER.fetch_add(1, Ordering::Relaxed) + 1;
    format!("toast-{n}")
}

/// Converts a width into the `i32` expected by `lipgloss`.
///
/// Values above `i32::MAX` are clamped instead of overflowing.
fn to_i32(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}

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
/// A toast expires once `duration` has elapsed since `created_at`. A toast with
/// a zero `duration` is *persistent*: it never expires on its own.
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
    /// Identifier of the toast. Generated by [`Toast::new`] and by
    /// [`ToastManager::add`] when it is empty.
    pub id: String,
    /// Text shown in the notification.
    pub message: String,
    /// How long the toast stays visible. Zero makes the toast persistent.
    pub duration: Duration,
    /// Severity of the toast.
    pub r#type: ToastType,
    /// Moment the toast was created.
    pub created_at: Instant,
}

impl Toast {
    /// Creates an [`ToastType::Info`] toast that expires after `duration`.
    ///
    /// A zero `duration` creates a persistent toast.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::{Toast, ToastType};
    ///
    /// let toast = Toast::new("Teste", Duration::from_secs(3));
    /// assert_eq!(toast.r#type, ToastType::Info);
    /// assert!(!toast.id.is_empty());
    /// ```
    pub fn new(message: &str, duration: Duration) -> Self {
        Self {
            id: next_toast_id(),
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

    /// Returns the toast with its identifier replaced.
    #[must_use]
    pub fn with_id(mut self, id: &str) -> Self {
        id.clone_into(&mut self.id);
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

    /// Reports whether the toast never expires on its own.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::Toast;
    ///
    /// assert!(Toast::new("ajuda", Duration::ZERO).persistent());
    /// assert!(!Toast::new("ajuda", Duration::from_secs(1)).persistent());
    /// ```
    pub fn persistent(&self) -> bool {
        self.duration.is_zero()
    }

    /// Reports whether the toast has expired, using the current time.
    ///
    /// A persistent toast never expires.
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
        if self.persistent() {
            return false;
        }
        now.saturating_duration_since(self.created_at) >= self.duration
    }

    /// Returns the time left before the toast expires, using the current time.
    ///
    /// Returns [`Duration::ZERO`] for persistent and already expired toasts.
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
        if self.persistent() {
            return Duration::ZERO;
        }
        self.duration
            .saturating_sub(now.saturating_duration_since(self.created_at))
    }

    /// Returns the instant at which the toast expires.
    ///
    /// Returns `None` for persistent toasts, which have no expiry instant. The
    /// Go original returns the zero `time.Time` in that case.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::Toast;
    ///
    /// let toast = Toast::new("x", Duration::from_secs(60));
    /// assert_eq!(toast.expires_at(), toast.created_at.checked_add(Duration::from_secs(60)));
    /// assert_eq!(Toast::new("x", Duration::ZERO).expires_at(), None);
    /// ```
    pub fn expires_at(&self) -> Option<Instant> {
        if self.persistent() {
            return None;
        }
        self.created_at.checked_add(self.duration)
    }

    /// Renders the toast as a rounded box with a colored icon.
    ///
    /// A `width` greater than zero is an upper bound: the message is truncated
    /// so the box never exceeds that many columns, and the box is right-aligned
    /// inside them. `0` means no limit and returns the box alone.
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
        let accent = self.r#type.color().to_lipgloss();

        // Columns left for the message. Zero means no limit.
        let mut text_width = 0;
        let mut show_icon = true;
        if width > 0 {
            let with_icon = width.saturating_sub(BORDER_WIDTH + ICON_WIDTH);
            if with_icon >= 1 {
                text_width = with_icon;
            } else {
                // No room for the icon next to the border: keep only the message.
                show_icon = false;
                let without_icon = width.saturating_sub(BORDER_WIDTH);
                if without_icon < 1 {
                    // Not even the border fits: return the truncated text alone.
                    return Style::new().max_width(to_i32(width)).render(&self.message);
                }
                text_width = without_icon;
            }
        }

        // Padding is added before truncation, so the limit covers it as in Go.
        let padded: Vec<String> = self
            .message
            .split('\n')
            .map(|line| format!(" {line} "))
            .collect();
        let mut message_style = Style::new()
            .foreground(Color(MESSAGE_FOREGROUND.to_owned()))
            .background(Color(MESSAGE_BACKGROUND.to_owned()));
        if text_width > 0 {
            message_style = message_style.max_width(to_i32(text_width));
        }
        let message = message_style.render(&padded.join("\n"));

        let content = if show_icon {
            let icon_style = Style::new()
                .foreground(Color(ICON_FOREGROUND.to_owned()))
                .background(accent.clone())
                .bold(true)
                .padding_2(0, 1);
            let icon = icon_style.render(&format!("[{}]", self.r#type.icon()));
            lipgloss::join_horizontal(lipgloss::CENTER, &[&icon, &message])
        } else {
            message
        };

        let border_style = Style::new()
            .border(lipgloss::rounded_border())
            .border_foreground(accent);
        let boxed = border_style.render(&content);

        if width > 0 {
            lipgloss::place_horizontal(to_i32(width), lipgloss::RIGHT, &boxed, &[])
        } else {
            boxed
        }
    }
}

/// Message delivered when the next toast expires.
///
/// Built by [`ToastManager::expired_msg`]. The application should call
/// [`ToastManager::cleanup`] when it receives one, then render again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpiredMsg {
    /// Identifiers of the toasts that were already expired when the message
    /// was built. Empty when the application already removed them.
    pub ids: Vec<String>,
}

/// Manages several [`Toast`]s and renders the visible ones as a stack.
///
/// Toasts are stored in insertion order. [`ToastManager::render`] shows the
/// most recent active toasts first, one below the other, up to
/// [`ToastManager::max_visible`].
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
    max_visible: usize,
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
    /// assert_eq!(manager.max_visible(), 5);
    /// assert_eq!(manager.count(), 0);
    /// ```
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_visible: DEFAULT_MAX_VISIBLE,
        }
    }

    /// Returns the maximum number of toasts rendered at the same time.
    ///
    /// Zero means no limit.
    #[must_use]
    pub fn max_visible(&self) -> usize {
        self.max_visible
    }

    /// Sets the maximum number of toasts rendered at the same time.
    ///
    /// Zero removes the limit, showing every active toast, as in Go.
    ///
    /// ```
    /// use charm_toast::ToastManager;
    ///
    /// let mut manager = ToastManager::new();
    /// manager.set_max_visible(3);
    /// assert_eq!(manager.max_visible(), 3);
    /// ```
    pub fn set_max_visible(&mut self, n: usize) {
        self.max_visible = n;
    }

    /// Adds a toast, keeping its `created_at` and generating an identifier when
    /// the toast has none.
    ///
    /// A toast added with an old `created_at` is not revived: it is already
    /// expired and is not counted as active.
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use charm_toast::{Toast, ToastManager};
    ///
    /// let mut manager = ToastManager::new();
    /// let start = Instant::now();
    /// manager.add(Toast::new("x", Duration::from_secs(1)).with_created_at(start));
    /// assert_eq!(manager.count_at(start + Duration::from_secs(2)), 0);
    /// ```
    pub fn add(&mut self, mut t: Toast) {
        if t.id.is_empty() {
            t.id = next_toast_id();
        }
        self.toasts.push(t);
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
        self.add(t);
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

    /// Returns copies of the active toasts, newest first.
    ///
    /// Expired toasts are removed first, as in Go. Changing the returned values
    /// does not change the manager.
    pub fn toasts(&mut self) -> Vec<Toast> {
        self.toasts_at(Instant::now())
    }

    /// Returns copies of the toasts active at `now`, newest first.
    pub fn toasts_at(&mut self, now: Instant) -> Vec<Toast> {
        self.cleanup_at(now);
        self.toasts.iter().rev().cloned().collect()
    }

    /// Removes the toast with the given identifier.
    ///
    /// Returns `true` when the toast existed and was removed.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::ToastManager;
    ///
    /// let mut manager = ToastManager::new();
    /// manager.add_info("Um", Duration::from_secs(60));
    /// let id = manager.toasts().remove(0).id;
    /// assert!(manager.remove(&id));
    /// assert!(!manager.remove(&id));
    /// ```
    pub fn remove(&mut self, id: &str) -> bool {
        match self.toasts.iter().position(|t| t.id == id) {
            Some(index) => {
                self.toasts.remove(index);
                true
            }
            None => false,
        }
    }

    /// Changes the toast with the given identifier using `update`.
    ///
    /// The closure receives a copy of the toast and returns the new version.
    /// The identifier is kept when the closure leaves it empty. Returns `true`
    /// when the toast existed and was updated.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::ToastManager;
    ///
    /// let mut manager = ToastManager::new();
    /// manager.add_info("antes", Duration::from_secs(60));
    /// let id = manager.toasts().remove(0).id;
    /// assert!(manager.update(&id, |toast| toast.with_message("depois")));
    /// assert_eq!(manager.toasts().remove(0).message, "depois");
    /// ```
    pub fn update(&mut self, id: &str, update: impl FnOnce(Toast) -> Toast) -> bool {
        let Some(slot) = self.toasts.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        let previous_id = slot.id.clone();
        let mut updated = update(slot.clone());
        if updated.id.is_empty() {
            updated.id = previous_id;
        }
        *slot = updated;
        true
    }

    /// Renders the visible toasts stacked vertically, using the current time.
    ///
    /// Returns an empty string when no toast is active.
    pub fn render(&self, width: usize) -> String {
        self.render_at(width, Instant::now())
    }

    /// Renders the toasts that are active at `now`, stacked vertically.
    ///
    /// The most recent toasts come first. At most
    /// [`ToastManager::max_visible`] toasts are shown; zero shows all of them.
    /// Expired toasts are skipped but not removed, because rendering takes
    /// `&self`. Call [`ToastManager::cleanup`] to remove them.
    pub fn render_at(&self, width: usize, now: Instant) -> String {
        let limit = if self.max_visible == 0 {
            usize::MAX
        } else {
            self.max_visible
        };
        let lines: Vec<String> = self
            .toasts
            .iter()
            .rev()
            .filter(|t| !t.is_expired_at(now))
            .take(limit)
            .map(|t| t.render(width))
            .collect();
        lines.join("\n")
    }

    /// Returns the time left until the next toast expires, using the current time.
    ///
    /// Returns [`Duration::ZERO`] when no toast expires on its own.
    pub fn next_expiry(&self) -> Duration {
        self.next_expiry_at(Instant::now())
    }

    /// Returns the time left until the next toast expires, as seen at `now`.
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use charm_toast::{Toast, ToastManager};
    ///
    /// let mut manager = ToastManager::new();
    /// let start = Instant::now();
    /// manager.add_at(Toast::new("curto", Duration::from_secs(2)), start);
    /// manager.add_at(Toast::new("longo", Duration::from_secs(9)), start);
    /// assert_eq!(manager.next_expiry_at(start), Duration::from_secs(2));
    /// ```
    pub fn next_expiry_at(&self, now: Instant) -> Duration {
        self.toasts
            .iter()
            .filter(|t| !t.persistent())
            .map(|t| t.remaining_at(now))
            .filter(|remaining| !remaining.is_zero())
            .min()
            .unwrap_or(Duration::ZERO)
    }

    /// Returns how long to wait before calling [`ToastManager::expired_msg`],
    /// using the current time.
    ///
    /// Returns `None` when no toast expires on its own, for example when only
    /// persistent toasts are active. Re-arm the timer after every
    /// [`ExpiredMsg`] and after every added toast.
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// use charm_toast::ToastManager;
    ///
    /// let mut manager = ToastManager::new();
    /// assert_eq!(manager.tick_delay(), None);
    ///
    /// manager.add_info("persistente", Duration::ZERO);
    /// assert_eq!(manager.tick_delay(), None);
    /// ```
    pub fn tick_delay(&self) -> Option<Duration> {
        self.tick_delay_at(Instant::now())
    }

    /// Returns the delay before the next expiry, as seen at `now`.
    ///
    /// The delay includes a grace period of one millisecond, as in Go.
    pub fn tick_delay_at(&self, now: Instant) -> Option<Duration> {
        let next = self.next_expiry_at(now);
        if next.is_zero() {
            return None;
        }
        Some(next + EXPIRY_GRACE)
    }

    /// Returns the identifiers of the toasts that are expired, using the current time.
    pub fn expired_ids(&self) -> Vec<String> {
        self.expired_ids_at(Instant::now())
    }

    /// Returns the identifiers of the toasts that are expired at `now`.
    ///
    /// ```
    /// use std::time::{Duration, Instant};
    ///
    /// use charm_toast::{Toast, ToastManager};
    ///
    /// let mut manager = ToastManager::new();
    /// let start = Instant::now();
    /// let toast = Toast::new("curto", Duration::from_millis(10));
    /// let id = toast.id.clone();
    /// manager.add_at(toast, start);
    /// assert_eq!(manager.expired_ids_at(start + Duration::from_secs(1)), vec![id]);
    /// ```
    pub fn expired_ids_at(&self, now: Instant) -> Vec<String> {
        self.toasts
            .iter()
            .filter(|t| t.is_expired_at(now))
            .map(|t| t.id.clone())
            .collect()
    }

    /// Builds the [`ExpiredMsg`] for the current time.
    ///
    /// Call it when the timer armed with [`ToastManager::tick_delay`] fires.
    pub fn expired_msg(&self) -> ExpiredMsg {
        self.expired_msg_at(Instant::now())
    }

    /// Builds the [`ExpiredMsg`] for `now`.
    pub fn expired_msg_at(&self, now: Instant) -> ExpiredMsg {
        ExpiredMsg {
            ids: self.expired_ids_at(now),
        }
    }
}
