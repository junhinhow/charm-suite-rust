//! Gerador de `src/spinners.rs` a partir da implementacao Go de referencia.
//!
//! Este script le `reference/go/charm-spinner/spinner.go` (UTF-8), extrai os
//! `Frames` e o `Interval` de cada um dos 42 spinners e emite o codigo Rust
//! equivalente.
//!
//! Os frames **nunca** sao digitados a mao: o script faz o parsing dos
//! literais Go, decodifica os escapes e converte cada string para uma
//! sequencia de code points Unicode. O arquivo gerado usa escapes
//! `\u{...}` (ASCII puro), portanto nao existe risco de mojibake no proprio
//! fonte Rust: o compilador reproduz exatamente os mesmos bytes UTF-8 do Go.
//!
//! Uso:
//!
//! ```text
//! rustc --edition 2021 scripts/generate_spinners.rs -o %TEMP%\generate_spinners.exe
//! %TEMP%\generate_spinners.exe reference/go/charm-spinner/spinner.go crates/charm-spinner/src/spinners.rs
//! ```

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::process::ExitCode;

/// Largura maxima de linha, igual ao `max_width` do `rustfmt.toml`.
const MAX_WIDTH: usize = 100;

/// Largura maxima de uma lista antes do layout vertical, igual ao
/// `array_width` padrao do `rustfmt` (60% de `max_width`).
const ARRAY_WIDTH: usize = 60;

/// Quantidade de spinners predefinidos no fonte Go.
const EXPECTED_SPINNERS: usize = 42;

/// Dados de um spinner extraidos do fonte Go.
struct SpinnerData {
    /// Nome do identificador Go (`Dots`).
    go_name: String,
    /// Frames ja decodificadas (texto puro, sem escapes).
    frames: Vec<String>,
    /// Intervalo entre frames, em milissegundos.
    interval_ms: u64,
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("uso: generate_spinners <spinner.go> <saida.rs>");
        return ExitCode::FAILURE;
    }

    let source = match fs::read_to_string(&args[1]) {
        Ok(text) => text,
        Err(err) => {
            eprintln!("falha ao ler `{}`: {err}", args[1]);
            return ExitCode::FAILURE;
        }
    };

    let spinners = match parse(&source) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("falha ao parsear `{}`: {err}", args[1]);
            return ExitCode::FAILURE;
        }
    };

    let code = generate(&spinners);
    if let Err(err) = fs::write(&args[2], code.as_bytes()) {
        eprintln!("falha ao escrever `{}`: {err}", args[2]);
        return ExitCode::FAILURE;
    }

    let total_frames: usize = spinners.iter().map(|s| s.frames.len()).sum();
    eprintln!(
        "gerado: {} spinners, {total_frames} frames -> {}",
        spinners.len(),
        args[2]
    );
    ExitCode::SUCCESS
}

/// Extrai os 42 spinners do fonte Go.
fn parse(source: &str) -> Result<Vec<SpinnerData>, String> {
    let mut spinners: Vec<SpinnerData> = Vec::new();
    let mut current: Option<SpinnerData> = None;

    for (number, line) in source.lines().enumerate() {
        let text = line.trim();

        if let Some(rest) = text.strip_suffix("= Spinner{") {
            if current.is_some() {
                return Err(format!("linha {number}: spinner anterior nao fechado"));
            }
            current = Some(SpinnerData {
                go_name: rest.trim().to_owned(),
                frames: Vec::new(),
                interval_ms: 0,
            });
            continue;
        }

        let Some(entry) = current.as_mut() else {
            continue;
        };

        if let Some(value) = text.strip_prefix("Frames:") {
            let value = value.trim().trim_end_matches(',').trim();
            let inner = value
                .strip_prefix("[]string{")
                .ok_or_else(|| format!("linha {number}: formato de Frames inesperado: {value}"))?;
            let inner = inner
                .strip_suffix('}')
                .ok_or_else(|| format!("linha {number}: lista de Frames nao fechada"))?;
            entry.frames =
                parse_go_string_list(inner).map_err(|err| format!("linha {number}: {err}"))?;
            continue;
        }

        if let Some(value) = text.strip_prefix("Interval:") {
            let value = value.trim().trim_end_matches(',').trim();
            let number = value.strip_suffix("* time.Millisecond").ok_or_else(|| {
                format!("linha {number}: formato de Interval inesperado: {value}")
            })?;
            entry.interval_ms = number
                .trim()
                .parse::<u64>()
                .map_err(|err| format!("linha {number}: intervalo invalido `{number}`: {err}"))?;
            continue;
        }

        if text == "}" {
            let entry = current.take().expect("entrada corrente");
            if entry.frames.is_empty() {
                return Err(format!("spinner `{}` sem frames", entry.go_name));
            }
            if entry.interval_ms == 0 {
                return Err(format!("spinner `{}` sem intervalo", entry.go_name));
            }
            spinners.push(entry);
            continue;
        }
    }

    if current.is_some() {
        return Err("bloco de spinner nao fechado ao fim do arquivo".to_owned());
    }
    if spinners.len() != EXPECTED_SPINNERS {
        return Err(format!(
            "esperados {EXPECTED_SPINNERS} spinners, encontrados {}",
            spinners.len()
        ));
    }
    Ok(spinners)
}

