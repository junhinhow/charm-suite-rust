//! macOS backend: `display notification` through `osascript`.

use std::process::Command;

#[cfg(target_os = "macos")]
use super::run_command;
#[cfg(target_os = "macos")]
use crate::Error;
use crate::Notification;

/// Sends the notification with AppleScript.
///
/// The Notification Center has no custom icons and `osascript` has no expiry
/// option, so [`Notification::icon`] and [`Notification::timeout`] are ignored.
#[cfg(target_os = "macos")]
pub(crate) fn send(notification: &Notification) -> Result<(), Error> {
    run_command(build_command(notification))
}

/// Builds the `osascript` invocation that shows the notification.
pub(crate) fn build_command(notification: &Notification) -> Command {
    let mut script = format!(
        r#"display notification "{}" with title "{}""#,
        escape_applescript(&notification.message),
        escape_applescript(&notification.title),
    );
    if notification.sound {
        script.push_str(r#" sound name "default""#);
    }

    let mut command = Command::new("osascript");
    command.arg("-e").arg(script);
    command
}

/// Escapes backslashes and double quotes for an AppleScript string literal.
fn escape_applescript(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::platform::ADVERSARIAL_PAYLOADS;

    fn sample(title: &str, message: &str) -> Notification {
        Notification {
            title: title.to_owned(),
            message: message.to_owned(),
            icon: String::new(),
            timeout: Duration::from_secs(5),
            sound: false,
        }
    }

    fn args_of(command: &Command) -> Vec<String> {
        command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    /// Counts the double quotes that are not escaped by an odd number of
    /// backslashes.
    fn count_unescaped_quotes(text: &str) -> usize {
        let bytes = text.as_bytes();
        let mut total = 0;
        for (i, &byte) in bytes.iter().enumerate() {
            if byte != b'"' {
                continue;
            }
            let backslashes = bytes[..i].iter().rev().take_while(|&&b| b == b'\\').count();
            if backslashes % 2 == 0 {
                total += 1;
            }
        }
        total
    }

    #[test]
    fn uses_osascript_with_an_e_flag() {
        let command = build_command(&sample("Hi", "There"));
        assert_eq!(command.get_program(), "osascript");
        assert_eq!(args_of(&command)[0], "-e");
    }

    #[test]
    fn builds_display_notification_script() {
        let command = build_command(&sample("Build", "passed"));
        assert_eq!(
            args_of(&command)[1],
            r#"display notification "passed" with title "Build""#
        );
    }

    #[test]
    fn escapes_double_quotes_in_title() {
        let command = build_command(&sample(r#"Say "hi""#, "ok"));
        assert!(args_of(&command)[1].ends_with(r#"with title "Say \"hi\"""#));
    }

    #[test]
    fn escapes_backslashes_in_message() {
        let command = build_command(&sample("T", r"C:\tmp"));
        assert!(args_of(&command)[1].contains(r#""C:\\tmp""#));
    }

    #[test]
    fn adds_default_sound_when_enabled() {
        let mut notification = sample("Hi", "There");
        notification.sound = true;
        let command = build_command(&notification);
        assert!(args_of(&command)[1].ends_with(r#" sound name "default""#));
    }

    #[test]
    fn has_no_sound_by_default() {
        let command = build_command(&sample("Hi", "There"));
        assert!(!args_of(&command)[1].contains("sound name"));
    }

    #[test]
    fn escaping_resists_adversarial_payloads() {
        for payload in ADVERSARIAL_PAYLOADS {
            let escaped = escape_applescript(payload);
            assert_eq!(
                count_unescaped_quotes(&escaped),
                0,
                "payload {payload:?} escaped to {escaped:?}"
            );
            // Every backslash of the payload must have been doubled.
            let expected = 2 * payload.matches('\\').count() + payload.matches('"').count();
            assert_eq!(
                escaped.matches('\\').count(),
                expected,
                "payload {payload:?} escaped to {escaped:?}"
            );
        }
    }

    #[test]
    fn literal_stays_closed_for_adversarial_payloads() {
        for payload in ADVERSARIAL_PAYLOADS {
            if payload.contains(['\r', '\n']) {
                // Line breaks are rejected before the command is built.
                continue;
            }
            let command = build_command(&sample(payload, payload));
            let script = &args_of(&command)[1];
            // Four unescaped quotes: opening and closing the message and the title.
            assert_eq!(
                count_unescaped_quotes(script),
                4,
                "payload {payload:?} produced {script:?}"
            );
        }
    }
}
