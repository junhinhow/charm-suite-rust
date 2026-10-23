//! Exemplo de renderização local de uma árvore JSON.

use charm_json::{with_max_depth, with_syntax_colors, Model, Parse};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sample = br#"{"name":"charm-json","tags":["json","yaml"]}"#;
    let root = Parse(sample, "")?;
    let mut model = Model::new(Some(root), [with_max_depth(8), with_syntax_colors(false)]);
    model.set_size(80, 24);
    println!("{}", model.view());
    Ok(())
}
