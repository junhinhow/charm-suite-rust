# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
