# charm-panes

Paineis redimensionaveis e acionados por teclado para interfaces de terminal.

## Instalacao

```bash
cargo add charm-panes
```

## Uso

```rust
use charm_panes::{Direction, Layout, Pane};

let mut layout = Layout::new(Direction::Horizontal);
layout.set_size(80, 24);

layout.add(Pane {
    id: "sidebar".to_string(),
    title: "Arquivos".to_string(),
    min_size: 20,
    fixed: true,
    content: "main.go\nREADME.md\ngo.mod".to_string(),
});
```

## Licenca

MIT
