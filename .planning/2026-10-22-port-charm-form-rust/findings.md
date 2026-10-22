# Findings & Decisions

## Requirements
-

## Research Findings
-

## Technical Decisions
| Decision | Rationale |
|----------|-----------|

## Issues Encountered
| Issue | Resolution |
|-------|------------|

## Resources
-
# Findings & Decisions

## Requirements
- Port the updated Go `charm-form` API to Rust in `crates/charm-form/` only, except the expressly requested root `KNOWN_DIFFERENCES.md` entry and Go parity driver.
- Preserve `#![forbid(unsafe_code)]`, `#![warn(missing_docs)]`, no `unwrap()`/`expect()` in library code, and idiomatic snake_case.
- Validate using exclusive `CARGO_TARGET_DIR=target/agent-sync-form`.

## Initial Findings
- Current Rust crate has a single `src/lib.rs`, basic parity test/golden, example, READMEs, changelog, and Cargo manifest.
- Recursive inventory found 13 Go files (not 7): root API/generator + tests, `renderer` implementation/tests, and CLI implementation/tests. All must be read for the requested full source/test port.
- Go README describes Answers, validator, normalizer, spec-only `run`, CLI, and Bubble Tea renderer; unlike the existing Rust implementation, it has an interactive TUI.
- Workspace already has `charmed-bubbletea`, `serde_json`, `serde_yaml_ng`, and `thiserror`; root `Cargo.toml` should not be changed unless adding the explicitly permitted regex workspace dependency is necessary.
- Existing `KNOWN_DIFFERENCES.md` still says `run` only outputs JSON and has no interactive form; update after runner behavior is established.

## Go API Contract (all 13 Go files read)
- Root package exports type constants, `FormSpec`, `FieldSpec`, `LoadFromFile`, `LoadFromBytes`, `NormalizeType`, `FormSpec.Normalize`, `FormSpec.Validate`, `FieldSpec.LabelText`, `FieldSpec.Regex`, `ToJSON`, `WriteJSON`, `Run`, `Answers.Validate/ToJSON/WriteJSON`, and `ValidateField`.
- Loading trims/case-folds field types (aliases `multi-select`, `multi_select`), preserves unknown types for validation, requires trimmed non-empty title/name/type, unique names, non-empty options, and syntactically valid regexes.
- Answer validation checks missing required values across fields and aggregates all errors; required text trims whitespace, required selects reject empty strings, required multiselect rejects empty lists, confirm accepts false, select/multiselect options are checked, and regex applies per selected item.
- `renderer` exports `ErrCancelado`, `Model` (`New`, `Answers`, `Cancelled`, `Done`, Bubble Tea `Init/Update/View`), `Options`, `Run`, and `RunWithOptions`; it handles input/text editing, selection, multiselect, confirm, paste, tab navigation, validation errors, cancellation, and interactive view. `TextBuffer` stays private in Go.
- Go's `cmd/charm-form` supports `-file`, `-out`, `-json=false`, and a positional YAML path; this is a CLI behavior requirement, not a Go package API.
- The requested Go parity driver path `reference/go/charm-form/cmd/parity/main.go` does not currently exist and needs to be created.

## Open Technical Checks
- `charmed-bubbletea` 0.2.4 provides `Model::{init,update,view}`, `Program::{new,with_input,with_output,run}`, typed `Message`, and `KeyMsg`/`KeyType`. It uses an Elm string view rather than Bubble Tea v2's `View` return type, so the renderer will implement this Rust runtime's trait directly.
- Its custom reader/writer mode supports deterministic tests; there is no public program window-size option, so `Options.width/height` will initialize the model's size as the Go option does.
- Rust regex crate is needed for validation at load time and answer checking. Go regexp uses RE2-like syntax, compatible with the Rust regex crate's supported safe subset for common patterns.
- Implementation structure: keep form/spec API in the crate root and expose a `renderer` module with `Model`, `Options`, `run`, and `run_with_options`; add a `charm-form` binary for Go CLI options/summary output.
