//! Renders one sample table with each border style.
//!
//! The crate is binary-only, so the renderer module is included directly from
//! `src/render.rs`.
//!
//! ```text
//! cargo run -p charm-table-cli --example charm_table_cli_demo
//! ```

#[path = "../src/render.rs"]
mod render;

use render::{render_table, BorderStyle, RenderOptions};

fn main() {
    let headers: Vec<String> = ["Linguagem", "Ano", "Tipo"]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    let rows: Vec<Vec<String>> = [
        ["Rust", "2015", "Sistemas"],
        ["Go", "2009", "Servidores"],
        ["Python", "1991", "Script"],
    ]
    .iter()
    .map(|row| row.iter().map(|cell| (*cell).to_owned()).collect())
    .collect();

    for style in [
        BorderStyle::Rounded,
        BorderStyle::Thick,
        BorderStyle::Double,
        BorderStyle::Ascii,
    ] {
        let options = RenderOptions {
            style,
            max_width: None,
            color: "63".to_owned(),
        };
        println!("{style:?}");
        println!("{}", render_table(&headers, &rows, &options));
        println!();
    }
}
