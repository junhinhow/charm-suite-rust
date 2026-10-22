# charm-i18n

Internacionalização para interfaces de terminal: catálogos de mensagens, detecção de locale, fallback e formatação estilo printf.

> **[Read in English](README.md)**

Este é um port em Rust da biblioteca Go [`github.com/junhinhow/charm-i18n`](https://github.com/junhinhow/charm-i18n). Consultas, fallback, detecção de locale e carregamento de JSON se comportam como no original, e o formatador reproduz a saída do `fmt.Sprintf` para os verbos mais comuns. A paridade é verificada por um golden file gerado pela implementação Go.

## Funcionalidades

- **Catálogos** organizados por locale e chave, com `add_locale` para adicionar ou substituir mensagens
- **Fallback**: chaves ausentes no locale ativo são buscadas no locale de fallback e, se não existirem, a própria chave é devolvida
- **Detecção de locale** a partir de `LC_ALL`, `LC_MESSAGES` e `LANG`, normalizando `pt_BR.UTF-8` para `pt-BR`
- **Mensagens formatadas** com `%s`, `%d`, `%v`, `%f`, `%q`, `%t`, `%%` e `{}` estilo Rust
- **Carregamento de JSON** a partir de um arquivo `{ "locale": { "chave": "texto" } }`

## Instalação

```toml
[dependencies]
charm-i18n = "0.1"
```

## Uso

```rust
use std::collections::BTreeMap;

use charm_i18n::Catalog;
use serde_json::json;

fn main() {
    let mut catalog = Catalog::new();
    catalog.add_locale(
        "en",
        BTreeMap::from([("greeting".to_owned(), "Hello, %s! You have %d messages.".to_owned())]),
    );
    catalog.add_locale(
        "pt-BR",
        BTreeMap::from([("greeting".to_owned(), "Ola, %s! Voce tem %d mensagens.".to_owned())]),
    );

    catalog.set_locale("pt-BR");
    println!("{}", catalog.tf("greeting", &[json!("Alice"), json!(5)]));
    // Ola, Alice! Voce tem 5 mensagens.

    catalog.set_locale("fr");
    println!("{}", catalog.t("missing.key")); // usa "en" como fallback ou devolve a chave
}
```

### Carregando traduções de JSON

```rust
use std::path::Path;

use charm_i18n::{detect_locale, Catalog};

fn main() -> Result<(), charm_i18n::Error> {
    let mut catalog = Catalog::new();
    catalog.load_json(Path::new("translations.json"))?;
    catalog.set_locale(&detect_locale());
    Ok(())
}
```

## Formatação

`Catalog::tf` formata o template traduzido com os argumentos, como o `fmt.Sprintf` do Go. Sem argumentos, o template é devolvido sem alterações.

| Diretiva | Significado |
|---|---|
| `%v` / `{}` | qualquer valor, formato padrão |
| `%s` | string (`%.3s` trunca) |
| `%q` | string entre aspas, ou caractere entre aspas para inteiros |
| `%d`, `%x`, `%X`, `%o`, `%b` | inteiros |
| `%f`, `%F`, `%e`, `%E` | ponto flutuante (precisão padrão 6) |
| `%t` | booleano |
| `%%` | `%` literal |

As flags `-`, `+`, espaço e `0`, além de largura e precisão, são suportadas.

Argumentos incompatíveis são reportados da mesma forma que o Go:

```rust
use charm_i18n::format::sprintf;
use serde_json::json;

assert_eq!(sprintf("%d", &[json!("Alice")]), "%!d(string=Alice)");
assert_eq!(sprintf("%d messages", &[]), "%!d(MISSING) messages");
assert_eq!(sprintf("Hello", &[json!("x")]), "Hello%!(EXTRA string=x)");
```

O formatador é público como `charm_i18n::format::sprintf`.

## Diferenças em relação ao Go

- `Catalog::t` devolve um `&str` ligado ao catálogo e à chave; a versão Go devolve uma cópia.
- `Catalog::has` verifica apenas o locale ativo, como a versão Go.
- Os argumentos são `serde_json::Value`. JSON não separa `int` de `int64` nem `float32` de `float64`, e não representa `NaN` nem infinito.
- Os placeholders `{}` e os escapes `{{` / `}}` são uma adição. O Go imprime chaves duplas literalmente.
- Diretivas não listadas acima (`%c`, `%T`, `%g`, `%p`, larguras `*`) são impressas como `%!verbo(...)` em vez de formatadas.
- A precisão não é aplicada ao valor exibido em diagnósticos de tipo incompatível (o Go imprime `%!f(int=03)` para `%.2f` com `3`).
- `detect_locale` mantém a capitalização do idioma e da região, como o Go.
- As mensagens de erro estão em inglês.

## Licença

MIT
