//! Programmatic extraction of the bitmap fonts and of the accent table from
//! the Go reference implementation of `charm-banner`.
//!
//! The Go sources store the glyphs as Unicode block patterns (`fonts.go`) and
//! the accent table as a Go map (`banner.go`). Copying them by hand is a
//! mojibake hazard, so this script parses the Go files (UTF-8) and emits
//! `src/fonts.rs` and `src/accents.rs`.
//!
//! Usage (from the workspace root):
//!
//! ```text
//! rustc --edition 2021 crates/charm-banner/xtask/extract_fonts.rs -o extract_fonts.exe
//! ./extract_fonts.exe reference/go/charm-banner crates/charm-banner/src
//! ```
//!
//! The script is a build-time helper: it is not part of the published library
//! and is never compiled by Cargo.

use std::fmt::Write as _;

/// A font declared in the Go source: Go variable name, Rust static name and
/// the expected geometry (columns x rows).
struct FontSpec {
    go_var: &'static str,
    rust_static: &'static str,
    label: &'static str,
    width: usize,
    height: usize,
}

const FONTS: &[FontSpec] = &[
    FontSpec {
        go_var: "smallFont",
        rust_static: "SMALL_FONT",
        label: "Small",
        width: 3,
        height: 3,
    },
    FontSpec {
        go_var: "mediumFont",
        rust_static: "MEDIUM_FONT",
        label: "Medium",
        width: 5,
        height: 5,
    },
    FontSpec {
        go_var: "largeFont",
        rust_static: "LARGE_FONT",
        label: "Large",
        width: 7,
        height: 7,
    },
];

/// Go map that associates accented letters with their base letter.
const BASE_RUNES_VAR: &str = "var baseRunes = map[rune]rune{";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: extract_fonts <reference/go/charm-banner> <crates/charm-banner/src>");
        std::process::exit(2);
    }
    if let Err(e) = run(&args[0], &args[1]) {
        eprintln!("erro: {e}");
        std::process::exit(1);
    }
}

fn run(go_dir: &str, src_dir: &str) -> Result<(), String> {
    let fonts_path = format!("{go_dir}/fonts.go");
    let banner_path = format!("{go_dir}/banner.go");
    let fonts_source = read(&fonts_path)?;
    let banner_source = read(&banner_path)?;

    let mut fonts_out = String::new();
    write_fonts_header(&mut fonts_out);
    for spec in FONTS {
        let glyphs = parse_font(&fonts_source, spec)?;
        write_font(&mut fonts_out, spec, &glyphs);
    }
    let fonts_file = format!("{src_dir}/fonts.rs");
    write(&fonts_file, &fonts_out)?;

    let mut accents = parse_base_runes(&banner_source)?;
    accents.sort_by_key(|&(key, _)| key);
    let mut accents_out = String::new();
    write_accents(&mut accents_out, &accents);
    let accents_file = format!("{src_dir}/accents.rs");
    write(&accents_file, &accents_out)?;

    println!("`{fonts_file}` e `{accents_file}` gerados a partir de `{go_dir}`");
    Ok(())
}

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("nao foi possivel ler `{path}`: {e}"))
}

fn write(path: &str, content: &str) -> Result<(), String> {
    std::fs::write(path, content.as_bytes())
        .map_err(|e| format!("nao foi possivel escrever `{path}`: {e}"))
}

fn write_fonts_header(out: &mut String) {
    out.push_str(
        "//! Glyph data of the three banner fonts.\n\
         //!\n\
         //! **Generated file - do not edit by hand.**\n\
         //!\n\
         //! Every pattern below was extracted programmatically from the Go\n\
         //! reference implementation (`reference/go/charm-banner/fonts.go`) by\n\
         //! `crates/charm-banner/xtask/extract_fonts.rs`, so the data is\n\
         //! byte-identical to the original Go fonts.\n\
         //!\n\
         //! Regenerate with:\n\
         //!\n\
         //! ```text\n\
         //! rustc --edition 2021 crates/charm-banner/xtask/extract_fonts.rs -o extract_fonts.exe\n\
         //! ./extract_fonts.exe reference/go/charm-banner crates/charm-banner/src\n\
         //! ```\n\
         \n\
         use std::collections::HashMap;\n\
         use std::sync::LazyLock;\n\
         \n\
         use crate::CharPattern;\n\
         \n",
    );
}

