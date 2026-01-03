1.  [ ] Define `Action` enum in `crates/cli/src/ui/tui/action.rs` <!-- id: 0 -->
2.  [ ] Create `App` struct in `crates/cli/src/ui/tui/app.rs` wrapping `TuiState` <!-- id: 1 -->
3.  [ ] Implement `update` function in `crates/cli/src/ui/tui/update.rs` handling basic actions <!-- id: 2 -->
4.  [ ] Refactor `event.rs` to yield `Action` instead of raw events <!-- id: 3 -->
5.  [ ] Extract Transcript UI into `crates/cli/src/ui/tui/ui/components/transcript.rs` <!-- id: 4 -->
6.  [ ] Extract Input UI into `crates/cli/src/ui/tui/ui/components/input.rs` <!-- id: 5 -->
7.  [ ] Extract Overlay UI into `crates/cli/src/ui/tui/ui/components/overlay.rs` <!-- id: 6 -->
8.  [ ] Implement main layout in `crates/cli/src/ui/tui/ui/mod.rs` <!-- id: 7 -->
9.  [ ] Rewrite main loop in `crates/cli/src/ui/tui/mod.rs` to use the new architecture <!-- id: 8 -->
10. [ ] Verify TUI functionality (manual test) <!-- id: 9 -->
