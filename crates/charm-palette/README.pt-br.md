# charm-palette

Visualizador e gerador de paletas de cor para o ecossistema
[charm](https://github.com/charmbracelet).

Swatches, a grade de 256 cores ANSI, gradientes true color e temas nomeados —
estilizados com [`lipgloss`](https://crates.io/crates/lipgloss).

> **[Read in English](README.md)**

Este crate e um port fiel da biblioteca Go
[`charm-palette`](https://github.com/junhinhow/charm-suite): o layout e
identico byte a byte (apos remover os codigos de escape ANSI).

## Instalacao

```toml
[dependencies]
charm-palette = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

## Uso

### Swatches

Desenha um bloco colorido por cor, com o valor hexadecimal abaixo.

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

### As 256 cores ANSI

```rust
use charm_palette::show_ansi256;

println!("{}", show_ansi256());
```

A grade e dividida em cores padrao (`0-15`), cubo de cores 6x6x6 (`16-231`) e
escala de cinza (`232-255`).

### Gradiente true color

A matiz varia horizontalmente e a luminosidade verticalmente.

```rust
use charm_palette::show_true_color;

println!("{}", show_true_color(48, 8));
```

### Temas nomeados

Cada cor e exibida como um bloco, seguido do nome e do valor hexadecimal.

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

Desenha varios temas lado a lado. As cores ausentes em um tema sao substituidas
por um marcador tracejado.

```rust
use std::collections::BTreeMap;

use charm_core::Rgb;
use charm_palette::compare_themes;

let mut dracula = BTreeMap::new();
dracula.insert("primary".to_string(), Rgb::rgb(0xff, 0x79, 0xc6));

let mut nord = BTreeMap::new();
nord.insert("primary".to_string(), Rgb::rgb(0x88, 0xc0, 0xd0));

println!("{}", compare_themes(&[("Dracula", &dracula), ("Nord", &nord)]));
```

## Referencia da API

| Funcao | Descricao |
|--------|-----------|
| `show_palette(&[Rgb], usize)` | Desenha swatches coloridos com o valor hex |
| `show_ansi256()` | Desenha as 256 cores ANSI em uma grade |
| `show_true_color(usize, usize)` | Desenha um gradiente true color |
| `show_theme(&str, &BTreeMap<String, Rgb>)` | Desenha um tema nomeado |
| `compare_themes(&[(&str, &BTreeMap<String, Rgb>)])` | Desenha varios temas lado a lado |

## Diferencas conhecidas em relacao ao Go original

* A implementacao Go recebe valores `image/color.Color` e os le through
  `RGBA()`, que devolve canais **premultiplicados**. Este port usa os canais
  brutos de `charm_core::Rgb`, de modo que os swatches mantem a cor pedida.
  Para cores totalmente opacas — o caso comum — as duas implementacoes
  produzem exatamente a mesma saida.
* O rotulo hexadecimal exibido ao lado de um swatch sempre mostra a tripla
  opaca `#rrggbb`, mesmo para cores translucidas.
* A implementacao Go percorre mapas Go, cuja ordem de iteracao e aleatoria.
  Este port usa `BTreeMap`, portanto as cores sao sempre renderizadas em ordem
  alfabetica.

## Testes

O crate inclui uma suite de testes de paridade que compara a saida renderizada
com os arquivos golden gerados pela implementacao Go original:

```bash
cargo test -p charm-palette
```

## Licenca

MIT
