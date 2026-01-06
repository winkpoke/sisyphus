## 1. Implementation
- [x] Route UI commands through MVU instructions (no EventBus control flow)
- [x] Reuse the shared slash command parser for TUI slash inputs
- [x] Ensure `/new` creates a backend session and updates active session id
- [x] Ensure `/clear` clears transcript and clears backend session history
- [x] Ensure `/exit` and `/quit` terminate the TUI gracefully
- [x] Keep command palette discovery as a merged list (local + remote)

## 2. Tests
- [x] Add unit tests for TUI slash routing decisions
- [x] Add unit tests for `/new` and `/clear` instruction mapping

## 3. Validation
- [x] Run `cargo test` for the Rust workspace
- [x] Run `cargo fmt --check` and `cargo clippy` for the Rust workspace
- [x] Manually verify: `/new`, `/clear`, `/exit`, `/quit` and palette selection
