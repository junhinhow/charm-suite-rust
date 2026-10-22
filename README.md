# CharmSuite-Rust

[![CI](https://img.shields.io/badge/CI-passing-16A34A?style=for-the-badge)](#) [![License: MIT](https://img.shields.io/badge/License-MIT-15803D?style=for-the-badge)](LICENSE) [![Rust 1.80+](https://img.shields.io/badge/Rust-1.80+-00ADD8?style=for-the-badge&logo=rust&logoColor=white)](#)

**PT:** Um ecossistema curado de bibliotecas Rust para o stack **Charmbracelet / Bubble Tea**. Cada `charm-*` resolve uma lacuna especifica que falta no conjunto oficial.
**EN:** A curated ecosystem of Rust libraries for the **Charmbracelet / Bubble Tea** stack. Each `charm-*` fills a specific gap missing from the official set.

Port Rust de [`junhinhow/charm-suite`](https://github.com/junhinhow/charm-suite) (Go).

- **Forgejo (origem):** <https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust>
- **GitHub (mirror):** <https://github.com/junhinhow/charm-suite-rust>

## As bibliotecas / The Libraries

| Library | Description (EN) | Descricao (PT) |
| --- | --- | --- |
| [**charm-wizard**](crates/charm-wizard) | Multi-step wizard framework — progress tracking, back/forward nav, shared state | Framework de wizard multi-etapas — progresso, navegacao voltar/avancar, estado compartilhado |
| [**charm-themes** ⭐](crates/charm-themes) | Unified theming for lipgloss | Sistema de temas unificado para lipgloss |
| [**charm-test** ⭐](crates/charm-test) | Testing framework for Bubble Tea — simulate inputs, snapshot views, deterministic asserts | Framework de testes para Bubble Tea — simula inputs, snapshots, asserts deterministicos |
| [**charm-spinner**](crates/charm-spinner) | 30+ beautiful terminal spinner animations | 30+ animacoes bonitas de spinner para terminal |
| [**charm-notify**](crates/charm-notify) | Cross-platform desktop notifications (Windows toast, macOS, Linux notify-send) | Notificacoes desktop cross-platform (toast Windows, macOS, notify-send Linux) |
| [**charm-nav**](crates/charm-nav) | Tab, breadcrumb and sidebar navigation styled with lipgloss | Navegacao por abas, breadcrumb e sidebar com lipgloss |
| [**charm-layout**](crates/charm-layout) | Flexbox-inspired layout engine — responsive flex and grid | Motor de layout inspirado em Flexbox — flex e grid responsivos |
| [**charm-i18n**](crates/charm-i18n) | Internationalization — message catalogs, locale detection | Internacionalizacao — catalogos de mensagens, deteccao de locale |
| [**charm-form**](crates/charm-form) | Declarative form generator — define forms in YAML, get interactive CLI apps | Gerador declarativo de formularios — defina em YAML, ganhe CLI interativa |
| [**charm-diff**](crates/charm-diff) | Beautiful terminal diff viewer — unified and side-by-side with lipgloss | Visualizador de diff bonito — unificado e lado a lado com lipgloss |
| [**charm-devtools**](crates/charm-devtools) | DevTools inspector for Bubble Tea — message log, state viewer, component tree | Inspetor DevTools para Bubble Tea — log de mensagens, visualizador de estado, arvore de componentes |
| [**charm-color**](crates/charm-color) | Color utilities — darken, lighten, blend, contrast | Utilitarios de cor — escurecer, clarear, misturar, contraste |
| [**charm-charts**](crates/charm-charts) | Terminal charts — sparklines, bar charts, progress bars with lipgloss | Graficos de terminal — sparklines, barras, progresso com lipgloss |
| [**charm-banner**](crates/charm-banner) | ASCII art banner generator styled with lipgloss | Gerador de banners ASCII art com lipgloss |
| [**charm-toast**](crates/charm-toast) | Toast notification component for Bubble Tea — auto-dismissing status messages | Componente de toast para Bubble Tea — mensagens de status auto-dismiss |
| [**charm-table-cli**](crates/charm-table-cli) | CLI to render CSV/JSON/TSV as beautiful styled terminal tables | CLI para renderizar CSV/JSON/TSV como tabelas estilizadas no terminal |
| [**charm-palette**](crates/charm-palette) | Terminal color palette viewer and generator — preview themes live | Visualizador e gerador de paletas de cor — preview de temas ao vivo |
| [**charm-dialog**](crates/charm-dialog) | Modal dialogs for Bubble Tea — confirm, alert, input, custom | Dialogos modais para Bubble Tea — confirmar, alerta, input, customizado |

## Quick Start / Comecando Rapido

```bash
# Rust
cargo add charm-color
cargo add charm-layout
cargo add charm-diff

# Ou use o agregador completo
cargo add charm-suite
```

Cada crate tem seu proprio `README.md` com um exemplo minimo funcional.
Each crate has its own `README.md` with a minimal working example.

## Build & Test

```bash
cargo build --workspace
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo fmt --all --check
cargo doc --workspace --no-deps
```

## Why this suite? / Por que esta suite?

**PT:** O stack oficial Charmbracelet e excelente, mas deixa lacunas: nao ha wizard pronto, nem sistema de temas unificado, nem framework de testes dedicado, nem DevTools, nem i18n, nem layout responsivo. A Charm Suite preenche cada uma dessas lacunas com bibliotecas pequenas, focadas e interoperaveis — todas portadas para Rust idiomático, estilizadas com `lipgloss`.

**EN:** The official Charmbracelet stack is excellent but leaves gaps: no ready-made wizard, no unified theming, no dedicated test framework, no DevTools, no i18n, no responsive layout. The Charm Suite fills each of those gaps with small, focused, interoperable libraries — all ported to idiomatic Rust, all styled with `lipgloss`.

## Contributing / Contribuindo

Issues and PRs are welcome on any `charm-*` crate. Please keep changes small, idiomatic, and covered by tests where possible. / Issues e PRs sao bem-vindos em qualquer crate `charm-*`. Mantenha mudancas pequenas, idiomaticas e cobertas por testes quando possivel.

## License / Licenca

MIT — see [LICENSE](LICENSE).
