//! Cross-platform desktop notifications for terminal interfaces.
//!
//! [`send`] shows a notification through the native mechanism of the current
//! operating system. The crate does not link to any notification library: it
//! **invokes external programs**:
//!
//! * Windows: `powershell` (BurntToast), falling back to `msg`;
//! * macOS: `osascript`;
//! * Linux: `notify-send` (from `libnotify`);
//! * other platforms: [`Error::UnsupportedPlatform`].
//!
//! This crate is a port of the Go [`charm-notify`] library.
//!
//! [`charm-notify`]: https://github.com/junhinhow/charm-notify
//!
//! ```no_run
//! use std::time::Duration;
//!
//! use charm_notify::{NotifyOptions, send};
//!
//! let opts = NotifyOptions::new()
//!     .timeout(Duration::from_secs(3))
//!     .sound(true);
//! send("Build finished", "All 42 tests passed", &opts)?;
//! # Ok::<(), charm_notify::Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::time::Duration;

mod platform;

/// Default time the notification stays on screen.
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors returned when a notification cannot be sent.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The title is empty.
    #[error("notification title cannot be empty")]
    EmptyTitle,
    /// The message is empty.
    #[error("notification message cannot be empty")]
    EmptyMessage,
    /// Desktop notifications are not implemented for this operating system.
    #[error("desktop notifications are not supported on this platform")]
    UnsupportedPlatform,
    /// The external program required on this platform is not installed or not in `PATH`.
    #[error("external program `{binary}` was not found in PATH")]
    BinaryNotFound {
        /// Name of the missing program.
        binary: String,
    },
    /// The external program ran but reported a failure.
    #[error("`{program}` exited unsuccessfully")]
    ProcessFailed {
        /// Name of the program that failed.
        program: String,
    },
    /// The external program could not be started.
    #[error("failed to run `{program}`: {source}")]
    Spawn {
        /// Name of the program that could not be started.
        program: String,
        /// Underlying I/O error.
        source: std::io::Error,
    },
}

/// A desktop notification, as passed to the platform backend.
///
/// Built by [`send`] from its arguments and the [`NotifyOptions`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    /// Title shown in bold.
    pub title: String,
    /// Body text of the notification.
    pub message: String,
    /// Optional icon (a path or an icon name, depending on the platform).
    pub icon: String,
    /// How long the notification stays on screen.
    pub timeout: Duration,
    /// Whether a sound is played.
    pub sound: bool,
}

/// Display options for [`send`], built with a fluent API.
///
/// The defaults are no icon, a 5 second timeout and no sound.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
///
/// use charm_notify::NotifyOptions;
///
/// let opts = NotifyOptions::new()
///     .icon("dialog-information")
///     .timeout(Duration::from_secs(10))
///     .sound(true);
/// assert_ne!(opts, NotifyOptions::new());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyOptions {
    icon: String,
    timeout: Duration,
    sound: bool,
}

impl Default for NotifyOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl NotifyOptions {
    /// Creates the default options: no icon, 5 second timeout, no sound.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_notify::NotifyOptions;
    ///
    /// let opts = NotifyOptions::new();
    /// assert_eq!(opts, NotifyOptions::default());
    /// ```
    pub fn new() -> Self {
        Self {
            icon: String::new(),
            timeout: DEFAULT_TIMEOUT,
            sound: false,
        }
    }

    /// Sets the icon. Its meaning depends on the platform (Linux accepts an
    /// icon name or path; other platforms ignore it).
    #[must_use]
    pub fn icon(mut self, icon: &str) -> Self {
        icon.clone_into(&mut self.icon);
        self
    }

    /// Sets how long the notification stays on screen.
    ///
    /// Timeouts below one second fall back to 5 seconds on Windows, where the
    /// fallback mechanism only accepts whole seconds.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enables or disables the notification sound.
    #[must_use]
    pub fn sound(mut self, enabled: bool) -> Self {
        self.sound = enabled;
        self
    }
}

/// Sends a desktop notification.
///
/// The notification is delivered by the native mechanism of the current
/// platform. Both `title` and `message` must be non-empty.
///
/// # Errors
///
/// * [`Error::EmptyTitle`] / [`Error::EmptyMessage`] when an argument is empty.
/// * [`Error::UnsupportedPlatform`] on operating systems without a backend.
/// * [`Error::BinaryNotFound`] when the external program is not installed.
/// * [`Error::ProcessFailed`] / [`Error::Spawn`] when the program fails.
///
/// # Examples
///
/// ```
/// use charm_notify::{Error, NotifyOptions, send};
///
/// let err = send("", "message", &NotifyOptions::new()).unwrap_err();
/// assert!(matches!(err, Error::EmptyTitle));
/// ```
pub fn send(title: &str, message: &str, opts: &NotifyOptions) -> Result<(), Error> {
    if title.is_empty() {
        return Err(Error::EmptyTitle);
    }
    if message.is_empty() {
        return Err(Error::EmptyMessage);
    }

    let notification = Notification {
        title: title.to_owned(),
        message: message.to_owned(),
        icon: opts.icon.clone(),
        timeout: opts.timeout,
        sound: opts.sound,
    };
    platform::send(&notification)
}
