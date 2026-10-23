//! `charm-table`: renders CSV, TSV, JSON and JSON Lines as styled terminal tables.
//!
//! This is a port of the Go `charm-table-cli` command. The table is written to
//! stdout; errors go to stderr, prefixed with `Erro:`, with exit code 1.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod parse;
mod render;

use std::fs::File;
use std::io::{self, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::Parser;
use thiserror::Error;

use crate::render::{BorderStyle, RenderError, RenderOptions};

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

    /// Largura maxima da tabela, bordas incluidas (0 = sem limite).
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

/// Errors reported on stderr. The messages follow the Go original.
#[derive(Debug, Error)]
enum AppError {
    /// The input file could not be opened.
    #[error("nao foi possivel abrir o arquivo \"{path}\": {source}")]
    OpenFile {
        /// Path that was requested.
        path: String,
        /// Underlying I/O error.
        #[source]
        source: io::Error,
    },
    /// The input could not be read.
    #[error("nao foi possivel ler a entrada: {0}")]
    Read(#[source] io::Error),
    /// Nothing was piped and no file was given.
    #[error("nenhuma entrada fornecida — use pipe ou passe um arquivo como argumento")]
    NoInput,
    /// The input could not be parsed.
    #[error(transparent)]
    Parse(#[from] parse::ParseError),
    /// The table could not be drawn.
    #[error(transparent)]
    Render(#[from] RenderError),
    /// The table could not be written to stdout.
    #[error("nao foi possivel escrever a tabela na saida: {0}")]
    Write(#[source] io::Error),
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli).and_then(|output| write_output(&output)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("Erro: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Reads the input, parses it and renders the table.
fn run(cli: &Cli) -> Result<String, AppError> {
    let data = read_input(cli.file.as_deref())?;
    let table = parse::detect_and_parse(&data, cli.use_header())?;
    let options = RenderOptions {
        style: cli.style,
        max_width: cli.max_width(),
        color: cli.color.clone(),
    };
    Ok(render::render_table(&table.headers, &table.rows, &options)?)
}

/// Writes the rendered table, followed by a newline, to stdout.
fn write_output(output: &str) -> Result<(), AppError> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{output}").map_err(AppError::Write)
}

/// Reads the whole input from `file`, or from stdin when no file is given.
///
/// Invalid UTF-8 is replaced with U+FFFD. Reading from an interactive terminal
/// is refused, as in the Go original.
fn read_input(file: Option<&Path>) -> Result<String, AppError> {
    let mut bytes = Vec::new();
    if let Some(path) = file {
        let mut handle = File::open(path).map_err(|source| AppError::OpenFile {
            path: path.display().to_string(),
            source,
        })?;
        handle.read_to_end(&mut bytes).map_err(AppError::Read)?;
    } else {
        let stdin = io::stdin();
        if stdin.is_terminal() {
            return Err(AppError::NoInput);
        }
        stdin
            .lock()
            .read_to_end(&mut bytes)
            .map_err(AppError::Read)?;
    }
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}
