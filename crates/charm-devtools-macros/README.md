# charm-devtools-macros

Derive macro for `charm-devtools`. It generates a `DevInspect` implementation
for named structs and explicitly marked fields.

```rust
use charm_devtools::DevInspect;

struct Button;
impl DevInspect for Button {}

#[derive(DevInspect)]
struct Screen {
    #[inspect]
    save: Button,
    #[devtools(inspect)]
    menu: Vec<Button>,
    #[inspect(skip)]
    cache: Button,
}

let tree = Screen {
    save: Button,
    menu: vec![Button, Button],
    cache: Button,
}
.inspect();
assert_eq!(tree.children[0].name, "save");
assert_eq!(tree.children[1].name, "menu[0]");
assert_eq!(tree.children[2].name, "menu[1]");
```

This crate is used indirectly: do not add it to your `Cargo.toml`. Enable the `derive` feature on `charm-devtools` instead (see its README for the install snippet). Fields without an inspection
attribute are ignored. Tuple and unit structs generate a root node only.

## Português (Brasil)

A macro derive gera uma implementação de `DevInspect` para structs nomeadas e
campos marcados explicitamente.

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

Ative a feature `derive` de `charm-devtools`. Campos sem atributo são ignorados;
structs tupla e unitárias geram somente o nó raiz.
