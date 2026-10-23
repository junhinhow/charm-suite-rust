# charm-command

A keyboard-driven command palette for Bubble Tea terminal applications. It
filters titles, descriptions, and groups with a ranked, accent-insensitive
subsequence matcher and renders a centered Lipgloss overlay.

```rust
use charmed_bubbletea::Model;
use charm_command::{Command, Palette};

let mut palette = Palette::new(vec![
    Command::new("save", "Save file")
        .with_description("Write the current buffer to disk")
        .with_group("File")
        .with_keys(["ctrl+s"]),
]);
palette.set_size(80, 24);
palette.open();
println!("{}", palette.view());
```

`Palette` implements `charmed_bubbletea::Model`. Forward key messages to its
`update` method while it is open; `Enter` invokes the selected command's
`with_run` callback and closes the palette. Clones share the same internal
state so application code can retain a handle while passing a clone to the
Bubble Tea runtime.

## Keyboard controls

| Key | Action |
| --- | --- |
| Up / `ctrl+p` | Previous result (clamped) |
| Down / `ctrl+n` | Next result (clamped) |
| `pgup` / `pgdown` | Previous / next page |
| `home` / `end` | First / last result |
| `enter` | Run selected command and close |
| `esc` | Close and clear the query |
| `backspace` / `ctrl+h` | Delete the final query character |
| `ctrl+u` | Clear the query |
| Printable characters | Append to query |

## Filtering and rendering

Matching is case- and common-Latin-accent-insensitive and operates on a
subsequence of title, group, or description. Scores weight title matches by 3,
group matches by 2, and descriptions by 1, with bonuses for exact/prefix,
word-boundary, early, and consecutive matches. Results are ordered by score,
then folded title, stably. An empty query returns all commands ordered by title.

Defaults: 80×24 viewport, 8 visible commands, 64-column box maximum, title
`Commands`, placeholder `Type a command…`. `set_size` controls the entire
overlay; a zero dimension returns an empty view. Shortcuts are hidden in narrow
layouts, and descriptions appear only when there is room.

## API

- `Command::new(id, title)` and `with_description`, `with_group`, `with_keys`,
  `with_run` configure an action.
- `Palette::new` / `Palette::with_options` construct a model.
- `set_commands`, `commands`, `add`, `remove` manage actions.
- `open`, `close`, `toggle`, `is_open` manage visibility.
- `set_query`, `query`, `filtered`, `selected` access search and selection.
- `set_size`, `set_placeholder`, `set_max_visible`, `set_theme`, `render`
  configure and display the overlay.
- `default_theme`, `with_max_visible`, `with_title`, `with_theme` provide
  defaults and construction options.

The matcher is implemented in this crate; no `charm-fuzzy` dependency is used.
Mouse interaction, asynchronous loading, and paste handling are intentionally
left to the host application.

## License

MIT
