# charm-notify

Cross-platform desktop notifications for terminal interfaces.

> **[Leia em Português](README.pt-br.md)**

This crate is a port of the Go [`charm-notify`](https://github.com/junhinhow/charm-notify)
library. It shows a native notification on Windows, macOS and Linux.

> **This crate invokes external programs.** It does not link to any
> notification library. Each platform uses a program that must be available:
>
> | Platform | Program | How to get it |
> |----------|---------|---------------|
> | Windows | `powershell` (BurntToast module), fallback `msg` | PowerShell ships with Windows; `msg` ships with Windows Pro/Enterprise |
> | macOS | `osascript` | Ships with macOS |
> | Linux | `notify-send` | Install `libnotify` (e.g. `apt install libnotify-bin`) |
>
> If the program is missing, [`send`] returns `Error::BinaryNotFound`.

## Installation

```toml
[dependencies]
charm-notify = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Usage

```rust
use std::time::Duration;

use charm_notify::{NotifyOptions, send};

fn main() -> Result<(), charm_notify::Error> {
    let opts = NotifyOptions::new()
        .icon("dialog-information")
        .timeout(Duration::from_secs(10))
        .sound(true);

    send("Build finished", "All 42 tests passed", &opts)?;
    Ok(())
}
```

### Options

| Method | Default | Description |
|--------|---------|-------------|
| `icon(&str)` | empty | Icon name or path (used on Linux) |
| `timeout(Duration)` | 5 seconds | How long the notification stays on screen |
| `sound(bool)` | `false` | Plays the default notification sound |

### Platform details

* **Windows** runs `New-BurntToastNotification` through PowerShell. If that
  fails, it runs `msg * /TIME:<seconds>`. Single quotes are doubled for
  PowerShell. The PowerShell process does not open a console window.
* **macOS** runs `osascript -e 'display notification ... with title ...'`.
  Backslashes and double quotes are escaped for AppleScript.
* **Linux** runs `notify-send <title> <message>`, adding `--icon=` when an icon
  is set and `--expire-time=<ms>` when the timeout is positive.
* **Other systems** return `Error::UnsupportedPlatform`.

## API Reference

| Item | Description |
|------|-------------|
| `send(&str, &str, &NotifyOptions)` | Send a notification |
| `NotifyOptions::new()` | Default options; the builder methods are `icon`, `timeout` and `sound` |
| `Notification` | Notification model passed to the platform backends |
| `Error` | `EmptyTitle`, `EmptyMessage`, `UnsupportedPlatform`, `BinaryNotFound`, `ProcessFailed`, `Spawn` |

## Known differences from the Go original

* Error messages are in English (crates.io convention). The Go messages are in
  Portuguese.
* The `Notification::builder()` shape from the roadmap is not used: the public
  API is `NotifyOptions`, as in the Go `Option` functions.
* The Linux "not found" error has no installation hint in its message; the
  hint is in the table above.

## Testing

Sending a real notification depends on the desktop session, so the tests check
the command built for each platform (program name, arguments and quoting)
instead of running it:

```bash
cargo test -p charm-notify
```

The test suite also covers argument validation and the detection of a missing
program. The example `charm_notify_demo` sends a real notification when run
manually.

## License

MIT
