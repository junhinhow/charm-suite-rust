//! Integration tests for the `charm-table` binary.
//!
//! The parity test runs the same matrix as `reference/go/charm-table-cli/cmd/parity`
//! and compares the transcript with `tests/golden/table.txt`.

use std::fmt::Write as _;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};

use charm_parity::assert_golden;

const CSV_SAMPLE: &str = concat!(
    "nome,idade,cidade,descricao\n",
    "Ana,31,São Paulo,Engenheira de software sênior\n",
    "Bruno,27,Rio de Janeiro,\"Analista, dados\"\n",
    "Carla,45,Belo Horizonte,\n",
    "Dário,,Curitiba,Gerente de produto com muitas responsabilidades\n",
);

const TSV_SAMPLE: &str = "id\tnome\tnota\n1\tAna\t9.5\n2\tBruno\t7\n3\tCarla\t10\n";

const SEMICOLON_SAMPLE: &str = "nome;idade\nAna;31\n";

const SINGLE_LINE_SAMPLE: &str = "nome,idade\n";

const JSON_ARRAY_SAMPLE: &str = r#"[{"meta": {"b": 1, "a": [1, 2.5, "x&y", null, true]}}, {"meta": "plain"}, {"meta": null}, {"meta": 3}, {}]"#;

const JSONL_SAMPLE: &str = concat!(
    "{\"v\": 1}\n",
    "{\"v\": 2.5}\n",
    "{\"v\": 1e-7}\n",
    "{\"v\": 12345678.5}\n",
    "{\"v\": 1e21}\n",
    "{\"v\": -0.0}\n",
    "{\"v\": 0.000012}\n",
    "{\"v\": 123456}\n",
    "{\"v\": true}\n",
    "{\"v\": \"texto\"}\n",
    "{\"v\": null}\n",
);

/// Same cases, in the same order, as `matrixCases` in the Go parity program.
const MATRIX: &[(&str, &[&str], &str)] = &[
    ("csv_rounded", &[], CSV_SAMPLE),
    ("csv_thick", &["--style", "thick"], CSV_SAMPLE),
    ("csv_double", &["--style", "double"], CSV_SAMPLE),
    ("csv_ascii", &["--style", "ascii"], CSV_SAMPLE),
    ("csv_no_header", &["--no-header"], CSV_SAMPLE),
    ("csv_max_width_40", &["--max-width", "40"], CSV_SAMPLE),
    ("csv_max_width_12", &["--max-width", "12"], CSV_SAMPLE),
    ("csv_color_hex", &["--color", "#ff0000"], CSV_SAMPLE),
    ("tsv_rounded", &[], TSV_SAMPLE),
    ("csv_semicolon_fallback", &[], SEMICOLON_SAMPLE),
    ("csv_single_line_header_quirk", &[], SINGLE_LINE_SAMPLE),
    ("json_array_nested", &[], JSON_ARRAY_SAMPLE),
    ("jsonl_numbers", &[], JSONL_SAMPLE),
];

fn run_charm(args: &[&str], stdin: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_charm-table"))
        .args(args)
        // O perfil de cor e detectado pelo ambiente. Em CI nao ha TTY e o
        // `lipgloss` cai para `NoColor`, o que removeria os codigos ANSI e
        // tornaria as assercoes de cor dependentes da maquina. Forcamos
        // truecolor para que os testes de cor sejam deterministicos.
        .env("COLORTERM", "truecolor")
        .env("TERM", "xterm-256color")
        .env_remove("NO_COLOR")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("falha ao iniciar charm-table");
    if let Some(mut pipe) = child.stdin.take() {
        if let Err(error) = pipe.write_all(stdin.as_bytes()) {
            // O filho pode encerrar antes de ler o stdin (por exemplo, quando o
            // `clap` rejeita um argumento invalido). Nesse caso o pipe quebra
            // com `BrokenPipe`, o que e esperado e nao deve falhar o teste.
            assert_eq!(
                error.kind(),
                std::io::ErrorKind::BrokenPipe,
                "falha ao escrever no stdin: {error}"
            );
        }
    }
    child
        .wait_with_output()
        .expect("falha ao aguardar charm-table")
}

