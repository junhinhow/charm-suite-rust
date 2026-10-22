# charm-wizard

Multi-step wizard framework for terminal interfaces, with shared state, forward/backward navigation and a styled progress indicator.

> **[Ler em português](README.pt-br.md)**

This is a Rust port of the Go library [`github.com/junhinhow/charm-wizard`](https://github.com/junhinhow/charm-wizard). The step sequencing, shared data, navigation and progress layout follow the original; colours match its palette. Run output is verified against a golden file generated from the Go implementation.

## Features

- **Ordered steps** with a name, a description and a closure that receives the shared data
- **Shared state** through a `DataMap` (`HashMap<String, serde_json::Value>`)
- **Navigation** with `back`, `forward` and `go_to`
- **Run modes**: `run` executes every step; `run_step` executes only the current one
- **Progress indicator**: step dots (completed, current, pending), a 30-cell bar and the percentage

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

wizard.run_step()?;      // runs step 0
wizard.forward();
wizard.run_step()?;      // runs step 1
# Ok::<(), charm_wizard::Error>(())
```

## Progress indicator

`render_progress(steps, current)` returns three lines:

```text
Step 2 of 4: Dependencies
[*] config --- [>] deps --- [ ] review --- [ ] deploy
██████████████░░░░░░░░░░░░░░░░ 50%
```

Completed steps are green `[*]`, the current step is yellow `[>]` and pending steps are gray `[ ]`. Colours match the Go implementation.

## Differences from Go

- `Wizard::run` writes to standard output, as in Go. `Wizard::run_to` writes to any `std::io::Write`; it is an addition used for testing.
- Steps return `charm_wizard::Error`. Use `Error::custom("message")` for messages from a step.
- `go_to` takes a `usize`, so Go's negative indices cannot be passed. Any out-of-range value returns `Error::InvalidIndex`.
- `render_progress` returns an empty string for an out-of-range index; Go panics.
- Error messages are in English.
- Output always contains ANSI escape sequences. Go emits colours only when stdout is a terminal.

## License

MIT
