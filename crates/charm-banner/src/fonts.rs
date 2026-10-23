//! Glyph data of the three banner fonts.
//!
//! **Generated file - do not edit by hand.**
//!
//! Every pattern below was extracted programmatically from the Go
//! reference implementation (`reference/go/charm-banner/banner.go`) by
//! `crates/charm-banner/xtask/extract_fonts.rs`, so the data is
//! byte-identical to the original Go fonts.
//!
//! Regenerate with:
//!
//! ```text
//! rustc --edition 2021 crates/charm-banner/xtask/extract_fonts.rs -o extract_fonts.exe
//! ./extract_fonts.exe reference/go/charm-banner/banner.go crates/charm-banner/src/fonts.rs
//! ```

use std::collections::HashMap;
use std::sync::LazyLock;

use crate::CharPattern;

/// Glyphs of the `Small` font (3 columns x 3 rows).
///
/// Extracted from `var smallFont` in the Go reference implementation.
#[rustfmt::skip]
pub static SMALL_FONT: LazyLock<HashMap<char, CharPattern>> = LazyLock::new(|| {
    let mut font = HashMap::with_capacity(41);
    font.insert('A', CharPattern { rows: &["▄█▄", "█▀█", "▀ ▀"] });
    font.insert('B', CharPattern { rows: &["██▄", "█▀█", "██▀"] });
    font.insert('C', CharPattern { rows: &["▄██", "█  ", "▀██"] });
    font.insert('D', CharPattern { rows: &["██▄", "█ █", "██▀"] });
    font.insert('E', CharPattern { rows: &["███", "██ ", "███"] });
    font.insert('F', CharPattern { rows: &["███", "██ ", "█  "] });
    font.insert('G', CharPattern { rows: &["▄██", "█▄█", "▀██"] });
    font.insert('H', CharPattern { rows: &["█ █", "███", "█ █"] });
    font.insert('I', CharPattern { rows: &["███", " █ ", "███"] });
    font.insert('J', CharPattern { rows: &["███", "  █", "██▀"] });
    font.insert('K', CharPattern { rows: &["█▄█", "██ ", "█ █"] });
    font.insert('L', CharPattern { rows: &["█  ", "█  ", "███"] });
    font.insert('M', CharPattern { rows: &["█▀█", "███", "█ █"] });
    font.insert('N', CharPattern { rows: &["█▀█", "█▐█", "█ █"] });
    font.insert('O', CharPattern { rows: &["▄█▄", "█ █", "▀█▀"] });
    font.insert('P', CharPattern { rows: &["██▄", "█▀ ", "█  "] });
    font.insert('Q', CharPattern { rows: &["▄█▄", "█ █", "▀█▄"] });
    font.insert('R', CharPattern { rows: &["██▄", "█▀ ", "█ █"] });
    font.insert('S', CharPattern { rows: &["▄██", "▀█▄", "██▀"] });
    font.insert('T', CharPattern { rows: &["███", " █ ", " █ "] });
    font.insert('U', CharPattern { rows: &["█ █", "█ █", "▀█▀"] });
    font.insert('V', CharPattern { rows: &["█ █", "█ █", " █ "] });
    font.insert('W', CharPattern { rows: &["█ █", "███", "█▀█"] });
    font.insert('X', CharPattern { rows: &["█ █", " █ ", "█ █"] });
    font.insert('Y', CharPattern { rows: &["█ █", " █ ", " █ "] });
    font.insert('Z', CharPattern { rows: &["██▄", " █ ", "▀██"] });
    font.insert('0', CharPattern { rows: &["▄█▄", "█ █", "▀█▀"] });
    font.insert('1', CharPattern { rows: &["▄█ ", " █ ", "███"] });
    font.insert('2', CharPattern { rows: &["▄█▄", "▄█▀", "███"] });
    font.insert('3', CharPattern { rows: &["██▄", " █▄", "██▀"] });
    font.insert('4', CharPattern { rows: &["█ █", "███", "  █"] });
    font.insert('5', CharPattern { rows: &["███", "██▄", "▀█▀"] });
    font.insert('6', CharPattern { rows: &["▄██", "██▄", "▀█▀"] });
    font.insert('7', CharPattern { rows: &["███", "  █", "  █"] });
    font.insert('8', CharPattern { rows: &["▄█▄", "▄█▄", "▀█▀"] });
    font.insert('9', CharPattern { rows: &["▄█▄", "▀██", "██▀"] });
    font.insert(' ', CharPattern { rows: &["   ", "   ", "   "] });
    font.insert('!', CharPattern { rows: &[" █ ", " █ ", " ▀ "] });
    font.insert('.', CharPattern { rows: &["   ", "   ", " ▀ "] });
    font.insert('-', CharPattern { rows: &["   ", "▀▀▀", "   "] });
    font.insert(':', CharPattern { rows: &[" ▀ ", "   ", " ▀ "] });
    font
});

