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

Ative a feature `derive` em `charm-devtools`. Campos sem atributo são ignorados;
structs tupla e unitárias geram somente o nó raiz. Campos `Vec<T>` inspecionados
geram filhos com nomes `campo[índice]`.
