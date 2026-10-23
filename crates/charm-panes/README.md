# charm-panes

Resizable, keyboard-driven panes for terminal interfaces built with Bubble Tea and Lip Gloss.

`charm-panes` is the interactive counterpart of a static split layout: panels
share the available width or height, and the focused panel can grow or shrink
while the others rebalance without ever crossing their minimum size.

## Install

```bash
cargo add charm-panes
```

## Usage

```rust
use charm_panes::{Direction, Layout, Pane};

let mut layout = Layout::new(Direction::Horizontal);
layout.set_size(80, 24);

layout.add(Pane {
    id: "sidebar".to_string(),
    title: "Files".to_string(),
    min_size: 20,
    fixed: true,
    content: "main.go\nREADME.md\ngo.mod".to_string(),
});
layout.add(Pane {
    id: "editor".to_string(),
    title: "main.go".to_string(),
    min_size: 30,
    content: "package main\n\nfunc main() {\n    println!(\"hello\");\n}".to_string(),
});
```

The layout itself does not start a program. Call `set_size` once (or let the
first window size message do it), then render with `view` or `view_model`.

| Key | Action |
| --- | --- |
| `tab` / `shift+tab` | next / previous pane |
| `alt+left` / `alt+right` | shrink / grow the focused pane (horizontal) |
| `alt+up` / `alt+down` | shrink / grow the focused pane (vertical) |
| `+` / `=` | grow the focused pane by one cell |
| `-` / `_` | shrink the focused pane by one cell |

## API

| Symbol | Description |
| --- | --- |
| `Direction`, `Horizontal`, `Vertical` | Layout axis. |
| `Pane` | id, title, content, minimum size and fixed flag. |
| `Layout::new(dir)` | Creates an empty layout. |
| `Layout::add(p)` | Appends a pane; an existing id is replaced in place. |
| `Layout::remove(id)`, `idrop(id)` | Removes a pane. |
| `Layout::pane(id)` | Returns a copy of a pane. |
| `Layout::set_content(id, s)`, `set_title(id, s)` | Updates a pane in place. |
| `Layout::ids()` | ids in layout order. |
| `Layout::set_size(w, h)`, `size()` | Available area. |
| `Layout::focus(i)`, `focused()` | Moves and reads the focus (clamped). |
| `Layout::focus_next()`, `focus_prev()` | Circular focus navigation. |
| `Layout::resize(delta)`, `resize_to(size)` | Resizes the focused pane. |
| `Layout::reset()` | Gives every pane an equal share. |
| `Layout::sizes()`, `visible()` | Current pane sizes and visible count. |
| `Layout::view()` | Renders dividers, titles and focus marker. |
| `Layout::update(msg)`, `init()`, `view_model()` | Bubble Tea integration. |

## Design notes

- **Exact geometry.** Space is split with the largest-remainder method.
- **Degradation.** When the minimum sizes do not fit, the first panes that fit
  remain visible and the last visible title shows `+N`.
- **Fixed panes.** A `Fixed` pane does not donate space when another pane is
  resized.
- **No styling imposed.** The library emits no colors or ANSI styles of its own.

## License

MIT