/// Glyphs of the `Medium` font (5 columns x 5 rows).
///
/// Extracted from `var mediumFont` in the Go reference implementation.
#[rustfmt::skip]
pub static MEDIUM_FONT: LazyLock<HashMap<char, CharPattern>> = LazyLock::new(|| {
    let mut font = HashMap::with_capacity(41);
    font.insert('A', CharPattern { rows: &["░▄██▄░", "█▀░░█▀", "█████░", "█░░░█░", "▀░░░▀░"] });
    font.insert('B', CharPattern { rows: &["████▄░", "█░░░█░", "████▀░", "█░░░█░", "████▀░"] });
    font.insert('C', CharPattern { rows: &["░▄███░", "█▀░░░░", "█░░░░░", "█▄░░░░", "░▀███░"] });
    font.insert('D', CharPattern { rows: &["████▄░", "█░░░█░", "█░░░█░", "█░░░█░", "████▀░"] });
    font.insert('E', CharPattern { rows: &["█████░", "█░░░░░", "████░░", "█░░░░░", "█████░"] });
    font.insert('F', CharPattern { rows: &["█████░", "█░░░░░", "████░░", "█░░░░░", "█░░░░░"] });
    font.insert('G', CharPattern { rows: &["░▄███░", "█▀░░░░", "█░▄██░", "█░░░█░", "░▀██▀░"] });
    font.insert('H', CharPattern { rows: &["█░░░█░", "█░░░█░", "█████░", "█░░░█░", "█░░░█░"] });
    font.insert('I', CharPattern { rows: &["█████░", "░░█░░░", "░░█░░░", "░░█░░░", "█████░"] });
    font.insert('J', CharPattern { rows: &["█████░", "░░░░█░", "░░░░█░", "█░░░█░", "░███▀░"] });
    font.insert('K', CharPattern { rows: &["█░░▄█░", "█▄█▀░░", "██▀░░░", "█░█▄░░", "█░░▀█░"] });
    font.insert('L', CharPattern { rows: &["█░░░░░", "█░░░░░", "█░░░░░", "█░░░░░", "█████░"] });
    font.insert('M', CharPattern { rows: &["█▀░▀█░", "██░██░", "█░█░█░", "█░░░█░", "█░░░█░"] });
    font.insert('N', CharPattern { rows: &["█▀░░█░", "██░░█░", "█░█░█░", "█░░██░", "█░░▀█░"] });
    font.insert('O', CharPattern { rows: &["░▄█▄░░", "█▀░▀█░", "█░░░█░", "█▄░▄█░", "░▀█▀░░"] });
    font.insert('P', CharPattern { rows: &["████▄░", "█░░░█░", "████▀░", "█░░░░░", "█░░░░░"] });
    font.insert('Q', CharPattern { rows: &["░▄█▄░░", "█▀░▀█░", "█░░░█░", "█▄░██░", "░▀█▄▀░"] });
    font.insert('R', CharPattern { rows: &["████▄░", "█░░░█░", "████▀░", "█░░█░░", "█░░░█░"] });
    font.insert('S', CharPattern { rows: &["░▄███░", "█▀░░░░", "░▀██▄░", "░░░░█░", "████▀░"] });
    font.insert('T', CharPattern { rows: &["█████░", "░░█░░░", "░░█░░░", "░░█░░░", "░░█░░░"] });
    font.insert('U', CharPattern { rows: &["█░░░█░", "█░░░█░", "█░░░█░", "█▄░▄█░", "░▀█▀░░"] });
    font.insert('V', CharPattern { rows: &["█░░░█░", "█░░░█░", "░█░█░░", "░█░█░░", "░░█░░░"] });
    font.insert('W', CharPattern { rows: &["█░░░█░", "█░░░█░", "█░█░█░", "██░██░", "█▀░▀█░"] });
    font.insert('X', CharPattern { rows: &["█░░░█░", "░█░█░░", "░░█░░░", "░█░█░░", "█░░░█░"] });
    font.insert('Y', CharPattern { rows: &["█░░░█░", "░█░█░░", "░░█░░░", "░░█░░░", "░░█░░░"] });
    font.insert('Z', CharPattern { rows: &["█████░", "░░░█▀░", "░░█░░░", "░█░░░░", "█████░"] });
    font.insert('0', CharPattern { rows: &["░▄█▄░░", "█▀░▀█░", "█░░░█░", "█▄░▄█░", "░▀█▀░░"] });
    font.insert('1', CharPattern { rows: &["░▄█░░░", "░░█░░░", "░░█░░░", "░░█░░░", "░███░░"] });
    font.insert('2', CharPattern { rows: &["░▄█▄░░", "▀░░▀█░", "░░▄█▀░", "░█▀░░░", "█████░"] });
    font.insert('3', CharPattern { rows: &["░▄██▄░", "▀░░░█░", "░░██░░", "▄░░░█░", "░▀██▀░"] });
    font.insert('4', CharPattern { rows: &["█░░█░░", "█░░█░░", "█████░", "░░░█░░", "░░░█░░"] });
    font.insert('5', CharPattern { rows: &["█████░", "█░░░░░", "████▄░", "░░░░█░", "████▀░"] });
    font.insert('6', CharPattern { rows: &["░▄██░░", "█▀░░░░", "████▄░", "█░░░█░", "░▀██▀░"] });
    font.insert('7', CharPattern { rows: &["█████░", "░░░░█░", "░░░█░░", "░░█░░░", "░░█░░░"] });
    font.insert('8', CharPattern { rows: &["░▄█▄░░", "█▀░▀█░", "░▄█▄░░", "█▄░▄█░", "░▀█▀░░"] });
    font.insert('9', CharPattern { rows: &["░▄█▄░░", "█░░░█░", "░▀███░", "░░░░█░", "░██▀░░"] });
    font.insert(' ', CharPattern { rows: &["░░░░░░", "░░░░░░", "░░░░░░", "░░░░░░", "░░░░░░"] });
    font.insert('!', CharPattern { rows: &["░░█░░░", "░░█░░░", "░░█░░░", "░░░░░░", "░░▀░░░"] });
    font.insert('.', CharPattern { rows: &["░░░░░░", "░░░░░░", "░░░░░░", "░░░░░░", "░░▀░░░"] });
    font.insert('-', CharPattern { rows: &["░░░░░░", "░░░░░░", "░▀▀▀░░", "░░░░░░", "░░░░░░"] });
    font.insert(':', CharPattern { rows: &["░░░░░░", "░░▀░░░", "░░░░░░", "░░▀░░░", "░░░░░░"] });
    font
});

