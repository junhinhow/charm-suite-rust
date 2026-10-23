# charm-spinner

> [Read in Portuguese](README.pt-br.md)

A collection of 42 beautiful terminal spinner animations for Rust, styled with
[lipgloss](https://crates.io/crates/lipgloss).

This crate is a Rust port of the Go library
[`github.com/junhinhow/charm-spinner`](https://github.com/junhinhow/charm-spinner)
and keeps its behaviour byte for byte.

## Installation

```toml
[dependencies]
charm-spinner = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Usage

```rust
use std::io::{self, Write};
use std::thread::sleep;

use charm_spinner::dots;

fn main() -> io::Result<()> {
    let spinner = dots();
    for frame in 0..40 {
        print!("\r{} Loading...", spinner.render(frame));
        io::stdout().flush()?;
        sleep(spinner.interval);
    }
    println!();
    Ok(())
}
```

A runnable version lives in `examples/charm_spinner_demo.rs`:

```bash
cargo run -p charm-spinner --example charm_spinner_demo -- Dots
cargo run -p charm-spinner --example charm_spinner_demo -- list
```

`render` cycles through the frames automatically: any index is valid, and a
spinner without frames renders an empty string instead of panicking.

## API

| Item | Description |
|------|-------------|
| `Spinner { frames, interval, style }` | A spinner animation: the ordered frames, the delay between them and an optional style |
| `Spinner::new(frames, interval)` | Creates a spinner without a style of its own |
| `Spinner::with_style(style)` | Sets the spinner's own style |
| `Spinner::render(frame)` | Renders the frame at `frame`, cycling with wrap-around |
| `dots()`, `line()`, `star()`, ... | The 42 predefined spinners |
| `all()` | Every predefined spinner, keyed by name (`"Dots"`, `"Line"`, ...) |
| `names()` | The names of the predefined spinners, in alphabetical order |
| `default_style()` | A copy of the package default style |
| `set_default_style(style)` | Replaces the package default style, safely even while other threads render |
| `DEFAULT_STYLE` | The built-in default style (empty), the initial value of the package default |

## Available spinners

| Name | Preview |
|------|---------|
| Dots | `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` |
| Line | `- \ \| /` |
| Star | `✶ ✸ ✹ ✺ ✹ ✷` |
| Moon | `🌑 🌒 🌓 🌔 🌕 🌖 🌗 🌘` |
| Earth | `🌍 🌎 🌏` |
| Clock | `🕐 🕑 🕒 🕓 🕔 🕕 🕖 🕗 🕘 🕙 🕚 🕛` |
| Bounce | `⠁ ⠂ ⠄ ⠂` |
| Bar | `▏ ▎ ▍ ▌ ▋ ▊ ▉ █ ▉ ▊ ▋ ▌ ▍ ▎ ▏` |
| Circle | `◐ ◓ ◑ ◒` |
| Square | `▖ ▘ ▝ ▗` |
| Triangle | `◢ ◣ ◤ ◥` |
| Arrow | `← ↖ ↑ ↗ → ↘ ↓ ↙` |
| Pulse | `█ ▓ ▒ ░ ▒ ▓` |
| Flip | `_ _ _ - \` \` ' ´ - _ _ _` |
| Grow | `▁ ▂ ▃ ▄ ▅ ▆ ▇ █ ▇ ▆ ▅ ▄ ▃ ▂ ▁` |
| Layer | `─ ━ ═ ━` |
| Toggle | `⊶ ⊷` |
| Balloon | `. o O ° O o .` |
| Noise | `▓ ▒ ░ ▒` |
| Fading | `░ ▒ ▓ █ ▓ ▒ ░ ` |
| Christmas | `🎄 🎅 ⛄ 🎁 🔔 ⭐` |
| Monkey | `🙈 🙉 🙊` |
| Hearts | `💛 💙 💜 💚 ❤️ 🧡` |
| Weather | `☀️ 🌤️ ⛅ 🌥️ ☁️ 🌧️ ⛈️ 🌩️ 🌧️ ☁️ 🌥️ ⛅ 🌤️` |
| Soccer | `⚽ 🥅 👟 💨` |
| Matrix | `ᚠ ᚡ ᚢ ᚣ ᚤ ᚥ ᚦ ᚧ ᚨ ᚩ` |
| DNA | `╔═╗ ╚═╝ ╔═╗ ═══ ╚═╝ ═══` |
| Orbit | `◜  ◝  ◞  ◟ ` |
| Rocket | `🚀 💨 ✨ 🌟 ✨ 💨` |
| Fire | `🔥 🔥 💥 ✨ 🔥 💥` |
| Ellipsis | `. .. ...` |
| Rectangle | `▛ ▜ ▙ ▟` |
| Diamond | `◆ ◇` |
| Hexagon | `⬢ ⬡` |
| BouncingBar | `[    ] [=   ] [==  ] [=== ] [ ===] [  ==] [   =]` |
| Digits | `0 1 2 3 4 5 6 7 8 9` |
| Quotes | `' " ❞ ❟` |
| Cat | `😺 😸 😹 😻 😼 😽` |
| Money | `💰 💵 💴 💶` |
| Beach | `🏖️ 🌴 🌊 ☀️` |
| Music | `♪ ♫ ♬` |
| Target | `◎ ◉ ⊙ ◉` |

Every spinner is available at once through `all()`, and `names()` lists their
names in alphabetical order:

```rust
use charm_spinner::{all, names};

let spinners = all();
assert_eq!(spinners.len(), 42);
assert_eq!(names().len(), 42);

for (name, spinner) in spinners {
    println!("{name}: {} frames every {} ms", spinner.frames.len(), spinner.interval.as_millis());
}
```

## Custom spinners

```rust
use std::time::Duration;

use charm_spinner::Spinner;

let custom = Spinner::new(
    vec!["🐱".to_owned(), "🐶".to_owned(), "🐭".to_owned()],
    Duration::from_millis(200),
);

assert_eq!(custom.render(0), "🐱");
assert_eq!(custom.render(3), "🐱"); // wraps around
```

## Styles

Each spinner can carry its own lipgloss style. When it does not, the package
default style applies:

```rust
use std::time::Duration;

use charm_spinner::{Spinner, Style};

let custom = Spinner::new(vec!["🐱".to_owned()], Duration::from_millis(200))
    .with_style(Style::new().bold(true));
```

The package default can be changed at any time, even while other threads are
rendering, with `set_default_style`:

```rust
use charm_spinner::{default_style, set_default_style, Style};

let previous = default_style();
set_default_style(Style::new().bold(true));
// ... render spinners ...
set_default_style(previous);
```

## Parity with the Go original

The frames are **never typed by hand**. They are extracted programmatically
from `reference/go/charm-spinner/spinner.go` by
`crates/charm-spinner/scripts/generate_spinners.rs`, which parses the Go
literals, decodes the escape sequences and converts every frame to its
Unicode code points.

The generated `src/spinners.rs` stores every frame as a `\u{...}` escape, so
the Rust source file is pure ASCII and can never be damaged by an encoding
round-trip (mojibake). The compiler reproduces exactly the same UTF-8 bytes
as the Go source.

The parity tests compare the Rust output with the golden files generated by
the Go program, byte for byte, for all 42 spinners and all 257 frames:

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-spinner/tests/golden/spinners.txt -- \
    go run ./reference/go/charm-spinner/cmd/parity report

cargo run -p charm-parity --bin parity -- gen \
    crates/charm-spinner/tests/golden/spinners_dump.txt -- \
    go run ./reference/go/charm-spinner/cmd/parity dump

cargo test -p charm-spinner
```

To regenerate the Rust source after a change in the Go reference:

```bash
rustc --edition 2021 crates/charm-spinner/scripts/generate_spinners.rs -o generate_spinners.exe
./generate_spinners.exe reference/go/charm-spinner/spinner.go crates/charm-spinner/src/spinners.rs
```

## Known differences

| Go | Rust |
|----|------|
| `spinner.Dots` (package variable) | `charm_spinner::dots()` (function) |
| `spinner.All()` / `spinner.Clone()` return maps | `charm_spinner::all()` returns a new `BTreeMap<&'static str, Spinner>` (sorted) on each call |
| `spinner.Names()` | `charm_spinner::names()` returns `Vec<&'static str>` (sorted) |
| `spinner.Style` (package variable) | `charm_spinner::DEFAULT_STYLE` is the initial value; the current one is `default_style()` / `set_default_style()` |
| `Spinner{Frames, Interval, Style}` | `Spinner { frames, interval, style: Option<Style> }`: a struct literal needs `style: None`, and `with_style` is the builder form |
| `Render(frame int)` accepts negative indexes | `render(frame: usize)` has no negative indexes; use `frames.len() - n` to count from the end |
| `Render` panics when the spinner has no frames | `render` returns an empty string |

## License

MIT
