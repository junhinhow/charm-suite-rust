# charm-layout

> [Read in Portuguese](README.pt-br.md)

A Flexbox-inspired layout engine for terminal UIs built on top of
[lipgloss](https://crates.io/crates/lipgloss). Create responsive flex and grid
layouts with proportional sizing, directional flow and gap spacing — just like
CSS Flexbox, but for the terminal.

This crate is a Rust port of the Go library
[`github.com/junhinhow/charm-layout`](https://github.com/junhinhow/charm-layout)
and keeps its behaviour byte for byte.

## Features

- **Flex layout** — row or column direction with proportional child sizing
  (ratios like `0.3` / `0.7`)
- **Grid layout** — auto-distributes children into equal-width columns across
  rows
- **Gap spacing** — configurable spacing between children and cells
- **Responsive** — `set_size(width, height)` adapts the layout to any terminal
  dimensions
- **Built on lipgloss** — uses `join_horizontal` / `join_vertical` internally
  for reliable rendering

## Installation

```bash
cargo add charm-layout
```

## Quick start

### Flex layout

```rust
use charm_layout::{Direction, Flex};

let flex = Flex::new(Direction::Row)
    .set_size(80, 24)
    .set_gap(1)
    .add_child(0.3, "Sidebar")
    .add_child(0.7, "Main Content");

println!("{}", flex.render());
```

### Column layout

```rust
use charm_layout::{Direction, Flex};

let flex = Flex::new(Direction::Column)
    .set_size(80, 24)
    .add_child(0.2, "Header")
    .add_child(0.6, "Body")
    .add_child(0.2, "Footer");

println!("{}", flex.render());
```

### Grid layout

```rust
use charm_layout::Grid;

let grid = Grid::new(3)
    .set_size(90, 30)
    .set_gap(1)
    .add_children(["Cell 1", "Cell 2", "Cell 3", "Cell 4", "Cell 5"]);

println!("{}", grid.render());
```

## API reference

### `Flex`

| Method | Description |
|--------|-------------|
| `Flex::new(direction)` | Create a new flex container (`Row` or `Column`) |
| `set_size(width, height)` | Set the available dimensions |
| `set_gap(gap)` | Set the spacing between children |
| `add_child(ratio, content)` | Add a child with proportional sizing |
| `render()` | Compute the layout and return the composed string |

### `Grid`

| Method | Description |
|--------|-------------|
| `Grid::new(columns)` | Create a grid with N columns |
| `set_size(width, height)` | Set the available dimensions |
| `set_gap(gap)` | Set the spacing between cells |
| `add_child(content)` | Add a single cell |
| `add_children(contents)` | Add multiple cells at once |
| `render()` | Compute the grid layout and return the composed string |

### Types

```rust
pub enum Direction {
    Row,    // Horizontal distribution
    Column, // Vertical distribution
}

pub struct FlexChild {
    pub ratio: f64,   // 0.0 to 1.0 — proportion of the available space
    pub content: String,
}
```

## Comparison with CSS Flexbox

| CSS Flexbox | charm-layout |
|-------------|--------------|
| `display: flex` | `Flex::new(Direction::Row)` |
| `flex-direction: column` | `Flex::new(Direction::Column)` |
| `flex: 0.3` | `add_child(0.3, content)` |
| `gap: 8px` | `set_gap(1)` |
| `display: grid; grid-template-columns: repeat(3, 1fr)` | `Grid::new(3)` |

## Known differences from the Go original

- `Grid::new(0)` clamps the column count to one, exactly like the Go
  constructor. Because `Grid::columns` is a public field, it can be set to
  zero after construction; in that case the Rust port behaves like a
  one-column grid instead of panicking with a division by zero like Go does.

## License

MIT
