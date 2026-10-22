# Changelog

All notable changes to this crate are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-10-22

### Added

- `Font` enum with `Small` (3 rows), `Medium` (5 rows) and `Large` (7 rows)
  fonts; `Medium` is the default.
- `render(text, &[RenderOption])`, the canonical rendering function, and
  `render_with(text, &RenderOptions)`, a builder-style alternative.
- `RenderOption::Font` and `RenderOption::Style` options, applied in order
  (the last one wins).
- `RenderOptions` builder with `new`, `with_font`, `with_style`, `font`,
  `style` and `to_options`.
- `CharPattern` and the `SMALL_FONT`, `MEDIUM_FONT` and `LARGE_FONT` glyph
  maps, plus `Font::glyphs()`, `Font::height()`, `Font::width()` and
  `Font::name()`.
- Font data extracted programmatically from the Go reference implementation
  (`xtask/extract_fonts.rs`), byte-identical to the original.
- Parity harness with the Go original: `reference/go/charm-banner/cmd/parity`
  and the golden file `tests/golden/banner.txt`.
- `examples/demo.rs` and bilingual documentation (English and Portuguese).

[0.1.0]: https://github.com/junhinhow/CharmSuite-Rust/releases/tag/charm-banner-v0.1.0
