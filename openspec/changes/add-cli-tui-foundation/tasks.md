## 1. Implementation
- [ ] 1.1 Add a new TUI module/crate boundary and UI selection flag
- [ ] 1.2 Implement terminal init/restore, panic-safe cleanup, and input event stream
- [ ] 1.3 Build a minimal TUI layout (transcript + input) with stable frame redraw
- [ ] 1.4 Wire backend event stream into the TUI event loop via typed AppEvent
- [ ] 1.5 Preserve current session-id switching behavior and clean shutdown

## 2. Validation
- [ ] 2.1 Add unit tests for core UI state transitions (no terminal required)
- [ ] 2.2 Add at least one integration test for interactive mode selection (TTY vs non-TTY)
- [ ] 2.3 Run formatting, lint, and test suite (cargo fmt, cargo clippy, cargo test)

