# charm-table-cli

> [Leia em portugues](README.pt-br.md)

A command-line tool that renders CSV, TSV, JSON and JSON Lines as styled
tables in the terminal, using [lipgloss](https://crates.io/crates/lipgloss)
for styling.

This crate is a Rust port of the Go command
[`github.com/junhinhow/charm-table-cli`](https://github.com/junhinhow/charm-table-cli).
The crate is named `charm-table-cli`, but the binary it installs is called
**`charm-table`**.

## Installation

```bash
cargo install --git https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust charm-table-cli
```

Not yet published on crates.io; install from git.

## Usage

```bash
# From a file
charm-table data.csv

# From a pipe
cat data.csv | charm-table

# Other styles and options
charm-table --style thick --color "#ff0000" data.csv
charm-table --no-header --max-width 60 data.csv
```

The input format is detected automatically:

| Input starts with | Format |
|---|---|
| `[` | JSON array of objects. The columns are the keys of every object. |
| `{` | JSON Lines: one object per line. |
| anything else | Delimited text. Tab-separated when the first line has a tab outside quotes, comma-separated otherwise. |

For delimited text, the first row is the header by default. With `--no-header`,
columns are named `Col1`, `Col2`, and so on, for every format.

## Flags

| Flag | Type | Default | Description |
|---|---|---|---|
| `--style <STYLE>` | `rounded`, `thick`, `double`, `ascii` | `rounded` | Border style. |
| `--header` / `--no-header` | switch | `--header` | Uses the first row as the header, or generates `Col1`, `Col2`, ... |
| `--max-width <N>` | integer | `0` | Maximum table width in columns, borders included. `0` means no limit. |
| `--color <COLOR>` | ANSI index or hex | `63` | Header color: an ANSI 256 index (`63`) or hex (`#ff0000`). |
| `[FILE]` | path | stdin | Input file. Without it, the input is read from stdin. |
| `-h`, `--help` | | | Prints help. |
| `-V`, `--version` | | | Prints the version. |

### Border styles

| `--style` | Characters | Example corners |
|---|---|---|
| `rounded` | `─│╭╮╰╯` | `╭ ╮ ╰ ╯` |
| `thick` | `━┃┏┓┗┛` | `┏ ┓ ┗ ┛` |
| `double` | `═║╔╗╚╝` | `╔ ╗ ╚ ╝` |
| `ascii` | `-\|+` | `+` |

`ascii` uses plain ASCII characters, so it works on terminals without Unicode
box drawing:

```text
+-------+-------+
| nome  | idade |
+-------+-------+
| Ana   | 31    |
+-------+-------+
```

## Example

Input (`people.csv`):

```csv
nome,idade,cidade
Ana,31,Recife
Bruno,27,Salvador
```

Command and output (ANSI codes removed):

```text
$ charm-table people.csv
╭───────┬───────┬──────────╮
│ nome  │ idade │ cidade   │
├───────┼───────┼──────────┤
│ Ana   │ 31    │ Recife   │
│ Bruno │ 27    │ Salvador │
╰───────┴───────┴──────────╯
```

Each column is as wide as its longest value, plus one space of padding on each
side, so every row stays on one line and the borders line up. The table is
followed by an empty line, as in the Go original.

Run the demo to see every border style:

```bash
cargo run -p charm-table-cli --example charm_table_cli_demo
```

## Behaviour

### Column order (JSON and JSON Lines)

The columns are the union of the keys of **all** records, so no column is
dropped. Inside each record the keys are sorted alphabetically, and keys that
only appear in later records are appended. The same input always produces the
same table.

```bash
echo '[{"name":"Alice","age":30},{"name":"Bob","city":"LA"}]' | charm-table
```

### Ragged records (CSV and TSV)

Rows with a different number of fields do not abort the program: the table
grows to the widest record and the missing cells are empty.

```bash
printf 'a,b,c\n1,2\n3,4,5,6\n' | charm-table
```

### `--max-width`

The limit includes the borders and the cell padding. The widest columns are
shrunk to a common cap, then any spare cell is given back, so the table fits
the limit whenever that is possible. A limit too small even for the borders is
reported as an error (exit code 1) instead of producing a broken table:

```text
$ printf 'a,b,c\n1,2,3\n' | charm-table --max-width 8
Erro: largura maxima 8 insuficiente para 3 colunas: sao necessarias ao menos 13 colunas
```

Cells that do not fit end with `...` when at least 4 cells are available,
and are cut without an ellipsis otherwise.

### Text width

Column widths are measured in **terminal cells**, not in bytes or characters.
CJK characters and emoji count as two cells, so they stay aligned. Line breaks
and tabs inside a cell are replaced by spaces, so a cell cannot break the
layout.

### Values

- Strings are shown as they are; `null` and missing keys are empty cells.
- Booleans are `true` and `false`.
- Numbers use the shortest decimal form that reads back the same value, never
  in exponent form: `1e-7` is `0.0000001` and `1e21` is
  `1000000000000000000000`.
- Arrays and objects are shown as compact JSON with sorted keys, and `<`, `>`,
  `&` are escaped as `\u003c`, `\u003e`, `\u0026`.

### Input limits and messages

- A JSON Lines record can be up to 16 MiB long. A longer line is an error.
- A leading byte order mark (BOM) is ignored.
- Whitespace-only input is an error (`entrada vazia`).
- Errors are printed on stderr as `Erro: ...`, with exit code 1.

## Parity with the Go original

The output is checked against a golden file generated by the Go program
(`reference/go/charm-table-cli/cmd/parity`). That program builds the real
binary and runs it on the same matrix of inputs and flags as the Rust
integration tests. The golden file covers CSV, TSV, JSON array and JSON Lines
input, every border style, `--no-header`, `--max-width`, `--color`, ragged
rows, quoted tabs and wide characters.

```bash
cargo run -p charm-parity --bin parity -- gen \
    crates/charm-table-cli/tests/golden/table.txt -- \
    go run ./reference/go/charm-table-cli/cmd/parity

cargo test -p charm-table-cli
```

## Known differences

| Go | Rust | Notes |
|---|---|---|
| `charm-table-cli` binary | `charm-table` binary | The crate has a different name from the binary. |
| `-style thick`, `-header=false` (single dash) | `--style thick`, `--no-header` | The Rust CLI uses clap, which accepts double dashes only. |
| `-style nope` falls back to `rounded` | Error, exit code 2 | Invalid values are rejected instead of silently replaced. |
| Stray quotes inside a field (Go `LazyQuotes`) | Parsed by the `csv` crate | Malformed quoting can give a different cell value. |
| Error text from the CSV reader: `record on line 4: ...` | `record N (line: L, byte: B): ...` | The prefix `erro ao ler dados CSV` matches; the detail comes from the `csv` crate. |
| Indexed ANSI colors (`38;5;63`) | Truecolor sequences (`38;2;95;95;255`) | `lipgloss` converts indexed colors to RGB. The color shown is the same. |
| Invalid UTF-8 is passed through | Replaced with U+FFFD | Rust strings must be valid UTF-8. |
| Width counted by the terminal library (`ansi.StringWidth`) | Width counted with `unicode-width`, per character | Some emoji sequences (ZWJ, variation selectors) can differ in width. |
| Error messages with accents (`não`, `é`) | The same messages without accents (`nao`, `e`) | The source is ASCII. The wording otherwise follows the Go messages. |

## License

MIT
