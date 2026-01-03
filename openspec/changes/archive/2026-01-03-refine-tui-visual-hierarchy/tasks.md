# Tasks: Refine TUI Visual Hierarchy

- [x] Update `TuiState` struct in `crates/cli/src/ui/tui/state.rs` to include `status`, `spinner_frame`, `active_model`, `token_usage`, and `context_title`. <!-- id: update-state-struct -->
- [x] Initialize new state fields in `TuiState::new`. <!-- id: init-state -->
- [x] Update `Tui::run` loop in `crates/cli/src/ui/tui/mod.rs` to increment `spinner_frame` on tick. <!-- id: update-loop -->
- [x] Implement Status Bar layout and rendering in `crates/cli/src/ui/tui/mod.rs`. <!-- id: implement-status-bar -->
- [x] Refine Transcript rendering with Padding and Dynamic Header in `crates/cli/src/ui/tui/mod.rs`. <!-- id: refine-transcript -->
- [x] Connect `Action` events to update status and token usage in `crates/cli/src/ui/tui/mod.rs`. <!-- id: connect-events -->
