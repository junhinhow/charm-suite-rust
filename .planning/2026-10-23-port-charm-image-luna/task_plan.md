# Plano: porte de charm-image

## Goal
Portar integralmente a referência `reference/go/charm-image` para `crates/charm-image`, sem editar outros crates nem o manifest raiz, e validar no diretório `target/agent-port-image-luna`.

## Fases
1. **Inspecionar a referência e padrões do workspace** — Status: complete
2. **Implementar crate, documentação, exemplo e testes/golden** — Status: complete
3. **Executar fmt, test, clippy e doc; registrar limitações** — Status: complete

## Next Step
Entrega concluída; validações e limitações registradas abaixo.

## Errors Encountered
| Erro | Tentativa | Resolução |
|---|---:|---|
| `go test ./...` tentou usar o go.work da raiz, que não lista o módulo de referência | 1 | Executado `GOWORK=off go test ./...` no módulo; passou |
| Clippy identificou `repeat().take()`, argumento por valor e conversão de rampa | 1 | Usado `repeat_n`, `Options: Copy` e conversão checked |
| Edição de teste deixou delimitador inválido | 1 | Corrigido o fechamento do closure antes de continuar validação |
