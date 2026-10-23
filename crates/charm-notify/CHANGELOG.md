# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Synchronised with the Go `charm-notify` library.

### Added

* `platform()`, `supported()` and `platform_capabilities()`, with the
  `Capabilities` struct, mirroring the Go `Platform`, `Supported` and
  `PlatformCapabilities`. Platform identifiers are `"linux"`, `"darwin"` and
  `"windows"`.
* `DEFAULT_TIMEOUT` is now public.
* `Error::MultilineText { field }`: the title or the message contains a line
  break. Rejected before any command is built, as in the Go original.
* `Error::WindowsFallbackFailed { burnt_toast, msg }`: on Windows, both
  attempts failed and both causes are kept.
* Windows: the icon is sent to BurntToast with `-Icon`.
* Tests ported from the Go suite: line-break rejection, the timeout rounding
  of the `msg` fallback, the escaping of adversarial payloads and the
  platform capabilities. The command tests of Linux and macOS now run on any
  host.
* `Error::BinaryNotFound` for `notify-send` now says how to install
  `libnotify` (`apt install libnotify-bin`), as the Go original does. The enum
  shape is unchanged; only the message is different.
* Tests for the options each backend ignores: the icon on macOS and on the
  `msg` fallback, and the timeout on BurntToast. Also a test that a successful
  command is reported as `Ok`.

### Changed

* **Breaking:** `Error::ProcessFailed` has a new field `stderr` that holds the
  trimmed standard error of the program. Patterns written as
  `ProcessFailed { program }` must add `..`.
* **Breaking:** the enum `Error` has new variants (`MultilineText`,
  `WindowsFallbackFailed`). Exhaustive `match` expressions need the new arms.
* Windows: BurntToast runs with `-NoProfile -NonInteractive -Command`.
* Windows: `msg` receives `/TIME` only when the timeout is positive, rounded up
  to whole seconds with a minimum of one second. A sub-second timeout becomes
  1 second, not 5.
* Linux: the `notify-send` lookup is done by the spawn itself; a missing
  program is still reported as `Error::BinaryNotFound`.

### Fixed

* The standard error of a failed command is no longer discarded.
* On Windows, the error of a failed BurntToast attempt is no longer dropped when
  the `msg` fallback also fails.

## [0.1.0] - 2026-10-22

### Added

* `send`: sends a desktop notification through the native mechanism of the
  current platform, with the same argument validation as the Go original.
* `NotifyOptions`: fluent options (`icon`, `timeout`, `sound`) with the same
  defaults as the Go original (5 second timeout, no sound, no icon).
* `Notification`: the notification model handed to the platform backends.
* `Error`: errors for empty arguments, unsupported platforms, missing external
  programs and failed processes (built with `thiserror`).
* Windows backend: BurntToast through PowerShell, with `msg.exe` as fallback.
* macOS backend: `display notification` through `osascript`.
* Linux backend: `notify-send`, with `--icon` and `--expire-time`.
* Unit tests that check the command built for each platform, including the
  quoting rules.
* Runnable example (`cargo run -p charm-notify --example charm_notify_demo`).

### Notes

* The crate invokes external programs (`powershell`, `msg`, `osascript`,
  `notify-send`). It does not link to any notification library.

[0.1.0]: https://forgejo.org/junhinhow/CharmSuite-Rust
