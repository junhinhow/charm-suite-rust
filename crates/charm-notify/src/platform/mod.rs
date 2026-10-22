//! Platform backends. Exactly one backend sends notifications on the target.
//!
//! The command builders of every backend are also compiled under `cfg(test)`,
//! so their command construction is tested on any host, not only on the
//! platform the backend targets.

use crate::Capabilities;

#[cfg(any(target_os = "linux", test))]
mod linux;
#[cfg(any(target_os = "macos", test))]
mod macos;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
mod other;
#[cfg(any(target_os = "windows", test))]
mod windows;

#[cfg(target_os = "linux")]
pub(crate) use linux::send;
#[cfg(target_os = "macos")]
pub(crate) use macos::send;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) use other::send;
#[cfg(target_os = "windows")]
pub(crate) use windows::send;

/// Platform identifier reported by [`crate::platform`].
#[cfg(target_os = "linux")]
pub(crate) const PLATFORM: Option<&str> = Some("linux");
/// Platform identifier reported by [`crate::platform`].
#[cfg(target_os = "macos")]
pub(crate) const PLATFORM: Option<&str> = Some("darwin");
/// Platform identifier reported by [`crate::platform`].
#[cfg(target_os = "windows")]
pub(crate) const PLATFORM: Option<&str> = Some("windows");
/// Platform identifier reported by [`crate::platform`].
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) const PLATFORM: Option<&str> = None;

/// Options honoured by the current backend.
#[cfg(target_os = "linux")]
pub(crate) const CAPABILITIES: Capabilities = Capabilities {
    icon: true,
    sound: false,
    timeout: true,
};
/// Options honoured by the current backend.
#[cfg(target_os = "macos")]
pub(crate) const CAPABILITIES: Capabilities = Capabilities {
    icon: false,
    sound: true,
    timeout: false,
};
/// Options honoured by the current backend.
#[cfg(target_os = "windows")]
pub(crate) const CAPABILITIES: Capabilities = Capabilities {
    icon: true,
    sound: true,
    timeout: true,
};
/// Options honoured by the current backend.
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) const CAPABILITIES: Capabilities = Capabilities {
    icon: false,
    sound: false,
    timeout: false,
};

#[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
use std::process::{Command, Stdio};

/// Process creation flag that stops Windows from opening a console window.
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Runs an external program and waits for it to finish.
///
/// Standard output and input are discarded; standard error is captured and
/// reported in [`Error::ProcessFailed`](crate::Error::ProcessFailed). A missing
/// program is reported as [`Error::BinaryNotFound`](crate::Error::BinaryNotFound).
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
        .stderr(Stdio::piped());

    let program = command.get_program().to_string_lossy().into_owned();
    match command.output() {
        Ok(output) if output.status.success() => Ok(()),
        Ok(output) => Err(crate::Error::ProcessFailed {
            program,
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        }),
        Err(source) if source.kind() == std::io::ErrorKind::NotFound => {
            Err(crate::Error::BinaryNotFound { binary: program })
        }
        Err(source) => Err(crate::Error::Spawn { program, source }),
    }
}

/// Texts that try to break out of a quoted literal in a generated command.
#[cfg(test)]
pub(crate) const ADVERSARIAL_PAYLOADS: &[&str] = &[
    "\"",
    "\"\"",
    "\" ; do shell script \"rm -rf /\" ; \"",
    "$(whoami)",
    "`whoami`",
    "'; rm -rf / ; '",
    "\"; rm -rf / ; \"",
    "\\",
    "\\\\",
    "\\n",
    "a\"b'c$d;e|f&g",
    "linha1\nlinha2",
    "linha1\rlinha2",
];

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
        assert!(
            matches!(err, Error::ProcessFailed { ref stderr, .. } if stderr.is_empty()),
            "unexpected error: {err:?}"
        );
    }

    #[test]
    fn captures_stderr_of_a_failed_command() {
        #[cfg(windows)]
        let command = {
            let mut command = Command::new("cmd");
            command.args(["/C", "echo detalhe do erro 1>&2 & exit 1"]);
            command
        };
        #[cfg(not(windows))]
        let command = {
            let mut command = Command::new("sh");
            command.args(["-c", "echo detalhe do erro >&2; exit 1"]);
            command
        };

        let err = run_command(command).unwrap_err();
        let Error::ProcessFailed { stderr, .. } = &err else {
            panic!("expected ProcessFailed, got {err:?}");
        };
        assert!(stderr.contains("detalhe do erro"), "stderr was {stderr:?}");
        assert!(err.to_string().contains("detalhe do erro"));
    }
}
