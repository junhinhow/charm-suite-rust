//! Linux backend: `notify-send` from `libnotify`.

use std::path::PathBuf;
use std::process::Command;

use super::run_command;
use crate::{Error, Notification};

/// Name of the program that shows the notification.
const PROGRAM: &str = "notify-send";

/// Sends the notification with `notify-send`.
///
/// # Errors
///
/// Returns [`Error::BinaryNotFound`] when `notify-send` is not in `PATH`,
/// which usually means `libnotify` is not installed.
pub(crate) fn send(notification: &Notification) -> Result<(), Error> {
    find_in_path(PROGRAM)?;
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

/// Looks for `binary` in every directory listed in `PATH`.
fn find_in_path(binary: &str) -> Result<PathBuf, Error> {
    std::env::var_os("PATH")
        .and_then(|paths| {
            std::env::split_paths(&paths)
                .map(|dir| dir.join(binary))
                .find(|candidate| candidate.is_file())
        })
        .ok_or_else(|| Error::BinaryNotFound {
            binary: binary.to_owned(),
        })
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
    fn orders_icon_before_expire_time() {
        let command = build_command(&sample("ico", Duration::from_secs(5)));
        assert_eq!(
            args_of(&command),
            ["Build", "passed", "--icon=ico", "--expire-time=5000"]
        );
    }

    #[test]
    fn missing_binary_is_detected_with_a_clear_error() {
        let err = find_in_path("charm-notify-no-such-binary").unwrap_err();
        assert!(
            matches!(err, Error::BinaryNotFound { ref binary } if binary == "charm-notify-no-such-binary")
        );
    }

    #[test]
    fn finds_an_existing_binary_in_path() {
        assert!(find_in_path("sh").is_ok());
    }
}
