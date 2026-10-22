# charm-diff

Beautiful terminal diff viewer for the [Charm](https://charm.sh) ecosystem. Produces unified and side-by-side diffs with [Lip Gloss](https://github.com/charmbracelet/lipgloss) styling.

> **[Read in Portuguese](README.pt-br.md)**

This is a Rust port of the Go library [`github.com/junhinhow/charm-diff`](https://github.com/junhinhow/charm-diff). The default LCS output is byte-for-byte identical to the original (ANSI codes aside), which is verified by golden files generated from the Go implementation.

## Features

- **Unified diff** — classic format with `+`/`-` markers and line numbers
- **Side-by-side diff** — two-column comparison at any terminal width
- **Configurable context lines** — show as many surrounding lines as you want (default: 3)
- **Predefined styles** — GitHub-inspired and minimal terminal themes
- **Custom styles** — build your own `DiffStyle` with any Lip Gloss style

## Install

```toml
[dependencies]
charm-diff = { git = "https://forgejo.multiservidores.com.br/ajr/CharmSuite-Rust", tag = "v0.1.0" }
```

Not yet published on crates.io; install from git.

## Usage

### Unified Diff

```rust
use charm_diff::unified_diff;

fn main() {
    let old = "fn hello() {\n    println!(\"hello\");\n}\n";
    let new = "fn hello() {\n    println!(\"hello, world!\");\n}\n";

    print!("{}", unified_diff(old, new));
}
```

### Side-by-Side Diff

```rust
use charm_diff::side_by_side_diff;

fn main() {
    let old = "fn hello() {\n    println!(\"hello\");\n}\n";
    let new = "fn hello() {\n    println!(\"hello, world!\");\n}\n";

    print!("{}", side_by_side_diff(old, new, 120));
}
```

### Custom Options

```rust
use charm_diff::{unified_diff_with_options, Algorithm, DiffStyle, Options};

fn main() {
    let opts = Options {
        context_lines: 5,
        style: DiffStyle::github(),
        algorithm: Algorithm::Myers,
    };

    print!("{}", unified_diff_with_options("a\nb\n", "a\nc\n", &opts));
}
```

A runnable example is available in [`examples/demo.rs`](examples/demo.rs):

```bash
cargo run -p charm-diff --example demo
```

## API Reference

### Functions

| Function | Description |
|----------|-------------|
| `unified_diff(old: &str, new: &str) -> String` | Unified diff with default options |
| `unified_diff_with_options(old: &str, new: &str, opts: &Options) -> String` | Unified diff with custom options |
| `side_by_side_diff(old: &str, new: &str, width: usize) -> String` | Side-by-side diff with default options |
| `side_by_side_diff_with_options(old: &str, new: &str, width: usize, opts: &Options) -> String` | Side-by-side diff with custom options |

### Types

| Type | Description |
|------|-------------|
| `Options` | Configuration: `context_lines: usize`, `style: DiffStyle`, `algorithm: Algorithm` |
| `Algorithm` | `Lcs` (default, Go parity) or `Myers` (linear auxiliary memory) |
| `DiffStyle` | Lip Gloss styles for each diff element |

### Style Constructors

| Constructor | Description |
|-------------|-------------|
| `DiffStyle::default()` | Returns `DiffStyle::terminal()` |
| `DiffStyle::terminal()` | Basic ANSI colors |
| `DiffStyle::github()` | GitHub-like colored backgrounds |

### Go → Rust mapping

| Go | Rust |
|-----|------|
| `DefaultOptions()` | `Options::default()` |
| `UnifiedDiff(old, new)` | `unified_diff(old, new)` |
| `UnifiedDiffWithOptions(old, new, opts)` | `unified_diff_with_options(old, new, &opts)` |
| `SideBySideDiff(old, new, width)` | `side_by_side_diff(old, new, width)` |
| `SideBySideDiffWithOptions(old, new, width, opts)` | `side_by_side_diff_with_options(old, new, width, &opts)` |
| `DefaultStyle()` | `DiffStyle::default()` |
| `TerminalStyle()` | `DiffStyle::terminal()` |
| `GitHubStyle()` | `DiffStyle::github()` |
| `Options{ContextLines: n, Style: s}` | `Options { context_lines: n, style: s, algorithm: Algorithm::Lcs }` |

## Algorithms

`Options` selects the diff algorithm with its `algorithm` field:

- **`Algorithm::Lcs` (default):** uses the classic dynamic-programming LCS
  table, with `O(n · m)` memory. Its tie-breaking matches the original Go
  implementation exactly, preserving byte-for-byte parity.
- **`Algorithm::Myers`:** uses Myers' bidirectional middle-snake divide-and-
  conquer algorithm with `O(n + m)` auxiliary memory. It produces a shortest
  edit script and is recommended for large files.

Both algorithms emit operations from the start of the input to the end. When
multiple shortest diffs are possible (for example, with repeated lines), Myers
may choose a different valid ordering of insertions and deletions than LCS;
therefore its rendered output can differ from the Go golden while still
transforming the old text into the new text with the minimum number of edits.

Operations are then grouped into *hunks* using `context_lines`; the side-by-side
layout pairs a deletion followed by an insertion into a single "modified line".

## Known differences

- **Truncation counts bytes, not characters.** The column padding/truncation
  used by the side-by-side layout decides whether to pad or truncate using the
  **byte** length of the line (`len(s)` in Go), and cuts the string at a byte
  offset. This is the behaviour of the Go original and it is preserved here.
  The Go implementation may therefore cut a multi-byte character in half and
  emit an invalid UTF-8 sequence; both implementations replace the broken
  bytes with `U+FFFD` when the styled output is produced, so the rendered
  result is identical. In Rust the cut is performed with
  `String::from_utf8_lossy`, which never yields an invalid `String`.
- **ANSI codes are always emitted.** The Go implementation only emits colours
  when stdout is a TTY, while the Rust port always renders the ANSI escape
  sequences. The parity harness strips the ANSI codes from both sides before
  comparing them.
- **`Options` and `DiffStyle` are owned values.** Go passes them by value as
  well, but Rust requires `&Options` in the `*_with_options` functions so that
  the (potentially expensive) style is not copied.

## Parity with the Go implementation

The golden file [`tests/golden/diff.txt`](tests/golden/diff.txt) is generated
by the Go program `reference/go/charm-diff/cmd/parity` and contains 20 samples
(identical files, insertions, deletions, modifications, adjacent and distant
changes, empty files, missing trailing newline, unicode, blank lines, the
context values `0`, `1`, `3` and `10`, tabs, long lines, repeated lines, a full
rewrite and the GitHub style), each rendered as a unified diff and as a
side-by-side diff at the widths 40, 80 and 120.

Regenerate it with:

```bash
cargo run -p charm-parity --bin parity -- gen crates/charm-diff/tests/golden/diff.txt -- go run ./reference/go/charm-diff/cmd/parity
```

And check it with:

```bash
cargo test -p charm-diff
```

## License

MIT
