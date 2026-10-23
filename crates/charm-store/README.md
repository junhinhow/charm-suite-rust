# charm-store

> [Read in Portuguese](README.pt-br.md)

Observable state for terminal UIs: a typed store with subscriptions, bounded
undo/redo and pluggable snapshots.

This crate is a Rust port of the Go library
[`github.com/junhinhow/charm-store`](https://github.com/junhinhow/charm-store)
and keeps its behaviour: equal values do not notify or grow the history, and
the notification order and the undo/redo semantics are the same.

## Installation

```toml
[dependencies]
charm-store = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Usage

```rust
use charm_store::Store;
use std::sync::{Arc, Mutex};

let counter = Store::new(0);
let seen = Arc::new(Mutex::new(Vec::new()));
let sink = Arc::clone(&seen);
let unsubscribe = counter.subscribe(move |v: i32| sink.lock().unwrap().push(v));

counter.set(1);
counter.set(1); // equal value: no notification and no history
counter.update(|v| v + 1);

counter.undo();
assert_eq!(counter.get(), 1);
assert_eq!(*seen.lock().unwrap(), vec![1, 2, 1]);
unsubscribe();
```

A runnable version lives in `examples/charm_store_demo.rs`:

```bash
cargo run -p charm-store --example charm_store_demo
```

## API

| Item | Description |
|------|-------------|
| `Store::new(initial)` | Creates a store with the default limit of 100 undo states |
| `get` / `set` / `update` | Reads and writes the state; `set` and `update` only notify on real changes |
| `subscribe(f)` | Registers an observer; returns an idempotent cancel function |
| `notify()` | Calls every observer with the current value, even if unchanged |
| `undo` / `redo` | Moves through the history and reports whether anything changed |
| `can_undo` / `can_redo` | Reports whether undo/redo is available |
| `history()` | Number of states available for undo (redo is not counted) |
| `set_history_limit(n)` | Sets the undo/redo depth; values below 1 are raised to 1 |
| `Codec` / `JsonCodec` | Serialization trait and its `serde_json` implementation |
| `snapshot(&codec)` / `restore(&codec, data)` | Serializes the current value and restores a snapshot |
| `select(&source, f)` | Derived store that only notifies when the mapped value changes |

## Terminal UI integration

The store is the source of truth. A subscriber can send a signal through a
channel, and the UI rereads `get()` when it wakes up:

```rust
use std::sync::mpsc::sync_channel;
use charm_store::Store;

let store = Store::new(0);
let (events, wake) = sync_channel::<()>(1);
let _unsubscribe = store.subscribe(move |_: i32| {
    // A full channel already holds a pending signal: coalesce.
    let _ = events.try_send(());
});

store.update(|v| v + 1);
wake.recv().unwrap();
assert_eq!(store.get(), 1);
```

Sending never blocks, so a slow UI never blocks `set`. Call `notify()` to push
the current value without changing it.

## Design notes

- **Concurrency:** every method takes `&self` and is safe to call from several
  threads. Observers run after the internal lock is released, in registration
  order, so a callback may call back into the store.
- **Change detection:** values are compared with `PartialEq`. A no-op `set` or
  `update` neither notifies nor grows the history. `NaN` always counts as
  different.
- **History:** `set` after an undo clears redo. `undo` and `redo` notify with
  the value they restore. A successful `restore` clears both stacks and always
  notifies.
- **Derived stores:** `select` returns a store with history disabled. Its
  subscription lives as long as its source.
- **Codec errors:** errors are wrapped with context and leave the store
  untouched. `restore` decodes before taking the lock, so a bad payload cannot
  corrupt the state.

## Parity with the Go original

The golden file `tests/golden/store.txt` is produced by the Go program in
`reference/go/charm-store/cmd/parity`, which exercises notifications, history,
limits, derived stores and snapshots:

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-store/tests/golden/store.txt -- \
    go -C reference/go/charm-store run ./cmd/parity

cargo test -p charm-store
```

The Go behavioural tests (`store_test.go`, `select_test.go`, `example_test.go`)
are ported to `tests/behavior.rs`.

## Known differences

| Go | Rust |
|----|------|
| `Subscribe(fn func(T))` accepts closures that mutate captured variables | `subscribe` takes `Fn(T) + Send + Sync`; observer state lives in a `Mutex` or atomics |
| Methods are safe for concurrent use with an internal `sync.RWMutex` | Same guarantee, with an internal `Mutex`; all methods take `&self` |
| `Update(nil)`, `Subscribe(nil)`, `Snapshot(nil)`, `Restore(nil)`, `Select(nil, ...)` | Not representable: the type system rejects them |
| `Subscribe` returns `func()` | Returns `impl Fn()`; the cancel behaviour is the same |
| `reflect.DeepEqual` | `PartialEq`; `NaN` is still always different |
| `Codec` takes and fills `any` | `Codec` has generic `marshal`/`unmarshal` returning `Result`; `unmarshal` returns the value |
| Errors in Portuguese (`store: falha ao gerar snapshot`) | Errors in English (`store: failed to create snapshot`) |
| `JSONCodec()` | `JsonCodec` (unit struct) |
| `Store` is a pointer shared by reference | `Store` is not `Clone`; share it with `Arc<Store<T>>` |
| `SetHistoryLimit(n)` accepts negative values | `set_history_limit(usize)`; `0` is raised to `1` |

## License

MIT
