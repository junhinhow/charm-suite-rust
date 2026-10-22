# charm-layout

> [Leia em ingles](README.md)

Motor de layout inspirado em Flexbox para interfaces de terminal, construido
sobre [lipgloss](https://crates.io/crates/lipgloss). Crie layouts flex e grid
responsivos, com dimensionamento proporcional, fluxo direcional e espacamento
entre elementos — como o Flexbox do CSS, mas para o terminal.

Este crate e um port Rust da biblioteca Go
[`github.com/junhinhow/charm-layout`](https://github.com/junhinhow/charm-layout)
e mantem o comportamento original byte a byte.

## Recursos

- **Layout flex** — direcao linha ou coluna com dimensionamento proporcional
  dos filhos (ratios como `0.3` / `0.7`)
- **Layout grid** — distribui os filhos automaticamente em colunas de largura
  igual
- **Espacamento** — espaco configuravel entre filhos e celulas
- **Responsivo** — `set_size(width, height)` adapta o layout a qualquer
  dimensao do terminal
- **Construido sobre lipgloss** — usa `join_horizontal` / `join_vertical` por
  dentro para uma renderizacao confiavel

## Instalacao

```bash
cargo add charm-layout
```

## Inicio rapido

### Layout flex

```rust
use charm_layout::{Direction, Flex};

let flex = Flex::new(Direction::Row)
    .set_size(80, 24)
    .set_gap(1)
    .add_child(0.3, "Sidebar")
    .add_child(0.7, "Main Content");

println!("{}", flex.render());
```

### Layout em coluna

```rust
use charm_layout::{Direction, Flex};

let flex = Flex::new(Direction::Column)
    .set_size(80, 24)
    .add_child(0.2, "Header")
    .add_child(0.6, "Body")
    .add_child(0.2, "Footer");

println!("{}", flex.render());
```

### Layout em grade

```rust
use charm_layout::Grid;

let grid = Grid::new(3)
    .set_size(90, 30)
    .set_gap(1)
    .add_children(["Cell 1", "Cell 2", "Cell 3", "Cell 4", "Cell 5"]);

println!("{}", grid.render());
```

## Referencia da API

### `Flex`

| Metodo | Descricao |
|--------|-----------|
| `Flex::new(direction)` | Cria um novo container flex (`Row` ou `Column`) |
| `set_size(width, height)` | Define as dimensoes disponiveis |
| `set_gap(gap)` | Define o espacamento entre os filhos |
| `add_child(ratio, content)` | Adiciona um filho com dimensionamento proporcional |
| `render()` | Calcula o layout e retorna a string composta |

### `Grid`

| Metodo | Descricao |
|--------|-----------|
| `Grid::new(columns)` | Cria uma grade com N colunas |
| `set_size(width, height)` | Define as dimensoes disponiveis |
| `set_gap(gap)` | Define o espacamento entre as celulas |
| `add_child(content)` | Adiciona uma celula |
| `add_children(contents)` | Adiciona varias celulas de uma vez |
| `render()` | Calcula o layout da grade e retorna a string composta |

### Tipos

```rust
pub enum Direction {
    Row,    // Distribuicao horizontal
    Column, // Distribuicao vertical
}

pub struct FlexChild {
    pub ratio: f64,   // 0.0 a 1.0 — proporcao do espaco disponivel
    pub content: String,
}
```

## Comparacao com o Flexbox do CSS

| Flexbox do CSS | charm-layout |
|----------------|--------------|
| `display: flex` | `Flex::new(Direction::Row)` |
| `flex-direction: column` | `Flex::new(Direction::Column)` |
| `flex: 0.3` | `add_child(0.3, content)` |
| `gap: 8px` | `set_gap(1)` |
| `display: grid; grid-template-columns: repeat(3, 1fr)` | `Grid::new(3)` |

## Diferencas conhecidas em relacao ao original em Go

- `Grid::new(0)` ajusta a quantidade de colunas para uma, exatamente como o
  construtor em Go. Como `Grid::columns` e um campo publico, ele pode ser
  alterado para zero depois da construcao; nesse caso o port Rust se comporta
  como uma grade de uma coluna, em vez de gerar um panic de divisao por zero
  como acontece em Go.

## Licenca

MIT
