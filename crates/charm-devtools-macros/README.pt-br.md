# charm-devtools-macros

Macro derive para `charm-devtools`. Gera a implementação de `DevInspect` para
structs com campos nomeados e marcados explicitamente.

```rust
#[derive(DevInspect)]
struct Screen {
    #[inspect]
    save_button: Button,
    #[devtools(inspect)]
    menu_items: Vec<Button>,
    #[inspect(skip)]
    cache: Button,
}
```

Este crate é usado indiretamente: não o adicione ao seu `Cargo.toml`. Ative a feature `derive` em `charm-devtools` (veja o README dele para o trecho de instalação). Campos sem atributo são ignorados;
structs tupla e unitárias geram somente o nó raiz. Campos `Vec<T>` inspecionados
geram filhos com nomes `campo[índice]`.
