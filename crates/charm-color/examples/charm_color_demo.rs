//! Demo of the `charm-color` API.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-color --example demo
//! ```

use charm_color::{blend, complementary, contrast, darken, from_lipgloss, lighten, palette};
use charm_core::Rgb;
use lipgloss::Style;

/// Builds a lipgloss style out of an [`Rgb`].
fn style_of(color: Rgb) -> Style {
    Style::new().foreground(color.to_lipgloss())
}

fn main() {
    let base = Rgb::from_hex("#3498db").unwrap_or_else(|_| Rgb::rgb(0x34, 0x98, 0xdb));

    println!("charm-color demo");
    println!("===============");
    println!();

    println!("base            {}", base.to_hex());
    println!("darken 0.25     {}", darken(base, 0.25).to_hex());
    println!("lighten 0.25    {}", lighten(base, 0.25).to_hex());
    println!("complementary   {}", complementary(base).to_hex());
    println!("contrast        {}", contrast(base).to_hex());
    println!(
        "blend w/ white  {}",
        blend(base, Rgb::white(), 0.25).to_hex()
    );
    println!("from lipgloss   {}", from_lipgloss("#ff8000").to_hex());
    println!();

    println!("palette (4 colors)");
    for (index, color) in palette(base, 4).iter().enumerate() {
        let swatch = style_of(*color).render("  ");
        println!("  [{index}] {swatch} {}", color.to_hex());
    }
    println!();

    println!("readable text on the base color");
    let foreground = contrast(base);
    let style = Style::new()
        .background(base.to_lipgloss())
        .foreground(foreground.to_lipgloss());
    println!("  {}", style.render("the quick brown fox"));
}
