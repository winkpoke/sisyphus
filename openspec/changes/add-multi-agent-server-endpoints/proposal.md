# Change: Add Multi-Agent Server Endpoints

## Why

The current Sisyphus server exposes a single global Agent instance via `AppState.agent` and `AppState.chat_service`, and all chat requests are routed through this single Agent. This prevents multi-agent scenarios where:

- Different sessions can use different Agents (with distinct instructions, modes, and permissions).
- Agents can own their own LLM model settings (e.g., different OpenAI models per Agent).
- Clients can explicitly select and switch the Agent used by a session.

Today:

- `ChatService` holds a single `Arc<Agent>` and always uses it for all sessions.
- The HTTP API does not expose any notion of Agent identity or allow per-session agent selection.
- `/api/v1/model` and `/api/v1/commands` implicitly reflect the single global Agent.

This contradicts the multi-agent architecture intent in `session-core` and `agent-core`, where Session is pure state and Agent is configurable behavior. It also prevents future workflows like planner → coder → reviewer within the same conversation.

## What Changes

At the specification level, introduce explicit multi-agent awareness in the server API:

1. **Agent Discovery Endpoints**
   - Add `GET /api/v1/agents` to list available Agents and their model settings.
   - Add `GET /api/v1/agents/:agent_id` to fetch metadata for a specific Agent.

2. **Session Creation with Agent Selection**
   - Extend `POST /api/v1/sessions` to accept an optional `agent_id` field.
   - Sessions default to a configured primary Agent when `agent_id` is omitted.

3. **Per-Session Agent Assignment API**
   - Add `GET /api/v1/sessions/:id` to return session metadata including `agent_id`.
   - Add `PUT /api/v1/sessions/:id/agent` to change the Agent associated with a session.
   - Enforce safe semantics when the session is busy (no concurrent agent switches during a turn).

4. **Chat Response Includes Agent Metadata**
   - Extend the chat response contract to include the effective `agent_id` used for the turn.
   - Clarify that the `model` field in the chat response comes from the selected Agent's model configuration.

5. **Clarify Legacy Endpoints**
   - Specify how `/api/v1/model` and `/api/v1/commands` behave in a multi-agent environment.
   - Prefer scoped or agent-aware endpoints (e.g., `GET /api/v1/agents/:agent_id/commands`) in new clients.

This proposal only defines the server API and behavior. It does not change the core Agent implementation details beyond what is needed to expose configuration already covered by `agent-core`.

## Impact

- **New Capability**: Multi-agent routing at the HTTP API layer.
- **Modified Capability**:
  - `server-core`: add Agent discovery and per-session Agent assignment.
  - `session-core`: extend Session state to track the current Agent identity.
  - `agent-core`: clarify that model settings are part of Agent configuration and are surfaced via the new endpoints.
- **Affected Code (implementation stage, not part of this proposal)**:
  - `crates/server/src/lib.rs` (AppState, routes, handlers)
  - `crates/core/src/service.rs` (ChatService routing to the selected Agent)
  - `crates/core/src/session.rs` (Session DTO/serialization for `agent_id`)
  - `crates/client/src/client.rs` (Client DTOs for chat and session operations)

## Out of Scope

- Implementing slash commands to switch Agents (e.g., `/agent use coder`).
- Redesigning the Agent configuration format beyond existing `agent-core` requirements.
- Adding new Agent types or dynamic Agent plugins; this change only provides API surfaces for Agents that already exist in the runtime.

