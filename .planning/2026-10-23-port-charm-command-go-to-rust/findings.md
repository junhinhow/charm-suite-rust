# Findings & Decisions

## Requirements
- Portar cada fonte Go (command/filter/theme/update/view), ambos READMEs e testes/exemplo; os testes Go foram lidos integralmente.
- Implementar Command/Palette/options, fuzzy subsequence insensitive a caixa/acentos, lifecycle/selection, teclado e render overlay Bubble Tea + lipgloss.
- Entregar manifest, README em inglês e pt-BR, CHANGELOG, exemplo, testes e golden dentro do crate.
- Rodar test, clippy, fmt e doc usando alvo `target/agent-port-command-luna`.
- Não alterar outros crates nem Cargo.toml raiz; não fazer commit/push. Git já mostrava alterações prévias em `Cargo.lock` e testes de `charm-mock`, que devem permanecer intocadas.

## Research Findings
- Fontes lidas: `command.go`, `filter.go`, `theme.go`, `update.go`, `view.go`, `command_test.go` (714 linhas), `example_test.go` e ambos READMEs.
- Defaults Go: 80x24, max 8 visíveis, box até 64 colunas, título `Commands`, placeholder `Type a command…`.
- API Go usa closure em `Run`; em Rust a equivalência será `Arc<dyn Fn() -> Option<Cmd> + Send + Sync>` e state compartilhado em `Arc<Mutex<_>>`, necessário para `Model` por valor.
- Workspace já define os deps charmed-bubbletea 0.2, lipgloss 0.1 e unicode-width 0.2; crates/* são membros automáticos.
- A dependência de bubbletea/lipgloss pode ser usada via declarations locais sem modificar manifest raiz.

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Fuzzy auto-contido | Não depender de charm-fuzzy; reproduzir pesos, desempates e folding latinos exatos. |
| Compartilhar estado com `Arc<Mutex>` | Conservar valor zero e copiar modelo Bubble Tea sem cópias divergentes do estado interno. |

## Issues Encountered
| Issue | Resolution |
|-------|------------|

## Resources
-
