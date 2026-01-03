# Tasks: Implement TUI Visual Polish

- [x] Create `crates/cli/src/ui/tui/theme.rs` with `Theme` struct and default semantic palette <!-- id: 0 -->
- [x] Refactor `crates/cli/src/ui/tui/mod.rs` to use `Theme` for transcript rendering instead of hardcoded colors <!-- id: 1 -->
- [x] Update `crates/cli/src/ui/tui/state.rs` to include `Toast` struct and `toast` field in `TuiState` <!-- id: 2 -->
- [x] Implement Toast rendering logic in `crates/cli/src/ui/tui/mod.rs` (overlay on top of existing UI) <!-- id: 3 -->
- [x] Implement `c` key handler in `crates/cli/src/ui/tui/mod.rs` to copy selected message and trigger Toast <!-- id: 4 -->
- [x] Implement streaming cursor/spinner rendering logic in `crates/cli/src/ui/tui/mod.rs` for `is_streaming` items <!-- id: 5 -->
- [x] Verify TUI color scheme matches requirements (User=Soft Blue, Assistant=Lavender, System=Grey) <!-- id: 6 -->
- [x] Verify "Copied!" toast appears and fades after 2 seconds <!-- id: 7 -->
- [x] Verify streaming cursor appears during assistant generation <!-- id: 8 -->