fn write_font(out: &mut String, spec: &FontSpec, glyphs: &[(char, Vec<String>)]) {
    let _ = writeln!(
        out,
        "/// Glyphs of the `{label}` font ({width} columns x {height} rows).",
        label = spec.label,
        width = spec.width,
        height = spec.height
    );
    let _ = writeln!(out, "///");
    let _ = writeln!(
        out,
        "/// Extracted from `var {go_var}` in the Go reference implementation.",
        go_var = spec.go_var
    );
    // The glyphs are emitted one per line, exactly as they are in the Go
    // source; `rustfmt::skip` keeps the generated file untouched.
    let _ = writeln!(out, "#[rustfmt::skip]");
    let _ = writeln!(
        out,
        "pub static {rust_static}: LazyLock<HashMap<char, CharPattern>> = LazyLock::new(|| {{",
        rust_static = spec.rust_static
    );
    let _ = writeln!(
        out,
        "    let mut font = HashMap::with_capacity({cap});",
        cap = glyphs.len()
    );
    for (key, rows) in glyphs {
        let row_literals = rows
            .iter()
            .map(|row| format!("{row:?}"))
            .collect::<Vec<_>>()
            .join(", ");
        let _ = writeln!(
            out,
            "    font.insert({key:?}, CharPattern {{ rows: &[{row_literals}] }});",
        );
    }
    let _ = writeln!(out, "    font");
    let _ = writeln!(out, "}});");
    let _ = writeln!(out);
}

fn write_accents(out: &mut String, accents: &[(char, char)]) {
    out.push_str(
        "//! Base letters of the accented characters.\n\
         //!\n\
         //! **Generated file - do not edit by hand.**\n\
         //!\n\
         //! Extracted programmatically from `var baseRunes` in the Go reference\n\
         //! implementation (`reference/go/charm-banner/banner.go`) by\n\
         //! `crates/charm-banner/xtask/extract_fonts.rs`.\n\
         //!\n\
         //! Regenerate with:\n\
         //!\n\
         //! ```text\n\
         //! rustc --edition 2021 crates/charm-banner/xtask/extract_fonts.rs -o extract_fonts.exe\n\
         //! ./extract_fonts.exe reference/go/charm-banner crates/charm-banner/src\n\
         //! ```\n\
         \n\
         /// Accented letters and their base letters, sorted by key.\n\
         #[rustfmt::skip]\n\
         pub(crate) static BASE_LETTERS: &[(char, char)] = &[\n",
    );
    for (key, base) in accents {
        let _ = writeln!(out, "    ({key:?}, {base:?}),");
    }
    out.push_str("];\n");
}

/// Parses a single `map[rune]charPattern{...}` literal from the Go source.
fn parse_font(source: &str, spec: &FontSpec) -> Result<Vec<(char, Vec<String>)>, String> {
    let marker = format!("var {} = map[rune]charPattern{{", spec.go_var);
    let start = source
        .find(&marker)
        .ok_or_else(|| format!("nao encontrei `{marker}`"))?;
    let body_start = start + marker.len();

    let mut glyphs: Vec<(char, Vec<String>)> = Vec::new();
    let mut rest = &source[body_start..];

    loop {
        let (line, tail) = match rest.split_once('\n') {
            Some((l, t)) => (l, t),
            None => (rest, ""),
        };
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }
        if !trimmed.is_empty() {
            glyphs.push(parse_entry(trimmed)?);
        }
        if tail.is_empty() {
            break;
        }
        rest = tail;
    }

    if glyphs.is_empty() {
        return Err(format!("fonte `{}` vazia", spec.go_var));
    }
    for (key, rows) in &glyphs {
        if rows.len() != spec.height {
            return Err(format!(
                "glifo `{key}` de `{}` tem {} linhas, esperado {}",
                spec.go_var,
                rows.len(),
                spec.height
            ));
        }
        if let Some(row) = rows.iter().find(|row| row.chars().count() != spec.width) {
            return Err(format!(
                "glifo `{key}` de `{}` tem a linha `{row}` fora da largura {}",
                spec.go_var, spec.width
            ));
        }
    }
    Ok(glyphs)
}

