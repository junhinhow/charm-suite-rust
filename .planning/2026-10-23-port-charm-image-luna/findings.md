# Descobertas

- A branch atual é `main`; já existem modificações não relacionadas (Cargo.lock, testes de charm-mock, novos charm-json/charm-logs e plano de outro trabalho). Preservá-las.
- Workspace inclui automaticamente `crates/*` via glob e depende de Rust 1.88; não alterar Cargo.toml raiz.
- A referência local está em `reference/go/charm-image` e contém fontes, testes Go, dois READMEs e protocolo Sixel além de iTerm2/Kitty.
- `target/agent-port-image-luna` interpretado como diretório de build dedicado via `CARGO_TARGET_DIR`, não como crate/caminho de código.
- A referência completa foi inspecionada: ASCII/Bayer 4×4, half-block truecolor, escalonamento e alpha compositing, detecção de ambiente, opções, PNG/base64, Kitty em chunks 4096, iTerm2, Sixel com paleta/RLE, erros e todos os testes/examples.
- A referência também inclui Sixel: incluir esse protocolo além dos explicitamente destacados Kitty/iTerm2 é necessário para paridade integral com Render/README/testes Go.
- Convenção do workspace: crate com módulo `src/lib.rs`, docs em inglês no README.md e pt-BR em README.pt-br.md, CHANGELOG, exemplo `examples/charm_<name>_demo.rs`, testes de integração e golden em `tests/golden`.
- Saída Sixel no Go usa conteúdo Sixel codificado em base64 dentro do DCS, apesar da limitação documentada. Preservar exatamente esse contrato.
- `image` é dependência direta necessária para receber `DynamicImage` e codificar PNG; `base64` também será crate-local para protocol payloads. Trabalhar com `CARGO_TARGET_DIR=target/agent-port-image-luna`.

## Decisões de API
- Expor `Protocol`, `Detect`, `DetectFrom`, `Options`/builder e `Render`, `RenderHalfBlock`, `RenderSixel`, `RenderKitty`, `RenderITerm2`, `AsciiArt`.
- Receber referências `&DynamicImage` para erro representável de imagem ausente? Melhor refletir diretamente as fontes Go através de `Option<&DynamicImage>` não é ergonômico: implementar `Render` sobre `Option<&DynamicImage>` e funções raw sobre `&DynamicImage`, para preservar nil/empty errors no Rust.
- `Render` usa `Result<String, Error>`; opções builder e background `image::Rgba<u8>`.
- A validação Go precisa de `GOWORK=off`, pois o módulo de referência não está listado no `go.work` raiz.
- Validação final: 11 testes unitários Rust e 1 doctest, clippy `-D warnings`, fmt check e docs passaram; suíte Go passou com `GOWORK=off`.
- Limitações: `&DynamicImage` não modela nil; sem ensaio em terminais reais; Sixel base64/DCS preservado como descrito no Go.
