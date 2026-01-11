# /agents UiCommand Design

## Context
The Sisyphus server supports multiple agents (e.g., "plan", "build") and allows sessions to be associated with specific agents. The server provides:
- `GET /api/v1/agents` - Lists all available agents
- `GET /api/v1/sessions/S1` - Returns session metadata including `agent_id`
- `PUT /api/v1/sessions/S1/agent` - Changes the agent for a session

However, there is currently no user-facing way to:
1. Discover what agents are available
2. See which agent is currently active for a session
3. Switch to a different agent

This design proposes adding a `/agents` **UiCommand** (not SlashCommand) to fill this gap.

## Goals / Non-Goals

### Goals
- Provide an intuitive way for users to list available agents and see the current session's agent
- Enable users to switch agents within a session via a simple command
- Maintain consistency with existing UiCommand patterns (`/new`, `/clear`, `/exit`)
- Leverage existing server APIs without requiring new server endpoints

### Non-Goals
- Implementing interactive agent selection (initial version will use argument-based selection)
- Implementing `/agents` support in the TUI frontend
- Adding server-side changes (all required APIs already exist)
- Implementing agent comparison or evaluation features
- Creating new SlashCommand templates or prompt-based agent selection

## Decisions

### Decision 1: /agents is a UiCommand, not a SlashCommand
The `/agents` command must be a **UiCommand** because:
1. It changes session state (associates a different agent with the session)
2. The `/new` and `/clear` commands are already UiCommands and also change session state
3. It calls server endpoints directly rather than being expanded into prompt text
4. UiCommands have precedence over SlashCommands with the same name

**Alternatives considered:**
- **SlashCommand**: Rejected because it would be a prompt template that doesn't directly mutate session state. Changing agents requires calling a server API, which UiCommands support but SlashCommands do not.

### Decision 2: Two modes of operation (list vs. switch)
The `/agents` command supports two modes:
1. **List mode** (no arguments): Display all available agents and highlight the current session's agent
2. **Switch mode** (with agent ID argument): Change the session's agent to the specified ID

**Rationale:**
- This mirrors common CLI patterns (e.g., `git branch` vs. `git branch <name>`)
- No additional command needed for switching (e.g., `/switch-agent`)
- Simple and discoverable: `/agents` shows options, `/agents <id>` makes the change

**Alternatives considered:**
- **Separate commands** (`/agents` to list, `/switch-agent <id>` to switch): Rejected because it adds cognitive overhead and two commands for one workflow.
- **Always interactive** (prompt user to select after listing): Rejected because it prevents scripting and is harder to implement initially.

### Decision 3: Minimal CLI changes, leverage existing client methods
The implementation will:
- Add two new methods to `Client` struct: `list_agents()` and `update_session_agent(session_id, agent_id)`
- Reuse an existing `get_session(session_id)` client method, or add one if missing
- Handle the `/agents` command in the existing command routing logic in `repl.rs`
- Reuse existing error handling patterns from `/new` and `/clear` commands

**Rationale:**
- Minimal code changes reduce risk
- Consistent with existing UI command implementation
- Easy to test and maintain

**Alternatives considered:**
- **Create a new `AgentManager` struct**: Rejected as over-engineering for a simple command.
- **Add interactive menu using reedline**: Deferred to future enhancement (see Open Questions).

### Decision 4: Display format for agents
When listing agents, the display will show:
- Agent ID (for use with the switch command)
- Agent name
- Agent description
- Agent model
- Current session's agent indicated with `*` or `(current)`

**Example output:**
```
Available Agents:
  * plan      - Planning agent for analysis and strategy (gpt-4-turbo)
    build     - Build agent for execution and implementation (gpt-4-turbo)
```

**Rationale:**
- Provides all necessary metadata for decision-making
- Makes the active agent obvious
- Clear mapping between name and ID for the switch command

**Alternatives considered:**
- **Table format**: Rejected because terminal width varies and tables can wrap poorly.
- **JSON output**: Rejected because this is a user-facing CLI command, not an API endpoint.

## Architecture

### Data Flow

1. **List mode** (`/agents`):
   ```
   User input: "/agents"
   CLI → Client.get_session(S1) → Server (GET /api/v1/sessions/S1) → Returns session metadata
   CLI → Client.list_agents() → Server (GET /api/v1/agents) → Returns agents
   CLI → Display agents with current agent marked using session.agent_id
   ```

2. **Switch mode** (`/agents <agent_id>`):
   ```
   User input: "/agents build"
   CLI → Client.update_session_agent(session_id, "build")
         → Server (PUT /api/v1/sessions/S1/agent)
         → Returns updated session metadata
   CLI → Display success message and update local session tracking if needed
   ```

### Code Locations