/// Parses the `var baseRunes = map[rune]rune{...}` literal of `banner.go`.
///
/// The entries are written as `'Á': 'A', 'À': 'A', ...`, possibly several per
/// line, so the parser collects every rune literal of the body in order and
/// pairs them up. Comment lines are skipped.
fn parse_base_runes(source: &str) -> Result<Vec<(char, char)>, String> {
    let start = source
        .find(BASE_RUNES_VAR)
        .ok_or_else(|| format!("nao encontrei `{BASE_RUNES_VAR}`"))?;
    let body = &source[start + BASE_RUNES_VAR.len()..];

    let mut runes: Vec<char> = Vec::new();
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed == "}" {
            break;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        let mut chars = trimmed.chars();
        while let Some(c) = chars.next() {
            if c != '\'' {
                continue;
            }
            let rune = chars
                .next()
                .ok_or_else(|| format!("literal de runa invalido: `{trimmed}`"))?;
            if rune == '\\' {
                return Err(format!("escape inesperado em `baseRunes`: `{trimmed}`"));
            }
            if chars.next() != Some('\'') {
                return Err(format!("literal de runa invalido: `{trimmed}`"));
            }
            runes.push(rune);
        }
    }

    if runes.is_empty() || runes.len() % 2 != 0 {
        return Err(format!(
            "`baseRunes` com {} runas, esperado um numero par e nao vazio",
            runes.len()
        ));
    }
    Ok(runes.chunks_exact(2).map(|pair| (pair[0], pair[1])).collect())
}

/// Parses a single entry such as `'A': {[]string{"▄█▄", "█▀█", "▀ ▀"}},`.
///
/// The key may be an escaped rune (`'\''` or `'\\'`).
fn parse_entry(line: &str) -> Result<(char, Vec<String>), String> {
    let body = line
        .strip_prefix('\'')
        .ok_or_else(|| format!("linha de glifo inesperada: `{line}`"))?;
    let mut chars = body.chars();
    let first = chars
        .next()
        .ok_or_else(|| format!("chave de glifo vazia: `{line}`"))?;
    let key = if first == '\\' {
        chars
            .next()
            .ok_or_else(|| format!("chave de glifo invalida: `{line}`"))?
    } else {
        first
    };
    let after_key = chars
        .as_str()
        .strip_prefix('\'')
        .ok_or_else(|| format!("chave de glifo invalida: `{line}`"))?;

    let open = after_key
        .find('{')
        .ok_or_else(|| format!("literal de glifo invalido: `{line}`"))?;
    let rows = scan_go_strings(&after_key[open + 1..]);
    if rows.is_empty() {
        return Err(format!("glifo `{key}` sem linhas: `{line}`"));
    }
    Ok((key, rows))
}

/// Extracts the Go string literals of a source fragment, honouring escapes.
fn scan_go_strings(fragment: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = fragment.chars();
    while let Some(c) = chars.next() {
        if c != '"' {
            continue;
        }
        let mut literal = String::new();
        while let Some(c) = chars.next() {
            match c {
                '"' => break,
                '\\' => {
                    let escape = chars.next().unwrap_or('\\');
                    match escape {
                        'n' => literal.push('\n'),
                        't' => literal.push('\t'),
                        'r' => literal.push('\r'),
                        '\\' => literal.push('\\'),
                        '"' => literal.push('"'),
                        '\'' => literal.push('\''),
                        'u' => {
                            let mut hex = String::new();
                            for _ in 0..4 {
                                if let Some(h) = chars.next() {
                                    hex.push(h);
                                }
                            }
                            match u32::from_str_radix(&hex, 16).ok().and_then(char::from_u32) {
                                Some(ch) => literal.push(ch),
                                None => literal.push('\u{fffd}'),
                            }
                        }
                        other => {
                            literal.push('\\');
                            literal.push(other);
                        }
                    }
                }
                other => literal.push(other),
            }
        }
        out.push(literal);
    }
    out
}
