# charm-image

Render images in the terminal using Kitty, iTerm2, Sixel, ANSI truecolor half-blocks, or a monochrome ASCII fallback.

> **[Leia em Português](README.pt-br.md)**

This is the Rust port of the local Go reference at `reference/go/charm-image`. Images are provided as [`image::DynamicImage`].

## Install

```toml
[dependencies]
charm-image = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
image = { version = "0.25", default-features = false, features = ["png", "jpeg"] }
```

Enable the image formats you need in your own `image` dependency. `charm-image` enables PNG encoding for Kitty and iTerm2.

## Usage

```rust,no_run
use charm_image::{render, Options, Protocol};
use image::ImageReader;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image = ImageReader::open("photo.png")?.decode()?;
    let output = render(&image, Options::default().width(60))?;
    print!("{output}");
    Ok(())
}
```

Force a protocol or use the ASCII renderer directly:

```rust
use charm_image::{ascii_art, render, Options, Protocol};
# use image::{DynamicImage, Rgba, RgbaImage};
# let image = DynamicImage::ImageRgba8(RgbaImage::from_pixel(1, 1, Rgba([255, 0, 0, 255])));
let output = render(&image, Options::default()
    .protocol(Protocol::HalfBlock)
    .width(40)
    .height(20))?;
let monochrome = ascii_art(&image, 40, 0, "");
# let _ = (output, monochrome);
# Ok::<(), charm_image::Error>(())
```

## Protocol detection

| Terminal indicator | Protocol |
| --- | --- |
| `TERM=dumb` or non-empty `NO_COLOR` | ASCII |
| `KITTY_WINDOW_ID` | Kitty |
| `TERM_PROGRAM=iTerm.app`, `WezTerm`, or `mintty` | iTerm2 |
| `TERM` or `COLORTERM` containing `sixel` | Sixel |
| Otherwise | ANSI truecolor half-block |

Detection is environment-based and does not query the terminal. An explicit `Options::protocol` overrides it.

## API

| Symbol | Description |
| --- | --- |
| `render`, `Options` | Main entry point and dimensions/protocol/dither/background settings |
| `Protocol`, `detect`, `detect_from` | Protocol values and environment detection |
| `ascii_art` | Monochrome renderer with configurable character ramp |
| `render_half_block` | Universal `▀` renderer using foreground/background truecolor |
| `render_kitty`, `render_iterm2`, `render_sixel` | Raw protocol escape sequences |
| `Error` | Empty input, invalid protocol, size and PNG-encoding errors |

## Behavior and limitations

- Natural width is the source width capped at 80 cells; missing height preserves approximate terminal aspect ratio (cells are treated as twice as tall as wide).
- Half-block, ASCII and Sixel composite transparent pixels over `Options::background`, or black by default. Kitty/iTerm2 preserve PNG alpha.
- ASCII dithering applies an ordered Bayer 4×4 luminance offset; it does not affect color output.
- Kitty payloads are PNG/base64 and split into 4096-byte APC chunks when needed. iTerm2 uses OSC 1337.
- Sixel uses an 8×16 raster per cell, 216-color cube plus 40 grays, and run-length encoding. As in the Go reference, the Sixel payload is base64 wrapped inside DCS; standard terminals expect raw Sixel, so decode this payload before sending it directly to one.
- `render` positions the cursor at row 1, column 1 before Sixel output; `render_sixel` returns only the raw DCS sequence.
- Terminal protocol behavior is covered by unit tests, not real terminal integration. Half-block requires truecolor support; scaling uses simple box averaging without gamma correction or image rotation.
- Rust APIs take `&DynamicImage`, so a nil image is unrepresentable; zero-sized images return `Error::EmptyImage`.

## License

MIT
