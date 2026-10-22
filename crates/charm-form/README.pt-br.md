# charm-form

Gerador declarativo de formulários para interfaces de terminal. Defina
formulários em YAML, carregue e valide as especificações, colete respostas
interativas e serialize o formulário ou as respostas em JSON.

> **[Read in English](README.md)**

Este crate porta a API pública e o comportamento do pacote Go
[`charm-form`](https://github.com/junhinhow/charm-form). A renderização
interativa usa o runtime Elm `charmed-bubbletea` do workspace.

## Instalação

```toml
[dependencies]
charm-form = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

## Formulário YAML

```yaml
title: Deploy Configuration
description: Configure deployment settings
fields:
  - name: environment
    type: select
    label: Target Environment
    options: [staging, production, development]
    required: true
  - name: version
    type: input
    label: Version
    placeholder: "v1.0.0"
    validate: "^v\\d+\\.\\d+\\.\\d+$"
  - name: dry_run
    type: confirm
    label: Dry run only?
```

## Execução interativa

O crate inclui o programa de linha de comando `charm-form`. Ele abre uma TUI e
grava as respostas coletadas em JSON por padrão:

```text
cargo run -p charm-form --bin charm-form -- -file crates/charm-form/examples/deploy.yaml
```

A CLI também aceita o caminho YAML como argumento posicional, `-out CAMINHO` e
`-json=false` para gravar um resumo legível em vez de JSON.

Também é possível executar o renderizador pela biblioteca:

```rust,no_run
use std::path::Path;

use charm_form::{load_from_file, renderer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec = load_from_file(Path::new("deploy.yaml"))?;
    let answers = renderer::run(spec)?;
    println!("{}", answers.to_json()?);
    Ok(())
}
```

`renderer::run_with_options` recebe streams `Read`/`Write` customizados e o
tamanho inicial do terminal, permitindo testes com entrada programada. Esc e
Ctrl+C cancelam o formulário.

### Tipos de campo e teclas

| Tipo | Resposta | Entrada |
|------|----------|---------|
| `input` | String | Enter confirma; setas editam o cursor |
| `text` | String | Enter insere quebra de linha; Ctrl+S confirma |
| `select` | String | Cima/baixo ou k/j; Enter confirma |
| `multiselect` | Vetor de strings | Cima/baixo, Espaço marca, Enter confirma |
| `confirm` | Booleano | y/n ou esquerda/direita; Enter confirma |

Tab e Shift+Tab navegam entre campos. A validação acontece ao confirmar; um
valor inválido mantém o foco no campo e exibe a mensagem de erro.

## API de especificação e validação

`FormSpec::load_from_file` / `load_from_bytes` normalizam os tipos e validam a
especificação. `normalize_type` aceita nomes sem diferenciar maiúsculas e
minúsculas, além dos aliases `multi-select` e `multi_select`. A validação
verifica:

* título não vazio e pelo menos um campo;
* nomes não vazios e únicos e tipos conhecidos;
* opções para `select` e `multiselect`;
* sintaxe válida da regex durante o carregamento.

`FieldSpec::label_text` usa o nome como rótulo alternativo;
`FieldSpec::regex` compila a regex opcional. `FormSpec::validate` também valida
especificações criadas diretamente em Rust.

## Respostas

As respostas são armazenadas como um mapa de nomes para
`serde_json::Value`:

```rust
use charm_form::{Answers, FormSpec};
use serde_json::Value;

let spec = FormSpec::load_from_bytes(
    b"title: Oi\nfields:\n  - name: quem\n    type: input\n    required: true\n",
)?;
let mut answers = Answers::new();
answers.insert("quem".to_owned(), Value::String("Ana".to_owned()));
answers.validate(&spec)?;
let json = answers.to_json()?;
# Ok::<(), charm_form::Error>(())
```

Respostas de texto e seleção são strings, confirmações são booleanos e
`multiselect` usa um vetor de strings. `Answers::validate` reúne todos os
erros de respostas inválidas ou obrigatórias ausentes; `to_json` e
`write_json` usam indentação de dois espaços.

`FormSpec::to_json`, `write_json` e `run` serializam a própria especificação. A
execução interativa é feita por `renderer::run`.

## Diferenças conhecidas em relação ao Go

* As mensagens de erro estão em inglês (convenção do crates.io); as mensagens
  Go estão em português.
* O renderizador Rust usa o runtime `charmed-bubbletea` e marcadores simples de
  seleção, em vez do estilo Lip Gloss v2 do Go. As teclas e a coleta de
  respostas foram portadas.

## Testes

```bash
cargo test -p charm-form
```

O golden de paridade é gerado pelo programa Go em
`reference/go/charm-form/cmd/parity`.

## Licença

MIT
