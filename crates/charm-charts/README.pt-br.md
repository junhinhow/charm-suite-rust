# charm-charts

Graficos de terminal para o ecossistema [charm](https://github.com/charmbracelet).

Sparklines, graficos de barras e barras de progresso — estilizados com
[`lipgloss`](https://crates.io/crates/lipgloss).

> **[Read in English](README.md)**

Este crate e um port fiel da biblioteca Go
[`charm-charts`](https://github.com/junhinhow/charm-suite): o algoritmo de
renderizacao, as regras de reamostragem e o layout sao identicos byte a byte
(apos remover os codigos de escape ANSI).

## Instalacao

```bash
cargo add charm-charts
```

## Graficos

### Sparkline

Grafico de linha compacto usando caracteres de bloco Unicode. Perfeito para
visualizacao inline de dados.

```rust
use charm_charts::Sparkline;
use lipgloss::{Color, Style};

let data = [4.0, 2.0, 7.0, 3.0, 8.0, 5.0, 9.0, 1.0, 6.0, 4.0];

let spark = Sparkline::new(&data)
    .width(30)
    .label("CPU")
    .style(Style::new().foreground(Color("212".to_string())));

println!("{}", spark.render());
// CPU ▃▁▆▂▇▄█ ▅▃
```

Sparklines de varias linhas para maior resolucao:

```rust
use charm_charts::Sparkline;

let spark = Sparkline::new(&[1.0, 2.0, 3.0]).height(3); // 1-8 linhas
```

### Grafico de barras

Graficos de barras horizontais ou verticais com rotulos e valores.

```rust
use charm_charts::BarChart;
use lipgloss::{Color, Style};

let labels = ["Go", "Rust", "Python", "JS"];
let values = [85.0, 72.0, 95.0, 68.0];

let chart = BarChart::new(&labels, &values)
    .width(50)
    .horizontal()
    .bar_style(Style::new().foreground(Color("99".to_string())));

println!("{}", chart.render());
//     Go ████████████████████████████████████████ 85.0
//   Rust ██████████████████████████████████       72.0
// Python █████████████████████████████████████████████ 95.0
//     JS ████████████████████████████████         68.0
```

Modo vertical:

```rust
use charm_charts::BarChart;

let chart = BarChart::new(&["a", "b"], &[3.0, 7.0]).vertical().bar_width(3);
```

### Barra de progresso

Barra de progresso horizontal com porcentagem e caracteres customizaveis.

```rust
use charm_charts::ProgressBar;
use lipgloss::{Color, Style};

let bar = ProgressBar::new(0.73)
    .width(30)
    .label("Upload")
    .fill_style(Style::new().foreground(Color("42".to_string())));

println!("{}", bar.render());
// Upload ██████████████████████░░░░░░░░░░  73%
```

## Estilizacao

Todos os graficos aceitam estilos do `lipgloss`, entao integrates o tema do
seu aplicativo sem esfuerzo:

```rust
use charm_charts::Sparkline;
use lipgloss::{Color, Style};

let primary = Color("ff79c6".to_string());
let spark = Sparkline::new(&[1.0, 2.0, 3.0]).style(Style::new().foreground(primary));
```

## Referencia da API

### `Sparkline`

| Metodo | Descricao |
|--------|-----------|
| `Sparkline::new(&[f64])` | Cria com os pontos de dados |
| `.width(n)` | Define a largura em caracteres |
| `.height(n)` | Define a altura em linhas (1-8, limitado) |
| `.style(s)` | Define o estilo do `lipgloss` |
| `.label(s)` | Define o rotulo de prefixo |
| `.render()` | Retorna a string estilizada |

### `BarChart`

| Metodo | Descricao |
|--------|-----------|
| `BarChart::new(&[&str], &[f64])` | Cria com rotulos e valores |
| `.width(n)` | Define a largura maxima |
| `.bar_width(n)` | Define a espessura das barras |
| `.horizontal()` / `.vertical()` | Define a orientacao (a ultima chamada vence) |
| `.style(s)` | Define o estilo geral |
| `.bar_style(s)` / `.label_style(s)` | Definem os estilos |
| `.show_values(bool)` | Alterna a exibicao dos valores |
| `.render()` | Retorna a string estilizada |

### `ProgressBar`

| Metodo | Descricao |
|--------|-----------|
| `ProgressBar::new(f64)` | Cria com o valor (0.0-1.0, limitado) |
| `.width(n)` | Define a largura da barra |
| `.fill_style(s)` / `.empty_style(s)` | Definem os estilos |
| `.label(s)` | Define o rotulo de prefixo |
| `.show_percentage(bool)` | Alterna a porcentagem |
| `.fill_char(c)` / `.empty_char(c)` | Caracteres customizados |
| `.render()` | Retorna a string estilizada |

## Diferencas conhecidas em relacao ao Go original

* A implementacao Go entra em panic ao renderizar um `Sparkline` com `width`
  igual a `0` (ela acessa um slice reamostrado vazio). Este port retorna uma
  string vazia.
* A implementacao Go entra em panic quando um `ProgressBar` recebe um `value`
  negativo pelo campo publico. Este port limita o valor a `0.0..=1.0`.
* A implementacao Go trunca os rotulos de um `BarChart` vertical por **bytes**,
  o que pode partir um caractere multibyte e gerar UTF-8 invalido. Este port
  mantem a semantica orientada a bytes, mas sempre devolve UTF-8 valido
  (bytes invalidos viram `U+FFFD`).

## Testes

O crate inclui uma suite de testes de paridade que compara a saida renderizada
com os arquivos golden gerados pela implementacao Go original:

```bash
cargo test -p charm-charts
```

## Licenca

MIT
