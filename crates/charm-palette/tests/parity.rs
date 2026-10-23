//! Parity tests against the Go original (`reference/go/charm-palette`).
//!
//! The golden file `tests/golden/palette.txt` is produced by the Go program
//! `reference/go/charm-palette/cmd/parity`, which exercises the whole public
//! API and prints a transcript of named blocks. The blocks below mirror that
//! program one to one.

use std::collections::BTreeMap;

use charm_core::Rgb;
use charm_palette::{compare_themes, show_ansi256, show_palette, show_theme, show_true_color};

/// Formats one named block of the transcript.
fn block(name: &str, content: &str) -> String {
    format!("=== {name} ===\n{content}\n")
}

/// Builds a theme map out of `(name, rgb)` pairs.
fn theme(entries: &[(&str, Rgb)]) -> BTreeMap<String, Rgb> {
    entries
        .iter()
        .map(|(name, rgb)| ((*name).to_owned(), *rgb))
        .collect()
}

/// Builds the full transcript, matching the Go program byte for byte.
#[allow(clippy::too_many_lines)]
fn transcript() -> String {
    let dracula = theme(&[
        ("primary", Rgb::rgb(0xff, 0x79, 0xc6)),
        ("accent", Rgb::rgb(0x50, 0xfa, 0x7b)),
        ("bg", Rgb::rgb(0x28, 0x28, 0x28)),
    ]);
    let mono = theme(&[("fg", Rgb::rgb(0xf8, 0xf8, 0xf2))]);
    let theme_one = theme(&[
        ("primary", Rgb::rgb(0xff, 0x00, 0x00)),
        ("bg", Rgb::rgb(0x11, 0x11, 0x11)),
    ]);
    let theme_two = theme(&[
        ("primary", Rgb::rgb(0x00, 0xff, 0x00)),
        ("fg", Rgb::rgb(0xff, 0xff, 0xff)),
    ]);
    let theme_three = theme(&[("primary", Rgb::rgb(0x00, 0x00, 0xff))]);
    let long_labels = theme(&[
        ("configuração", Rgb::rgb(99, 102, 241)),
        ("nome-de-etapa-muito-comprido", Rgb::rgb(168, 85, 247)),
    ]);

    let blocks = vec![
        // show_palette
        block(
            "palette_basic",
            &show_palette(
                &[
                    Rgb::rgb(0xff, 0x00, 0x00),
                    Rgb::rgb(0x00, 0xff, 0x00),
                    Rgb::rgb(0x00, 0x00, 0xff),
                    Rgb::rgb(0xff, 0xff, 0xff),
                ],
                48,
            ),
        ),
        block(
            "palette_narrow",
            &show_palette(&[Rgb::rgb(0x28, 0x28, 0x28), Rgb::rgb(0xfa, 0xbd, 0x2f)], 8),
        ),
        block(
            "palette_width_5",
            &show_palette(&[Rgb::rgb(0x28, 0x28, 0x28), Rgb::rgb(0xfa, 0xbd, 0x2f)], 5),
        ),
        block(
            "palette_vertical_width_20",
            &show_palette(
                &[
                    Rgb::rgb(255, 89, 94),
                    Rgb::rgb(255, 202, 58),
                    Rgb::rgb(138, 201, 38),
                    Rgb::rgb(25, 130, 196),
                    Rgb::rgb(106, 76, 147),
                    Rgb::rgb(1, 2, 3),
                ],
                20,
            ),
        ),
        block(
            "palette_single",
            &show_palette(&[Rgb::rgb(0xff, 0x79, 0xc6)], 12),
        ),
        block(
            "palette_contrast_dark",
            &show_palette(&[Rgb::rgb(15, 23, 42)], 20),
        ),
        block(
            "palette_contrast_light",
            &show_palette(&[Rgb::rgb(240, 230, 140)], 20),
        ),
        block("palette_empty", &show_palette(&[], 40)),
        block("palette_zero_width", &show_palette(&[Rgb::rgb(1, 2, 3)], 0)),
        // show_ansi256
        block("ansi256", &show_ansi256()),
        // show_true_color
        block("truecolor_16x4", &show_true_color(16, 4)),
        block("truecolor_1x1", &show_true_color(1, 1)),
        block("truecolor_zero", &show_true_color(0, 0)),
        // show_theme
        block("theme_multi", &show_theme("Dracula", &dracula)),
        block("theme_single", &show_theme("Mono", &mono)),
        block("theme_empty", &show_theme("Empty", &BTreeMap::new())),
        block("theme_long_labels", &show_theme("Longos", &long_labels)),
        // compare_themes
        block("compare_two", &compare_themes(&[&theme_one, &theme_two])),
        block("compare_one", &compare_themes(&[&theme_three])),
        block("compare_empty", &compare_themes(&[])),
        block("compare_long_labels", &compare_themes(&[&long_labels])),
    ];

    blocks.join("\n")
}

#[test]
fn palette_matches_the_go_implementation() {
    charm_parity::assert_golden("palette", &transcript());
}
