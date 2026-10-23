# charm-input

Entradas de data, hora, intervalo e texto mascarado para Bubble Tea.

> [Read in English](README.md)

## Instalação

```bash
cargo add charm-input
```

## Uso

```rust
use charm_input::Date;
use chrono::NaiveDate;

let mut date = Date::new(NaiveDate::from_ymd_opt(2026, 10, 22).expect("valid"));
date.set_size(40, 8);
date.focus();
```

Cada componente implementa `charmed_bubbletea::Model`.

## Diferenças conhecidas

- Os componentes não consultam `time::now`; o destaque de hoje vem de `set_today`.
- `NO_COLOR` troca a barra do intervalo e o cursor mascarado para ASCII.

## Licença

MIT
