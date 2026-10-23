//! Programmatic extraction of the bitmap fonts from the Go reference
//! implementation of `charm-banner`.
//!
//! The Go source (`reference/go/charm-banner/banner.go`) stores the glyphs as
//! Unicode block patterns. Copying them by hand is a mojibake hazard, so this
//! script parses the Go file (UTF-8) and emits `src/fonts.rs`.
//!
//! Usage (from the workspace root):
//!
//! ```text
//! rustc --edition 2021 crates/charm-banner/xtask/extract_fonts.rs -o extract_fonts.exe
//! ./extract_fonts.exe reference/go/charm-banner/banner.go crates/charm-banner/src/fonts.rs
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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() != 2 {
        eprintln!("usage: extract_fonts <banner.go> <fonts.rs>");
        std::process::exit(2);
    }
    if let Err(e) = run(&args[0], &args[1]) {
        eprintln!("erro: {e}");
        std::process::exit(1);
    }
}

fn run(go_path: &str, out_path: &str) -> Result<(), String> {
    let source = std::fs::read_to_string(go_path)
        .map_err(|e| format!("nao foi possivel ler `{go_path}`: {e}"))?;

    let mut out = String::new();
    write_header(&mut out);

    for spec in FONTS {
        let glyphs = parse_font(&source, spec)?;
        write_font(&mut out, spec, &glyphs)?;
    }

    std::fs::write(out_path, out.as_bytes())
        .map_err(|e| format!("nao foi possivel escrever `{out_path}`: {e}"))?;
    println!("`{out_path}` gerado a partir de `{go_path}`");
    Ok(())
}

fn write_header(out: &mut String) {
    out.push_str(
        "//! Glyph data of the three banner fonts.\n\
         //!\n\
         //! **Generated file - do not edit by hand.**\n\
         //!\n\
         //! Every pattern below was extracted programmatically from the Go\n\
         //! reference implementation (`reference/go/charm-banner/banner.go`) by\n\
         //! `crates/charm-banner/xtask/extract_fonts.rs`, so the data is\n\
         //! byte-identical to the original Go fonts.\n\
         //!\n\
         //! Regenerate with:\n\
         //!\n\
         //! ```text\n\
         //! rustc --edition 2021 crates/charm-banner/xtask/extract_fonts.rs -o extract_fonts.exe\n\
         //! ./extract_fonts.exe reference/go/charm-banner/banner.go crates/charm-banner/src/fonts.rs\n\
         //! ```\n\
         \n\
         use std::collections::HashMap;\n\
         use std::sync::LazyLock;\n\
         \n\
         use crate::CharPattern;\n\
         \n",
    );
}

fn write_font(
    out: &mut String,
    spec: &FontSpec,
    glyphs: &[(char, Vec<String>)],
) -> Result<(), String> {
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
            "    font.insert({key}, CharPattern {{ rows: &[{rows}] }});",
            key = format!("{key:?}"),
            rows = row_literals
        );
    }
    let _ = writeln!(out, "    font");
    let _ = writeln!(out, "}});");
    let _ = writeln!(out);
    Ok(())
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
    }
    Ok(glyphs)
}

/// Parses a single entry such as `'A': {[]string{"▄█▄", "█▀█", "▀ ▀"}},`.
fn parse_entry(line: &str) -> Result<(char, Vec<String>), String> {
    let rest = line
        .strip_prefix('\'')
        .ok_or_else(|| format!("linha de glifo inesperada: `{line}`"))?;
    let end = rest
        .find('\'')
        .ok_or_else(|| format!("chave de glifo invalida: `{line}`"))?;
    let key = rest[..end]
        .chars()
        .next()
        .ok_or_else(|| format!("chave de glifo vazia: `{line}`"))?;

    let after = &rest[end + 1..];
    let open = after
        .find('{')
        .ok_or_else(|| format!("literal de glifo invalido: `{line}`"))?;
    let rows = scan_go_strings(&after[open + 1..]);
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
