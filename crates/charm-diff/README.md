# charm-diff

Beautiful terminal diff viewer for the [Charm](https://charm.sh) ecosystem. Produces unified and side-by-side diffs with [Lip Gloss](https://github.com/charmbracelet/lipgloss) styling.

> **[Read in Portuguese](README.pt-br.md)**

This is a Rust port of the Go library [`github.com/junhinhow/charm-diff`](https://github.com/junhinhow/charm-diff). The default engine reproduces Git's `xdiff` behavior, including edit alignment and hunk construction; parity is checked against Go-generated golden files and randomized `git diff` comparisons.

## Features

- **Unified diff** — classic format with `+`/`-` markers and line numbers
- **Git-compatible hunks** — hunk ranges, context, and edit ordering follow `git diff -U<N>`
- **Side-by-side diff** — two-column comparison at any terminal width
- **Configurable context lines** — show as many surrounding lines as you want (default: 3)
- **Linear auxiliary memory** — the default xdiff engine handles large inputs without an LCS table
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
        algorithm: Algorithm::Xdiff,
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
| `Algorithm` | `Xdiff` (default, Git/Go parity), `Lcs`, or `Myers` |
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
| `Options{ContextLines: n, Style: s}` | `Options { context_lines: n, style: s, algorithm: Algorithm::Xdiff }` |

## Algorithms

`Options` retains a Rust-specific `algorithm` selector; the Go API has no such field. The default is `Algorithm::Xdiff`, which is the engine used by the updated Go library:

- **`Algorithm::Xdiff` (default):** uses Git's linear-space Myers engine,
  including record cleanup and change compaction. Its unified hunk layout
  matches `git diff -U<N>` and the Go implementation.
- **`Algorithm::Lcs`:** uses the classic dynamic-programming LCS table, with
  `O(n · m)` memory. Kept for compatibility with previous Rust releases.
- **`Algorithm::Myers`:** uses a linear-space middle-snake algorithm and
  computes a shortest edit script, but does not apply xdiff's cleanup and
  compaction rules.

`Lcs` and `Myers` may choose different valid insertion/deletion alignments than
Git when repeated lines make multiple scripts possible; use the default
`Xdiff` when Go/Git parity matters.

Operations are then grouped into *hunks* using `context_lines`; the side-by-side
layout pairs a deletion followed by an insertion into a single "modified line".

## Known differences

- **`context_lines` is unsigned.** Rust uses `usize`, so negative values cannot
  be represented; pass `0` for the Go behavior of normalizing negative context
  to no context.
- **The algorithm selector is Rust-only.** Go always uses xdiff. The Rust
  `Lcs` and `Myers` alternatives are retained for compatibility and may render
  a different valid alignment on ambiguous repeated lines.
- **ANSI codes are always emitted.** The Go implementation only emits colours
  when stdout is a TTY, while the Rust port always renders the ANSI escape
  sequences. The parity harness strips the ANSI codes from both sides before
  comparing them.
- **`Options` and `DiffStyle` are owned values.** Go passes them by value as
  well, but Rust requires `&Options` in the `*_with_options` functions so that
  the (potentially expensive) style is not copied.

## Parity with the Go implementation

The golden file [`tests/golden/diff.txt`](tests/golden/diff.txt) is generated
by the Go program `reference/go/charm-diff/cmd/parity` and contains 20 regular samples
(identical files, insertions, deletions, modifications, adjacent and distant
changes, empty files, missing trailing newline, unicode, blank lines, the
context values `0`, `1`, `3` and `10`, tabs, long lines, repeated lines, a full
rewrite and the GitHub style), each rendered as a unified diff and as a
side-by-side diff at widths 15, 40, 80 and 120. A 10,000-line case also checks
dynamic five-digit line-number formatting. The Rust tests additionally compare
200 deterministic randomized unified diffs with Git.

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
