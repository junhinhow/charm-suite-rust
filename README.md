# CharmSuite-Rust

<!-- MSRV: 1.88 (workspace.package.rust-version). Atualize este badge se a MSRV mudar. -->
[![CI](https://github.com/junhinhow/charm-suite-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/junhinhow/charm-suite-rust/actions/workflows/ci.yml) [![License: MIT](https://img.shields.io/badge/License-MIT-15803D?style=for-the-badge)](LICENSE) [![Rust 1.88+](https://img.shields.io/badge/Rust-1.88+-00ADD8?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/) [![Forgejo](https://img.shields.io/badge/Forgejo-repo-FB923C?style=for-the-badge)](https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust)

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

## Installation / Instalação

> **Ainda não publicado no crates.io.** A publicação está planejada, mas sem data definida. Por enquanto, instale via git. / **Not yet published on crates.io.** Publishing is planned, but there is no date yet. For now, install from git.

Os crates são um monorepo: cada um fica em `crates/<nome>/`. Adicione a biblioteca desejada ao `Cargo.toml` do seu projeto. / The crates live in a monorepo: each one is in `crates/<name>/`. Add the library you need to your project's `Cargo.toml`.

```toml
# Cargo.toml
[dependencies]
charm-color = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

- **`tag = "v0.1.0"`** fixa uma versão estável. Para acompanhar o desenvolvimento, use `branch = "main"` no lugar de `tag`. / **`tag = "v0.1.0"`** pins a stable release. To follow development, use `branch = "main"` instead of `tag`.
- **Espelho no GitHub / GitHub mirror:** `https://github.com/junhinhow/charm-suite-rust` funciona igual: basta trocar a URL. / works the same way: just swap the URL.

Exemplo rápido / Quick example:

```rust
use charm_color::{darken, Rgb};

fn main() {
    let red = Rgb::rgb(255, 0, 0);
    println!("{}", darken(red, 0.3).to_hex());
}
```

Instalação do binário `charm-table` (CLI) / Installing the `charm-table` binary (CLI):

```bash
cargo install --git https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust charm-table-cli
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

**MSRV (Minimum Supported Rust Version): 1.88.** Declared in `workspace.package.rust-version` and checked by the `msrv` CI job. It is driven by a transitive dependency (`darling` 0.24) that requires Rust 1.88. / **MSRV (versao minima do Rust suportada): 1.88.** Declarada em `workspace.package.rust-version` e verificada pelo job `msrv` do CI. Ela e determinada por uma dependencia transitiva (`darling` 0.24) que exige Rust 1.88.

## Why this suite? / Por que esta suite?

**PT:** O stack oficial Charmbracelet e excelente, mas deixa lacunas: nao ha wizard pronto, nem sistema de temas unificado, nem framework de testes dedicado, nem DevTools, nem i18n, nem layout responsivo. A Charm Suite preenche cada uma dessas lacunas com bibliotecas pequenas, focadas e interoperaveis — todas portadas para Rust idiomático, estilizadas com `lipgloss`.

**EN:** The official Charmbracelet stack is excellent but leaves gaps: no ready-made wizard, no unified theming, no dedicated test framework, no DevTools, no i18n, no responsive layout. The Charm Suite fills each of those gaps with small, focused, interoperable libraries — all ported to idiomatic Rust, all styled with `lipgloss`.

## Contributing / Contribuindo

> **⚠️ Mirror somente-leitura / Read-only mirror**
>
> **EN:** The Forgejo repository is the source of truth. The GitHub repository is a **read-only push mirror**: issues and pull requests opened there are **not** seen by the maintainers. Please open issues and PRs on Forgejo: <https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust>
>
> **PT:** O repositório Forgejo é a origem. O GitHub é um **espelho somente-leitura**: issues e PRs abertos lá **não chegam** aos mantenedores. Abra issues e PRs no Forgejo: <https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust>

Issues and PRs are welcome on any `charm-*` crate. Please keep changes small, idiomatic, and covered by tests where possible. / Issues e PRs sao bem-vindos em qualquer crate `charm-*`. Mantenha mudancas pequenas, idiomaticas e cobertas por testes quando possivel.

- [CONTRIBUTING.md](CONTRIBUTING.md): build, testes, Conventional Commits, fluxo de PR, golden files e release. / build, tests, Conventional Commits, PR flow, golden files and release.
- [KNOWN_DIFFERENCES.md](KNOWN_DIFFERENCES.md): diferenças conhecidas em relação ao Go original. / known differences from the Go original.
- [SECURITY.md](SECURITY.md): como reportar vulnerabilidades. / how to report vulnerabilities.

## License / Licenca

MIT — see [LICENSE](LICENSE).
