# charm-logs

A log viewer for [`charmed-bubbletea`](https://crates.io/crates/charmed-bubbletea), ported from the Go `charm-logs` component. It provides a bounded circular buffer, live-safe appends, follow/navigation, level and text filters, ANSI-safe rendering, and RFC 3339 plain-text export.

## Install

```toml
[dependencies]
charm-logs = "0.1"
charmed-bubbletea = "0.2"
```

## Example

```rust
use charm_logs::{Entry, Level, Model};
use charmed_bubbletea::{KeyMsg, Message, Model as TeaModel};

let mut logs = Model::new();
logs.set_size(80, 24);
logs.append_one(Entry::new(Level::Info, "application started").with_source("startup"));
assert!(logs.view().contains("application started"));

// Forward Bubble Tea messages with logs.update(message). `append_one` returns
// an optional Cmd that may be returned by the parent application's update.
let _ = KeyMsg::from_char('k');
let _ = Message::new(());
```

## API and controls

`Level` orders `Debug < Info < Warn < Error`; `parse_level` also accepts `warning`. `Entry` contains `time`, `level`, `source`, and `message`. `Model::new` defaults to capacity 1000, follow enabled, and 80×24. Use `with_max_entries(n)` or `Model::with_options` to configure capacity; values below 10 clamp to 10.

`append` / `append_one` produce an optional `Cmd` containing `AppendedMsg`. `entries` returns a copy of currently visible entries; `len` includes filtered entries. `export(&mut writer)` emits one sanitized line per entry. The `e` shortcut writes visible entries to `logs.txt` and emits `ExportedMsg`.

| Key | Action |
|---|---|
| `j` / Down, `k` / Up | Navigate; moving up disables follow |
| `g`, `G` | Oldest entry; newest entry and enable follow |
| `/`, Enter, Esc, Backspace | Search, accept, cancel, edit |
| `l`, `f`, `c`, `e` | Cycle minimum severity, toggle follow, clear, export |

Appends are safe from other threads because the shared buffer is mutex-protected. UI mutation belongs on the Bubble Tea update thread. See [Portuguese README](README.pt-br.md).

## License

MIT
