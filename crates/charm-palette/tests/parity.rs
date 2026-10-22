//! Parity tests against the Go original (`reference/go/charm-palette`).
//!
//! The golden file `tests/golden/palette.txt` is produced by the Go program
//! `reference/go/charm-palette/cmd/parity`, which exercises the whole public
//! API and prints a transcript of named blocks.
//!
//! The Go original walks over Go maps, whose iteration order is randomised.
//! The multi-key cases are therefore sorted by their visible (ANSI-stripped)
//! content on **both** sides, so the transcript stays deterministic.

use std::collections::BTreeMap;

use charm_core::Rgb;
use charm_palette::{compare_themes, show_ansi256, show_palette, show_theme, show_true_color};

/// Formats one named block of the transcript.
fn block(name: &str, content: &str) -> String {
    format!("=== {name} ===\n{content}\n")
}

/// Removes the ANSI escape sequences, so blocks can be sorted by content.
fn strip_ansi(text: &str) -> String {
    charm_core::strip_ansi(text)
}

/// Makes the map-ordered output of `show_theme` deterministic.
fn sort_theme_blocks(out: &str) -> String {
    let trimmed = out.strip_suffix("\n\n").unwrap_or(out);
    let mut parts: Vec<&str> = trimmed.split("\n\n").collect();
    if parts.is_empty() {
        return out.to_owned();
    }
    let head = parts.remove(0);
    parts.sort_by_cached_key(|part| strip_ansi(part));
    format!("{head}\n\n{}\n\n", parts.join("\n\n"))
}

/// Makes the map-ordered output of `compare_themes` deterministic.
fn sort_compare_themes(out: &str) -> String {
    let trimmed = out.strip_suffix('\n').unwrap_or(out);
    let lines: Vec<&str> = trimmed.split('\n').collect();
    if lines.len() < 3 {
        return out.to_owned();
    }
    let header = lines[0];
    let body = &lines[1..];

    let mut pairs: Vec<(&str, &str)> = Vec::new();
    let mut i = 0;
    while i + 1 < body.len() {
        pairs.push((body[i], body[i + 1]));
        i += 2;
    }
    pairs.sort_by_cached_key(|(key, _)| strip_ansi(key));

    let mut out_lines: Vec<&str> = vec![header];
    for (key, swatch) in pairs {
        out_lines.push(key);
        out_lines.push(swatch);
    }
    format!("{}\n", out_lines.join("\n"))
}

/// Builds a theme map out of `(name, rgb)` pairs.
fn theme(entries: &[(&str, Rgb)]) -> BTreeMap<String, Rgb> {
    entries
        .iter()
        .map(|(name, rgb)| ((*name).to_owned(), *rgb))
        .collect()
}

/// Builds the full transcript, matching the Go program byte for byte.
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

    let blocks = vec![
        // ----------------------------------------------------------
        // show_palette
        // ----------------------------------------------------------
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
            "palette_single",
            &show_palette(&[Rgb::rgb(0xff, 0x79, 0xc6)], 12),
        ),
        block(
            "palette_wrap5",
            &show_palette(
                &[Rgb::rgb(0x28, 0x28, 0x28), Rgb::rgb(0xfa, 0xbd, 0x2f)],
                10,
            ),
        ),
        block("palette_empty", &show_palette(&[], 40)),
        // ----------------------------------------------------------
        // show_ansi256
        // ----------------------------------------------------------
        block("ansi256", &show_ansi256()),
        // ----------------------------------------------------------
        // show_true_color
        // ----------------------------------------------------------
        block("truecolor_16x4", &show_true_color(16, 4)),
        block("truecolor_1x1", &show_true_color(1, 1)),
        block("truecolor_zero", &show_true_color(0, 0)),
        // ----------------------------------------------------------
        // show_theme
        // ----------------------------------------------------------
        block(
            "theme_multi",
            &sort_theme_blocks(&show_theme("Dracula", &dracula)),
        ),
        block("theme_single", &show_theme("Mono", &mono)),
        block("theme_empty", &show_theme("Empty", &BTreeMap::new())),
        // ----------------------------------------------------------
        // compare_themes
        // ----------------------------------------------------------
        block(
            "compare_two",
            &sort_compare_themes(&compare_themes(&[("one", &theme_one), ("two", &theme_two)])),
        ),
        block("compare_one", &compare_themes(&[("one", &theme_three)])),
        block("compare_empty", &compare_themes(&[])),
    ];

    blocks.join("\n")
}

#[test]
fn palette_matches_the_go_implementation() {
    charm_parity::assert_golden("palette", &transcript());
}
