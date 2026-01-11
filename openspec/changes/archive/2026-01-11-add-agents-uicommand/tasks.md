## 1. Server Client API Enhancements

- [x] 1.1 Add `list_agents()` method to `Client` struct in `crates/client/src/client.rs`
- [x] 1.2 Ensure `get_session(session_id)` exists for session metadata lookup
- [x] 1.3 Add `update_session_agent(session_id, agent_id)` method to `Client` struct
- [x] 1.4 Add tests for agent-related client methods

## 2. CLI REPL Implementation

- [x] 2.1 Add `/agents` command handler to REPL command routing in `crates/cli/src/ui/repl.rs`
- [x] 2.2 Implement agent listing display (call `GET /api/v1/sessions/:id`, `GET /api/v1/agents`)
- [x] 2.3 Implement agent selection logic (call `PUT /api/v1/sessions/:id/agent`)
- [x] 2.4 Add error handling for invalid agent IDs and busy sessions

## 3. TUI Implementation

- [x] 3.1 Define `AgentResponse` in `crates/cli/src/ui/tui/action.rs` (or reuse from client) and add `AgentListReceived`, `AgentSwitched` actions
- [x] 3.2 Update `TuiState` in `crates/cli/src/ui/tui/state.rs` to include `AgentSelectionState` and add `InputMode::AgentSelection`
- [x] 3.3 Implement `AgentSelection` UI component in `crates/cli/src/ui/tui/ui/components/agent_selection.rs` (or similar)
- [x] 3.4 Update `draw` in `crates/cli/src/ui/tui/ui/mod.rs` to render agent selection when in `AgentSelection` mode
- [x] 3.5 Handle `/agents` command dispatch in `crates/cli/src/ui/tui/app.rs` (fetch agents async)
- [x] 3.6 Handle key events for `InputMode::AgentSelection` in `crates/cli/src/ui/tui/update.rs` (navigation, selection, cancel)
- [x] 3.7 Handle `AgentSwitched` action to show success toast

## 4. Verification

- [x] 4.1 Test `/agents` with no arguments shows list and current agent (REPL)
- [x] 4.2 Test `/agents <agent_id>` switches to specified agent (REPL)
- [x] 4.3 Test error handling for invalid agent ID (REPL)
- [x] 4.4 Test error handling when session is busy (REPL)
- [x] 4.5 Test subsequent chat requests use the new agent (REPL)
- [x] 4.6 Verify `/agents` command opens selection modal in TUI
- [x] 4.7 Verify agent list navigation and selection in TUI
- [x] 4.8 Verify agent switch updates session and shows confirmation in TUI