/// Converte o conteudo de um `[]string{...}` Go em vetor de strings.
fn parse_go_string_list(inner: &str) -> Result<Vec<String>, String> {
    let chars: Vec<char> = inner.chars().collect();
    let mut index = 0usize;
    let mut frames = Vec::new();

    loop {
        while index < chars.len() && (chars[index].is_whitespace() || chars[index] == ',') {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }
        if chars[index] != '"' {
            return Err(format!(
                "esperado literal de string na posicao {index}, encontrado `{}`",
                chars[index]
            ));
        }
        index += 1;

        let mut body = String::new();
        loop {
            if index >= chars.len() {
                return Err("literal de string nao terminado".to_owned());
            }
            let current = chars[index];
            if current == '"' {
                index += 1;
                break;
            }
            if current == '\\' {
                index += 1;
                if index >= chars.len() {
                    return Err("escape nao terminado".to_owned());
                }
                let escaped = chars[index];
                index += 1;
                match escaped {
                    'n' => body.push('\n'),
                    't' => body.push('\t'),
                    'r' => body.push('\r'),
                    'a' => body.push('\u{7}'),
                    'b' => body.push('\u{8}'),
                    'f' => body.push('\u{c}'),
                    'v' => body.push('\u{b}'),
                    '\\' => body.push('\\'),
                    '\'' => body.push('\''),
                    '"' => body.push('"'),
                    'x' => {
                        let (value, used) = read_radix(&chars, index, 2, 16)?;
                        index += used;
                        body.push(char::from_u32(value).ok_or("escape \\x invalido")?);
                    }
                    'u' => {
                        let (value, used) = read_radix(&chars, index, 4, 16)?;
                        index += used;
                        body.push(char::from_u32(value).ok_or("escape \\u invalido")?);
                    }
                    'U' => {
                        let (value, used) = read_radix(&chars, index, 8, 16)?;
                        index += used;
                        body.push(char::from_u32(value).ok_or("escape \\U invalido")?);
                    }
                    '0'..='7' => {
                        let mut value = escaped.to_digit(8).expect("digito octal");
                        let mut used = 0;
                        while used < 2 && index < chars.len() && chars[index].is_digit(8) {
                            value = value * 8 + chars[index].to_digit(8).expect("digito octal");
                            index += 1;
                            used += 1;
                        }
                        body.push(char::from_u32(value).ok_or("escape octal invalido")?);
                    }
                    other => return Err(format!("escape desconhecido: \\{other}")),
                }
                continue;
            }
            body.push(current);
            index += 1;
        }
        frames.push(body);
    }

    Ok(frames)
}

/// Le `count` digitos na base `radix` a partir de `index`.
fn read_radix(
    chars: &[char],
    index: usize,
    count: usize,
    radix: u32,
) -> Result<(u32, usize), String> {
    if index + count > chars.len() {
        return Err("escape numerico truncado".to_owned());
    }
    let mut value = 0u32;
    for offset in 0..count {
        let digit = chars[index + offset].to_digit(radix).ok_or_else(|| {
            format!(
                "digito invalido em escape numerico: `{}`",
                chars[index + offset]
            )
        })?;
        value = value * radix + digit;
    }
    Ok((value, count))
}

