# Change: Refactor Session Architecture

## Why
Current Sisyphus architecture tightly couples `Session` within `Agent`. This prevents:
1.  **Multiple Sessions**: Managing multiple independent chats or sub-tasks.
2.  **Agent Switching**: Dynamically changing agents (e.g., "Planner" -> "Coder") within the same session history.
3.  **Concurrency**: Future support for parallel execution or background tasks.

OpenCode separates `Session` (State) from `Agent` (Configuration). Sisyphus should align with this to support advanced agentic workflows.

## What Changes
- **Decouple Session**: Remove `Session` ownership from `Agent`. Make `Session` a standalone entity holding history and status.
- **Introduce SessionManager**: A central registry to manage, load, and save sessions by ID.
- **Stateless Agent Execution**: Refactor `Agent` (or introduce `AgentRunner`) to take a `Session` and `AgentConfig` as arguments for execution, rather than owning the state.
- **Session Locking**: Implement a locking mechanism (like OpenCode's `busy` status) to prevent concurrent user inputs on the same session, while allowing for future parallel tool execution.

## Impact
- **New Capability**: `session-core`
- **Modified Capability**: `agent-core` (Interaction with session)
- **Affected Code**:
  - `crates/core/src/session.rs` (Refactor)
  - `crates/core/src/session/manager.rs` (New)
  - `crates/core/src/agent.rs` (Refactor signature)
  - `crates/cli/src/main.rs` (Update loop to use Manager)
