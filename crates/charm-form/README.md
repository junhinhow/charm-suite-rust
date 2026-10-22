# charm-form

Declarative form generator for terminal interfaces. Define forms in YAML, load
and validate them, collect interactive answers, and serialize either the form
specification or the answers as JSON.

> **[Leia em Português](README.pt-br.md)**

This crate ports the public API and behavior of the Go
[`charm-form`](https://github.com/junhinhow/charm-form) package. Interactive
rendering uses the workspace's `charmed-bubbletea` Elm runtime.

## Installation

```toml
[dependencies]
charm-form = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

## YAML form

```yaml
title: Deploy Configuration
description: Configure deployment settings
fields:
  - name: environment
    type: select
    label: Target Environment
    options: [staging, production, development]
    required: true
  - name: version
    type: input
    label: Version
    placeholder: "v1.0.0"
    validate: "^v\\d+\\.\\d+\\.\\d+$"
  - name: dry_run
    type: confirm
    label: Dry run only?
```

## Run interactively

The crate includes a `charm-form` command-line program. It opens a TUI and
writes collected answers as JSON by default:

```text
cargo run -p charm-form --bin charm-form -- -file crates/charm-form/examples/deploy.yaml
```

The CLI also accepts a positional YAML path, `-out PATH`, and `-json=false` to
write a readable summary instead of JSON.

Library callers can run the renderer directly:

```rust,no_run
use std::path::Path;

use charm_form::{load_from_file, renderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec = load_from_file(Path::new("deploy.yaml"))?;
    let answers = renderer::run(spec)?;
    println!("{}", answers.to_json()?);
    Ok(())
}
```

`renderer::run_with_options` accepts custom `Read`/`Write` streams and an
initial terminal size, making scripted runs testable. Escape and Ctrl+C cancel
the form.

### Field types and keys

| Type | Answer | Input |
|------|--------|-------|
| `input` | String | Enter confirms; arrows edit the cursor |
| `text` | String | Enter inserts a newline; Ctrl+S confirms |
| `select` | String | Up/down or k/j, then Enter |
| `multiselect` | String array | Up/down, Space toggles, Enter confirms |
| `confirm` | Boolean | y/n or left/right, then Enter |

Tab and Shift+Tab navigate between fields. Field validation runs on confirm;
invalid values leave the cursor on the field and show the validation message.

## Specification and validation API

`FormSpec::load_from_file` / `load_from_bytes` normalize types and validate the
specification. `normalize_type` accepts case-insensitive names and the
`multi-select` / `multi_select` aliases. Form validation checks:

* a non-empty title and at least one field;
* non-empty, unique field names and a recognized type;
* options for `select` and `multiselect`;
* valid regex syntax at load time.

`FieldSpec::label_text` falls back to the field name; `FieldSpec::regex`
compiles its optional regex. `FormSpec::validate` can validate a constructed
specification directly.

## Answers

Answers are stored as a map of names to `serde_json::Value`:

```rust
use charm_form::{Answers, FormSpec};
use serde_json::Value;

let spec = FormSpec::load_from_bytes(
    b"title: Hi\nfields:\n  - name: who\n    type: input\n    required: true\n",
)?;
let mut answers = Answers::new();
answers.insert("who".to_owned(), Value::String("Ana".to_owned()));
answers.validate(&spec)?;
let json = answers.to_json()?;
# Ok::<(), charm_form::Error>(())
```

Text and select answers are strings, confirm answers are booleans, and
multiselect answers are arrays of strings. `Answers::validate` reports every
invalid or missing required answer; `to_json` and `write_json` serialize the
answer map with two-space indentation.

`FormSpec::to_json`, `write_json`, and `run` serialize the form definition
itself. The interactive runner is `renderer::run`.

## Known differences from Go

* Error messages are in English (crates.io convention), while Go messages are
  in Portuguese.
* The Rust renderer uses the `charmed-bubbletea` runtime and plain selection
  markers instead of Go's Lip Gloss v2 styling. Key bindings and collection
  behavior are ported.

## Testing

```bash
cargo test -p charm-form
```

The parity golden is generated from the Go reference program at
`reference/go/charm-form/cmd/parity`.

## License

MIT
