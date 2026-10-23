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
> If the program is missing, [`send`] returns `Error::BinaryNotFound`. For
> `notify-send` the message also says how to install `libnotify`.

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
| `icon(&str)` | empty | Icon name or path (Linux and Windows/BurntToast) |
| `timeout(Duration)` | 5 seconds | How long the notification stays on screen; zero uses the platform default |
| `sound(bool)` | `false` | Plays the default notification sound (macOS and Windows/BurntToast) |

### Platform support

Not every option is honoured on every operating system. The table shows what
each backend actually does:

| Option | Linux (`notify-send`) | macOS (`osascript`) | Windows (BurntToast) | Windows (`msg` fallback) |
|--------|-----------------------|---------------------|----------------------|--------------------------|
| `icon` | `--icon=` | ignored (no custom icons) | `-Icon '<path>'` | ignored |
| `sound` | ignored (no option) | `sound name "default"` | `-Sound Default` | ignored |
| `timeout` | `--expire-time=<ms>` | ignored (no option) | ignored | `/TIME:<seconds>` |

On Windows the toast is attempted with BurntToast first and falls back to
`msg`; each path honours a different subset of the options.

You can check the current platform at runtime:

```rust
println!("{:?}", charm_notify::platform());               // Some("linux"), Some("darwin"), Some("windows") or None
println!("{}", charm_notify::supported());                // true when the platform has a backend
println!("{:?}", charm_notify::platform_capabilities());  // e.g. Capabilities { icon: true, sound: false, timeout: true }
```

The platform identifiers follow the Go original: `"darwin"` is macOS.

### Timeout semantics

`timeout(d)` behaves as follows:

* `d > 0`: the notification expires after `d`.
* `d == 0`: the notification uses the platform default; no explicit expiry
  is sent.

The Windows `msg` fallback only accepts whole seconds and rounds up, so
`timeout(Duration::from_millis(500))` becomes **1 second** there. A zero
timeout sends no `/TIME` argument at all.

### Text constraints

The title and the message are mandatory and **must not contain line breaks**.
`send` returns `Error::MultilineText { field }` when they do, because a line
break would turn the payload into a multi-line command on the platforms that
build a script (AppleScript and PowerShell).

### Errors

Command failures carry the captured standard error of the program, so the
message explains the cause instead of only a bare exit status:

```rust
match charm_notify::send("Build", "done", &charm_notify::NotifyOptions::new()) {
    Err(charm_notify::Error::ProcessFailed { program, stderr }) => {
        eprintln!("`{program}` failed: {stderr}");
    }
    Err(err) => eprintln!("{err}"),
    Ok(()) => {}
}
```

On Windows, when both BurntToast and `msg` fail, the error is
`Error::WindowsFallbackFailed` and keeps both causes (for example, "BurntToast
is not installed"), so the real cause is never lost.

## Platform Details

* **Windows** runs `New-BurntToastNotification` through
  `powershell -NoProfile -NonInteractive -Command`. Single quotes are doubled
  for PowerShell. The process does not open a console window. If that fails,
  it runs `msg * [/TIME:<seconds>] "<title>: <message>"`.
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
| `DEFAULT_TIMEOUT` | The default timeout (5 seconds) |
| `platform()` | Platform identifier (`"linux"`, `"darwin"`, `"windows"`) or `None` |
| `supported()` | Whether the current platform has a backend |
| `platform_capabilities()` | Which options the current platform honours (`Capabilities`) |
| `Notification` | Notification model passed to the platform backends |
| `Error` | `EmptyTitle`, `EmptyMessage`, `MultilineText`, `UnsupportedPlatform`, `BinaryNotFound`, `ProcessFailed`, `WindowsFallbackFailed`, `Spawn` |

## Known differences from the Go original

* Error messages are in English (crates.io convention). The Go messages are in
  Portuguese.
* The `Notification::builder()` shape from the roadmap is not used: the public
  API is `NotifyOptions`, as in the Go `Option` functions.
* The Go `Send` takes functional options and skips `nil` ones; the Rust
  `send` takes a `&NotifyOptions` value, so there is no `nil` case.
* Go's `ErrCommandNotFound` and `ErrMultilineText` sentinels are replaced by
  the `Error::BinaryNotFound` and `Error::MultilineText` variants, and the Go
  `CommandError` by the `Error::ProcessFailed` variant with `program` and
  `stderr` fields. The Go `Args` field is not kept.
* The Go `Platform()` returns `""` when unsupported; the Rust `platform()`
  returns `None`.

## Testing

Sending a real notification depends on the desktop session, so the tests check
the command built for each platform (program name, arguments and quoting)
instead of running it. The builders of all backends compile in every test
build, so the Linux and macOS command tests also run on Windows:

```bash
cargo test -p charm-notify
```

The test suite also covers argument validation, the escaping of adversarial
payloads, the captured stderr of failed commands and the reported
capabilities. The example `charm_notify_demo` sends a real notification when
run manually.

## License

MIT
