# Task Plan: Port charm-test and charm-devtools

## Goal
Implement both Rust crates to the supplied API, with Go parity goldens, documentation, examples, and clean validation.

## Next Step
Report the completed ports, validation results, and runtime-driven API deviations.

## Current Phase
Phase 5

## Phases

### Phase 1: Requirements & Discovery
- [x] Understand user intent
- [x] Identify constraints
- [x] Document in findings.md
- **Status:** complete

### Phase 2: Planning & Structure
- [x] Define approach
- [x] Create project structure
- **Status:** complete

### Phase 3: Implementation
- [x] Execute the plan
- [x] Write to files before executing
- **Status:** complete

### Phase 4: Testing & Verification
- [x] Verify requirements met
- [x] Document test results
- **Status:** complete

### Phase 5: Delivery
- [x] Review outputs
- [x] Deliver to user
- **Status:** complete

## Decisions Made
| Decision | Rationale |
|----------|-----------|
| Implement a synchronous driver using `Cmd::execute()` and explicit Batch/Sequence handling | Matches Go's deterministic command processing; runtime simulator does not retain message history and has different lifecycle details |
| Use `std::any::type_name::<Message>()` and `Debug` for DevTools message entries | `Message` is erased and offers neither concrete type metadata nor concrete debug forwarding |
| Capture `current_state` from the embedded model's view and component tree from `DevInspect` | Rust has no runtime reflection or `Debug` bound on `Model`; explicit trait enables custom structure |

## Errors Encountered
| Error | Resolution |
|-------|------------|
| `Message` cannot be cloned after being consumed by `Model::update` | Exposed `MessageRecord` diagnostics in `Simulator::messages()` instead of an impossible `&[Message]` history; documented in the crate README |
| Runtime lacks concrete message `type_name` and payload `Debug` access | Record the erased wrapper's type/value representation; identify known built-in key/resize message types explicitly |
| Workspace dependency key is `charmed-bubbletea` while the library target is `bubbletea` | Rust source imports through the inherited `charmed_bubbletea` dependency name |
