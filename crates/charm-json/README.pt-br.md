# charm-json

Explorador interativo de árvores JSON/YAML para Rust e Bubble Tea. O parsing mantém a ordem de entrada, expõe caminhos RFC 6901 e o modelo oferece recolhimento, busca, mensagens de cópia, cores de sintaxe e limite de profundidade exibida.

```rust
use charm_json::{with_max_depth, Model, Parse, with_syntax_colors};
let root = Parse(br#"{"nome":"Charm","itens":[1,2]}"#, "")?;
let mut model = Model::new(Some(root), [with_max_depth(8), with_syntax_colors(false)]);
model.set_size(80, 24);
assert!(model.view().contains("nome"));
# Ok::<(), charm_json::ParseError>(())
```

Use `Parse(data, "json")`, `Parse(data, "yaml")` ou `Parse(data, "")` para detecção automática. `ParseJSON` e `ParseYAML` fazem parsing explícito. A árvore oferece `Node::{find, flatten, collapse, expand, toggle, collapse_all, expand_all}`. O modelo navega com `j/k` e setas, `enter/espaço`, `h/l`, `g/G`, `/`, `esc` e `y` (emite `YankedMsg`; o clipboard é responsabilidade do aplicativo hospedeiro).

## Licença

MIT
