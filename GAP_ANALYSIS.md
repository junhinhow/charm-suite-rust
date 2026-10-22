# Gap Analysis — Go (monorepo atualizado) vs Rust (port atual)

Data da análise: 2026-10-22. Origem Go: `D:\Codes\Projetos\Debian 008\Forgejo\CharmSuite` (commit `d5152ab`).
Destino Rust: `D:\Codes\Projetos\Debian 008\Forgejo\CharmSuite-Rust` (commit `ddf40d1`).

## 1. Resumo

O repositório Go deixou de ser um índice de 18 repos separados e virou um **monorepo de 37 libs** (`libs/`), com workspace Go (`go.work`), CI unificada, scripts de bootstrap (`scripts/`) e um `AUDIT.md` com correções aplicadas.

A Rust atual cobre **22 crates** (18 libs portadas + `charm-core` + `charm-suite` + `charm-devtools-macros` + `charm-showcase`). Faltam **19 libs novas** (2 delas vazias) e a API das 18 já portadas **cresceu** (de ~4.560 LOC para ~10.797 LOC).

| Métrica | Go (atual) | Rust (atual) | Gap |
|---|---|---|---|
| Bibliotecas | 37 (35 com código) | 18 | 17 novas |
| LOC de produção | ~30.000 | ~12.000 | ~18.000 |
| LOC de teste | ~12.000 | ~3.000 | ~9.000 |
| Crates publicáveis | 35 | 18 | 17 |

## 2. Bibliotecas novas no Go (17 com código + 2 vazias + 1 Go-only)

### Portáveis para Rust (17)

| Lib | LOC | Dificuldade | Dependências Go | Equiv. Rust sugerido |
|---|---|---|---|---|
| `charm-a11y` | 378 | 🟢 Baixa | std | `charm-a11y` (trivial) |
| `charm-store` | 319 | 🟢 Baixa | std | `charm-store` (generics) |
| `charm-router` | 353 | 🟢 Baixa | bubbletea | `charm-router` |
| `charm-keys` | 741 | 🟡 Média | bubbletea | `charm-keys` |
| `charm-fuzzy` | 661 | 🟡 Média | lipgloss | `charm-fuzzy` |
| `charm-panes` | 975 | 🟡 Média | lipgloss | `charm-panes` |
| `charm-progress` | 876 | 🟡 Média | lipgloss | `charm-progress` |
| `charm-mock` | 916 | 🟡 Média | std | `charm-mock` |
| `charm-input` | 845 | 🟡 Média | lipgloss + bubbletea | `charm-input` |
| `charm-logs` | 994 | 🟡 Média | lipgloss + bubbletea | `charm-logs` |
| `charm-json` | 1113 | 🟡 Média | lipgloss + bubbletea | `charm-json` |
| `charm-command` | 1159 | 🟡 Média | lipgloss + bubbletea | `charm-command` |
| `charm-config` | 1620 | 🔴 Alta | yaml + bubbletea | `charm-config` |
| `charm-image` | 757 | 🔴 Alta | image (Go std) | `charm-image` (crate `image`) |
| `charm-record` | 636 | 🔴 Alta | reflect + bubbletea | `charm-record` |
| `charm-update` | 1524 | 🔴 Alta | net/http + archive | `charm-update` |
| `charm-a11y` | 378 | 🟢 Baixa | std | (já listado) |

### Não portáveis / fora de escopo

| Lib | LOC | Motivo |
|---|---|---|
| `charm-lint` | 1264 | É um **analyzer do `go vet`** (`golang.org/x/tools/go/analysis`). Não existe equivalente em Rust — o linting em Rust é feito por `clippy` + `rustc`. Fora de escopo. |
| `charm-cli` | 0 | Vazio (stub). |
| `charm-gallery` | 0 | Vazio (stub). |

## 3. Bibliotecas existentes — drift de API

Todas as 18 portadas cresceram. Tabela de LOC Go (antigo → novo) e nº de símbolos exportados (novo):

| Lib | LOC antes | LOC agora | Δ | Símbolos | Status Rust |
|---|---|---|---|---|---|
| `charm-diff` | 522 | 1282 | +760 | 56 | ✅ sincronizado (onda 1) |
| `charm-form` | 87 | 1078 | +991 | 64 | ✅ sincronizado (onda 1) |
| `charm-test` | 417 | 1220 | +803 | 70 | ✅ sincronizado (onda 1) |
| `charm-devtools` | 324 | 618 | +294 | 40 | ✅ sincronizado (onda 1) |
| `charm-layout` | 242 | 692 | +450 | 33 | ✅ sincronizado (onda 1) |
| `charm-i18n` | 361 | 1132 | +771 | 43 | ✅ sincronizado (onda 1) |
| `charm-themes` | 234 | 820 | +586 | 31 | ⬜ pendente |
| `charm-toast` | 381 | 534 | +153 | 39 | ⬜ pendente |
| `charm-charts` | 405 | 605 | +200 | 37 | ⬜ pendente |
| `charm-palette` | 240 | 490 | +250 | 22 | ⬜ pendente |
| `charm-nav` | 212 | 337 | +125 | 20 | ⬜ pendente |
| `charm-dialog` | 324 | 412 | +88 | 21 | ⬜ pendente |
| `charm-wizard` | 241 | 469 | +228 | 37 | ⬜ pendente |
| `charm-banner` | 316 | 395 | +79 | 12 | ⬜ pendente |
| `charm-color` | 295 | 299 | +4 | 20 | ⬜ pendente |
| `charm-spinner` | 206 | 316 | +110 | 9 | ⬜ pendente |
| `charm-notify` | 141 | 484 | +343 | 25 | ⬜ pendente |
| `charm-table-cli` | 341 | 620 | +279 | 29 | ⬜ pendente |

