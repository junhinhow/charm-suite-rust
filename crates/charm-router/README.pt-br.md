# charm-router

Pilha de telas para Bubble Tea v2: navegacao push/pop, guards de saida, params tipados e breadcrumb resistente a resize.

## Instalacao

```toml
[dependencies]
charm-router = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

## Uso

As telas implementam `Screen` e retornam `(ScreenUpdate, Option<Cmd>)` em
`update`: use `ScreenUpdate::Keep` para manter a instância ou
`ScreenUpdate::Replace(Box<dyn Screen>)` para substituí-la. A API oferece
`push`, `pop`, `replace`, `reset`, guards, parâmetros JSON tipados, tamanho
persistente do terminal, títulos/profundidade e `breadcrumb(width)`.

## Licenca

MIT
