# Changelog

## Unreleased

- Added deterministic virtual `tick`/`every` timers, clock advancement, bounded message/command processing, and `SimulatorOptions`.
- Added stable rendered-content snapshots, key parse errors, aliases, and supported modifier parsing; refreshed the counter Insta snapshot.
- Documented limits caused by the runtime's string-only view and erased timer commands.

## 0.1.0

- Initial Rust port of the deterministic Bubble Tea simulator, assertions, key mapping, and Insta snapshots.
- Added the canonical counter example and Go-generated view parity fixture.
