# charm-wizard

Multi-step wizard framework for terminal interfaces, with shared state, forward/backward navigation and a styled progress indicator.

> **[Ler em português](README.pt-br.md)**

This is a Rust port of the Go library [`github.com/junhinhow/charm-wizard`](https://github.com/junhinhow/charm-wizard). The step sequencing, shared data, navigation and progress layout follow the original; colours and texts match its output. Run output is verified against a golden file generated from the Go implementation.

## Features

- **Ordered steps** with a name, a description and a closure that receives the shared data
- **Shared state** through a `DataMap` (`HashMap<String, serde_json::Value>`), with typed getters
- **Navigation** with `back`, `forward` and `go_to`. Going back undoes the data written by the step being left, and a failed step cannot be skipped
- **Run modes**: `run` executes the steps from the current one to the end; `run_step` executes only the current one
- **Progress indicator**: step dots (completed, current, pending), a 30-cell bar, the percentage and `progress()` as a fraction

## Install

```toml
[dependencies]
charm-wizard = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Usage

```rust
use charm_wizard::{DataMap, Error, Wizard};
use serde_json::json;

fn main() -> Result<(), Error> {
    let mut wizard = Wizard::new()
        .with_title("Project Setup")
        .add_step("config", "Configuration", |data: &mut DataMap| {
            data.insert("project".into(), json!("charm-suite"));
            Ok(())
        })
        .add_step("deploy", "Deploy", |data: &mut DataMap| {
            if data.contains_key("project") {
                Ok(())
            } else {
                Err(Error::custom("project is missing"))
            }
        });

    wizard.run()?;
    Ok(())
}
```

Steps can also be driven manually:

```rust
use charm_wizard::{DataMap, Wizard};

let mut wizard = Wizard::new()
    .add_step("one", "First", |_: &mut DataMap| Ok(()))
    .add_step("two", "Second", |_: &mut DataMap| Ok(()));

wizard.run_step()?;      // runs step 0 and prints its progress
wizard.forward();
wizard.run_step()?;      // runs step 1
# Ok::<(), charm_wizard::Error>(())
```

## Navigation and undo

`back` moves to the previous step and restores the shared data to the state it had before the step being left ran. `forward` refuses to move when the current step failed, and `go_to` jumps to any valid index:

```rust
use charm_wizard::{DataMap, Error, Wizard};
use serde_json::json;

let mut wizard = Wizard::new()
    .add_step("a", "A", |d: &mut DataMap| {
        d.insert("a".into(), json!(1));
        Ok(())
    })
    .add_step("b", "B", |d: &mut DataMap| {
        d.insert("b".into(), json!(2));
        Ok(())
    });

wizard.run_step()?;
wizard.forward();
wizard.run_step()?;
assert_eq!(wizard.get_int("b"), Some(2));

wizard.back();
assert_eq!(wizard.get_int("b"), None);   // undone
assert_eq!(wizard.get_int("a"), Some(1));
# Ok::<(), Error>(())
```

`run` starts at the current step, so `go_to(2)` followed by `run` executes only the third step and the ones after it.

## Shared data

| Method | Returns |
|--------|---------|
| `set_data(key, value)` | Builder; inserts a `serde_json::Value` |
| `data()` | `&DataMap` with all values |
| `get_string(key)` | `Option<&str>`; `None` when missing or of another type |
| `get_bool(key)` | `Option<bool>` |
| `get_int(key)` | `Option<i64>`; `None` for fractional numbers |

## Progress indicator

`render_progress(steps, current)` returns three lines:

```text
Etapa 2 de 4: Dependencies
[*] config --- [>] deps --- [ ] review --- [ ] deploy
██████████████░░░░░░░░░░░░░░░░ 50%
```

Completed steps are green `[*]`, the current step is yellow `[>]` and pending steps are gray `[ ]`. Colours match the Go implementation. The output of `run` ends with `✓ <title> concluído com sucesso!`, as in Go.

## Differences from Go

- `Wizard::run` writes to standard output, as in Go, and redraws each progress block in place when standard output is a terminal. `Wizard::run_to` and `Wizard::run_step_to` write to any `std::io::Write` and never redraw: they replace Go's `WithWriter`.
- Steps are closures and cannot be `nil`. Use `|_: &mut DataMap| Ok(())` for a step without work.
- `data()` returns a shared reference. Go returns the map itself; change values through steps or with `set_data` while building the wizard.
- `steps()` returns a slice of the steps, not a copy.
- `get_int` returns `i64`, where Go uses `int`.
- Steps return `charm_wizard::Error`. Use `Error::custom("message")` for messages from a step.
- `go_to` takes a `usize`, so Go's negative indices cannot be passed. Any out-of-range value returns `Error::InvalidIndex`.
- Error messages are in English. The progress and success texts are in Portuguese, as in Go.
- Output always contains ANSI escape sequences. Go emits colours only when stdout is a terminal.

## License

MIT
