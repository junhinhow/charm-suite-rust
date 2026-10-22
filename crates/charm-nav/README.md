# charm-nav

> [Read in Portuguese](README.pt-br.md)

Navigation components for terminal UIs, styled with
[lipgloss](https://crates.io/crates/lipgloss).

This crate is a Rust port of the Go library
[`github.com/junhinhow/charm-nav`](https://github.com/junhinhow/charm-nav) and
keeps its behaviour byte for byte.

## Components

### Tabs

Horizontal tab bar with active/inactive styling and configurable separators.

```rust
use charm_nav::Tabs;

let mut tabs = Tabs::new(&["Home", "Settings", "About"]);
tabs.next();
println!("{}", tabs.render());
// Home | Settings | About
//         ^^^^^^^^ (active, bold + underline)
```

**API:**

| Method | Description |
|--------|-------------|
| `Tabs::new(items)` | Create a new tab bar |
| `set_active(index)` | Set the active tab by index (out of range values are ignored) |
| `next()` | Move to the next tab (wraps around) |
| `previous()` | Move to the previous tab (wraps around) |
| `active()` | Get the active tab index |
| `items()` | Get all tab labels |
| `render()` | Render the styled tab bar |

**Customizable fields:** `separator`, `active_style`, `inactive_style`

---

### Breadcrumb

Trail navigation with a push/pop stack and a configurable separator.

```rust
use charm_nav::Breadcrumb;

let mut breadcrumb = Breadcrumb::new(&["Home", "Settings"]);
breadcrumb.push("Profile");
println!("{}", breadcrumb.render());
// Home > Settings > Profile
//                    ^^^^^^^ (active, bold)
```

**API:**

| Method | Description |
|--------|-------------|
| `Breadcrumb::new(items)` | Create a new breadcrumb trail |
| `push(item)` | Append an item |
| `pop()` | Remove and return the last item (empty string when empty) |
| `items()` | Get all breadcrumb items |
| `render()` | Render the styled breadcrumb |

**Customizable fields:** `separator`, `active_style`, `inactive_style`

---

### Sidebar

Vertical menu with a cursor indicator and a selection highlight.

```rust
use charm_nav::Sidebar;

let mut sidebar = Sidebar::new(&["Dashboard", "Users", "Settings", "Logs"]);
sidebar.set_selected(1);
println!("{}", sidebar.render());
//   Dashboard
// ▸ Users        (selected, highlighted)
//   Settings
//   Logs
```

**API:**

| Method | Description |
|--------|-------------|
| `Sidebar::new(items)` | Create a new sidebar menu |
| `set_selected(index)` | Set the selected item by index (out of range values are ignored) |
| `next()` | Move to the next item (wraps around) |
| `previous()` | Move to the previous item (wraps around) |
| `selected()` | Get the selected item index |
| `items()` | Get all menu items |
| `render()` | Render the styled sidebar |

**Customizable fields:** `width`, `cursor`, `selected_style`, `unselected_style`

## Default styles

| Component | Style | Value |
|-----------|-------|-------|
| Tabs | `active_style` | bold, underline, foreground `#7D56F4` |
| Tabs | `inactive_style` | foreground `#666666` |
| Tabs | `separator` | `" \| "` |
| Breadcrumb | `active_style` | bold, foreground `#7D56F4` |
| Breadcrumb | `inactive_style` | foreground `#666666` |
| Breadcrumb | `separator` | `" > "` |
| Sidebar | `selected_style` | bold, foreground `#7D56F4`, background `#1A1A2E` |
| Sidebar | `unselected_style` | foreground `#666666` |
| Sidebar | `width` | `20` |
| Sidebar | `cursor` | `"▸ "` |

## Installation

```bash
cargo add charm-nav
```

## License

MIT
