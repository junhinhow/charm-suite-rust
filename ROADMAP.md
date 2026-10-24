# Roadmap — CharmSuite-Rust

> Última atualização: 2026-10-25
> Estado atual: **v0.2.0** | CI verde | 38 crates (~42k LOC src) | Paridade funcional completa

Este roadmap reflete o estado real do workspace após as correções de qualidade de outubro/2026. A paridade funcional com o projeto Go original foi atingida para todas as bibliotecas portadas; o repositório Go de referência foi reestruturado e não possui mais arquivos `.go` para comparação direta. O foco agora é estabilização e qualidade (Onda 3).

---

## ✅ Concluído (até v0.2.0)

- 38 crates no workspace (37 publicáveis + agregador), totalizando ~42.000 LOC de produção
- Paridade funcional completa: Onda 1 (18 libs) e Onda 2 (sincronização de API) concluídas
- Referência Go reestruturada; sem libs pendentes de port
- Testes unitários e de paridade em todos os crates
- CI Forgejo com jobs `fmt`, `clippy`, `test` e `doc` passando
- Correção de testes dependentes de ambiente (`charm-palette`, `charm-update`)
- Documentação de diferenças deliberadas (`KNOWN_DIFFERENCES.md`)

---

## 🎯 Onda 3 — Estabilização e Qualidade (v0.3.0)

**Objetivo:** Sinalizar maturidade do workspace e garantir confiabilidade para adoção externa.

| # | Tarefa | Critério de Aceite | Prioridade |
|---|--------|-------------------|------------|
| 1 | Atualizar `GAP_ANALYSIS.md` | Sincronizar LOC/status reais da Onda 2; validar lista de 19 libs pendentes | Alta |
| 2 | ~~Adicionar job Linux nativo na CI~~ | ✅ Já atendido: `.github/workflows/ci.yml` roda test/clippy/fmt/doc em `ubuntu-latest` nativo | ~~Alta~~ |
| 3 | ~~Testes de integração entre crates~~ | ✅ Suite `tests/integration/` criada com 4 testes passando (themes+layout, panes+layout, core+text, router+nav) | ~~Média~~ |
| 4 | Benchmarks basais | `benches/` em charm-layout, charm-diff, charm-fuzzy com Criterion | Média |
| 5 | Documentação API completa | `cargo doc --workspace` sem warnings; exemplos inline em todos os itens públicos | Média |
| 6 | Exemplos funcionais executáveis | 1 exemplo por crate principal rodando com `cargo run --example` | Baixa |
| 7 | Completar Onda 2 (sync API) | Sincronizar 12 libs com drift (charm-themes, charm-toast, etc.) | Alta |
| 8 | Iniciar Onda 3 (libs novas) | Portar 19 libs pendentes do Go (a11y, store, router, keys...) | Média |
| 9 | Limpeza de branches órfãs | Remover `target/agent-port-config-luna` local | Baixa |
| 10 | Tag v0.3.0 | Release notes + changelog atualizado (após Ondas 2+3) | Marco |

---

## 🔮 Onda 4 — Ecossistema e Adoção (v0.4.0+)

**Objetivo:** Tornar a suíte utilizável por terceiros e atrair contribuidores.

| # | Tarefa | Descrição |
|---|--------|-----------|
| 1 | Crate `charm-suite` como meta-crate | Re-export unificado com feature flags por módulo |
| 2 | Guia de migração Go→Rust | Documento comparativo com exemplos lado-a-lado |
| 3 | Templates de projeto | `cargo generate` template para apps Bubble Tea usando CharmSuite |
| 4 | Showcase interativo | App `charm-showcase` com navegação por todos os componentes |
| 5 | Publicação no crates.io | Política de versionamento, MSRV, metadata completa |
| 6 | Contributing guide | Padrões de código, processo de PR, setup de dev environment |
| 7 | Integração com ecossistema Charm | Compatibilidade verificada com Bubble Tea, Lip Gloss, Bubbles upstream |

---

## 🚫 Fora de Escopo

- Portar novas libs do Go além das 19 listadas no GAP_ANALYSIS.md
- Reimplementar `charm-lint` (específico de `go vet`)
- Suporte a plataformas além de Linux/macOS/Windows desktop
- Bindings para outras linguagens

---

## Princípios

1. **Qualidade > quantidade**: nenhum crate novo até que os existentes tenham testes de integração e docs completas
2. **CI como verdade**: nada mergeia sem passar em Linux nativo + Windows
3. **Paridade funcional, não sintática**: diferenças idiomáticas de Rust são bem-vindas se documentadas
4. **Release semântico**: breaking changes = major; novas features = minor; fixes = patch
5. **Documentação viva**: README, GAP_ANALYSIS e ROADMAP devem refletir o estado real, não aspirações

---

## Como Contribuir

1. Escolha uma tarefa da Onda 3
2. Abra issue discutindo a abordagem
3. PR com testes + docs + passagem na CI
4. Revisão por pelo menos 1 maintainer

---

*Este documento é atualizado a cada release ou mudança significativa no escopo.*