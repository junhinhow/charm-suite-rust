# charm-router

Screen stack for Bubble Tea v2: push/pop navigation, exit guards, typed params and a resize-safe breadcrumb.

## Install

```toml
[dependencies]
charm-router = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

## Usage

Screens implement `Screen`; their `update` returns `(ScreenUpdate, Option<Cmd>)`.
Use `ScreenUpdate::Keep` to retain the current value or
`ScreenUpdate::Replace(Box<dyn Screen>)` to return a replacement. The router
also provides `push`, `pop`, `replace`, `reset`, guards, typed JSON parameters,
sticky terminal size, titles/depth and `breadcrumb(width)`.

## License

MIT
