## 1. Implementation
- [ ] Route UI commands through MVU instructions (no EventBus control flow)
- [ ] Reuse the shared slash command parser for TUI slash inputs
- [ ] Ensure `/new` creates a backend session and updates active session id
- [ ] Ensure `/clear` clears transcript and clears backend session history
- [ ] Ensure `/exit` and `/quit` terminate the TUI gracefully
- [ ] Keep command palette discovery as a merged list (local + remote)

## 2. Tests
- [ ] Add unit tests for TUI slash routing decisions
- [ ] Add unit tests for `/new` and `/clear` instruction mapping

## 3. Validation
- [ ] Run `cargo test` for the Rust workspace
- [ ] Run `cargo fmt --check` and `cargo clippy` for the Rust workspace
- [ ] Manually verify: `/new`, `/clear`, `/exit`, `/quit` and palette selection
