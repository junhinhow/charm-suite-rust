# charm-a11y

> [Read in Portuguese](README.pt-br.md)

Accessibility profiles and graceful degradation for terminal applications:
respect `NO_COLOR`, fall back to ASCII, reduce motion, and emit linear text for
screen readers. The crate is built on [lipgloss](https://crates.io/crates/lipgloss).

This crate is a Rust port of the Go library
[`github.com/junhinhow/charm-a11y`](https://github.com/junhinhow/charm-a11y).
Its output matches the Go version after ANSI removal, as checked by golden files
generated from the Go reference.

## Installation

```toml
[dependencies]
charm-a11y = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
lipgloss = "0.1"
```

Not yet published on crates.io; install from git.

## Usage

```rust
use charm_a11y::Profile;
use lipgloss::{rounded_border, Style};

fn main() {
    let profile = Profile::default_profile();

    let title = profile.style(
        Style::new()
            .bold(true)
            .foreground("#7C3AED")
            .border(rounded_border()),
    );
    println!("{}", title.render(&format!("{} pronto", profile.symbol("✓", "ok"))));
    println!("{}", profile.describe());
}
```

With `TERM=dumb` the same program prints an ASCII border, no color, and a plain
text status line. With `A11Y_REDUCED_MOTION=1`, `profile.frames` returns a
single frame.

A runnable version lives in `examples/charm_a11y_demo.rs`:

```bash
cargo run -p charm-a11y --example charm_a11y_demo
NO_COLOR=1 cargo run -p charm-a11y --example charm_a11y_demo
TERM=dumb cargo run -p charm-a11y --example charm_a11y_demo
```

## API

| Item | Description |
|------|-------------|
| `Profile::detect()` | Reads the process environment on each call. |
| `Profile::detect_from(getenv)` | Same detection with an injectable environment; a missing variable is `None`. |
| `Profile::default_profile()` | Process-wide profile, detected once on first use. |
| `Profile` (`Default`) | Zero value: no known restriction, `unicode` is `false`. |
| `Profile { no_color, dumb, high_contrast, reduced_motion, screen_reader, unicode, width }` | Public capability fields; `width` is `0` when unknown. |
| `strip_ansi(s)` | Removes CSI, OSC, DCS/APC/PM/SOS strings and two-byte escapes. |
| `transliterate(s)` | Replaces known glyphs with ASCII equivalents. |
| `profile.sanitize(s)` | Strips ANSI and, without Unicode, transliterates glyphs. |
| `profile.symbol(unicode, ascii)` | Picks the Unicode or ASCII variant. |
| `profile.style(style)` | Removes colors, boosts contrast, and converts borders to ASCII. |
| `profile.border(border)` | Returns an ASCII-safe border when needed. |
| `profile.frames(&frames)` | Returns one frame under reduced motion. |
| `profile.announce(message)` | Formats a linear, ANSI-free message for screen readers. |
| `profile.describe()` | One-line, stable summary prefixed with `a11y: `. |
| `profile.to_string()` (`Display`) | Compact stable form used by `describe`. |

## Environment variables

| Variable | Accepted values | Effect |
|----------|-----------------|--------|
| `NO_COLOR` | Any non-empty value | `no_color`; also `unicode = false` |
| `CLICOLOR` | `0` | `no_color`; also `unicode = false` |
| `TERM` | `dumb` (case-insensitive) | `no_color`, `dumb`, `screen_reader`; `unicode = false` |
| `ASCII` | `1` or `true` | `unicode = false` |
| `A11Y_ASCII` | `1` or `true` | `unicode = false` |
| `A11Y_HIGH_CONTRAST` | `1` or `true` | `high_contrast = true` |
| `A11Y_REDUCED_MOTION` | `1` or `true` | `reduced_motion = true` |
| `A11Y_SCREEN_READER` | `1` or `true` | `screen_reader = true` |
| `COLUMNS` | Positive integer | Sets `width`; otherwise `0` (unknown) |

Notes:

- `NO_COLOR` follows the [no-color.org](https://no-color.org) specification: it
  takes effect only when set to a **non-empty** value. `NO_COLOR=0` still
  disables color.
- The `A11Y_*` and `ASCII` values are trimmed and matched case-insensitively.
- `unicode` is disabled whenever color is disabled, not only for `TERM=dumb`.
  There is no override that forces Unicode while color is off.

## ASCII transliteration

`sanitize` and `border` use the same table as the Go version (222 glyphs,
extracted from the Go source). Unlisted glyphs are preserved because there is no
universal ASCII equivalent. Multi-character replacements are intentional, so
width can change.

| Group | Examples |
|-------|----------|
| Box drawing | `─` `━` → `-`; `│` `┃` → `|`; corners and junctions (`┌`, `╭`, `┼`, `╬`) → `+`; `═` → `=` |
| Blocks and shades | `█` `▓` → `#`; `▒` → `:`; `░` → `.`; `▀` → `^`; `▄` → `_`; `▌` → `[`; `▐` → `]` |
| Arrows and triangles | `→` → `->`; `←` → `<-`; `↑` → `^`; `↓` → `v`; `↔` → `<->`; `▶` `►` → `>`; `◀` → `<` |
| Marks and symbols | `●` `○` → `o`; `✓` `✔` → `v`; `✗` `✘` → `x`; `•` → `*`; `…` → `...`; `“` `”` → `"`; `≤` → `<=`; `⚠` → `(!)` |

## Known differences

- `Profile::default_profile()` replaces the Go `Default()` function. The Rust
  [`Default`] trait gives the zero value, as in Go.
- `detect_from` takes `Fn(&str) -> Option<String>`. There is no `nil` getter;
  use `|_| None` for an empty environment.
- `announce` takes an already formatted message. Use `format!` where Go passes a
  format string and arguments; the result is the same.
- `width` is a `usize`, so negative values do not exist. Invalid or negative
  `COLUMNS` gives `0`, as in Go.
- `style` consumes its `Style` argument (Go passes the style by value).
- lipgloss 0.1.x has no `UnderlineColor`, `BorderForegroundBlend`, or getters for
  per-side border colors. Those parts of the Go test suite are checked by the
  rendered output instead.
- `strip_ansi` is a dedicated parser that keeps control characters such as tabs,
  as Go's `ansi.Strip` does. The `strip_ansi` in `charm_core` (through lipgloss
  0.1.x) removes tabs, so the two can differ on control characters.
- `transliterate` is public. It is an extra; the Go version keeps it internal.
- Border parts that mix known and unknown glyphs are interned, because
  `lipgloss::Border` stores `&'static str`. The leak is bounded by the number of
  distinct parts.
- Strings are always valid UTF-8, so the Go invalid-byte tests use replacement
  characters.

See also [`KNOWN_DIFFERENCES.md`](../../KNOWN_DIFFERENCES.md) at the workspace root.

## License

MIT