fn stdout_of(args: &[&str], stdin: &str) -> String {
    let output = run_charm(args, stdin);
    assert!(
        output.status.success(),
        "charm-table falhou: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("saida nao e UTF-8")
}

fn stderr_of(args: &[&str], stdin: &str) -> (bool, String) {
    let output = run_charm(args, stdin);
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn plain(text: &str) -> String {
    charm_core::strip_ansi(text)
}

/// Keeps only letters and digits, so the assertions ignore borders, padding and
/// line breaks.
fn squashed(text: &str) -> String {
    plain(text)
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect()
}

fn widest_line(text: &str) -> usize {
    text.lines()
        .map(|line| charm_core::display_width(&plain(line)))
        .max()
        .unwrap_or(0)
}

#[test]
fn golden_transcript_matches_go() {
    let mut transcript = String::new();
    for (name, args, input) in MATRIX {
        let _ = write!(transcript, "### {name}\n{}", stdout_of(args, input));
    }
    assert_golden("table", &transcript);
}

#[test]
fn border_styles_draw_their_corners() {
    let cases = [
        ("rounded", "╭", "╮"),
        ("thick", "┏", "┓"),
        ("double", "╔", "╗"),
        ("ascii", "┌", "┐"),
    ];
    for (style, left, right) in cases {
        let out = stdout_of(&["--style", style], CSV_SAMPLE);
        let top = out.lines().next().expect("tabela sem linhas");
        assert!(top.starts_with(left), "{style}: {top}");
        assert!(top.ends_with(right), "{style}: {top}");
    }
}

#[test]
fn every_table_line_has_the_same_width() {
    let out = stdout_of(&[], CSV_SAMPLE);
    let widths: Vec<usize> = out
        .lines()
        .map(|line| charm_core::display_width(&plain(line)))
        .collect();
    assert!(widths.windows(2).all(|pair| pair[0] == pair[1]), "{out}");
}

#[test]
fn invalid_style_is_rejected() {
    let (ok, _) = stderr_of(&["--style", "neon"], CSV_SAMPLE);
    assert!(!ok);
}

#[test]
fn no_header_generates_column_names() {
    let out = squashed(&stdout_of(&["--no-header"], CSV_SAMPLE));
    assert!(out.contains("Col1Col2Col3Col4"), "{out}");
    assert!(
        out.contains("nomeidade"),
        "nome deve virar linha de dados: {out}"
    );
}

#[test]
fn last_header_flag_wins() {
    let out = squashed(&stdout_of(&["--no-header", "--header"], CSV_SAMPLE));
    assert!(!out.contains("Col1"), "{out}");
    let out = squashed(&stdout_of(&["--header", "--no-header"], CSV_SAMPLE));
    assert!(out.contains("Col1"), "{out}");
}

#[test]
fn max_width_truncates_cells_and_shrinks_table() {
    let unlimited = stdout_of(&[], CSV_SAMPLE);
    let limited = stdout_of(&["--max-width", "40"], CSV_SAMPLE);
    assert!(limited.contains("..."), "{limited}");
    assert!(widest_line(&limited) < widest_line(&unlimited));
}

#[test]
fn zero_max_width_means_no_limit() {
    assert_eq!(
        stdout_of(&["--max-width", "0"], CSV_SAMPLE),
        stdout_of(&[], CSV_SAMPLE)
    );
}

#[test]
fn color_hex_and_ansi_index_are_emitted() {
    let hex = stdout_of(&["--color", "#ff0000"], CSV_SAMPLE);
    assert!(hex.contains("38;2;255;0;0"), "{hex}");

    // lipgloss emits indexed colors as truecolor; the Go original emits `38;5;63`.
    let ansi = stdout_of(&["--color", "63"], CSV_SAMPLE);
    assert!(ansi.contains("38;2;95;95;255"), "{ansi}");
}

#[test]
fn reads_input_from_file_argument() {
    let path: PathBuf = [env!("CARGO_TARGET_TMPDIR"), "charm-table-sample.csv"]
        .iter()
        .collect();
    std::fs::write(&path, CSV_SAMPLE).expect("falha ao escrever arquivo temporario");
    let from_file = stdout_of(&[path.to_str().expect("caminho nao UTF-8")], "");
    assert_eq!(from_file, stdout_of(&[], CSV_SAMPLE));
}

#[test]
fn empty_stdin_is_an_error() {
    let (ok, stderr) = stderr_of(&[], "");
    assert!(!ok);
    assert_eq!(stderr, "Erro: entrada vazia\n");
}

#[test]
fn invalid_json_is_an_error() {
    let (ok, stderr) = stderr_of(&[], "[{\"a\":");
    assert!(!ok);
    assert!(
        stderr.starts_with("Erro ao processar dados: JSON invalido"),
        "{stderr}"
    );
}

#[test]
fn ragged_csv_is_an_error() {
    let (ok, stderr) = stderr_of(&[], "a,b\n1,2,3\n");
    assert!(!ok);
    assert!(
        stderr.starts_with("Erro ao processar dados: erro ao processar dados delimitados"),
        "{stderr}"
    );
}

#[test]
fn missing_file_is_an_error() {
    let (ok, stderr) = stderr_of(&["arquivo-que-nao-existe-charm.csv"], "");
    assert!(!ok);
    assert!(
        stderr.starts_with("Erro ao ler entrada: nao foi possivel abrir arquivo"),
        "{stderr}"
    );
}
