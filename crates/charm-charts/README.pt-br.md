# charm-charts

Gráficos de terminal para o ecossistema [charm](https://github.com/charmbracelet).

Sparklines, gráficos de barras e barras de progresso — estilizados com
[`lipgloss`](https://crates.io/crates/lipgloss).

> **[Read in English](README.md)**

Este crate é um port fiel da biblioteca Go
[`charm-charts`](https://github.com/junhinhow/charm-suite): o algoritmo de
renderização, as regras de reamostragem e o layout são idênticos byte a byte
(após remover os códigos de escape ANSI).

## Instalação

```toml
[dependencies]
charm-charts = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

## Gráficos

### Sparkline

Gráfico de linha compacto usando caracteres de bloco Unicode. Perfeito para
visualização inline de dados.

```rust
use charm_charts::Sparkline;
use lipgloss::{Color, Style};

let data = [12.0, 18.0, 9.0, 24.0, 31.0, 27.0, 15.0, 8.0, 19.0, 22.0];

let spark = Sparkline::new(&data)
    .label("cpu")
    .style(Style::new().foreground(Color("212".to_string())));

println!("{}", spark.render());
// cpu ▁▁▂▂▃▂▂▁  ▁▃▄▅▆▆▇█▇▇▇▆▆▅▄▃▂▁▁   ▁▂▃▃▄▄▄▄
```

O gráfico ocupa sempre exatamente a largura pedida: quando há menos pontos que
colunas, a série é expandida por interpolação linear; quando há mais pontos,
cada faixa é reduzida pela média.

Sparklines de várias linhas para maior resolução:

```rust
use charm_charts::Sparkline;

let spark = Sparkline::new(&[1.0, 2.0, 3.0]).height(3); // 1-8 linhas
```

O rótulo fica sempre na primeira linha do gráfico; as demais linhas recebem
um recuo para que as colunas continuem alinhadas.

### Gráfico de barras

Gráficos de barras horizontais ou verticais com rótulos e valores.

```rust
use charm_charts::BarChart;
use lipgloss::{Color, Style};

let chart = BarChart::new(&["Go", "Rust"], &[85.0, 72.0]).width(30);

println!("{}", chart.render());
//   Go ████████████████████ 85.0
// Rust █████████████████    72.0
```

`.bar_width(n)` define a espessura de cada barra: no modo horizontal, cada
barra ocupa `n` linhas (o rótulo e o valor aparecem na primeira linha do grupo);
no modo vertical, define a largura de cada coluna.

Modo vertical:

```rust
use charm_charts::BarChart;

let chart = BarChart::new(&["a", "b"], &[3.0, 7.0]).vertical().bar_width(3);
```

Gráficos verticais respeitam `.width(n)` (as colunas encolhem para caber) e
`.show_values(bool)` (os valores são exibidos acima das colunas).

### Barra de progresso

Barra de progresso horizontal com porcentagem e caracteres customizáveis.

```rust
use charm_charts::ProgressBar;
use lipgloss::{Color, Style};

let bar = ProgressBar::new(0.73)
    .width(10)
    .label("Upload")
    .fill_style(Style::new().foreground(Color("42".to_string())));

println!("{}", bar.render());
// Upload ███████░░░  73%
```

## Estilização

Todos os gráficos aceitam estilos do `lipgloss`, então se integram ao tema do
seu aplicativo:

```rust
use charm_charts::Sparkline;
use lipgloss::{Color, Style};

let primary = Color("ff79c6".to_string());
let spark = Sparkline::new(&[1.0, 2.0, 3.0]).style(Style::new().foreground(primary));
```

## Referência da API

### `Sparkline`

| Método | Descrição |
|--------|-----------|
| `Sparkline::new(&[f64])` | Cria com os pontos de dados |
| `.width(n)` | Define a largura em caracteres (limitada a 1-500) |
| `.height(n)` | Define a altura em linhas (limitada a 1-8) |
| `.style(s)` | Define o estilo do `lipgloss` |
| `.label(s)` | Define o rótulo de prefixo |
| `.render()` | Retorna a string estilizada |

### `BarChart`

| Método | Descrição |
|--------|-----------|
| `BarChart::new(&[&str], &[f64])` | Cria com rótulos e valores |
| `.width(n)` | Define a largura máxima (limitada a 1-500) |
| `.bar_width(n)` | Define a espessura das barras (limitada a 1-20) |
| `.horizontal()` / `.vertical()` | Define a orientação (a última chamada vence) |
| `.style(s)` | Define o estilo geral |
| `.bar_style(s)` / `.label_style(s)` | Definem os estilos |
| `.show_values(bool)` | Alterna a exibição dos valores |
| `.render()` | Retorna a string estilizada |

### `ProgressBar`

| Método | Descrição |
|--------|-----------|
| `ProgressBar::new(f64)` | Cria com o valor (0.0-1.0, limitado) |
| `.width(n)` | Define a largura da barra (limitada a 0-500) |
| `.fill_style(s)` / `.empty_style(s)` | Definem os estilos |
| `.label(s)` | Define o rótulo de prefixo |
| `.show_percentage(bool)` | Alterna a porcentagem |
| `.fill_char(c)` / `.empty_char(c)` | Caracteres customizados |
| `.render()` | Retorna a string estilizada |

## Notas

- Larguras e alturas inválidas são limitadas, em vez de causar pânico.
- Os rótulos são medidos pela largura de exibição (nunca por bytes), então
  letras acentuadas e caracteres CJK continuam alinhados. Rótulos que não cabem
  na coluna são encurtados com `…`, sem quebrar um caractere.

## Diferenças conhecidas em relação ao Go original

* A implementação Go entra em pânico quando um `ProgressBar` recebe um `value`
  negativo pelo campo público. Este port limita o valor a `0.0..=1.0`.
* A implementação Go entra em pânico quando um `BarChart` vertical tem um
  `bar_width` maior que a coluna que `width` deixa para ele. Este port reduz a
  barra à largura da coluna.

## Testes

O crate inclui uma suíte de testes de paridade que compara a saída renderizada
com os arquivos golden gerados pela implementação Go original:

```bash
cargo test -p charm-charts
```

## Licença

MIT