- **Client API** (`crates/client/src/client.rs`):
  - `pub async fn list_agents(&self) -> Result<Vec<Agent>>`
  - `pub async fn get_session(&self, session_id: &str) -> Result<Session>`
  - `pub async fn update_session_agent(&self, session_id: &str, agent_id: &str) -> Result<Session>`

- **CLI REPL** (`crates/cli/src/ui/repl.rs`):
  - Add `/agents` case to existing command routing match statement
  - Implement display logic for list mode
  - Implement switch logic with error handling

### Error Handling

The command must handle these error cases:
1. **Network/API errors**: Server unreachable, timeout, 5xx errors
2. **Not found errors**: Invalid agent ID (4xx from server)
3. **Session busy errors**: Agent change rejected because session is busy
4. **Empty agent list**: No agents configured on server

All errors should:
- Display user-friendly error messages
- Not crash the CLI
- Leave the session in a consistent state (no partial updates)

## TUI Interaction Design

### Agent Selection Flow
1. **Trigger**: User types `/agents` in the input field or selects it from the command palette.
2. **State Transition**:
   - The application fetches the list of available agents from the server.
   - The application enters `InputMode::AgentSelection`.
   - A modal or overlay appears, listing the available agents.
3. **Selection UI**:
   - Displays a list of agents (Name + ID + Description).
   - Highlights the currently active agent for the session.
   - Allows navigation using Up/Down arrow keys.
4. **Action**:
   - Pressing `Enter` selects the highlighted agent.
   - The application sends a `PUT` request to update the session's agent.
   - On success, a toast notification confirms the switch ("Switched to agent: <name>").
   - The modal closes, and the application returns to `InputMode::Normal`.
   - Pressing `Esc` cancels the operation and returns to `InputMode::Normal` without changes.

### State Management
- **`AgentSelectionState`**: Holds the list of `AgentResponse` objects and the currently `selected_index`.
- **`TuiState`**: Stores an optional `AgentSelectionState`.
- **`InputMode`**: Adds `AgentSelection` variant.

### Error Handling
- If fetching agents fails, show a toast error.
- If updating the agent fails, show a toast error and remain in (or return to) normal mode.

## Risks / Trade-offs

### Risk 1: Agent ID confusion
Users might not understand the difference between agent name and agent ID.

**Mitigation:**
- Display both ID and name when listing
- Use agent names as IDs in the default configuration (e.g., "plan", "build")
- Provide clear error messages if an invalid ID is used

### Risk 2: Session state inconsistency
If the CLI crashes after sending the agent change request but before updating local state, there could be inconsistency.

**Mitigation:**
- Update local session tracking only after receiving success response from server
- Store session ID as the source of truth (already the case)
- Consider fetching updated session metadata after agent change

### Risk 3: TUI implementation complexity
The CLI may present different UX constraints between REPL output and future TUI overlays.

**Mitigation:**
- Limit the initial scope to REPL
- Revisit TUI integration as a follow-up change with `cli-tui` spec deltas

## Migration Plan

No migration is needed because:
- Existing sessions and agents are not affected
- The command is additive (adds new functionality)
- All required server APIs already exist

**Implementation Steps:**
1. Add or reuse client methods (`get_session`, `list_agents`, `update_session_agent`)
2. Implement `/agents` handler in REPL
3. Test with various scenarios (list, switch, errors)
4. Propose a follow-up change for TUI integration if needed

**Rollback:**
- If issues arise, remove the `/agents` command handler
- Remove added client methods (no breaking changes)

## Open Questions

### Q1: Should `/agents` support interactive selection via reedline menu?
**Context:** The existing `/` keybinding triggers a completion menu for commands. Could we provide similar interactive selection for agents?

**Discussion:**
- Pro: Better UX, less typing, harder to make mistakes
- Con: More complex to implement, may be overkill for initial version

**Recommendation:** Implement argument-based selection first (`/agents <id>`). Add interactive selection as a follow-up enhancement if user feedback indicates demand.

### Q2: Should the command display agent capabilities or metadata beyond basic info?
**Context:** Agents may have additional metadata (capabilities, configuration, etc.) in the future.

**Discussion:**
- Pro: Helps users choose the right agent
- Con: Clutters display, may not be available in current API

**Recommendation:** Start with basic metadata (id, name, description, model). Extend display if/when additional metadata becomes available via the API and user testing indicates value.

### Q3: Should changing agent clear the session context?
**Context:** Some users might expect a "fresh start" when switching agents, others might want to preserve context.

**Discussion:**
- Pro: Avoids confusion from agent-specific context
- Con: Loses valuable conversation history

**Recommendation:** Preserve context (current behavior of agent change API). Add a separate `/new` + `/agents` workflow if users want both fresh session and new agent. Document this pattern in help.
