# Design: Multi-Agent Server Endpoints

## Context

Sisyphus currently exposes a single Agent instance via the server `AppState`. All HTTP chat requests are routed through this Agent, and the chat response includes a single `model` value that reflects the underlying provider configuration.

OpenSpec `session-core` defines Session as a pure state container, and `agent-core` defines configurable Agents with metadata, permissions, and model settings. To support multi-agent scenarios (e.g., planner/coder/reviewer, per-session Agent selection), the server must:

- Make Agents addressable via HTTP.
- Allow sessions to be created with, and later switch to, a specific Agent.
- Ensure chat executes against the Agent selected for that session.

## High-Level Architecture

### Agent Registry

Introduce an in-memory Agent registry owned by the server runtime:

- Maps a stable `agent_id` to an `Agent` instance and its `AgentConfig`.
- Provides a read-only API for the HTTP layer to list and fetch agents.
- Encapsulates any logic for choosing the default/primary Agent.

This keeps `Agent` runtime ownership out of `Session`, preserving the `session-core` requirement that Session is pure state and the `agent-core` requirement that Agents are configurable behavior.

### Session-Agent Association

Extend `Session` to include an `agent_id` field:

- Serializable and persisted along with other session metadata.
- Represents the currently selected Agent for that session.
- Initialized when the session is created (from request or default agent).

Session remains unaware of Agent internals; it only stores the identity required to route work.

### Chat Routing

Refactor `ChatService` to:

- Load the `Session` via `SessionManager`.
- Read `session.agent_id` to determine which Agent should handle the turn.
- Resolve the Agent via the Agent registry.
- Execute the chat turn using the selected Agent.
- Apply command effects (`NewSession`, `ClearHistory`, etc.) as already defined.

The HTTP handler remains a thin adapter, consistent with the `server-core` requirement that handlers delegate command handling and effect application to core services.

### Endpoint Responsibilities

- `/api/v1/agents` and `/api/v1/agents/:agent_id` expose Agent metadata (including model) for discovery and UI.
- `POST /api/v1/sessions` accepts an optional `agent_id` and initializes the session accordingly.
- `GET /api/v1/sessions/:id` returns session metadata including `agent_id`.
- `PUT /api/v1/sessions/:id/agent` updates the `agent_id` safely when the session is not busy.
- `POST /api/v1/sessions/:id/chat` always uses the `agent_id` stored on the Session when selecting an Agent.

## Trade-offs

- **Pros**
  - Keeps Session and Agent decoupled while enabling multi-agent routing.
  - API is explicit and discoverable; clients can choose and switch Agents without slash commands.
  - Aligns with existing specs and patterns: thin HTTP handlers and centralized ChatService orchestration.

- **Cons**
  - Requires additional state in sessions (`agent_id`) and a new registry abstraction.
  - Legacy endpoints (`/api/v1/model`, `/api/v1/commands`) need clear behavior in a multi-agent world.

## Alternatives Considered

1. **Embed Agent reference in Session**
   - Rejected: violates Session serialization/persistence requirements and tightly couples state to runtime objects.

2. **Expose Agent switching only via slash commands**
   - Rejected: conflicts with the requirement that HTTP handlers remain thin and places multi-agent routing in user-entered strings instead of explicit APIs.

3. **Per-request Agent override parameter on `/chat`**
   - Rejected: makes it harder to reason about which Agent is associated with a session and can conflict with session-scoped UX/permissions.

