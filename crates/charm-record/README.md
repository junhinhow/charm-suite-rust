# charm-record

Record and replay Bubble Tea sessions as deterministic, versioned JSON cassettes.

> [Leia em Português](README.pt-br.md)

## Usage

```rust
use charm_record::{register_msg, RecordMessage, Recorder, Replayer};
use charmed_bubbletea::{Cmd, Message, Model};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)] struct Add(i32);
impl RecordMessage for Add { const TYPE_NAME: &'static str = "app.Add"; }
struct Counter(i32);
impl Model for Counter {
    fn init(&self) -> Option<Cmd> { None }
    fn update(&mut self, msg: Message) -> Option<Cmd> {
        if let Some(add) = msg.downcast::<Add>() { self.0 += add.0; }
        None
    }
    fn view(&self) -> String { self.0.to_string() }
}
register_msg::<Add>();
let mut recorder = Recorder::new(Counter(0), 80, 24);
recorder.update(Add(1));
let cassette = recorder.cassette();
let mut replay = Replayer::new(Counter(0), &cassette)?;
replay.run()?;
assert_eq!(replay.view(), "1");
# Ok::<(), charm_record::Error>(())
```

`load` / `load_bytes` read format v1. `Recorder::save` emits two-space JSON with a trailing newline and sorted metadata. `Event::at` is relative milliseconds and replay never waits. `record_command(name)` writes a named `cmd` marker; replay skips it with a warning. Returned Bubble Tea commands are forwarded and are never executed by this crate.

## Rust-specific type handling

`charmed-bubbletea::Message` erases the concrete type and is consumed by `Model::update`; Rust has no Go-style runtime reflection. Therefore, serializable custom messages implement `RecordMessage` (serde `Serialize` + `DeserializeOwned` and a stable `TYPE_NAME`) and are replayable only after `register_msg::<T>()`. `Recorder::update` takes the typed value before wrapping it as a runtime message. Use `update_unrecorded` for messages that should only be forwarded.

The Rust runtime exposes `KeyMsg` press events but no key-release message; Go key-release cassette events are skipped with a warning. Common Go rune/control key payloads are decoded, but modifier combinations beyond the Rust runtime's `alt` flag cannot be represented exactly. `update_signal` handles built-in quit, interrupt, suspend, resume and focus messages; paste and mouse messages are not exposed by this runtime version. Terminal dimensions are `u16`, so negative Go dimensions cannot be represented. The `step` method returns whether it delivered a message, rather than returning a consumed, non-cloneable runtime `Message`. For custom Go cassette messages, set `TYPE_NAME` to the Go cassette name and keep the JSON shape compatible to enable decoding.

## API

- `Cassette`, `Event`, `VERSION`, `META_UNSERIALIZABLE`, `BubbleTeaSignal`
- `RecordMessage`, `register_msg`, `load`, `load_bytes`
- `Recorder::{new, update, update_key, update_resize, update_signal, update_unrecorded, record_command, cassette, save, reset, view}`
- `Replayer::{new, step, run, view, model, frames, warnings}`
- `Error`, `VersionError`

## License

MIT
