# charm-test

Deterministic test utilities for Bubble Tea models. `Simulator` delivers window-size and key messages, runs synchronous commands (including batches and sequences), and makes view assertions and Insta snapshots straightforward.

## Installation

```toml
[dev-dependencies]
charm-test = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Usage

```rust
use charmed_bubbletea::{Cmd, KeyMsg, KeyType, Message, Model};
use charm_test::{Simulator, require_view_contains};

struct Counter(u32);
impl Model for Counter {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, message: Message) -> Option<Cmd> {
        if let Some(key) = message.downcast_ref::<KeyMsg>() {
            if key.key_type == KeyType::Runes && key.runes == ['+'] { self.0 += 1; }
        }
        None
    }
    fn view(&self) -> String { format!("Count: {}", self.0) }
}

let mut sim = Simulator::new(Counter(0));
sim.send_key("+");
require_view_contains(&sim, "Count: 1");
```

`require_view`, `require_view_contains`, `require_view_not_contains`, and `require_view_lines` compare ANSI-stripped output. `require_snapshot` uses Insta; update snapshots with `INSTA_UPDATE=always`.

## Known differences

Bubble Tea's `Message` is consumed by `Model::update` and does not implement `Clone`. Accordingly, `Simulator::messages()` returns `MessageRecord` diagnostics rather than the consumed `Message` values from the Go API. The runtime also erases concrete message payload types; records can report known built-in message types but use the erased wrapper's `Debug` form for values. For unknown multi-character key names, Rust preserves their runes because `KeyMsg` has no equivalent `Text` field for Go's zero-code key message.
