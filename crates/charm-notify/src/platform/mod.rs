//! Platform backends. Exactly one of the submodules is compiled for the target.

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod other;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub(crate) use linux::send;
#[cfg(target_os = "macos")]
pub(crate) use macos::send;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) use other::send;
#[cfg(target_os = "windows")]
pub(crate) use windows::send;

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use std::process::{Command, Stdio};

/// Process creation flag that stops Windows from opening a console window.
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Runs an external program and waits for it to finish.
///
/// Standard streams are discarded, as the Go original does. A missing program
/// is reported as [`Error::BinaryNotFound`](crate::Error::BinaryNotFound).
#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
pub(crate) fn run_command(mut command: Command) -> Result<(), crate::Error> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let program = command.get_program().to_string_lossy().into_owned();
    match command.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => Err(crate::Error::ProcessFailed { program }),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            Err(crate::Error::BinaryNotFound { binary: program })
        }
        Err(source) => Err(crate::Error::Spawn { program, source }),
    }
}

#[cfg(all(
    test,
    any(target_os = "windows", target_os = "macos", target_os = "linux")
))]
mod tests {
    use super::*;
    use crate::Error;

    #[test]
    fn missing_binary_is_reported_without_panicking() {
        let err = run_command(Command::new("charm-notify-no-such-binary")).unwrap_err();
        assert!(
            matches!(err, Error::BinaryNotFound { ref binary } if binary == "charm-notify-no-such-binary")
        );
    }

    #[test]
    fn non_zero_exit_is_a_process_failure() {
        #[cfg(windows)]
        let command = {
            let mut command = Command::new("cmd");
            command.args(["/C", "exit 3"]);
            command
        };
        #[cfg(not(windows))]
        let command = {
            let mut command = Command::new("sh");
            command.args(["-c", "exit 3"]);
            command
        };

        let err = run_command(command).unwrap_err();
        assert!(matches!(err, Error::ProcessFailed { .. }));
    }
}
