# charm-themes

Sistema de temas unificado para aplicacoes de terminal.

Defina uma paleta de cores uma vez, use em todos os lugares — lipgloss, bubbles
e qualquer outro componente do seu TUI.

> **[Read in English](README.md)**

## O Problema

Cada sistema de estilos pede as mesmas cores de novo:

- **lipgloss** usa `lipgloss::Style` com cores inline
- **bubbles** cada componente tem seu proprio struct `Styles`
- Renderizadores de Markdown usam seu proprio `StyleConfig`

Ao construir um app TUI, voce acaba definindo as mesmas cores tres ou mais
vezes em formatos diferentes. Mudar o tema significa atualizar cada definicao
de estilo.

## A Solucao

```rust
use charm_themes::themes;

// Escolha um tema
let theme = themes::dracula();

// Use com lipgloss — estilos semanticos prontos
let styles = theme.lipgloss();
println!("{}", styles.title.render("Ola"));
println!("{}", styles.error.render("Algo deu errado"));
println!("{}", styles.border.render("Conteudo em caixa"));

// Ou use as cores diretamente em estilos customizados
let custom = lipgloss::Style::new()
    .foreground(theme.primary.to_lipgloss())
    .background(theme.background.to_lipgloss());
println!("{}", custom.render("Estilo customizado"));
```

## Temas Inclusos

| Tema | Background | Primary | Accent |
|---|---|---|---|
| **Gruvbox** | `#282828` | `#fabd2f` | `#83a598` |
| **Dracula** | `#282a36` | `#bd93f9` | `#ff79c6` |
| **Nord** | `#2e3440` | `#88c0d0` | `#b48ead` |
| **Catppuccin** | `#1e1e2e` | `#cba6f7` | `#f5c2e7` |
| **Tokyo Night** | `#1a1b26` | `#7aa2f7` | `#bb9af7` |

Todos os temas tambem sao acessiveis pelo nome:

```rust
use charm_themes::themes;

let all = themes::themes();
assert_eq!(all.len(), 5);
assert!(all.contains_key("tokyo-night"));
```

## Estrutura do Tema

```rust
pub struct Theme {
    pub name: String,
    pub background: Rgb,   // fundo principal
    pub foreground: Rgb,   // texto principal
    pub primary: Rgb,      // titulos, headings
    pub secondary: Rgb,    // subtitulos, info secundaria
    pub accent: Rgb,       // destaques, enfase
    pub success: Rgb,      // verde / positivo
    pub warning: Rgb,      // amarelo / cuidado
    pub error: Rgb,        // vermelho / negativo
    pub info: Rgb,         // azul / informativo
    pub muted: Rgb,        // cinza / desabilitado
    pub syntax: SyntaxColors, // cores de syntax highlighting
}
```

`Rgb` vem do [`charm-core`](https://crates.io/crates/charm-core) e e o tipo de
cor canonico da Charm Suite. A conversao para `lipgloss::Color` e feita por
`to_lipgloss()` e `from_lipgloss_str()`.

## Estilos Lipgloss Pre-Construidos

```rust
let styles = charm_themes::themes::gruvbox().lipgloss();

styles.base;        // texto normal
styles.bold;        // negrito
styles.italic;      // italico
styles.faint;       // desbotado

styles.success;     // mensagem de sucesso
styles.warning;     // aviso
styles.error;       // erro
styles.info;        // informacao

styles.title;       // titulo (primary, bold)
styles.subtitle;    // subtitulo (secondary)
styles.highlight;   // destaque (accent, bold)
styles.border;      // caixa com borda arredondada
styles.code;        // codigo inline
```

## Cores de Syntax

Cada tema tambem carrega uma paleta para realce de codigo:

```rust
use charm_themes::themes;

let syntax = themes::nord().syntax;
assert_eq!(syntax.keyword.to_hex(), "#81a1c1");
assert_eq!(syntax.string.to_hex(), "#a3be8c");
```

## Paridade com o Go original

As cores **nunca sao digitadas a mao**. elas sao extraidas programaticamente
do codigo Go original (`reference/go/charm-themes/themes.go`) pelo script
`scripts/extract-themes.ps1`, e o teste de paridade compara a saida Rust com
um golden file gerado pela implementacao Go:

```text
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-themes/tests/golden/themes.txt -- \
    go run ./reference/go/charm-themes/cmd/parity
```

## Licenca

MIT
