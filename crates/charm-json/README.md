# charm-json

Interactive JSON/YAML tree explorer for Rust and Bubble Tea. Parsing keeps input order, exposes RFC 6901 paths, and the model supports collapse, search, yank messages, syntax colors, and a display-depth limit.

```rust
use charm_json::{with_max_depth, Model, Parse, with_syntax_colors};
let root = Parse(br#"{"name":"Charm","items":[1,2]}"#, "")?;
let mut model = Model::new(Some(root), [with_max_depth(8), with_syntax_colors(false)]);
model.set_size(80, 24);
assert!(model.view().contains("name"));
# Ok::<(), charm_json::ParseError>(())
```

Use `Parse(data, "json")`, `Parse(data, "yaml")`, or `Parse(data, "")` for automatic detection. `ParseJSON` and `ParseYAML` are available for explicit parsing. The tree API is `Node::{find, flatten, collapse, expand, toggle, collapse_all, expand_all}`. Model navigation supports `j/k` and arrows, `enter/space`, `h/l`, `g/G`, `/`, `esc`, and `y` (emits `YankedMsg`; clipboard access belongs to the host app).

## License

MIT
