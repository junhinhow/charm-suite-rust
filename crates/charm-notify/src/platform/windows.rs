//! Windows backend: BurntToast through PowerShell, with `msg.exe` as fallback.

use std::process::Command;

use super::run_command;
use crate::{Error, Notification};

/// Sends the notification with BurntToast, falling back to `msg.exe` when
/// PowerShell or the module is unavailable.
pub(crate) fn send(notification: &Notification) -> Result<(), Error> {
    run_command(build_command(notification))
        .or_else(|_| run_command(build_fallback_command(notification)))
}

/// Builds the PowerShell invocation that shows a BurntToast notification.
pub(crate) fn build_command(notification: &Notification) -> Command {
    let mut script = format!(
        "New-BurntToastNotification -Text '{}','{}'",
        escape_powershell(&notification.title),
        escape_powershell(&notification.message),
    );
    if notification.sound {
        script.push_str(" -Sound Default");
    }

    let mut command = Command::new("powershell");
    command.args(["-Command", &script]);
    command
}

/// Builds the `msg.exe` invocation used when BurntToast fails.
pub(crate) fn build_fallback_command(notification: &Notification) -> Command {
    let timeout = notification.timeout.as_secs();
    let seconds = if timeout < 1 { 5 } else { timeout };
    let text = format!("{}: {}", notification.title, notification.message);

    let mut command = Command::new("msg");
    command.arg("*").arg(format!("/TIME:{seconds}")).arg(text);
    command
}

/// Escapes single quotes by doubling them, the PowerShell convention.
fn escape_powershell(text: &str) -> String {
    text.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

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
    fn burnt_toast_uses_powershell() {
        let command = build_command(&sample("Hi", "There"));
        assert_eq!(command.get_program(), "powershell");
        assert_eq!(args_of(&command)[0], "-Command");
    }

    #[test]
    fn burnt_toast_doubles_single_quotes() {
        let command = build_command(&sample("It's", "Ok 'now'"));
        assert_eq!(
            args_of(&command)[1],
            "New-BurntToastNotification -Text 'It''s','Ok ''now'''"
        );
    }

    #[test]
    fn burnt_toast_adds_default_sound_when_enabled() {
        let mut notification = sample("Hi", "There");
        notification.sound = true;
        let command = build_command(&notification);
        assert!(args_of(&command)[1].ends_with(" -Sound Default"));
    }

    #[test]
    fn burnt_toast_has_no_sound_by_default() {
        let command = build_command(&sample("Hi", "There"));
        assert!(!args_of(&command)[1].contains("-Sound"));
    }

    #[test]
    fn fallback_uses_msg_with_title_and_message() {
        let command = build_fallback_command(&sample("Deploy", "done"));
        assert_eq!(command.get_program(), "msg");
        assert_eq!(args_of(&command), ["*", "/TIME:5", "Deploy: done"]);
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
    fn fallback_uses_five_seconds_for_sub_second_timeouts() {
        let mut notification = sample("Deploy", "done");
        notification.timeout = Duration::from_millis(400);
        assert_eq!(
            args_of(&build_fallback_command(&notification))[1],
            "/TIME:5"
        );
    }
}
