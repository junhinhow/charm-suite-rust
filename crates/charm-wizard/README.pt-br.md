# charm-wizard

Framework de wizard multi-etapas para interfaces de terminal, com estado compartilhado, navegação para frente e para trás e indicador de progresso estilizado.

> **[Read in English](README.md)**

Este é um port em Rust da biblioteca Go [`github.com/junhinhow/charm-wizard`](https://github.com/junhinhow/charm-wizard). A sequência de etapas, os dados compartilhados, a navegação e o layout do progresso seguem o original; as cores e os textos são os mesmos da saída do original. A saída de execução é verificada contra um golden file gerado pela implementação Go.

## Funcionalidades

- **Etapas ordenadas** com nome, descrição e uma closure que recebe os dados compartilhados
- **Estado compartilhado** por meio de um `DataMap` (`HashMap<String, serde_json::Value>`), com getters tipados
- **Navegação** com `back`, `forward` e `go_to`. Voltar desfaz os dados gravados pela etapa abandonada, e uma etapa que falhou não pode ser ignorada
- **Modos de execução**: `run` executa as etapas a partir da atual até o fim; `run_step` executa apenas a atual
- **Indicador de progresso**: pontos por etapa (concluída, atual, pendente), barra de 30 células, percentual e `progress()` como fração

## Instalação

```toml
[dependencies]
charm-wizard = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Ainda não publicado no crates.io; instale via git.

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

wizard.run_step()?;      // executa a etapa 0 e imprime o progresso
wizard.forward();
wizard.run_step()?;      // executa a etapa 1
# Ok::<(), charm_wizard::Error>(())
```

## Navegação e desfazer

`back` volta uma etapa e restaura os dados compartilhados ao estado que tinham antes de a etapa abandonada ser executada. `forward` se recusa a avançar quando a etapa atual falhou, e `go_to` vai para qualquer índice válido:

```rust
use charm_wizard::{DataMap, Error, Wizard};
use serde_json::json;

let mut wizard = Wizard::new()
    .add_step("a", "A", |d: &mut DataMap| {
        d.insert("a".into(), json!(1));
        Ok(())
    })
    .add_step("b", "B", |d: &mut DataMap| {
        d.insert("b".into(), json!(2));
        Ok(())
    });

wizard.run_step()?;
wizard.forward();
wizard.run_step()?;
assert_eq!(wizard.get_int("b"), Some(2));

wizard.back();
assert_eq!(wizard.get_int("b"), None);   // desfeito
assert_eq!(wizard.get_int("a"), Some(1));
# Ok::<(), Error>(())
```

`run` começa na etapa atual: `go_to(2)` seguido de `run` executa somente a terceira etapa e as seguintes.

## Dados compartilhados

| Método | Devolve |
|--------|---------|
| `set_data(chave, valor)` | Builder; insere um `serde_json::Value` |
| `data()` | `&DataMap` com todos os valores |
| `get_string(chave)` | `Option<&str>`; `None` quando falta ou tem outro tipo |
| `get_bool(chave)` | `Option<bool>` |
| `get_int(chave)` | `Option<i64>`; `None` para números fracionários |

## Indicador de progresso

`render_progress(steps, current)` devolve três linhas:

```text
Etapa 2 de 4: Dependencies
[*] config --- [>] deps --- [ ] review --- [ ] deploy
██████████████░░░░░░░░░░░░░░░░ 50%
```

Etapas concluídas aparecem em verde `[*]`, a etapa atual em amarelo `[>]` e as pendentes em cinza `[ ]`. As cores são as mesmas da implementação Go. A saída de `run` termina com `✓ <título> concluído com sucesso!`, como no Go.

## Diferenças em relação ao Go

- `Wizard::run` escreve na saída padrão, como no Go, e redesenha cada bloco de progresso no lugar quando a saída padrão é um terminal. `Wizard::run_to` e `Wizard::run_step_to` escrevem em qualquer `std::io::Write` e nunca redesenham: substituem o `WithWriter` do Go.
- As etapas são closures e não podem ser `nil`. Use `|_: &mut DataMap| Ok(())` para uma etapa sem trabalho.
- `data()` devolve uma referência compartilhada. O Go devolve o próprio mapa; altere valores pelas etapas ou com `set_data` ao montar o wizard.
- `steps()` devolve uma fatia das etapas, e não uma cópia.
- `get_int` devolve `i64`, onde o Go usa `int`.
- As etapas devolvem `charm_wizard::Error`. Use `Error::custom("mensagem")` para mensagens dentro de uma etapa.
- `go_to` recebe `usize`, então índices negativos do Go não podem ser passados. Qualquer valor fora da faixa devolve `Error::InvalidIndex`.
- As mensagens de erro estão em inglês. Os textos de progresso e de conclusão estão em português, como no Go.
- A saída sempre contém sequências de escape ANSI. O Go só emite cores quando a saída padrão é um terminal.

## Licença

MIT
