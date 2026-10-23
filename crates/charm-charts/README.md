# charm-charts

Terminal charts for the [charm](https://github.com/charmbracelet) ecosystem.

Sparklines, bar charts and progress bars — styled with [`lipgloss`](https://crates.io/crates/lipgloss).

> **[Leia em Português](README.pt-br.md)**

This crate is a faithful port of the Go [`charm-charts`](https://github.com/junhinhow/charm-suite)
library: the rendering algorithm, the resampling rules and the layout are
byte-for-byte identical (after the ANSI escape codes are stripped).

## Installation

```toml
[dependencies]
charm-charts = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Charts

### Sparkline

Compact line chart using Unicode block characters. Perfect for inline data
visualization.

```rust
use charm_charts::Sparkline;
use lipgloss::{Color, Style};

let data = [12.0, 18.0, 9.0, 24.0, 31.0, 27.0, 15.0, 8.0, 19.0, 22.0];

let spark = Sparkline::new(&data)
    .label("cpu")
    .style(Style::new().foreground(Color("212".to_string())));

println!("{}", spark.render());
// cpu ▁▁▂▂▃▂▂▁  ▁▃▄▅▆▆▇█▇▇▇▆▆▅▄▃▂▁▁   ▁▂▃▃▄▄▄▄
```

The chart always occupies exactly the requested width: when there are fewer
data points than columns the series is expanded by linear interpolation, and
when there are more points they are reduced by averaging each bucket.

Multi-row sparklines for higher resolution:

```rust
use charm_charts::Sparkline;

let spark = Sparkline::new(&[1.0, 2.0, 3.0]).height(3); // 1-8 rows
```

The label is always rendered on the first line of the chart, followed by the
remaining lines indented so the columns stay aligned.

### Bar Chart

Horizontal or vertical bar charts with labels and values.

```rust
use charm_charts::BarChart;
use lipgloss::{Color, Style};

let chart = BarChart::new(&["Go", "Rust"], &[85.0, 72.0]).width(30);

println!("{}", chart.render());
//   Go ████████████████████ 85.0
// Rust █████████████████    72.0
```

`.bar_width(n)` sets the thickness of each bar: in horizontal mode every bar
occupies `n` rows (the label and the value are shown on the first row of the
group), in vertical mode it sets the width of each column.

Vertical mode:

```rust
use charm_charts::BarChart;

let chart = BarChart::new(&["a", "b"], &[3.0, 7.0]).vertical().bar_width(3);
```

Vertical charts honor `.width(n)` (the columns shrink to fit) and
`.show_values(bool)` (the values are printed above the columns).

### Progress Bar

Horizontal progress bar with percentage and custom characters.

```rust
use charm_charts::ProgressBar;
use lipgloss::{Color, Style};

let bar = ProgressBar::new(0.73)
    .width(10)
    .label("Upload")
    .fill_style(Style::new().foreground(Color("42".to_string())));

println!("{}", bar.render());
// Upload ███████░░░  73%
```

## Styling

All charts accept `lipgloss` styles, so they integrate seamlessly with your
app's theme:

```rust
use charm_charts::Sparkline;
use lipgloss::{Color, Style};

let primary = Color("ff79c6".to_string());
let spark = Sparkline::new(&[1.0, 2.0, 3.0]).style(Style::new().foreground(primary));
```

## API Reference

### `Sparkline`

| Method | Description |
|--------|-------------|
| `Sparkline::new(&[f64])` | Create with data points |
| `.width(n)` | Set width in characters (clamped to 1-500) |
| `.height(n)` | Set height in rows (clamped to 1-8) |
| `.style(s)` | Set `lipgloss` style |
| `.label(s)` | Set prefix label |
| `.render()` | Return styled string |

### `BarChart`

| Method | Description |
|--------|-------------|
| `BarChart::new(&[&str], &[f64])` | Create with labels and values |
| `.width(n)` | Set max width (clamped to 1-500) |
| `.bar_width(n)` | Set bar thickness (clamped to 1-20) |
| `.horizontal()` / `.vertical()` | Set orientation (last call wins) |
| `.style(s)` | Set the overall style |
| `.bar_style(s)` / `.label_style(s)` | Set styles |
| `.show_values(bool)` | Toggle value display |
| `.render()` | Return styled string |

### `ProgressBar`

| Method | Description |
|--------|-------------|
| `ProgressBar::new(f64)` | Create with value (0.0-1.0, clamped) |
| `.width(n)` | Set bar width (clamped to 0-500) |
| `.fill_style(s)` / `.empty_style(s)` | Set styles |
| `.label(s)` | Set prefix label |
| `.show_percentage(bool)` | Toggle percentage |
| `.fill_char(c)` / `.empty_char(c)` | Custom characters |
| `.render()` | Return styled string |

## Notes

- Invalid widths and heights are clamped instead of causing a panic.
- Labels are measured by their display width (never by bytes), so accented
  letters and CJK characters stay aligned, and labels that do not fit their
  column are shortened with `…` without breaking a character.

## Known differences from the Go original

* The Go implementation panics when a `ProgressBar` receives a negative `value`
  through the public field. This port clamps the value to `0.0..=1.0`.
* The Go implementation panics when a vertical `BarChart` has a `bar_width`
  larger than the column that `width` leaves for it. This port shrinks the bar
  to the column.

## Testing

The crate ships a parity test suite that compares the rendered output with the
golden files generated by the original Go implementation:

```bash
cargo test -p charm-charts
```

## License

MIT