/// Glyphs of the `Large` font (7 columns x 7 rows).
///
/// Extracted from `var largeFont` in the Go reference implementation.
#[rustfmt::skip]
pub static LARGE_FONT: LazyLock<HashMap<char, CharPattern>> = LazyLock::new(|| {
    let mut font = HashMap::with_capacity(41);
    font.insert('A', CharPattern { rows: &["░░▄███▄░░", "░█▀░░░█▀░", "░█░░░░░█░", "░███████░", "░█░░░░░█░", "░█░░░░░█░", "░▀░░░░░▀░"] });
    font.insert('B', CharPattern { rows: &["░█████▄░░", "░█░░░░█░░", "░█████▀░░", "░█░░░░█░░", "░█░░░░█░░", "░█████▀░░", "░░░░░░░░░"] });
    font.insert('C', CharPattern { rows: &["░░▄████░░", "░█▀░░░░░░", "░█░░░░░░░", "░█░░░░░░░", "░█░░░░░░░", "░█▄░░░░░░", "░░▀████░░"] });
    font.insert('D', CharPattern { rows: &["░█████▄░░", "░█░░░░█▄░", "░█░░░░░█░", "░█░░░░░█░", "░█░░░░░█░", "░█░░░░█▀░", "░█████▀░░"] });
    font.insert('E', CharPattern { rows: &["░██████░░", "░█░░░░░░░", "░█░░░░░░░", "░█████░░░", "░█░░░░░░░", "░█░░░░░░░", "░██████░░"] });
    font.insert('F', CharPattern { rows: &["░██████░░", "░█░░░░░░░", "░█░░░░░░░", "░█████░░░", "░█░░░░░░░", "░█░░░░░░░", "░█░░░░░░░"] });
    font.insert('G', CharPattern { rows: &["░░▄████░░", "░█▀░░░░░░", "░█░░░░░░░", "░█░░▄██░░", "░█░░░░█░░", "░█▄░░░█░░", "░░▀███▀░░"] });
    font.insert('H', CharPattern { rows: &["░█░░░░█░░", "░█░░░░█░░", "░█░░░░█░░", "░██████░░", "░█░░░░█░░", "░█░░░░█░░", "░█░░░░█░░"] });
    font.insert('I', CharPattern { rows: &["░██████░░", "░░░█░░░░░", "░░░█░░░░░", "░░░█░░░░░", "░░░█░░░░░", "░░░█░░░░░", "░██████░░"] });
    font.insert('J', CharPattern { rows: &["░██████░░", "░░░░░░█░░", "░░░░░░█░░", "░░░░░░█░░", "░░░░░░█░░", "░█░░░░█░░", "░░████▀░░"] });
    font.insert('K', CharPattern { rows: &["░█░░░▄█░░", "░█░░█▀░░░", "░█▄█▀░░░░", "░██▀░░░░░", "░█░█▄░░░░", "░█░░▀█░░░", "░█░░░▀█░░"] });
    font.insert('L', CharPattern { rows: &["░█░░░░░░░", "░█░░░░░░░", "░█░░░░░░░", "░█░░░░░░░", "░█░░░░░░░", "░█░░░░░░░", "░██████░░"] });
    font.insert('M', CharPattern { rows: &["░█▀░░░▀█░", "░██▄░▄██░", "░█░███░█░", "░█░░█░░█░", "░█░░░░░█░", "░█░░░░░█░", "░█░░░░░█░"] });
    font.insert('N', CharPattern { rows: &["░█▀░░░░█░", "░██▄░░░█░", "░█░█▄░░█░", "░█░░█░░█░", "░█░░▀█░█░", "░█░░░▀██░", "░█░░░░▀█░"] });
    font.insert('O', CharPattern { rows: &["░░▄███▄░░", "░█▀░░░▀█░", "░█░░░░░█░", "░█░░░░░█░", "░█░░░░░█░", "░█▄░░░▄█░", "░░▀███▀░░"] });
    font.insert('P', CharPattern { rows: &["░█████▄░░", "░█░░░░█░░", "░█░░░░█░░", "░█████▀░░", "░█░░░░░░░", "░█░░░░░░░", "░█░░░░░░░"] });
    font.insert('Q', CharPattern { rows: &["░░▄███▄░░", "░█▀░░░▀█░", "░█░░░░░█░", "░█░░░░░█░", "░█░░░█░█░", "░█▄░░▀██░", "░░▀███▄▀░"] });
    font.insert('R', CharPattern { rows: &["░█████▄░░", "░█░░░░█░░", "░█░░░░█░░", "░█████▀░░", "░█░░█░░░░", "░█░░░█░░░", "░█░░░░█░░"] });
    font.insert('S', CharPattern { rows: &["░░▄████░░", "░█▀░░░░░░", "░░█▄░░░░░", "░░░▀██▄░░", "░░░░░░█░░", "░░░░░▄█░░", "░████▀░░░"] });
    font.insert('T', CharPattern { rows: &["░███████░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░"] });
    font.insert('U', CharPattern { rows: &["░█░░░░█░░", "░█░░░░█░░", "░█░░░░█░░", "░█░░░░█░░", "░█░░░░█░░", "░█▄░░▄█░░", "░░▀██▀░░░"] });
    font.insert('V', CharPattern { rows: &["░█░░░░░█░", "░█░░░░░█░", "░░█░░░█░░", "░░█░░░█░░", "░░░█░█░░░", "░░░█░█░░░", "░░░░█░░░░"] });
    font.insert('W', CharPattern { rows: &["░█░░░░░█░", "░█░░░░░█░", "░█░░░░░█░", "░█░░█░░█░", "░█░███░█░", "░██▀░▀██░", "░█▀░░░▀█░"] });
    font.insert('X', CharPattern { rows: &["░█░░░░░█░", "░░█░░░█░░", "░░░█░█░░░", "░░░░█░░░░", "░░░█░█░░░", "░░█░░░█░░", "░█░░░░░█░"] });
    font.insert('Y', CharPattern { rows: &["░█░░░░░█░", "░░█░░░█░░", "░░░█░█░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░"] });
    font.insert('Z', CharPattern { rows: &["░███████░", "░░░░░░█░░", "░░░░░█░░░", "░░░░█░░░░", "░░░█░░░░░", "░░█░░░░░░", "░███████░"] });
    font.insert('0', CharPattern { rows: &["░░▄███▄░░", "░█▀░░░▀█░", "░█░░░░░█░", "░█░░░░░█░", "░█░░░░░█░", "░█▄░░░▄█░", "░░▀███▀░░"] });
    font.insert('1', CharPattern { rows: &["░░░▄█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░████░░░"] });
    font.insert('2', CharPattern { rows: &["░░▄███▄░░", "░▀░░░░▀█░", "░░░░░░░█░", "░░░░▄██▀░", "░░▄█▀░░░░", "░█▀░░░░░░", "░███████░"] });
    font.insert('3', CharPattern { rows: &["░░▄███▄░░", "░▀░░░░▀█░", "░░░░░░░█░", "░░░░██▀░░", "░░░░░░░█░", "░▄░░░░▄█░", "░░▀███▀░░"] });
    font.insert('4', CharPattern { rows: &["░█░░░░█░░", "░█░░░░█░░", "░█░░░░█░░", "░███████░", "░░░░░░█░░", "░░░░░░█░░", "░░░░░░█░░"] });
    font.insert('5', CharPattern { rows: &["░███████░", "░█░░░░░░░", "░█░░░░░░░", "░██████▄░", "░░░░░░░█░", "░░░░░░▄█░", "░█████▀░░"] });
    font.insert('6', CharPattern { rows: &["░░▄████░░", "░█▀░░░░░░", "░█░░░░░░░", "░██████▄░", "░█░░░░░█░", "░█▄░░░▄█░", "░░▀███▀░░"] });
    font.insert('7', CharPattern { rows: &["░███████░", "░░░░░░░█░", "░░░░░░█░░", "░░░░░█░░░", "░░░░█░░░░", "░░░█░░░░░", "░░░█░░░░░"] });
    font.insert('8', CharPattern { rows: &["░░▄███▄░░", "░█▀░░░▀█░", "░█▄░░░▄█░", "░░▄███▄░░", "░█▀░░░▀█░", "░█▄░░░▄█░", "░░▀███▀░░"] });
    font.insert('9', CharPattern { rows: &["░░▄███▄░░", "░█░░░░░█░", "░█▄░░░▄█░", "░░▀████▀░", "░░░░░░░█░", "░░░░░░█░░", "░░████▀░░"] });
    font.insert(' ', CharPattern { rows: &["░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░"] });
    font.insert('!', CharPattern { rows: &["░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░█░░░░", "░░░░░░░░░", "░░░░▀░░░░"] });
    font.insert('.', CharPattern { rows: &["░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░▀░░░░"] });
    font.insert('-', CharPattern { rows: &["░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░", "░░▀▀▀▀░░░", "░░░░░░░░░", "░░░░░░░░░", "░░░░░░░░░"] });
    font.insert(':', CharPattern { rows: &["░░░░░░░░░", "░░░░░░░░░", "░░░░▀░░░░", "░░░░░░░░░", "░░░░▀░░░░", "░░░░░░░░░", "░░░░░░░░░"] });
    font
});
