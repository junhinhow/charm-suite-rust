//! Demo of the `charm-palette` crate.
//!
//! Run with:
//!
//! ```text
//! cargo run -p charm-palette --example demo
//! ```

use std::collections::BTreeMap;

use charm_core::Rgb;
use charm_palette::{compare_themes, show_ansi256, show_palette, show_theme, show_true_color};

fn theme(entries: &[(&str, Rgb)]) -> BTreeMap<String, Rgb> {
    entries
        .iter()
        .map(|(name, rgb)| ((*name).to_owned(), *rgb))
        .collect()
}

fn main() {
    // ------------------------------------------------------------------
    // A palette of swatches
    // ------------------------------------------------------------------
    println!("charm-palette — demo");
    println!();
    println!(
        "{}",
        show_palette(
            &[
                Rgb::rgb(0xff, 0x5f, 0x87),
                Rgb::rgb(0xff, 0xb8, 0x6c),
                Rgb::rgb(0xff, 0xe0, 0x6c),
                Rgb::rgb(0x8b, 0xe2, 0x6c),
                Rgb::rgb(0x6c, 0xd8, 0xe0),
                Rgb::rgb(0x6c, 0xa8, 0xe0),
            ],
            60,
        )
    );
    println!();

    // ------------------------------------------------------------------
    // A named theme
    // ------------------------------------------------------------------
    let dracula = theme(&[
        ("background", Rgb::rgb(0x28, 0x28, 0x28)),
        ("current", Rgb::rgb(0x44, 0x47, 0x5a)),
        ("foreground", Rgb::rgb(0xf8, 0xf8, 0xf2)),
        ("comment", Rgb::rgb(0x62, 0x72, 0xa4)),
        ("cyan", Rgb::rgb(0x8b, 0xe2, 0xfc)),
        ("green", Rgb::rgb(0x50, 0xfa, 0x7b)),
        ("orange", Rgb::rgb(0xff, 0xb8, 0x6c)),
        ("pink", Rgb::rgb(0xff, 0x79, 0xc6)),
        ("purple", Rgb::rgb(0xbd, 0x93, 0xf9)),
        ("red", Rgb::rgb(0xff, 0x55, 0x55)),
        ("yellow", Rgb::rgb(0xff, 0xf1, 0x8a)),
    ]);
    println!("{theme}", theme = show_theme("Dracula", &dracula));

    // ------------------------------------------------------------------
    // Two themes side by side
    // ------------------------------------------------------------------
    let nord = theme(&[
        ("background", Rgb::rgb(0x2e, 0x34, 0x40)),
        ("foreground", Rgb::rgb(0xd8, 0xde, 0xe9)),
        ("red", Rgb::rgb(0xbf, 0x61, 0x6a)),
        ("green", Rgb::rgb(0xa3, 0xbe, 0x8c)),
        ("blue", Rgb::rgb(0x81, 0xa1, 0xc1)),
    ]);
    println!(
        "{}",
        compare_themes(&[("Dracula", &dracula), ("Nord", &nord)])
    );

    // ------------------------------------------------------------------
    // The 256 color grid
    // ------------------------------------------------------------------
    println!("{}", show_ansi256());

    // ------------------------------------------------------------------
    // A true color gradient
    // ------------------------------------------------------------------
    println!("{}", show_true_color(48, 6));
}
