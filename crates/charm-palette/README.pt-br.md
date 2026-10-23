# charm-palette

Visualizador e gerador de paletas de cores para o ecossistema
[charm](https://github.com/charmbracelet).

Amostras de cor, a grade ANSI de 256 cores, gradientes true color e temas
nomeados — estilizados com [`lipgloss`](https://crates.io/crates/lipgloss).

> **[Read in English](README.md)**

Este crate é um port fiel da biblioteca Go
[`charm-palette`](https://github.com/junhinhow/charm-suite): o layout é idêntico
byte a byte (após remover os códigos de escape ANSI).

## Instalação

```toml
[dependencies]
charm-palette = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

## Uso

### Amostras de cor

Desenha um bloco colorido por cor, com o valor hexadecimal abaixo. Quando a
largura é pequena demais para o rótulo `#rrggbb` de cada cor, as amostras são
empilhadas uma por linha, para que a largura pedida nunca seja ultrapassada.

```rust
use charm_core::Rgb;
use charm_palette::show_palette;

let colors = [
    Rgb::rgb(0xff, 0x5f, 0x87),
    Rgb::rgb(0xff, 0xb8, 0x6c),
    Rgb::rgb(0x8b, 0xe2, 0x6c),
];

println!("{}", show_palette(&colors, 36));
```

O rótulo hexadecimal usa uma cor de contraste (preto ou branco, escolhida pela
luminância relativa da WCAG), para continuar legível em amostras claras e
escuras.

### As 256 cores ANSI

```rust
use charm_palette::show_ansi256;

println!("{}", show_ansi256());
```

A grade é dividida nas cores padrão (`0-15`), no cubo de cores 6x6x6
(`16-231`) e na escala de cinza (`232-255`).

### Gradiente true color

A matiz varia na horizontal e a luminosidade varia na vertical.

```rust
use charm_palette::show_true_color;

println!("{}", show_true_color(48, 8));
```

### Temas nomeados

Cada cor é exibida como uma amostra, seguida do nome e do valor hexadecimal. Os
nomes são ordenados alfabeticamente, então a saída é determinística.

```rust
use std::collections::BTreeMap;

use charm_core::Rgb;
use charm_palette::show_theme;

let mut colors = BTreeMap::new();
colors.insert("primary".to_string(), Rgb::rgb(0xff, 0x79, 0xc6));
colors.insert("background".to_string(), Rgb::rgb(0x28, 0x28, 0x28));

println!("{}", show_theme("Dracula", &colors));
```

### Comparando temas

Renderiza vários temas lado a lado. Cada tema vira uma coluna, titulada pela
posição (`Theme 1`, `Theme 2`, ...), e cada nome de cor vira uma linha. Cores que
faltam em um tema aparecem como um marcador tracejado.

```rust
use std::collections::BTreeMap;

use charm_core::Rgb;
use charm_palette::compare_themes;

let mut dracula = BTreeMap::new();
dracula.insert("primary".to_string(), Rgb::rgb(0xff, 0x79, 0xc6));

let mut nord = BTreeMap::new();
nord.insert("primary".to_string(), Rgb::rgb(0x88, 0xc0, 0xd0));

println!("{}", compare_themes(&[&dracula, &nord]));
```

### Entrada inválida

As funções acima devolvem uma string vazia para entrada inválida. As variantes
`try_*` devolvem um [`Error`] que informa o problema:

```rust
use charm_palette::{try_show_palette, Error};

assert_eq!(try_show_palette(&[], 40), Err(Error::NoColors));
```

## Referência da API

| Função | Descrição |
|--------|-----------|
| `show_palette(&[Rgb], usize)` | Renderiza amostras coloridas com o valor hex |
| `show_ansi256()` | Renderiza as 256 cores ANSI em grade |
| `show_true_color(usize, usize)` | Renderiza um gradiente true color |
| `show_theme(&str, &BTreeMap<String, Rgb>)` | Renderiza um tema nomeado |
| `compare_themes(&[&BTreeMap<String, Rgb>])` | Renderiza vários temas lado a lado |
| `try_show_palette(&[Rgb], usize)` | Como `show_palette`, devolvendo `Result` |
| `try_show_true_color(usize, usize)` | Como `show_true_color`, devolvendo `Result` |
| `try_show_theme(&str, &BTreeMap<String, Rgb>)` | Como `show_theme`, devolvendo `Result` |
| `try_compare_themes(&[&BTreeMap<String, Rgb>])` | Como `compare_themes`, devolvendo `Result` |

### Erros

| Variante | Descrição |
|----------|-----------|
| `Error::NoColors` | Nenhuma cor foi informada |
| `Error::NoThemes` | Nenhum tema foi informado |
| `Error::InvalidWidth(usize)` | A largura de exibição é zero |
| `Error::InvalidDimensions { width, height }` | Uma dimensão do gradiente é zero |

## Diferenças conhecidas em relação ao Go original

* A implementação Go recebe valores `image/color.Color`, que podem ser nulos.
  Este port recebe `charm_core::Rgb`, que não pode ser nulo; por isso não há
  `ErrNilColor`.
* As funções Go `*Err` devolvem `error`; as funções `try_*` deste port devolvem
  `Result<String, Error>`.
* `try_show_true_color` informa `Error::InvalidDimensions` onde o
  `ShowTrueColorErr` do Go informa `ErrInvalidWidth`.
* `compare_themes` recebe os temas sem nomes. A função Go também não recebe
  nomes, então as colunas são tituladas pela posição.

## Testes

O crate inclui uma suíte de testes de paridade que compara a saída renderizada
com os arquivos golden gerados pela implementação Go original:

```bash
cargo test -p charm-palette
```

## Licença

MIT
