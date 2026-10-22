# charm-banner

Gerador de banners em arte ASCII para interfaces de terminal — estilizado com [Lip Gloss](https://github.com/charmbracelet/lipgloss).

> [Read in English](README.md)

Este crate e um port Rust da biblioteca Go
[`charm-banner`](https://github.com/junhinhow/charm-banner) e mantem o
comportamento **byte-a-byte identico**, inclusive os dados das fontes bitmap,
que foram extraidos programaticamente do codigo Go original.

## Funcionalidades

- Converte texto em arte ASCII grande usando elementos de bloco Unicode (`▀▄█░▒▓`)
- Tres tamanhos de fonte integrados: **Small** (3 linhas), **Medium** (5 linhas) e **Large** (7 linhas)
- Suporte completo a estilos [Lip Gloss](https://github.com/charmbracelet/lipgloss)
- Suporta letras A-Z, digitos 0-9 e pontuacao comum (`!`, `.`, `-`, `:`)
- Caracteres sem glifo usam o glifo do espaco, exatamente como no Go original

## Instalacao

```toml
[dependencies]
charm-banner = "0.1"
```

## Uso

```rust
use charm_banner::{render, Font, RenderOption};
use lipgloss::{Color, Style};

fn main() {
    // Banner simples (a fonte Medium e a padrao).
    println!("{}", render("HELLO", &[]));

    // Com outro tamanho de fonte.
    println!("{}", render("GO", &[RenderOption::Font(Font::Large)]));

    // Com estilo Lip Gloss.
    let style = Style::new().bold(true).foreground(Color("#ff79c6".to_owned()));
    println!("{}", render("CHARM", &[RenderOption::Style(style)]));
}
```

Executando o demo:

```bash
cargo run -p charm-banner --example demo
```

## API

A API canonica e uma funcao que recebe uma lista de opcoes:

| Item | Descricao |
|------|-----------|
| `render(text: &str, opts: &[RenderOption]) -> String` | Renderiza texto como banner em arte ASCII |
| `RenderOption::Font(Font)` | Define o tamanho da fonte: `Small`, `Medium`, `Large` |
| `RenderOption::Style(lipgloss::Style)` | Aplica um estilo Lip Gloss na saida |
| `render_with(text: &str, opts: &RenderOptions) -> String` | Igual a `render`, com opcoes no estilo builder |
| `RenderOptions::new()` / `::with_font(..)` / `::with_style(..)` | Opcoes no estilo builder |
| `Font::height()` / `Font::width()` / `Font::name()` / `Font::glyphs()` | Metadados e mapa de glifos da fonte |
| `CharPattern { rows }` | O padrao bitmap de um caractere |
| `SMALL_FONT` / `MEDIUM_FONT` / `LARGE_FONT` | Os mapas de glifos das tres fontes |

Duas formas equivalentes de renderizar um banner:

```rust
use charm_banner::{render, render_with, Font, RenderOption, RenderOptions};

// 1. Canonica: uma lista de opcoes.
let a = render("HI", &[RenderOption::Font(Font::Small)]);

// 2. Builder: o mesmo resultado, com uma struct de opcoes.
let b = render_with("HI", &RenderOptions::new().with_font(Font::Small));

assert_eq!(a, b);
```

As opcoes sao aplicadas em ordem, portanto a ultima vence — exatamente como a
lista variadica `Option` do Go original.

## Fontes

| Fonte | Linhas | Colunas nominais | Glifos |
|-------|--------|------------------|--------|
| `Font::Small` | 3 | 3 | 41 |
| `Font::Medium` | 5 | 5 | 41 |
| `Font::Large` | 7 | 7 | 41 |

Cada fonte contem os glifos de `A`-`Z`, `0`-`9`, o espaco e a pontuacao `!`,
`.`, `-` e `:`. Todos os demais caracteres sao renderizados com o glifo do
espaco.

Os dados das fontes ficam em `src/fonts.rs`, arquivo **gerado** por
`xtask/extract_fonts.rs` diretamente de `reference/go/charm-banner/banner.go`:

```bash
rustc --edition 2021 crates/charm-banner/xtask/extract_fonts.rs -o extract_fonts
./extract_fonts reference/go/charm-banner/banner.go crates/charm-banner/src/fonts.rs
```

## Paridade com a implementacao Go

O harness de paridade compara a saida Rust com a do Go original byte-a-byte.
O golden file `tests/golden/banner.txt` e gerado pelo programa Go
`reference/go/charm-banner/cmd/parity`, que despeja todos os glifos das tres
fontes e um conjunto de banners renderizados:

```bash
cargo run -p charm-parity --bin charm-parity -- gen \
    crates/charm-banner/tests/golden/banner.txt -- \
    go run ./reference/go/charm-banner/cmd/parity

cargo test -p charm-banner
```

A comparacao remove os codigos ANSI (o port Rust sempre os emite, o Go so
quando stdout e um TTY) e mantem os espacos no fim de cada linha, de modo que
o espacamento original dos glifos tambem e verificado.

## Diferencas conhecidas

 Sao peculiaridades do Go original que o port reproduz de proposito:

- **Linhas dos glifos "desalinhadas".** `fontWidth` anuncia 3, 5 e 7 colunas,
  mas os glifos das fontes do Go tem na verdade 3, 6 e 9 colunas. A funcao
  nunca e usada durante a renderizacao, e o port mantem os dados originais
  intactos.
- **Coluna em branco depois de caracteres multibyte.** O codigo Go compara o
  deslocamento *em bytes* do rune com o comprimento *em bytes* da string, de
  modo que uma coluna em branco tambem e adicionada depois de um caractere
  com mais de um byte (por exemplo `É`). O port reproduz isso byte-a-byte.
- **Maiusculas.** O Go usa o mapeamento *simples* de maiusculas do Unicode
  (`ß` continua `ß`); o `str::to_uppercase` do Rust usa o mapeamento *completo*
  (`ß` vira `SS`). O port implementa o mapeamento simples para que a
  quantidade de glifos renderizados nao mude.

## Licenca

[MIT](LICENSE)
