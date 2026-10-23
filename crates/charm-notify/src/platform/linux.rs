//! Linux backend: `notify-send` from `libnotify`.

use std::process::Command;

#[cfg(target_os = "linux")]
use super::run_command;
#[cfg(target_os = "linux")]
use crate::Error;
use crate::Notification;

/// Name of the program that shows the notification.
const PROGRAM: &str = "notify-send";

/// Sends the notification with `notify-send`.
///
/// The icon and the expiry time are sent only when set. `notify-send` has no
/// sound option, so [`Notification::sound`] is ignored.
///
/// # Errors
///
/// Returns [`Error::BinaryNotFound`] when `notify-send` is not in `PATH`,
/// which usually means `libnotify` is not installed.
#[cfg(target_os = "linux")]
pub(crate) fn send(notification: &Notification) -> Result<(), Error> {
    run_command(build_command(notification))
}

/// Builds the `notify-send` invocation.
pub(crate) fn build_command(notification: &Notification) -> Command {
    let mut command = Command::new(PROGRAM);
    command.arg(&notification.title).arg(&notification.message);

    if !notification.icon.is_empty() {
        command.arg(format!("--icon={}", notification.icon));
    }
    if !notification.timeout.is_zero() {
        command.arg(format!(
            "--expire-time={}",
            notification.timeout.as_millis()
        ));
    }
    command
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn sample(icon: &str, timeout: Duration) -> Notification {
        Notification {
            title: "Build".to_owned(),
            message: "passed".to_owned(),
            icon: icon.to_owned(),
            timeout,
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
    fn passes_title_and_message_first() {
        let command = build_command(&sample("", Duration::ZERO));
        assert_eq!(command.get_program(), PROGRAM);
        assert_eq!(args_of(&command), ["Build", "passed"]);
    }

    #[test]
    fn adds_icon_when_set() {
        let command = build_command(&sample("dialog-information", Duration::ZERO));
        assert_eq!(
            args_of(&command),
            ["Build", "passed", "--icon=dialog-information"]
        );
    }

    #[test]
    fn adds_expire_time_in_milliseconds() {
        let command = build_command(&sample("", Duration::from_millis(1500)));
        assert_eq!(args_of(&command), ["Build", "passed", "--expire-time=1500"]);
    }

    #[test]
    fn omits_expire_time_for_zero_timeout() {
        let command = build_command(&sample("", Duration::ZERO));
        assert!(!args_of(&command)
            .iter()
            .any(|arg| arg.starts_with("--expire-time")));
    }

    #[test]
    fn orders_icon_before_expire_time() {
        let command = build_command(&sample("ico", Duration::from_secs(5)));
        assert_eq!(
            args_of(&command),
            ["Build", "passed", "--icon=ico", "--expire-time=5000"]
        );
    }

    #[test]
    fn ignores_sound_because_notify_send_has_no_option_for_it() {
        let mut notification = sample("", Duration::ZERO);
        notification.sound = true;
        assert_eq!(args_of(&build_command(&notification)), ["Build", "passed"]);
    }

    #[test]
    fn keeps_shell_metacharacters_as_plain_arguments() {
        let mut notification = sample("", Duration::ZERO);
        notification.message = "$(whoami); `id` \"x\"".to_owned();
        let command = build_command(&notification);
        assert_eq!(args_of(&command)[1], "$(whoami); `id` \"x\"");
    }
}
