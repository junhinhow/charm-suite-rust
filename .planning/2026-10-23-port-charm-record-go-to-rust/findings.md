# Findings & Decisions

## Requirements
- Pedido: portar integralmente a referência `reference/go/charm-record`, inclusive arquivos Go, READMEs e testes, para `crates/charm-record`; sem commit/push e build no target dedicado.
- Referência lida: `cassette.go`, `codec.go`, `recorder.go`, `replay.go`, `doc.go`, `record_test.go`, `codec_test.go`, `example_test.go`, ambos READMEs, `go.mod` e `go.sum`.
- Workspace usa `charmed-bubbletea` 0.2.4 (API distinta do Go Bubble Tea v2). `Message` é `Box<dyn Any + Send>`, sem `Clone` nem acesso ao `TypeId`; `Model::update(&mut self, Message) -> Option<Cmd>` e `Cmd` é one-shot `FnOnce`.
- Mensagens embutidas Rust expostas relevantes: `WindowSizeMsg { width: u16, height: u16 }`, `KeyMsg { key_type, runes, alt, paste }`, `QuitMsg`, `InterruptMsg`, `SuspendMsg`, `ResumeMsg`, `FocusMsg`, `BlurMsg`; não há KeyReleaseMsg no runtime.
- Tratamento embutido implementado via `update_resize`, `update_key` e `update_signal`; o payload de tecla preserva todo enum `KeyType` via discriminante estável desta versão local, runes, alt e paste.
- API adaptada: `Recorder::update<T: RecordMessage>` toma mensagem tipada para serializar antes de encaminhar; `register_msg::<T>()` exige nome estável `RecordMessage::TYPE_NAME`. `update_key` e `update_resize` tratam os tipos embutidos dedicadamente. Mensagens não `RecordMessage` podem ser encaminhadas sem gravação via `update_unrecorded`.
- Comandos permanecem retornados sem execução. `record_command` grava evento nomeado apenas para interoperabilidade com cassetes externos, replay o ignora com warning.
- Diferenças de Rust/serde e nomes estáveis de tipos precisam constar nos READMEs e `KNOWN_DIFFERENCES.md`.

## Research Findings
-

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Não tentar introspecção dinâmica | O Bubble Tea Rust apaga o tipo concreto. O registro precisa ser explicitamente usado antes da perda de tipo; a assinatura tipada torna isso verificável em compilação. |

## Issues Encountered
| Issue | Resolution |
|-------|------------|

## Resources
-