/// Emite o arquivo Rust com os 42 spinners.
fn generate(spinners: &[SpinnerData]) -> String {
    let mut out = String::new();

    out.push_str(
        "//! Predefined spinner animations extracted from the Go `charm-spinner` package.\n\
//!\n\
//! This file is **machine generated** by `scripts/generate_spinners.rs` from\n\
//! `reference/go/charm-spinner/spinner.go`. Do not edit it by hand.\n\
//!\n\
//! Every frame is stored as a `\\u{...}` escape, so this source file is pure\n\
//! ASCII and can never be damaged by an encoding round-trip (mojibake). The\n\
//! compiled strings are byte-identical to the Go source; the parity tests in\n\
//! `tests/parity.rs` prove it against the original implementation.\n\
\n",
    );
    out.push_str("use std::collections::BTreeMap;\n");
    out.push_str("use std::time::Duration;\n\n");
    out.push_str("use crate::Spinner;\n\n");

    for spinner in spinners {
        let rust_name = spinner.go_name.to_lowercase();
        let code_points = spinner
            .frames
            .iter()
            .map(|frame| {
                frame
                    .chars()
                    .map(|ch| format!("{:04X}", ch as u32))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect::<Vec<_>>()
            .join(" / ");

        let _ = writeln!(
            out,
            "/// The `{}` spinner: {} frames at {} ms per frame.",
            spinner.go_name,
            spinner.frames.len(),
            spinner.interval_ms
        );
        out.push_str("///\n");
        let _ = writeln!(out, "/// Frames (Unicode code points): {code_points}");
        let _ = writeln!(out, "pub fn {rust_name}() -> Spinner {{");
        out.push_str("    Spinner::new(\n");

        // O layout segue o mesmo criterio do `rustfmt`: uma unica linha
        // quando a lista cabe em `ARRAY_WIDTH`, um elemento por linha caso
        // contrario (mantendo o arquivo gerado identico ao `cargo fmt`).
        let items: Vec<String> = spinner
            .frames
            .iter()
            .map(|frame| format!("{}.to_owned()", rust_string(frame)))
            .collect();
        let items_width: usize =
            items.iter().map(String::len).sum::<usize>() + 2 * items.len().saturating_sub(1);
        let single_line = format!("        vec![{}],", items.join(", "));
        if items_width <= ARRAY_WIDTH && single_line.len() <= MAX_WIDTH {
            out.push_str(&single_line);
            out.push('\n');
        } else {
            out.push_str("        vec![\n");
            for item in &items {
                let _ = writeln!(out, "            {item},");
            }
            out.push_str("        ],\n");
        }

        let _ = writeln!(
            out,
            "        Duration::from_millis({}),\n    )\n}}\n",
            spinner.interval_ms
        );
    }

    out.push_str("/// Returns every predefined spinner, keyed by its display name.\n");
    out.push_str("///\n");
    out.push_str("/// The keys are exactly the ones used by the Go implementation\n");
    out.push_str("/// (`\"Dots\"`, `\"Line\"`, `\"Star\"`, ...), and the iteration order is\n");
    out.push_str("/// alphabetical, matching Go's `sort.Strings` ordering.\n");
    out.push_str("///\n");
    out.push_str("/// # Examples\n");
    out.push_str("///\n");
    out.push_str("/// ```\n");
    out.push_str("/// let spinners = charm_spinner::all();\n");
    out.push_str("/// assert_eq!(spinners.len(), 42);\n");
    out.push_str("/// assert!(spinners.contains_key(\"Dots\"));\n");
    out.push_str("/// ```\n");
    out.push_str("pub fn all() -> BTreeMap<&'static str, Spinner> {\n");
    out.push_str("    let mut spinners = BTreeMap::new();\n");
    for spinner in spinners {
        let _ = writeln!(
            out,
            "    spinners.insert(\"{}\", {}());",
            spinner.go_name,
            spinner.go_name.to_lowercase()
        );
    }
    out.push_str("    spinners\n}\n");

    out
}

/// Converte uma string em um literal Rust ASCII usando escapes `\\u{...}`.
fn rust_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        let _ = write!(out, "\\u{{{:04X}}}", ch as u32);
    }
    out.push('"');
    out
}