## 4. Convenções novas no Go (importantes para o Rust)

O `AUDIT.md` do Go estabelece convenções que **não existiam na versão antiga** e que o Rust deve respeitar quando portar:

1. **Mensagens de erro e comentários em pt-BR** (o Rust usa inglês no rustdoc — manter; mas mensagens de erro do Go são pt-BR, então a paridade de golden exige pt-BR nos erros).
2. **`lipgloss.Width` / `ansi.StringWidth`** em vez de `len()` para largura de exibição — o Rust já usa `unicode-width`; verificar que todos os crates novos façam o mesmo.
3. **Testes de regressão** para cada bug corrigido.
4. **README bilíngue sincronizado** (`README.md` en + `README.pt-br.md` pt-BR).
5. **Commits Conventional Commits** em pt-BR.

## 5. Planejamento proposto

### Onda 2 — Sincronizar as 12 libs restantes (API drift)
Prioridade: libs que cresceram mais e que são usadas por outras.
1. `charm-themes` (586 LOC de drift)
2. `charm-toast` (153)
3. `charm-charts` (200)
4. `charm-palette` (250)
5. `charm-nav` (125)
6. `charm-dialog` (88)
7. `charm-wizard` (228)
8. `charm-banner` (79)
9. `charm-color` (4)
10. `charm-spinner` (110)
11. `charm-notify` (343)
12. `charm-table-cli` (279)

Esforço estimado: ~60-80h (1-2 semanas).

### Onda 3 — Portar as 17 libs novas
Ordem por dificuldade (fáceis primeiro, para ganhar ritmo e validar o processo):
1. `charm-a11y` (378) — fácil
2. `charm-store` (319) — fácil
3. `charm-router` (353) — fácil
4. `charm-keys` (741) — média
5. `charm-fuzzy` (661) — média
6. `charm-panes` (975) — média
7. `charm-progress` (876) — média
8. `charm-mock` (916) — média
9. `charm-input` (845) — média
10. `charm-logs` (994) — média
11. `charm-json` (1113) — média
12. `charm-command` (1159) — média
13. `charm-config` (1620) — difícil
14. `charm-image` (757) — difícil (crate `image`)
15. `charm-record` (636) — difícil (reflect)
16. `charm-update` (1524) — difícil (rede + archives)

Esforço estimado: ~120-160h (3-4 semanas).

### Onda 4 — Atualizar o agregador e a documentação
- `charm-suite` (agregador) ganha features para as 17 libs novas.
- README raiz com 35 libs.
- Atualizar `ROADMAP.md` e `KNOWN_DIFFERENCES.md`.

### Total estimado
~180-240h (4-6 semanas de 1 dev full-time). Com 2 devs em paralelo: ~3 semanas.

## 6. O que NÃO vou portar

- `charm-lint` (1264 LOC) — analyzer do `go vet`; em Rust o linting é feito por `clippy` e `rustc`. Fora de escopo; se quiser, vira um clippy lint plugin (trabalho à parte).
- `charm-cli` e `charm-gallery` — vazios (stubs) na versão Go.

## 7. Subagentes indisponíveis

O provedor `opencode-go` está com limite de uso nos subagentes (erros "Go usage limit exceeded"). Até o limite resetar, o trabalho será feito diretamente nesta sessão, em lotes menores, para manter qualidade.

## 8. Decisão pendente para você

1. **Prioridade**: começar pela Onda 2 (sync das 12) ou pela Onda 3 (libs novas)? A Onda 2 garante que o Rust atual fique em dia com o Go; a Onda 3 adiciona funcionalidades novas.
2. **`charm-lint`**: confirmar que fica fora (é tooling de Go, sem equivalente direto).
3. **Onda 3 completa ou incremental?**: as 17 libs novas são muitas. Posso começar com as 4 mais fáceis (a11y, store, router, keys) e reportar, ou tentar fazer todas de uma vez.

Minha recomendação: **Onda 2 primeiro** (sincronizar o que já existe), porque o drift pode ter mudanças de API que afetam usuários, e depois Onda 3 incremental (4 libs por vez).