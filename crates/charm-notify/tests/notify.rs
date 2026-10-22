//! Integration tests for the public API of `charm-notify`.
//!
//! These tests never display a real notification: they only exercise the
//! argument validation, which runs before any platform program is invoked.
//! The command construction of each platform is tested inside the crate, next
//! to the backends (`src/platform/*.rs`), where the builders are visible.

use std::time::Duration;

use charm_notify::{
    platform, platform_capabilities, send, supported, Capabilities, Error, NotifyOptions,
    DEFAULT_TIMEOUT,
};

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
fn title_with_line_break_is_rejected() {
    for title in ["linha1\nlinha2", "linha1\rlinha2"] {
        let err = send(title, "mensagem", &NotifyOptions::new()).unwrap_err();
        assert!(matches!(err, Error::MultilineText { field: "title" }), "{err:?}");
    }
}

#[test]
fn message_with_line_break_is_rejected() {
    for message in ["linha1\nlinha2", "linha1\rlinha2"] {
        let err = send("titulo", message, &NotifyOptions::new()).unwrap_err();
        assert!(matches!(err, Error::MultilineText { field: "message" }), "{err:?}");
    }
}

#[test]
fn empty_checks_run_before_line_break_checks() {
    let err = send("", "linha1\nlinha2", &NotifyOptions::new()).unwrap_err();
    assert!(matches!(err, Error::EmptyTitle));
}

#[test]
fn default_timeout_is_five_seconds() {
    assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(5));
    assert_eq!(
        NotifyOptions::new(),
        NotifyOptions::new().timeout(DEFAULT_TIMEOUT)
    );
}

#[test]
fn platform_identifier_matches_the_target() {
    let expected = if cfg!(target_os = "linux") {
        Some("linux")
    } else if cfg!(target_os = "macos") {
        Some("darwin")
    } else if cfg!(target_os = "windows") {
        Some("windows")
    } else {
        None
    };
    assert_eq!(platform(), expected);
    assert_eq!(supported(), expected.is_some());
}

#[test]
fn capabilities_match_the_target() {
    let expected = if cfg!(target_os = "linux") {
        Capabilities {
            icon: true,
            sound: false,
            timeout: true,
        }
    } else if cfg!(target_os = "macos") {
        Capabilities {
            icon: false,
            sound: true,
            timeout: false,
        }
    } else if cfg!(target_os = "windows") {
        Capabilities {
            icon: true,
            sound: true,
            timeout: true,
        }
    } else {
        Capabilities::default()
    };
    assert_eq!(platform_capabilities(), expected);
}

#[test]
fn process_failure_reports_captured_stderr() {
    let err = Error::ProcessFailed {
        program: "osascript".to_owned(),
        stderr: "execution error: boom".to_owned(),
    };
    assert_eq!(
        err.to_string(),
        "`osascript` exited unsuccessfully: execution error: boom"
    );
}

#[test]
fn process_failure_without_stderr_says_so() {
    let err = Error::ProcessFailed {
        program: "msg".to_owned(),
        stderr: String::new(),
    };
    assert_eq!(
        err.to_string(),
        "`msg` exited unsuccessfully: no error output"
    );
}

#[test]
fn windows_fallback_failure_keeps_both_causes() {
    let err = Error::WindowsFallbackFailed {
        burnt_toast: Box::new(Error::BinaryNotFound {
            binary: "powershell".to_owned(),
        }),
        msg: Box::new(Error::BinaryNotFound {
            binary: "msg".to_owned(),
        }),
    };
    let text = err.to_string();
    assert!(text.contains("BurntToast"), "{text}");
    assert!(text.contains("msg.exe"), "{text}");
    assert!(text.contains("powershell"), "{text}");
    assert!(text.contains("`msg`"), "{text}");
}

#[test]
fn error_messages_are_clear() {
    assert_eq!(
        Error::EmptyTitle.to_string(),
        "notification title cannot be empty"
    );
    assert_eq!(
        Error::MultilineText { field: "message" }.to_string(),
        "notification message cannot contain line breaks"
    );
    assert_eq!(
        Error::BinaryNotFound {
            binary: "notify-send".to_owned()
        }
        .to_string(),
        "external program `notify-send` was not found in PATH"
    );
}
