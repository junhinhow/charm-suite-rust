# Changelog

All notable changes to this crate are documented in this file.

## [0.1.0]

### Added

- `Date`, `Time`, `Range` and `Masked` components ported from the Go
  `charm-input`, each implementing `charmed_bubbletea::Model`.
- `DateSelectedMsg` emitted by `Date` on Enter.
- Golden-file parity test generated from the Go original.
- Runnable demo in `examples/charm_input_demo.rs`.
- English and Portuguese (`pt-BR`) documentation.
