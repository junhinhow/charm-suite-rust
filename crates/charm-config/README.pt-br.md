# charm-config

Loader de configuração em camadas, validação de schema e editor Bubble Tea para
Rust. Este crate porta o comportamento de `reference/go/charm-config`.

```toml
[dependencies]
charm-config = { path = "../charm-config" }
serde_json = "1"
```

```rust,no_run
use charm_config::{Editor, Field, Loader, Schema};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = Loader::new()
        .defaults(json!({"server":{"host":"localhost","port":8080}}))
        .file("config.yaml")
        .env("APP");
    if let Some(error) = loader.error() { eprintln!("config: {error}"); }
    println!("{}:{}", loader.string("server.host", "localhost"), loader.int("server.port", 8080));

    let schema = Schema::new(vec![
        Field::new("server.host", "string").required(true).doc("Host do banco"),
        Field::new("server.port", "int").required(true).doc("Porta TCP"),
    ]);
    let editor = Editor::new(loader, schema);
    let _result = charmed_bubbletea::Program::new(editor).run()?;
    Ok(())
}
```

## Formatos e precedência

`Loader::file` reconhece `.json`, `.yaml` e `.yml` (sem diferenciar maiúsculas).
Arquivos ausentes ou inválidos são ignorados e reportados por `Loader::error`;
o loader continua utilizável. A referência Go não implementa TOML. A precedência
é padrões (0), arquivos (100), ambiente (200) e flags (300); camadas próprias
podem definir qualquer prioridade. Mapas são mesclados recursivamente; listas e
escalares são substituídos. Prioridades iguais mantêm a ordem de inserção, com a
última camada vencendo.

Variáveis de ambiente convertem nomes para caminhos separados por ponto
(`APP_DATABASE_HOST` → `database.host`). Os valores continuam strings até a
leitura por getters tipados. Segmentos vazios e sufixos com pontos são ignorados.

## Schema e editor

Tipos suportados: `string`, `int`, `bool`, `duration` e `list`. A validação
retorna todos os erros na ordem do schema. Strings conversíveis são aceitas para
inteiro, booleano e duração. `Field::default_value` preenche somente o snapshot
do editor; não altera `Loader::all` nem satisfaz validação de campo obrigatório.

O editor implementa `charmed_bubbletea::Model`. Use setas ou `j`/`k` para
navegar, Enter para editar, Tab para alternar valores/documentação, `s` para
salvar e `q` para sair (alterações pendentes pedem confirmação). Listas inline
são separadas por vírgulas. O save grava os valores efetivos na camada de arquivo
de maior prioridade, mantém o formato JSON/YAML, ordena as chaves e define modo
`0600` no Unix. Comentários e formatação original não são preservados.

## Exemplo

Execute `cargo run -p charm-config --example charm_config_demo` na raiz do
workspace. Consulte também o [README em inglês](README.md).
