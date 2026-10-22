# charm-form

Gerador declarativo de formulários para interfaces de terminal. Os formulários
são escritos em YAML, validados e serializados para JSON.

> **[Read in English](README.md)**

Este crate é um port da biblioteca Go
[`charm-form`](https://github.com/junhinhow/charm-form): o schema YAML, as
regras de validação e o layout do JSON são os mesmos.

## Instalação

```bash
cargo add charm-form
```

## Uso

Um formulário é descrito em YAML:

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

Carregue o formulário e depois serialize-o ou execute-o:

```rust
use std::path::Path;

use charm_form::FormSpec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spec = FormSpec::load_from_file(Path::new("deploy.yaml"))?;

    // JSON formatado, com indentação de dois espaços.
    println!("{}", spec.to_json()?);

    // Imprime o mesmo JSON na saída padrão.
    spec.run()?;
    Ok(())
}
```

### Tipos de campo

| Tipo | Descrição |
|------|-----------|
| `input` | Entrada de texto em uma linha (aceita `placeholder` e `validate`) |
| `text` | Texto em múltiplas linhas |
| `confirm` | Pergunta de sim/não |
| `select` | Uma escolha entre `options` (obrigatório) |
| `multiselect` | Várias escolhas entre `options` (obrigatório) |

### Validação

O carregamento falha com um [`Error`] quando:

* o `title` está ausente ou vazio;
* não há campos;
* um campo não tem `name` ou não tem `type`;
* um campo tem um tipo fora de `input`, `select`, `confirm`, `multiselect`, `text`;
* um campo `select` ou `multiselect` não tem `options`.

## Referência da API

| Função | Descrição |
|--------|-----------|
| `FormSpec::load_from_file(&Path)` / `load_from_file` | Carrega e valida um arquivo YAML |
| `FormSpec::load_from_bytes(&[u8])` / `load_from_bytes` | Carrega e valida bytes YAML |
| `FormSpec::to_json(&self)` / `to_json` | Serializa para JSON formatado |
| `FormSpec::run(&self)` / `run` | Imprime o formulário em JSON |

## Diferenças conhecidas em relação ao Go

* As mensagens de erro estão em inglês (convenção do crates.io). As regras de
  validação e a ordem em que são verificadas são as mesmas; as mensagens do Go
  estão em português.
* `run` apenas imprime o formulário em JSON. Como no Go, é um placeholder para
  um formulário interativo. A integração com `charmed-huh` está prevista para
  uma versão major futura.

## Testes

A suíte de paridade compara a saída JSON com um golden file gerado pela
implementação Go:

```bash
cargo test -p charm-form
```

## Licença

MIT
