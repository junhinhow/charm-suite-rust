//! Multi-step wizard framework for terminal interfaces.
//!
//! `charm-wizard` is the Rust port of the Go package
//! [`charm-wizard`](https://github.com/junhinhow/charm-suite). A [`Wizard`]
//! runs a sequence of [`Step`]s that share a [`DataMap`], supports moving
//! backward and forward, and prints a styled progress indicator with
//! [`render_progress`]. Going back undoes the data written by the step being
//! left, and a failed step blocks [`Wizard::forward`].
//!
//! # Examples
//!
//! ```
//! use charm_wizard::{DataMap, Error, Wizard};
//! use serde_json::json;
//!
//! let mut wizard = Wizard::new()
//!     .with_title("Deploy")
//!     .add_step("config", "Configuration", |data: &mut DataMap| {
//!         data.insert("env".into(), json!("staging"));
//!         Ok(())
//!     })
//!     .add_step("deploy", "Deployment", |data: &mut DataMap| {
//!         if data.get("env").is_some() {
//!             Ok(())
//!         } else {
//!             Err(Error::custom("missing environment"))
//!         }
//!     });
//!
//! wizard.run_to(&mut Vec::<u8>::new())?;
//! assert_eq!(wizard.data()["env"], "staging");
//! # Ok::<(), Error>(())
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod progress;
mod wizard;

#[cfg(test)]
mod tests;

pub use progress::render_progress;
pub use wizard::{DataMap, Step, StepFn, Wizard};

/// Errors produced by a [`Wizard`] or by one of its steps.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The wizard has no registered steps.
    #[error("wizard has no registered steps")]
    NoSteps,
    /// A step index is outside the registered steps.
    #[error("invalid step index: {index} (total: {total})")]
    InvalidIndex {
        /// Index that was requested.
        index: usize,
        /// Number of registered steps.
        total: usize,
    },
    /// A step returned an error. The original error is available through
    /// [`std::error::Error::source`].
    #[error("step `{name}` failed")]
    StepFailed {
        /// Name of the failing step.
        name: String,
        /// Error returned by the step.
        #[source]
        source: Box<Error>,
    },
    /// A message produced by a step; see [`Error::custom`].
    #[error("{0}")]
    Custom(String),
    /// Writing the wizard output failed.
    #[error("failed to write wizard output: {0}")]
    Io(#[from] std::io::Error),
}

impl Error {
    /// Creates an [`Error::Custom`] from a message, for use inside steps.
    ///
    /// # Examples
    ///
    /// ```
    /// use charm_wizard::Error;
    ///
    /// let err = Error::custom("disk is full");
    /// assert_eq!(err.to_string(), "disk is full");
    /// ```
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }
}
