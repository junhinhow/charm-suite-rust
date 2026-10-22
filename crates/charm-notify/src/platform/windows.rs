//! Windows backend: BurntToast through PowerShell, with `msg.exe` as fallback.

use std::process::Command;
use std::time::Duration;

#[cfg(target_os = "windows")]
use super::run_command;
#[cfg(target_os = "windows")]
use crate::Error;
use crate::Notification;

/// Sends the notification with BurntToast, falling back to `msg.exe` when
/// PowerShell or the module is unavailable.
///
/// When both attempts fail, the error of each one is kept in
/// [`Error::WindowsFallbackFailed`] so the real cause is not lost.
#[cfg(target_os = "windows")]
pub(crate) fn send(notification: &Notification) -> Result<(), Error> {
    match run_command(build_command(notification)) {
        Ok(()) => Ok(()),
        Err(burnt_toast) => run_command(build_fallback_command(notification)).map_err(|msg| {
            Error::WindowsFallbackFailed {
                burnt_toast: Box::new(burnt_toast),
                msg: Box::new(msg),
            }
        }),
    }
}

/// Builds the PowerShell invocation that shows a BurntToast notification.
///
/// The icon (`-Icon`) and the sound (`-Sound Default`) are added only when
/// set. BurntToast has no expiry option, so the timeout is not sent.
pub(crate) fn build_command(notification: &Notification) -> Command {
    let mut script = format!(
        "New-BurntToastNotification -Text '{}','{}'",
        escape_powershell(&notification.title),
        escape_powershell(&notification.message),
    );
    if !notification.icon.is_empty() {
        script = format!("{script} -Icon '{}'", escape_powershell(&notification.icon));
    }
    if notification.sound {
        script.push_str(" -Sound Default");
    }

    let mut command = Command::new("powershell");
    command.args(["-NoProfile", "-NonInteractive", "-Command", &script]);
    command
}

/// Builds the `msg.exe` invocation used when BurntToast fails.
///
/// `msg.exe` has no icon option, so the icon is ignored here. The expiry is
/// passed as `/TIME` in whole seconds and omitted for a zero timeout.
pub(crate) fn build_fallback_command(notification: &Notification) -> Command {
    let mut command = Command::new("msg");
    command.arg("*");
    if let Some(seconds) = timeout_seconds(notification.timeout) {
        command.arg(format!("/TIME:{seconds}"));
    }
    command.arg(format!("{}: {}", notification.title, notification.message));
    command
}

/// Converts a timeout to the whole seconds accepted by `msg.exe`.
///
/// Rounds up and never goes below one second. A zero timeout means "use the
/// system default" and yields `None`.
#[cfg(any(target_os = "windows", test))]
pub(crate) fn timeout_seconds(timeout: Duration) -> Option<u64> {
    if timeout.is_zero() {
        return None;
    }
    let seconds = timeout.as_secs() + u64::from(timeout.subsec_nanos() > 0);
    Some(seconds.max(1))
}

/// Escapes single quotes by doubling them, the PowerShell convention.
fn escape_powershell(text: &str) -> String {
    text.replace('\'', "''")
}

#[cfg(test)]
mod tests {
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

    #[test]
    fn burnt_toast_uses_powershell_without_profile_and_console() {
        let command = build_command(&sample("Hi", "There"));
        assert_eq!(command.get_program(), "powershell");
        let args = args_of(&command);
        assert_eq!(args[..3], ["-NoProfile", "-NonInteractive", "-Command"]);
    }

    #[test]
    fn burnt_toast_doubles_single_quotes() {
        let command = build_command(&sample("It's", "Ok 'now'"));
        assert_eq!(
            args_of(&command)[3],
            "New-BurntToastNotification -Text 'It''s','Ok ''now'''"
        );
    }

    #[test]
    fn burnt_toast_adds_icon_when_set() {
        let mut notification = sample("T", "M");
        notification.icon = "C:/icons/app.ico".to_owned();
        let command = build_command(&notification);
        assert!(args_of(&command)[3].ends_with(" -Icon 'C:/icons/app.ico'"));
    }

    #[test]
    fn burnt_toast_adds_icon_and_sound_in_order() {
        let mut notification = sample("T", "M");
        notification.icon = "C:/icons/app.ico".to_owned();
        notification.sound = true;
        let script = args_of(&build_command(&notification))[3].clone();
        assert!(script.ends_with(" -Icon 'C:/icons/app.ico' -Sound Default"));
    }

    #[test]
    fn burnt_toast_adds_default_sound_when_enabled() {
        let mut notification = sample("Hi", "There");
        notification.sound = true;
        let command = build_command(&notification);
        assert!(args_of(&command)[3].ends_with(" -Sound Default"));
    }

    #[test]
    fn burnt_toast_has_no_icon_or_sound_by_default() {
        let command = build_command(&sample("Hi", "There"));
        let script = &args_of(&command)[3];
        assert!(!script.contains("-Icon"));
        assert!(!script.contains("-Sound"));
    }

    #[test]
    fn burnt_toast_doubles_quotes_in_icon() {
        let mut notification = sample("T", "M");
        notification.icon = "it's.ico".to_owned();
        let script = args_of(&build_command(&notification))[3].clone();
        assert!(script.ends_with(" -Icon 'it''s.ico'"));
    }

    #[test]
    fn escaping_resists_adversarial_payloads() {
        for payload in ADVERSARIAL_PAYLOADS {
            let escaped = escape_powershell(payload);
            // Every single quote of the payload must appear doubled.
            assert_eq!(
                escaped.matches('\'').count(),
                2 * payload.matches('\'').count(),
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
            let script = &args_of(&command)[3];
            // Four delimiters, plus the doubled quotes of the title and the message.
            let expected = 4 + 4 * payload.matches('\'').count();
            assert_eq!(
                script.matches('\'').count(),
                expected,
                "payload {payload:?} produced {script:?}"
            );
        }
    }

    #[test]
    fn fallback_uses_msg_with_title_and_message() {
        let command = build_fallback_command(&sample("Deploy", "done"));
        assert_eq!(command.get_program(), "msg");
        assert_eq!(args_of(&command), ["*", "/TIME:5", "Deploy: done"]);
    }

    #[test]
    fn fallback_omits_time_for_zero_timeout() {
        let mut notification = sample("Deploy", "done");
        notification.timeout = Duration::ZERO;
        assert_eq!(
            args_of(&build_fallback_command(&notification)),
            ["*", "Deploy: done"]
        );
    }

    #[test]
    fn fallback_uses_whole_seconds_from_timeout() {
        let mut notification = sample("Deploy", "done");
        notification.timeout = Duration::from_secs(12);
        assert_eq!(
            args_of(&build_fallback_command(&notification))[1],
            "/TIME:12"
        );
    }

    #[test]
    fn fallback_rounds_sub_second_timeouts_up_to_one_second() {
        let mut notification = sample("Deploy", "done");
        notification.timeout = Duration::from_millis(400);
        assert_eq!(
            args_of(&build_fallback_command(&notification))[1],
            "/TIME:1"
        );
    }

    #[test]
    fn timeout_seconds_rounds_up() {
        let cases = [
            (Duration::ZERO, None),
            (Duration::from_millis(500), Some(1)),
            (Duration::from_secs(1), Some(1)),
            (Duration::from_millis(1500), Some(2)),
            (Duration::from_secs(10), Some(10)),
        ];
        for (timeout, expected) in cases {
            assert_eq!(timeout_seconds(timeout), expected, "timeout {timeout:?}");
        }
    }
}
