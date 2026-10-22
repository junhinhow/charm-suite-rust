# charm-i18n

Internationalization for terminal interfaces: message catalogs, locale detection, fallback and printf-style formatting.

> **[Ler em português](README.pt-br.md)**

This is a Rust port of the Go library [`github.com/junhinhow/charm-i18n`](https://github.com/junhinhow/charm-i18n). Lookups, fallback, locale detection and JSON loading behave like the original, and the formatter reproduces `fmt.Sprintf` output for the common verbs. Parity is verified by a golden file generated from the Go implementation.

## Features

- **Catalogs** organized by locale and key, with `add_locale` to add or replace messages
- **Fallback**: keys missing from the active locale are looked up in the fallback locale, then returned as the key itself
- **Locale detection** from `LC_ALL`, `LC_MESSAGES` and `LANG`, normalizing `pt_BR.UTF-8` to `pt-BR`
- **Formatted messages** with `%s`, `%d`, `%v`, `%f`, `%q`, `%t`, `%%` and Rust-style `{}`
- **JSON loading** from a `{ "locale": { "key": "text" } }` file

## Install

```toml
[dependencies]
charm-i18n = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Usage

```rust
use std::collections::BTreeMap;

use charm_i18n::Catalog;
use serde_json::json;

fn main() {
    let mut catalog = Catalog::new();
    catalog.add_locale(
        "en",
        BTreeMap::from([("greeting".to_owned(), "Hello, %s! You have %d messages.".to_owned())]),
    );
    catalog.add_locale(
        "pt-BR",
        BTreeMap::from([("greeting".to_owned(), "Ola, %s! Voce tem %d mensagens.".to_owned())]),
    );

    catalog.set_locale("pt-BR");
    println!("{}", catalog.tf("greeting", &[json!("Alice"), json!(5)]));
    // Ola, Alice! Voce tem 5 mensagens.

    catalog.set_locale("fr");
    println!("{}", catalog.t("missing.key")); // falls back to "en" or returns the key
}
```

### Loading translations from JSON

```rust
use std::path::Path;

use charm_i18n::{detect_locale, Catalog};

fn main() -> Result<(), charm_i18n::Error> {
    let mut catalog = Catalog::new();
    catalog.load_json(Path::new("translations.json"))?;
    catalog.set_locale(&detect_locale());
    Ok(())
}
```

## Formatting

`Catalog::tf` formats the translated template with the arguments, like Go's `fmt.Sprintf`. Without arguments the template is returned unchanged.

| Directive | Meaning |
|---|---|
| `%v` / `{}` | any value, default format |
| `%s` | string (`%.3s` truncates) |
| `%q` | quoted string, or quoted character for integers |
| `%d`, `%x`, `%X`, `%o`, `%b` | integers |
| `%f`, `%F`, `%e`, `%E` | floats (default precision 6) |
| `%t` | boolean |
| `%%` | literal `%` |

Flags `-`, `+`, space and `0`, plus width and precision, are supported.

Mismatched arguments are reported the way Go reports them:

```rust
use charm_i18n::format::sprintf;
use serde_json::json;

assert_eq!(sprintf("%d", &[json!("Alice")]), "%!d(string=Alice)");
assert_eq!(sprintf("%d messages", &[]), "%!d(MISSING) messages");
assert_eq!(sprintf("Hello", &[json!("x")]), "Hello%!(EXTRA string=x)");
```

The formatter is public as `charm_i18n::format::sprintf`.

## Differences from Go

- `Catalog::t` returns a `&str` tied to the catalog and the key; the Go version returns a copy.
- `Catalog::has` checks only the active locale, as the Go version does.
- Arguments are `serde_json::Value`. JSON does not separate `int` from `int64` or `float32` from `float64`, and cannot represent `NaN` or infinity.
- `{}` placeholders and `{{` / `}}` escapes are an addition. Go prints doubled braces literally.
- Directives not listed above (`%c`, `%T`, `%g`, `%p`, `*` widths) print as `%!verb(...)` instead of being formatted.
- Precision is not applied to the value shown in a type-mismatch diagnostic (Go prints `%!f(int=03)` for `%.2f` with `3`).
- `detect_locale` keeps the case of the language and region, like Go.
- Error messages are in English.

## License

MIT
