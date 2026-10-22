# Findings & Decisions

## Requirements
- Update only `crates/charm-diff/`, the root `KNOWN_DIFFERENCES.md`, and Go parity driver if needed; do not alter unrelated crates or root `Cargo.toml`.
- Read all four Go source files completely; port dynamic line numbering, hunk generation/merging, and `xdiff` semantics; retain existing Rust API where practical.
- Port new Go tests; regenerate golden with the prescribed parity command; update bilingual READMEs, changelog, and differences docs.
- Enforce `forbid(unsafe_code)`, `warn(missing_docs)`, no `unwrap`/`expect` in library; validate fmt/tests/clippy with `target/agent-sync-diff`.

## Research Findings
- Existing selected plan belonged to an already-completed unrelated task; initialized and pinned this work to `.planning/2026-10-22-port-charm-diff-update`.
- `PORTING_GUIDE.md` requires Go-generated goldens, bilingual crate documentation, and no unrelated crate/root manifest edits.
- `KNOWN_DIFFERENCES.md` currently claims LCS `O(n*m)` memory for charm-diff; this must be revisited if the new xdiff algorithm changes that complexity.
- Read all four Go implementation files fully. Go's exported API remains the same four render functions, `Options{ContextLines, Style}`, and `DiffStyle` constructors; there are no new public fields/functions. The behavior change is the default engine: exact git-compatible xdiff (linear-space Myers plus record cleanup/change compaction), context-based script hunks, dynamic line number widths, and side-by-side minimum-width clamping.
- Go `Options` still normalizes negative context to zero and a zero `DiffStyle` to `DefaultStyle()`. Rust `Options` currently uses `usize` and contains an additional Rust-only `algorithm` selector; retain source compatibility and preserve explicit LCS/Myers options while adding a default `Xdiff` mode.
- Go side-by-side width now budgets `2*(number_width+1)+3` overhead and clamps to `2*(number_width+1)+3+2` (minimum one display cell each side); text is measured/truncated in terminal display cells, not bytes.
- Read all Go unit, hunk, golden, and regression tests. Golden regression covers real git output including 200 deterministic random cases, negative contexts, CRLF, dynamic side-by-side widths, multibyte truncation, and a 10,000-line memory case.
- The reference checkout currently has no `cmd/parity` directory/file (recursive listing returned nothing), despite the documented generation command. A Go parity driver must be added if the requested command is to succeed.
- Existing Rust implementation already contains a separate Myers algorithm and LCS option, but default is LCS and its hunk construction uses operation ranges rather than Go's change script/index mapping. Its number width is fixed at four; side-by-side currently imposes a 10-cell minimum per column, exceeding requested widths.
- `charm_core` already exposes `display_width` and display-cell `truncate`; its existing `pad_or_truncate` is byte-based and must not be reused for the new Go behavior.
- Verified the complete reference source/tests: `Options` in the updated Go source still has only `ContextLines` and `Style`; no public Go option fields or render functions were added. `normalizeOptions` clamps negative context to zero and replaces only a zero style. Preserve Rust's existing `usize` field and Rust-only `Algorithm`; make xdiff the default and document the unsigned-context/API difference.
- Current worktree already contains a partial Rust xdiff port, dynamic width helpers, xdiff hunk builder, modified golden, and an updated Go `cmd/parity/main.go`; keep and finish these task changes rather than recreate them.
- Initial validation passes: Rust tests/doctests (24 unit + 2 parity) and Go tests pass. Existing golden currently matches, but still run the prescribed generator after completing the ports/tests.

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| User's detailed implementation brief is the approved task contract | User explicitly asked for autonomous execution and enumerated implementation/test/docs scope |
| Keep xdiff in a separate Rust module under its Go semantic name | Enables direct testing and clear separation from default diff behavior |
| Run every Cargo command with `CARGO_TARGET_DIR=target/agent-sync-diff` | Explicit user constraint avoids target-directory contention |
| Preserve the existing Rust `Algorithm` customization but add `Xdiff` and make it the default | Go exposes no algorithm field; xdiff is now the Go default, while existing Rust LCS/Myers callers should not break |
| Add the missing `reference/go/charm-diff/cmd/parity/main.go` driver | Required golden command points to a nonexistent driver in the supplied reference checkout |

## Issues Encountered
| Issue | Resolution |
|-------|------------|

## Resources
-
