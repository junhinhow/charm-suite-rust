# Task Plan: Porte de charm-config para Rust

## Goal
Portar reference/go/charm-config integralmente para crates/charm-config na branch target/agent-port-config-luna.

## Next Step
Entregar o porte e registrar diferenças da representação Rust.

## Current Phase
Phase 1

## Phases

### Phase 1: Requirements & Discovery
- [x] Ler integralmente Go, READMEs, testes e exemplo da referência
- [x] Conferir convenções Rust do workspace e estado existente
- [x] Documentar restrições e diferenças esperadas
- **Status:** complete

### Phase 2: Planning & Structure
- [x] Definir representação e dependências
- [x] Criar estrutura do crate dentro do escopo
- **Status:** complete

### Phase 3: Implementation
- [x] Implementar Loader, merge, conversões, Schema e Editor Bubble Tea
- [x] Adicionar documentação, exemplo, testes portados e golden
- **Status:** complete

### Phase 4: Testing & Verification
- [x] Rodar test, clippy, fmt e doc
- [x] Registrar diferenças e resultados
- **Status:** complete

### Phase 5: Delivery
- [x] Revisar escopo e alterações
- [x] Reportar resultado
- **Status:** complete

## Test Results
- `cargo test -p charm-config`: aprovado (16 testes de integração + 1 doctest).
- `cargo clippy -p charm-config --all-targets -- -D warnings`: aprovado.
- `cargo fmt --check --package charm-config`: aprovado.
- `cargo doc -p charm-config --no-deps`: aprovado.
- Aviso transitivo do Cargo: `proc-macro-error2 v2.0.1` poderá ser rejeitado por uma futura versão Rust; fora do escopo/dependências deste crate.

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Usar serde_json::Value como árvore pública dinâmica e serde_json::Map ordenado | Facilita merge profundo, cópias, normalização e serialização determinística com serde |
| Somente JSON/YAML no carregamento e save | São os únicos formatos presentes na implementação Go e nos READMEs; TOML não integra a referência |
| Dependências já declaradas no workspace serão referenciadas via workspace | Evita mudanças desnecessárias na raiz |

## Errors Encountered
| Error | Resolution |
|-------|------------|
