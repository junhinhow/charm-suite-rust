# Findings & Decisions

## Requirements
- Port `charm-test` and `charm-devtools`, without modifying the root manifest or unrelated crates.
- Use unique example filenames, bilingual READMEs, changelogs, Go-generated golden outputs, and the specified isolated Cargo target directory.
- `charm-test` initialization and resize both deliver `WindowSizeMsg`; keys map named keys and single Unicode characters; assertions ignore ANSI.
- `charm-devtools` must keep a 100-entry oldest-first history, expose `DevInspect`, and preserve inner view while closed.

## Research Findings
- `Cmd::execute(self) -> Option<Message>` is public and synchronous. `BatchMsg(pub Vec<Cmd>)` and `SequenceMsg(pub Vec<Cmd>)` are public, so the driver can execute and flatten both command forms.
- `Message` has no `type_name()` and its public `Debug` implementation is opaque (`Message { .. }`); concrete payload type names and payload debug values cannot be recovered after type erasure.
- `Message` supports `is`, `downcast`, and `downcast_ref`; `WindowSizeMsg`, `QuitMsg`, `BatchMsg`, and `SequenceMsg` are runtime types.
- `ProgramSimulator` exists but has its own queued stepping lifecycle and does not expose the full incoming message history required by `charm-test`; a small custom synchronous driver is a better fit.
- Go key parser aliases include `escape`, `pageup`, `pagedown`; unknown multi-character keys become key messages carrying text rather than being discarded. Single Unicode chars are to be handled by Rust `chars()` per the port contract.
- Go counter model displays `Count: N`, increments on k/up, decrements without going negative on j/down, and q/ctrl+c blanks the view.
- Project has no existing Rust `charm-test` or `charm-devtools` crates. Initial `git diff --stat` showed an existing `Cargo.lock` modification (+18 lines); it predates this work and must be preserved.

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Fallback `MessageEntry.type_name` is the erased Rust message wrapper type and value is its public Debug representation | The library cannot inspect the erased concrete payload; this is the fallback specified by the roadmap |
| Inspector model state view uses the model's current rendered `view()` | `Model` has no Debug bound and reflection is unavailable |
| Custom DevInspect nodes define component hierarchy; default node uses `type_name::<Self>()` | Implements ADR-8 explicit introspection |

## Issues Encountered
| Issue | Resolution |
|-------|------------|
| `reference/go/go.work` is absent at that path | Continue locating workspace file at repository root before running parity generation |
| Workspace dependency key `charmed-bubbletea` imports as `charmed_bubbletea`, despite the dependency library target being named `bubbletea` | Use the inherited dependency's Cargo identifier (`charmed_bubbletea`) in Rust imports; public `type_name` values still use the library's actual `bubbletea` name |
| First compile attempt could not import `BatchMsg` / `SequenceMsg` from the crate root | Import these public message types from `charmed_bubbletea::message` |
| First assertion test expected a panic on a one-line view with expected count 1 | Corrected the test to request 2 lines; implementation already counted the view correctly |
| Rust API request for `&[Message]` history conflicts with consumed, non-Clone `Message` inputs | Exposed safe `MessageRecord` diagnostics instead and documented the unavoidable signature deviation |

## Resources
-
