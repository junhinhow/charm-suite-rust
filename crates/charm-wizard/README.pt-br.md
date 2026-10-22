# charm-wizard

Framework de wizard multi-etapas para interfaces de terminal, com estado compartilhado, navegação para frente e para trás e indicador de progresso estilizado.

> **[Read in English](README.md)**

Este é um port em Rust da biblioteca Go [`github.com/junhinhow/charm-wizard`](https://github.com/junhinhow/charm-wizard). A sequência de etapas, os dados compartilhados, a navegação e o layout do progresso seguem o original; as cores são as mesmas da paleta do original. A saída de execução é verificada contra um golden file gerado pela implementação Go.

## Funcionalidades

- **Etapas ordenadas** com nome, descrição e uma closure que recebe os dados compartilhados
- **Estado compartilhado** por meio de um `DataMap` (`HashMap<String, serde_json::Value>`)
- **Navegação** com `back`, `forward` e `go_to`
- **Modos de execução**: `run` executa todas as etapas; `run_step` executa apenas a atual
- **Indicador de progresso**: pontos por etapa (concluída, atual, pendente), barra de 30 células e percentual

## Instalação

```toml
[dependencies]
charm-wizard = "0.1"
```

## Uso

```rust
use charm_wizard::{DataMap, Error, Wizard};
use serde_json::json;

fn main() -> Result<(), Error> {
    let mut wizard = Wizard::new()
        .with_title("Configuração do Projeto")
        .add_step("config", "Configuração", |data: &mut DataMap| {
            data.insert("project".into(), json!("charm-suite"));
            Ok(())
        })
        .add_step("deploy", "Deploy", |data: &mut DataMap| {
            if data.contains_key("project") {
                Ok(())
            } else {
                Err(Error::custom("projeto ausente"))
            }
        });

    wizard.run()?;
    Ok(())
}
```

As etapas também podem ser conduzidas manualmente:

```rust
use charm_wizard::{DataMap, Wizard};

let mut wizard = Wizard::new()
    .add_step("um", "Primeira", |_: &mut DataMap| Ok(()))
    .add_step("dois", "Segunda", |_: &mut DataMap| Ok(()));

wizard.run_step()?;      // executa a etapa 0
wizard.forward();
wizard.run_step()?;      // executa a etapa 1
# Ok::<(), charm_wizard::Error>(())
```

## Indicador de progresso

`render_progress(steps, current)` devolve três linhas:

```text
Step 2 of 4: Dependencies
[*] config --- [>] deps --- [ ] review --- [ ] deploy
██████████████░░░░░░░░░░░░░░░░ 50%
```

Etapas concluídas aparecem em verde `[*]`, a etapa atual em amarelo `[>]` e as pendentes em cinza `[ ]`. As cores são as mesmas da implementação Go.

## Diferenças em relação ao Go

- `Wizard::run` escreve na saída padrão, como no Go. `Wizard::run_to` escreve em qualquer `std::io::Write`; é uma adição usada nos testes.
- As etapas devolvem `charm_wizard::Error`. Use `Error::custom("mensagem")` para mensagens dentro de uma etapa.
- `go_to` recebe `usize`, então índices negativos do Go não podem ser passados. Qualquer valor fora da faixa devolve `Error::InvalidIndex`.
- `render_progress` devolve string vazia para índice fora da faixa; o Go entra em panic.
- As mensagens de erro estão em inglês.
- A saída sempre contém sequências de escape ANSI. O Go só emite cores quando a saída padrão é um terminal.

## Licença

MIT
