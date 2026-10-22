# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-22

### Added

- `Theme` struct with semantic color roles (`background`, `foreground`,
  `primary`, `secondary`, `accent`, `success`, `warning`, `error`, `info`,
  `muted`) and a `syntax` palette.
- `SyntaxColors` struct for code highlighting (`keyword`, `string`, `number`,
  `comment`, `function`, `type`, `operator`, `constant`).
- Five built-in themes: `gruvbox()`, `dracula()`, `nord()`, `catppuccin()` and
  `tokyo_night()`.
- `themes()` returning every built-in theme indexed by name
  (`gruvbox`, `dracula`, `nord`, `catppuccin`, `tokyo-night`).
- `Theme::lipgloss()` deriving a `LipglossStyles` bundle with `base`, `bold`,
  `italic`, `faint`, `success`, `warning`, `error`, `info`, `title`,
  `subtitle`, `highlight`, `border` and `code` styles.
- Parity test against the original Go implementation, backed by a golden file
  generated from `reference/go/charm-themes/cmd/parity`.
- `scripts/extract-themes.ps1`, which generates `src/themes.rs` directly from
  the Go source so the color values are never transcribed by hand.

[0.1.0]: https://forgejo.org/junhinhow/CharmSuite-Rust
