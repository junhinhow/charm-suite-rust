# charm-diff

Visualizador de diffs no terminal para o ecossistema [Charm](https://charm.sh). Produz diffs unificados e lado a lado com estilizacao [Lip Gloss](https://github.com/charmbracelet/lipgloss).

> **[Read in English](README.md)**

Esta e uma portacao para Rust da biblioteca Go [`github.com/junhinhow/charm-diff`](https://github.com/junhinhow/charm-diff). A saida e byte a byte identica a original (desconsiderando os codigos ANSI), o que e verificado por *golden files* gerados a partir da implementacao Go.

## Funcionalidades

- **Diff unificado** — formato classico com marcadores `+`/`-` e numeros de linha
- **Diff lado a lado** — comparacao em duas colunas para qualquer largura de terminal
- **Linhas de contexto configuraveis** — mostre quantas linhas ao redor quiser (padrao: 3)
- **Estilos predefinidos** — temas inspirados no GitHub e terminal minimalista
- **Estilos customizados** — crie o seu proprio `DiffStyle` com qualquer estilo Lip Gloss

## Instalacao

```toml
[dependencies]
charm-diff = "0.1"
```

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
use charm_diff::{unified_diff_with_options, DiffStyle, Options};

fn main() {
    let opts = Options {
        context_lines: 5,
        style: DiffStyle::github(),
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
| `Options` | Configuracao: `context_lines: usize`, `style: DiffStyle` |
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
| `Options{ContextLines: n, Style: s}` | `Options { context_lines: n, style: s }` |

## Algoritmo

O diff e calculado com o classico algoritmo de programacao dinamica de
**sequencia comum mais longa** (LCS):

1. A tabela completa `(old_len + 1) x (new_len + 1)` e preenchida com o
   tamanho da maior subsequencia comum de cada par de prefixos.
2. A tabela e percorrida de tras para frente para reconstruir a lista de
   operacoes (`Equal`, `Insert`, `Delete`).
3. A lista e invertida, porque foi construida de tras para frente.
4. As operacoes sao agrupadas em *hunks* mantendo `context_lines` linhas de
   contexto ao redor de cada mudanca; hunks sobrepostos sao mesclados.
5. O layout lado a lado agrupa uma remocao seguida de uma insercao em uma
   unica "linha modificada".

As regras de desempate da reconstrucao sao identicas as do original Go, de
modo que a **ordem das operacoes** — e portanto a saida renderizada — e a
mesma.

> **Nota sobre complexidade:** a tabela LCS usa `O(n · m)` de memoria,
> exatamente como o original Go. Isso e mantido de proposito para garantir a
> paridade bit a bit; uma variante otimizada (espaco linear) e uma melhoria
> possivel para depois da 1.0.

## Diferencas conhecidas

- **O truncamento conta bytes, e nao caracteres.** O preenchimento/ truncamento
  das colunas do layout lado a lado decide entre preencher ou truncar usando o
  comprimento em **bytes** da linha (`len(s)` no Go), e corta a string em um
  offset de byte. Esse e o comportamento do original Go e ele foi preservado
  aqui. Portanto, a implementacao Go pode cortar um caractere multibyte ao meio
  e emitir uma sequencia UTF-8 invalida; as duas implementacoes substituem os
  bytes quebrados por `U+FFFD` no momento de produzir a saida estilizada, de
  modo que o resultado renderizado e identico. Em Rust o corte e feito com
  `String::from_utf8_lossy`, que nunca devolve uma `String` invalida.
- **Os codigos ANSI sao sempre emitidos.** A implementacao Go so emite cores
  quando stdout e um TTY, enquanto o port Rust sempre renderiza as sequencias
  de escape ANSI. O harness de paridade remove os codigos ANSI dos dois lados
  antes de compara-los.
- **`Options` e `DiffStyle` sao valores proprios.** O Go tambem os passa por
  valor, mas o Rust exige `&Options` nas funcoes `*_with_options` para nao
  copiar o estilo (que pode ser caro).

## Paridade com a implementacao Go

O golden file [`tests/golden/diff.txt`](tests/golden/diff.txt) e gerado pelo
programa Go `reference/go/charm-diff/cmd/parity` e contem 20 amostras
(arquivos identicos, insercoes, remocoes, modificacoes, mudancas adjacentes e
distantes, arquivos vazios, ausencia de newline final, unicode, linhas em
branco, os valores de contexto `0`, `1`, `3` e `10`, tabs, linhas longas,
linhas repetidas, uma reescrita completa e o estilo GitHub), cada uma
renderizada como diff unificado e como diff lado a lado nas larguras 40, 80 e
120.

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
