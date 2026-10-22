//! `charm-table`: renders CSV, TSV, JSON and JSON Lines as styled terminal tables.
//!
//! This is a port of the Go `charm-table-cli` command. The table is written to
//! stdout; errors go to stderr with exit code 1.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod parse;
mod render;

use std::fs::File;
use std::io::{self, IsTerminal, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use thiserror::Error;

use crate::render::{BorderStyle, RenderOptions};

/// Command-line options. The flags mirror the Go `flag` set.
#[derive(Debug, Parser)]
#[command(
    name = "charm-table",
    version,
    about = "Renderiza CSV, TSV, JSON e JSONL como tabelas estilizadas no terminal."
)]
struct Cli {
    /// Estilo da borda: rounded, thick, double, ascii.
    #[arg(long, value_enum, default_value = "rounded")]
    style: BorderStyle,

    /// Primeira linha como cabecalho (padrao).
    #[arg(long, overrides_with = "no_header")]
    header: bool,

    /// Gera cabecalhos Col1, Col2, ... em vez de usar a primeira linha.
    #[arg(long = "no-header", overrides_with = "header")]
    no_header: bool,

    /// Largura maxima da tabela (0 = sem limite).
    #[arg(
        long,
        value_name = "N",
        default_value_t = 0,
        allow_negative_numbers = true
    )]
    max_width: i64,

    /// Cor do cabecalho: numero ANSI ou hex (ex.: 63 ou #ff0000).
    #[arg(long, default_value = "63")]
    color: String,

    /// Arquivo de entrada. Sem ele, a entrada e lida do stdin.
    file: Option<PathBuf>,
}

impl Cli {
    /// Whether the first row is the header. Defaults to `true`; `--no-header`
    /// turns it off, and the last of `--header`/`--no-header` wins.
    fn use_header(&self) -> bool {
        self.header || !self.no_header
    }

    /// The width limit, or `None` when `--max-width` is 0 or negative.
    fn max_width(&self) -> Option<usize> {
        usize::try_from(self.max_width)
            .ok()
            .filter(|width| *width > 0)
    }
}

/// Errors reported on stderr. The messages match the Go original.
#[derive(Debug, Error)]
enum AppError {
    /// The input could not be read.
    #[error("Erro ao ler entrada: {0}")]
    Input(#[source] io::Error),
    /// The input has no bytes at all.
    #[error("Erro: entrada vazia")]
    Empty,
    /// The input could not be parsed.
    #[error("Erro ao processar dados: {0}")]
    Parse(#[source] parse::ParseError),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(output) => {
            println!("{output}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

/// Reads the input, parses it and renders the table.
fn run(cli: &Cli) -> Result<String, AppError> {
    let data = read_input(cli.file.as_deref()).map_err(AppError::Input)?;
    if data.is_empty() {
        return Err(AppError::Empty);
    }

    let table = parse::detect_and_parse(&data, cli.use_header()).map_err(AppError::Parse)?;
    let options = RenderOptions {
        style: cli.style,
        max_width: cli.max_width(),
        color: cli.color.clone(),
    };
    Ok(render::render_table(&table.headers, &table.rows, &options))
}

/// Reads the whole input from `file`, or from stdin when no file is given.
///
/// Invalid UTF-8 is replaced with U+FFFD. Reading from an interactive terminal
/// is refused, as in the Go original.
fn read_input(file: Option<&Path>) -> io::Result<String> {
    let mut bytes = Vec::new();
    if let Some(path) = file {
        let mut handle = File::open(path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!("nao foi possivel abrir arquivo {}: {err}", path.display()),
            )
        })?;
        handle.read_to_end(&mut bytes)?;
    } else {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            return Err(io::Error::other(
                "nenhuma entrada fornecida — use pipe ou passe um arquivo como argumento",
            ));
        }
        stdin.lock().read_to_end(&mut bytes)?;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
