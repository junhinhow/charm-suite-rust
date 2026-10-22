# charm-diff

Visualizador de diffs no terminal para o ecossistema [Charm](https://charm.sh). Produz diffs unificados e lado a lado com estilizacao [Lip Gloss](https://github.com/charmbracelet/lipgloss).

> **[Read in English](README.md)**

Esta e uma portacao para Rust da biblioteca Go [`github.com/junhinhow/charm-diff`](https://github.com/junhinhow/charm-diff). O motor padrao reproduz o comportamento do `xdiff` do Git, incluindo o alinhamento das edicoes e a construcao dos hunks; a paridade e verificada com *golden files* gerados pelo Go e comparacoes aleatorias com `git diff`.

## Funcionalidades

- **Diff unificado** — formato classico com marcadores `+`/`-` e numeros de linha
- **Hunks compativeis com o Git** — intervalos, contexto e ordem das edicoes seguem `git diff -U<N>`
- **Diff lado a lado** — comparacao em duas colunas para qualquer largura de terminal
- **Linhas de contexto configuraveis** — mostre quantas linhas ao redor quiser (padrao: 3)
- **Memoria auxiliar linear** — o motor xdiff padrao lida com entradas grandes sem uma tabela LCS
- **Estilos predefinidos** — temas inspirados no GitHub e terminal minimalista
- **Estilos customizados** — crie o seu proprio `DiffStyle` com qualquer estilo Lip Gloss

## Instalacao

```toml
[dependencies]
charm-diff = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

## Uso

### Diff Unificado

```rust
use charm_diff::unified_diff;

fn main() {
    let old = "fn hello() {\n    println!(\"hello\");\n}\n";
    let new = "fn hello() {\n    println!(\"hello, world!\");\n}\n";

    print!("{}", unified_diff(old, new));
}
```

### Diff Lado a Lado

```rust
use charm_diff::side_by_side_diff;

fn main() {
    let old = "fn hello() {\n    println!(\"hello\");\n}\n";
    let new = "fn hello() {\n    println!(\"hello, world!\");\n}\n";

    print!("{}", side_by_side_diff(old, new, 120));
}
```

### Opcoes Personalizadas

```rust
use charm_diff::{unified_diff_with_options, Algorithm, DiffStyle, Options};

fn main() {
    let opts = Options {
        context_lines: 5,
        style: DiffStyle::github(),
        algorithm: Algorithm::Xdiff,
    };

    print!("{}", unified_diff_with_options("a\nb\n", "a\nc\n", &opts));
}
```

Ha um exemplo executavel em [`examples/demo.rs`](examples/demo.rs):

```bash
cargo run -p charm-diff --example demo
```

## Referencia da API

### Funcoes

| Funcao | Descricao |
|--------|-----------|
| `unified_diff(old: &str, new: &str) -> String` | Diff unificado com opcoes padrao |
| `unified_diff_with_options(old: &str, new: &str, opts: &Options) -> String` | Diff unificado com opcoes customizadas |
| `side_by_side_diff(old: &str, new: &str, width: usize) -> String` | Diff lado a lado com opcoes padrao |
| `side_by_side_diff_with_options(old: &str, new: &str, width: usize, opts: &Options) -> String` | Diff lado a lado com opcoes customizadas |

### Tipos

| Tipo | Descricao |
|------|-----------|
| `Options` | Configuracao: `context_lines: usize`, `style: DiffStyle`, `algorithm: Algorithm` |
| `Algorithm` | `Xdiff` (padrao, paridade Git/Go), `Lcs` ou `Myers` |
| `DiffStyle` | Estilos Lip Gloss para cada elemento do diff |

### Construtores de Estilo

| Construtor | Descricao |
|------------|-----------|
| `DiffStyle::default()` | Retorna `DiffStyle::terminal()` |
| `DiffStyle::terminal()` | Cores ANSI basicas |
| `DiffStyle::github()` | Fundos coloridos estilo GitHub |

### Mapeamento Go → Rust

| Go | Rust |
|----|------|
| `DefaultOptions()` | `Options::default()` |
| `UnifiedDiff(old, new)` | `unified_diff(old, new)` |
| `UnifiedDiffWithOptions(old, new, opts)` | `unified_diff_with_options(old, new, &opts)` |
| `SideBySideDiff(old, new, width)` | `side_by_side_diff(old, new, width)` |
| `SideBySideDiffWithOptions(old, new, width, opts)` | `side_by_side_diff_with_options(old, new, width, &opts)` |
| `DefaultStyle()` | `DiffStyle::default()` |
| `TerminalStyle()` | `DiffStyle::terminal()` |
| `GitHubStyle()` | `DiffStyle::github()` |
| `Options{ContextLines: n, Style: s}` | `Options { context_lines: n, style: s, algorithm: Algorithm::Xdiff }` |

## Algoritmos

`Options` mantem um seletor `algorithm` especifico do Rust; a API Go nao possui esse campo. O padrao e `Algorithm::Xdiff`, o motor usado pela biblioteca Go atualizada:

- **`Algorithm::Xdiff` (padrao):** usa o motor Myers de espaco linear do Git,
  incluindo limpeza de registros e compactacao de mudancas. O layout dos hunks
  unificados corresponde ao `git diff -U<N>` e a implementacao Go.
- **`Algorithm::Lcs`:** usa a tabela classica de LCS por programacao dinamica,
  com memoria `O(n · m)`. Mantido por compatibilidade com versoes Rust anteriores.
- **`Algorithm::Myers`:** usa o algoritmo middle-snake de espaco linear e
  calcula uma sequencia minima de edicoes, sem as regras de limpeza e
  compactacao do xdiff.

`Lcs` e `Myers` podem escolher alinhamentos validos diferentes do Git quando
linhas repetidas permitem varias sequencias; use o `Xdiff` padrao quando a
paridade com Go/Git for importante.

As operacoes sao entao agrupadas em *hunks* conforme `context_lines`; o layout
lado a lado combina uma remocao seguida de uma insercao em uma unica "linha
modificada".

## Diferencas conhecidas

- **`context_lines` e sem sinal.** O Rust usa `usize`, portanto valores
  negativos nao podem ser representados; use `0` para obter o comportamento Go
  de normalizar contexto negativo para nenhum contexto.
- **O seletor de algoritmo e especifico do Rust.** O Go sempre usa xdiff. As
  alternativas Rust `Lcs` e `Myers` sao mantidas por compatibilidade e podem
  renderizar outro alinhamento valido em linhas repetidas ambiguas.
- **Os codigos ANSI sao sempre emitidos.** A implementacao Go so emite cores
  quando stdout e um TTY, enquanto o port Rust sempre renderiza as sequencias
  de escape ANSI. O harness de paridade remove os codigos ANSI dos dois lados
  antes de compara-los.
- **`Options` e `DiffStyle` sao valores proprios.** O Go tambem os passa por
  valor, mas o Rust exige `&Options` nas funcoes `*_with_options` para nao
  copiar o estilo (que pode ser caro).

## Paridade com a implementacao Go

O golden file [`tests/golden/diff.txt`](tests/golden/diff.txt) e gerado pelo
programa Go `reference/go/charm-diff/cmd/parity` e contem 20 amostras regulares
(arquivos identicos, insercoes, remocoes, modificacoes, mudancas adjacentes e
distantes, arquivos vazios, ausencia de newline final, unicode, linhas em
branco, os valores de contexto `0`, `1`, `3` e `10`, tabs, linhas longas,
linhas repetidas, uma reescrita completa e o estilo GitHub), cada uma
renderizada como diff unificado e como diff lado a lado nas larguras 15, 40, 80
e 120. Um caso com 10.000 linhas tambem verifica a largura dinamica de cinco
digitos. Os testes Rust tambem comparam 200 diffs unificados aleatorios e
deterministicos com o Git.

Regere com:

```bash
cargo run -p charm-parity --bin parity -- gen crates/charm-diff/tests/golden/diff.txt -- go run ./reference/go/charm-diff/cmd/parity
```

E verifique com:

```bash
cargo test -p charm-diff
```

## Licenca

MIT
