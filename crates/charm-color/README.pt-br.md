# charm-color

Utilitários de manipulação de cores para interfaces de terminal — escurecer,
clarear, misturar, contraste, cor complementar e paletas harmônicas.

> **[Read in English](README.md)**

Este crate é um port Rust da biblioteca Go
[`charm-color`](https://github.com/junhinhow/charm-color). Todas as funções
reproduzem a implementação original byte a byte, incluindo suas regras de
truncamento e arredondamento.

## Instalação

```bash
cargo add charm-color
```

## Uso

```rust
use charm_color::{blend, complementary, contrast, darken, lighten, palette};
use charm_core::Rgb;

fn main() {
    let vermelho = Rgb::rgb(255, 0, 0);

    println!("escuro:       {}", darken(vermelho, 0.3).to_hex());
    println!("claro:        {}", lighten(vermelho, 0.3).to_hex());
    println!("complementar: {}", complementary(vermelho).to_hex());
    println!("cor do texto: {}", contrast(vermelho).to_hex());
    println!("mistura:      {}", blend(vermelho, Rgb::white(), 0.25).to_hex());

    for (indice, cor) in palette(vermelho, 5).iter().enumerate() {
        println!("  paleta[{indice}]: {}", cor.to_hex());
    }
}
```

Uma versão executável deste exemplo está em `examples/demo.rs`:

```bash
cargo run -p charm-color --example demo
```

## API

| Função | Descrição |
|--------|-----------|
| `darken(color, amount)` | Escurece uma cor pelo fator (0.0-1.0) |
| `lighten(color, amount)` | Clareia uma cor pelo fator (0.0-1.0) |
| `blend(c1, c2, t)` | Interpola entre duas cores |
| `contrast(color)` | Retorna preto ou branco para legibilidade |
| `complementary(color)` | Retorna a cor complementar (oposta) |
| `palette(base, n)` | Gera `n` cores harmônicas por rotação de hue |
| `from_lipgloss(&str)` | Converte uma string de cor do lipgloss (`"#rrggbb"`, ANSI-256) |

`Rgb::to_hex`, `Rgb::from_hex` e `Rgb::from_hex_or_black` vêm do crate
[`charm-core`](https://crates.io/crates/charm-core) e ficam disponíveis através
do tipo [`Rgb`].

## Paridade com o original Go

O crate acompanha um *golden file* (`tests/golden/color_ops.txt`) gerado pela
implementação Go original, e um teste que compara as duas saídas byte a byte:

```bash
# regenera o golden file a partir da referência Go
cargo run -p charm-parity --bin parity -- \
    gen crates/charm-color/tests/golden/color_ops.txt -- \
    go run -C reference/go/charm-color ./cmd/parity

# roda o teste de paridade
cargo test -p charm-color --test parity
```

## Diferenças conhecidas

O port reproduz o comportamento do Go, incluindo algumas peculiaridades que
vale explicitar:

- **Alfa pré-multiplicado.** A implementação Go lê as cores através de
  `color.NRGBA.RGBA()`, que pré-multiplica os canais RGB pelo canal alfa. O
  port faz o mesmo, então operações em cores translúcidas atuam sobre valores
  pré-multiplicados.
- **Truncamento, não arredondamento.** `darken`, `lighten` e `blend` truncam
  os canais resultantes (`255 * 0.5` vira `127`, e não `128`), enquanto a
  conversão HSL para RGB de `palette` arredonda. As duas regras foram
  preservadas.
- **`from_lipgloss` e índices fora de faixa.** O original Go repassa a
  string para `lipgloss.Color`, que interpreta valores acima de `255` como
  um valor RGB de 24 bits empacotado e valores negativos como seu valor
  absoluto. Este port delega para
  `charm_core::Rgb::from_lipgloss_str`, que resolve os índices `0..=255` e
  cai para preto em qualquer outro caso.
- **`from_hex` é estrito.** O original Go faz o parse com `fmt.Sscanf`, que
  aceita entradas incomuns (por exemplo `"12 456"`).
  `charm_core::Rgb::from_hex_or_black` aceita apenas `#rrggbb` e `rrggbb` e
  devolve preto opaco para o resto, assim como o original Go faz com strings
  malformadas.

## Licença

[MIT](LICENSE)
