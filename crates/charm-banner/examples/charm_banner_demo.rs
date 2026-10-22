//! Demo of the `charm-banner` crate.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-banner --example demo
//! ```

use charm_banner::{render, Font, RenderOption};
use lipgloss::{Color, Style};

fn main() {
    let text = "Charm";

    println!("# Small (3x3)");
    println!("{}", render(text, &[RenderOption::Font(Font::Small)]));
    println!();

    println!("# Medium (5x5, default)");
    println!("{}", render(text, &[]));
    println!();

    println!("# Large (7x7)");
    println!("{}", render(text, &[RenderOption::Font(Font::Large)]));
    println!();

    println!("# Styled");
    let style = Style::new()
        .bold(true)
        .foreground(Color("#ff79c6".to_owned()));
    println!(
        "{}",
        render(
            "Rust",
            &[RenderOption::Style(style), RenderOption::Font(Font::Small)]
        )
    );
    println!();

    println!("# Numbers and punctuation");
    println!(
        "{}",
        render("Rust 1.0!", &[RenderOption::Font(Font::Medium)])
    );
}
