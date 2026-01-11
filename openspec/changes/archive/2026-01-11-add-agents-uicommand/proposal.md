# Change: Add /agents UiCommand for primary agent selection

## Why
Users need an easy way to discover and switch between available agents within a session. Currently, while the server supports multiple agents and agent selection, there is no user-facing command to view available agents or change the active agent for a session.

## What Changes
- Add a new UiCommand `/agents` to list available agents and enable agent switching
- Display current agent for the active session
- Provide an argument-based selection mechanism to change the session's agent
- Call existing server endpoints (`GET /api/v1/agents`, `GET /api/v1/sessions/:id`, and `PUT /api/v1/sessions/:id/agent`)
- Implement TUI integration for agent selection using a dedicated selection mode

## Impact
- Affected specs: `cli-architecture`, `tui-architecture`
- Affected code: `crates/cli/src/ui/repl.rs` (REPL), `crates/cli/src/ui/tui/` (TUI)
- Uses existing server APIs: `GET /api/v1/agents`, `GET /api/v1/sessions/:id`, `PUT /api/v1/sessions/:id/agent`
