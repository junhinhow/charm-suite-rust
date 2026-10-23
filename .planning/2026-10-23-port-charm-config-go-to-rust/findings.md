# Findings & Decisions

## Requirements
- Criar `crates/charm-config` sem alterar crates existentes; dependências de workspace apenas se inevitável.
- Portar Loader/Source, merge profundo/cópia, variáveis de ambiente, JSON/YAML, conversões tipadas, Schema/Field, normalização e editor TUI Bubble Tea com save JSON/YAML.
- Portar todos os casos de `loader_test.go`, `schema_test.go`, `editor_test.go` e `example_test.go`; criar README(s), CHANGELOG, exemplo e golden.
- A referência não tem parsing TOML; logo não adicionar TOML como extensão fora do contrato Go.
- Sem commit/push. Branch criada: `target/agent-port-config-luna`.

## Research Findings
- Arquivos Go lidos integralmente: `editor.go`, `loader.go`, `merge.go`, `schema.go`, `values.go`; testes e example lidos integralmente; ambos READMEs lidos.
- Formatos da referência: `.json`, `.yaml`, `.yml`; JSON/YAML top-level precisam ser objetos não vazios; rejeitar conteúdo vazio e documentos/trailing content adicionais.
- Prioridades: defaults 0, file 100, env 200, flags 300; empate estável por ordem de inserção.
- Editor captura snapshot; default de Field entra no Editor, mas não em Loader nem em validação; valores convertíveis são normalizados; salva no arquivo de maior prioridade, modo 0600.
- Repo já inclui charmed-bubbletea, lipgloss, serde, serde_json, serde_yaml_ng no workspace. Toml não está listado nem na implementação de referência.
- Alterações preexistentes não relacionadas: Cargo.lock, crates/charm-mock/tests/go_port.rs, crates/charm-mock/tests/parity.rs, untracked crates/charm-json/ e crates/charm-logs/; manter intocadas.

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| serde_json::Value / Map | Mapa determinístico, adequado para JSON/YAML e configurações dinâmicas |
| Somente JSON/YAML | Únicos formatos existentes na referência |

## Diferenças/Ressalvas do porte
- `Editor::values()` usa `serde_json::Value`; durações normalizadas são mantidas como texto canônico (p.ex. `"1m30s"`) porque JSON não possui escalar duration. O getter `Loader::duration` expõe `Duration` assinado em nanossegundos.
- Os tipos inteiros dinâmicos usam `i64` (limite fixo Rust), enquanto o Go usa `int` dependente da arquitetura e aceita vários tipos de slices/maps; a árvore Rust é `serde_json::Value`.
- API idiomática Rust toma posse de `Loader` em construtores/builder e `Schema::validate` devolve `Vec<String>` em vez de `[]error` Go.
- Nenhum TOML foi adicionado: não aparece nos arquivos, extensões, READMEs nem testes da referência.
- Não houve alteração em outros crates nem dependências de workspace. O workspace já tinha dependências necessárias.

## Issues Encountered
| Issue | Resolution |
|-------|------------|

## Resources
-
