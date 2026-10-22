//! Fallback backend for operating systems without a notification mechanism.

use crate::{Error, Notification};

/// Always fails: there is no notification backend for this platform.
pub(crate) fn send(_notification: &Notification) -> Result<(), Error> {
    Err(Error::UnsupportedPlatform)
}
