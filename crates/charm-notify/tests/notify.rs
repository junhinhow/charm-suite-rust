//! Integration tests for the public API of `charm-notify`.
//!
//! These tests never display a real notification: they only exercise the
//! argument validation, which runs before any platform program is invoked.
//! The command construction of each platform is tested inside the crate, next
//! to the backends (`src/platform/*.rs`), where the builders are visible.

use std::time::Duration;

use charm_notify::{send, Error, NotifyOptions};

#[test]
fn empty_title_is_rejected() {
    let err = send("", "message", &NotifyOptions::new()).unwrap_err();
    assert!(matches!(err, Error::EmptyTitle));
}

#[test]
fn empty_message_is_rejected() {
    let err = send("title", "", &NotifyOptions::new()).unwrap_err();
    assert!(matches!(err, Error::EmptyMessage));
}

#[test]
fn title_is_checked_before_message() {
    let err = send("", "", &NotifyOptions::new()).unwrap_err();
    assert!(matches!(err, Error::EmptyTitle));
}

#[test]
fn default_options_match_new() {
    assert_eq!(NotifyOptions::default(), NotifyOptions::new());
}

#[test]
fn builder_methods_change_the_options() {
    let opts = NotifyOptions::new()
        .icon("dialog-information")
        .timeout(Duration::from_secs(10))
        .sound(true);
    assert_ne!(opts, NotifyOptions::new());
    assert_eq!(
        opts,
        NotifyOptions::new()
            .icon("dialog-information")
            .timeout(Duration::from_secs(10))
            .sound(true)
    );
}

#[test]
fn builder_is_order_independent() {
    let a = NotifyOptions::new()
        .sound(true)
        .timeout(Duration::from_secs(2));
    let b = NotifyOptions::new()
        .timeout(Duration::from_secs(2))
        .sound(true);
    assert_eq!(a, b);
}

#[test]
fn error_messages_are_clear() {
    assert_eq!(
        Error::EmptyTitle.to_string(),
        "notification title cannot be empty"
    );
    assert_eq!(
        Error::BinaryNotFound {
            binary: "notify-send".to_owned()
        }
        .to_string(),
        "external program `notify-send` was not found in PATH"
    );
}
