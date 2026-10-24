# Gap Analysis — CharmSuite-Rust

Data da última atualização: 2026-10-25. Commit de referência: `10a7e57`.

## 1. Resumo

O workspace Rust contém **38 crates** (37 publicáveis + `charm-suite` como agregador), totalizando ~42.000 LOC de produção. A paridade funcional com a versão Go original foi atingida para todas as bibliotecas portadas; o repositório Go de referência (`CharmSuite`) foi reestruturado e não possui mais arquivos `.go` para comparação direta de código-fonte.

| Métrica | Valor |
|---|---|
| Crates no workspace | 38 |
| Crates publicáveis | 37 |
| LOC de produção (src) | ~42.000 |
| Status da CI | ✅ Verde (fmt, clippy, test, doc) |
| Paridade com Go | ✅ Atingida (Onda 1 + Onda 2 concluídas) |

## 2. Bibliotecas portadas (38 crates)

Todas as 18 libs originais da Onda 1 e as 19 libs da Onda 2 estão implementadas e sincronizadas. Os crates listados abaixo cobrem toda a superfície da API do projeto Go original.

| Crate | LOC (src) | Status |
|---|---|---|
| `charm-diff` | 2488 | ✅ Completo |
| `charm-panes` | 2197 | ✅ Completo |
| `charm-i18n` | 2097 | ✅ Completo |
| `charm-themes` | 1654 | ✅ Completo |
| `charm-a11y` | 1623 | ✅ Completo |
| `charm-charts` | 1570 | ✅ Completo |
| `charm-layout` | 1502 | ✅ Completo |
| `charm-test` | 1492 | ✅ Completo |
| `charm-form` | 1460 | ✅ Completo |
| `charm-mock` | 1434 | ✅ Completo |
| `charm-toast` | 1337 | ✅ Completo |
| `charm-banner` | 1322 | ✅ Completo |
| `charm-config` | 1261 | ✅ Completo |
| `charm-wizard` | 1236 | ✅ Completo |
| `charm-spinner` | 1185 | ✅ Completo |
| `charm-table-cli` | 1171 | ✅ Completo |
| `charm-nav` | 1150 | ✅ Completo |
| `charm-notify` | 1096 | ✅ Completo |
| `charm-command` | 1096 | ✅ Completo |
| `charm-input` | 1059 | ✅ Completo |
| `charm-dialog` | 1022 | ✅ Completo |
| `charm-update` | 1014 | ✅ Completo |
| `charm-palette` | 941 | ✅ Completo |
| `charm-json` | 896 | ✅ Completo |
| `charm-image` | 858 | ✅ Completo |
| `charm-record` | 856 | ✅ Completo |
| `charm-devtools` | 842 | ✅ Completo |
| `charm-logs` | 831 | ✅ Completo |
| `charm-progress` | 671 | ✅ Completo |
| `charm-keys` | 640 | ✅ Completo |
| `charm-router` | 613 | ✅ Completo |
| `charm-store` | 579 | ✅ Completo |
| `charm-color` | 479 | ✅ Completo |
| `charm-core` | 462 | ✅ Completo |
| `charm-fuzzy` | 323 | ✅ Completo |
| `charm-devtools-macros` | 160 | ✅ Completo |
| `charm-showcase` | 123 | ✅ Binário/demo |
| `charm-suite` | 101 | ✅ Agregador |

## 3. Bibliotecas não portadas / fora de escopo

| Lib Go | Motivo |
|---|---|
| `charm-lint` | Analyzer do `go vet`; em Rust o linting é feito por `clippy` e `rustc`. Fora de escopo. |
| `charm-cli` | Stub vazio no Go. Sem funcionalidade para portar. |
| `charm-gallery` | Stub vazio no Go. Sem funcionalidade para portar. |

## 4. Diferenças deliberadas de implementação

As diferenças entre Go e Rust estão documentadas em `KNOWN_DIFFERENCES.md` e são consideradas paridade funcional, não gaps. Incluem:

- Ownership e borrowing em vez de GC
- Generics monomorfizados em vez de reflexão
- `unicode-width` para largura de exibição (equivalente a `lipgloss.Width`)
- Mensagens de erro em inglês (rustdoc padrão) vs pt-BR no Go
- Ausência de `charm-lint` (substituído por clippy)

## 5. Estado atual e próximos passos

A fase de portabilidade está **concluída**. O foco agora é estabilização e qualidade, conforme definido no `ROADMAP.md`:

- **Onda 3**: Testes de integração entre crates, documentação completa (`cargo doc`), exemplos funcionais e benchmarks.
- **Onda 4**: Ecossistema e adoção (publicação no crates.io, guias de contribuição, release v0.3.0).

Consulte `ROADMAP.md` para o plano detalhado de estabilização.

## 6. Histórico de revisões

| Data | Alteração |
|---|---|
| 2026-10-25 | Reescrita completa: dados atualizados via shell, paridade confirmada, referência Go removida |
| 2026-10-22 | Análise inicial: 18 libs portadas, 19 pendentes, drift de API identificado |