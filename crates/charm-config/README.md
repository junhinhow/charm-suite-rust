# charm-config

Layered configuration loader, schema validation, and Bubble Tea editor for Rust.
This crate ports the behavior of `reference/go/charm-config`.

```toml
[dependencies]
charm-config = { path = "../charm-config" }
serde_json = "1"
```

```rust,no_run
use charm_config::{Editor, Field, Loader, Schema};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = Loader::new()
        .defaults(json!({"server":{"host":"localhost","port":8080}}))
        .file("config.yaml")
        .env("APP");
    if let Some(error) = loader.error() { eprintln!("config: {error}"); }
    println!("{}:{}", loader.string("server.host", "localhost"), loader.int("server.port", 8080));

    let schema = Schema::new(vec![
        Field::new("server.host", "string").required(true).doc("Database host"),
        Field::new("server.port", "int").required(true).doc("TCP port"),
    ]);
    let editor = Editor::new(loader, schema);
    let _result = charmed_bubbletea::Program::new(editor).run()?;
    Ok(())
}
```

## Formats and precedence

`Loader::file` recognizes `.json`, `.yaml`, and `.yml` (case-insensitive).
Missing or invalid files are skipped and reported by `Loader::error`; the
loader remains usable. The Go reference does not implement TOML. Precedence is
defaults (0), files (100), environment (200), flags (300); custom layers can
choose any priority. Maps merge recursively, while arrays and scalar values are
replaced. Equal priorities preserve insertion order, with later layers winning.

Environment names are case-insensitive for the prefix, then mapped to lowercase
dot paths (`APP_DATABASE_HOST` → `database.host`). Values remain strings until
read with typed getters. Empty segments and suffixes containing dots are ignored.

## Schema and editor

Supported field types are `string`, `int`, `bool`, `duration`, and `list`.
Validation returns all errors in schema order. Convertible strings are accepted
for integer, boolean, and duration fields. `Field::default_value` fills only the
editor snapshot; it neither changes `Loader::all` nor satisfies required-field
validation.

The editor implements `charmed_bubbletea::Model`. Use arrows or `j`/`k` to
navigate, Enter to edit, Tab to switch values/docs, `s` to save, and `q` to quit
(unsaved changes require confirmation). Inline list values use comma-separated
items. Save materializes effective editor values into the highest-priority
file layer, preserves JSON/YAML format, sorts keys, and sets mode `0600` on Unix.
Comments and original formatting are not preserved.

## Public API

`Loader::{new,add,defaults,file,env,flags,get,string,int,boolean,duration,all,sources,error}`;
`Source::new`; `Schema::new` and `Schema::validate`; `Field::{new,required,default_value,doc}`;
`Editor::{new,set_size,values,dirty,save}`; `Duration` is a signed nanosecond
duration compatible with Go's `time.Duration` range and textual units.

## Example

Run `cargo run -p charm-config --example charm_config_demo` from the workspace.
See [README.pt-br.md](README.pt-br.md) for the Portuguese version.
