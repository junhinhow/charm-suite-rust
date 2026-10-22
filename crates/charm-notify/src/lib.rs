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
//! Not every option is honoured on every platform. [`platform_capabilities`]
//! reports what the current platform respects.
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
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Errors returned when a notification cannot be sent.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The title is empty.
    #[error("notification title cannot be empty")]
    EmptyTitle,
    /// The message is empty.
    #[error("notification message cannot be empty")]
    EmptyMessage,
    /// The title or the message contains a line break.
    ///
    /// Line breaks are rejected because they turn the payload into a
    /// multi-line command on the platforms that build a script.
    #[error("notification {field} cannot contain line breaks")]
    MultilineText {
        /// Which text was rejected: `"title"` or `"message"`.
        field: &'static str,
    },
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
    ///
    /// `stderr` holds the trimmed error output of the program; it is empty
    /// when the program wrote nothing to stderr.
    #[error("`{program}` exited unsuccessfully{}", describe_stderr(.stderr))]
    ProcessFailed {
        /// Name of the program that failed.
        program: String,
        /// Trimmed standard error output captured from the program.
        stderr: String,
    },
    /// On Windows, both BurntToast and the `msg.exe` fallback failed.
    #[error("BurntToast failed ({burnt_toast}) and msg.exe failed ({msg})")]
    WindowsFallbackFailed {
        /// Error from the BurntToast attempt.
        burnt_toast: Box<Error>,
        /// Error from the `msg.exe` fallback.
        msg: Box<Error>,
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

/// Formats the captured stderr for [`Error::ProcessFailed`].
fn describe_stderr(stderr: &str) -> String {
    if stderr.is_empty() {
        ": no error output".to_owned()
    } else {
        format!(": {stderr}")
    }
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
    /// How long the notification stays on screen. A zero duration means the
    /// platform default.
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

    /// Sets the icon. Its meaning depends on the platform: Linux accepts an
    /// icon name or path, Windows (BurntToast) accepts a path, and macOS
    /// ignores it.
    #[must_use]
    pub fn icon(mut self, icon: &str) -> Self {
        icon.clone_into(&mut self.icon);
        self
    }

    /// Sets how long the notification stays on screen.
    ///
    /// A zero duration uses the platform default and sends no explicit expiry.
    /// On Windows the `msg.exe` fallback only accepts whole seconds, so a
    /// timeout below one second becomes one second there.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enables or disables the notification sound.
    ///
    /// Sound is honoured on macOS and Windows (BurntToast), not on Linux.
    #[must_use]
    pub fn sound(mut self, enabled: bool) -> Self {
        self.sound = enabled;
        self
    }
}

/// Options that the current platform's backend honours.
///
/// Returned by [`platform_capabilities`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    /// The icon is honoured.
    pub icon: bool,
    /// The sound is honoured.
    pub sound: bool,
    /// The timeout is honoured.
    pub timeout: bool,
}

/// Returns the platform identifier used to send notifications.
///
/// The identifiers follow the Go original: `"linux"`, `"darwin"` (macOS) or
/// `"windows"`. Returns `None` on platforms without a backend.
///
/// # Examples
///
/// ```
/// let supported = charm_notify::platform().is_some();
/// assert_eq!(supported, charm_notify::supported());
/// ```
pub fn platform() -> Option<&'static str> {
    platform::PLATFORM
}

/// Reports whether the current platform has a notification backend.
///
/// # Examples
///
/// ```
/// if charm_notify::supported() {
///     println!("notifications are available");
/// }
/// ```
pub fn supported() -> bool {
    platform().is_some()
}

/// Reports which [`NotifyOptions`] the current platform honours.
///
/// On Windows the timeout is only honoured by the `msg.exe` fallback, and the
/// icon and sound only by BurntToast.
///
/// # Examples
///
/// ```
/// let caps = charm_notify::platform_capabilities();
/// if !caps.sound {
///     println!("sound is ignored on this platform");
/// }
/// ```
pub fn platform_capabilities() -> Capabilities {
    platform::CAPABILITIES
}

/// Checks that a text has no line break.
fn validate_single_line(field: &'static str, value: &str) -> Result<(), Error> {
    if value.contains(['\r', '\n']) {
        Err(Error::MultilineText { field })
    } else {
        Ok(())
    }
}

/// Sends a desktop notification.
///
/// The notification is delivered by the native mechanism of the current
/// platform. `title` and `message` must be non-empty and must not contain
/// line breaks.
///
/// # Errors
///
/// * [`Error::EmptyTitle`] / [`Error::EmptyMessage`] when an argument is empty.
/// * [`Error::MultilineText`] when the title or the message has a line break.
/// * [`Error::UnsupportedPlatform`] on operating systems without a backend.
/// * [`Error::BinaryNotFound`] when the external program is not installed.
/// * [`Error::ProcessFailed`] / [`Error::Spawn`] when the program fails.
/// * [`Error::WindowsFallbackFailed`] on Windows when both BurntToast and
///   `msg.exe` fail.
///
/// # Examples
///
/// ```
/// use charm_notify::{Error, NotifyOptions, send};
///
/// let err = send("", "message", &NotifyOptions::new()).unwrap_err();
/// assert!(matches!(err, Error::EmptyTitle));
///
/// let err = send("title", "two\nlines", &NotifyOptions::new()).unwrap_err();
/// assert!(matches!(err, Error::MultilineText { field: "message" }));
/// ```
pub fn send(title: &str, message: &str, opts: &NotifyOptions) -> Result<(), Error> {
    if title.is_empty() {
        return Err(Error::EmptyTitle);
    }
    if message.is_empty() {
        return Err(Error::EmptyMessage);
    }
    validate_single_line("title", title)?;
    validate_single_line("message", message)?;

    let notification = Notification {
        title: title.to_owned(),
        message: message.to_owned(),
        icon: opts.icon.clone(),
        timeout: opts.timeout,
        sound: opts.sound,
    };
    platform::send(&notification)
}
