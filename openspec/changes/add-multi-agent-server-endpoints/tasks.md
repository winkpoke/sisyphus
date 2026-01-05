# Tasks: Add Multi-Agent Server Endpoints

1. **Agent Registry and Metadata**
   - [ ] Introduce an internal Agent registry that exposes `id`, `name`, `description`, and model settings for each Agent.
   - [ ] Ensure registry integrates with existing AgentConfig and provider initialization.

2. **Server AppState and Routing**
   - [ ] Update `AppState` to reference the Agent registry instead of a single Agent.
   - [ ] Wire new routes into the Axum router for `/api/v1/agents` and related endpoints.

3. **Agent Discovery Endpoints**
   - [ ] Implement `GET /api/v1/agents` to list Agents with their model names.
   - [ ] Implement `GET /api/v1/agents/:agent_id` to return a single Agent's metadata.

4. **Session-Agent Association**
   - [ ] Extend the Session data model and DTOs to include `agent_id`.
   - [ ] Default new sessions to a configured primary Agent when `agent_id` is not specified.
   - [ ] Update `POST /api/v1/sessions` to accept an optional `agent_id` and validate it.

5. **Session Agent Management Endpoints**
   - [ ] Implement `GET /api/v1/sessions/:id` to return session metadata including `agent_id`.
   - [ ] Implement `PUT /api/v1/sessions/:id/agent` to update the session's `agent_id` with proper validation and busy-state handling.

6. **ChatService and Chat Handler Updates**
   - [ ] Refactor `ChatService` to resolve the Agent from the session's `agent_id` on each turn.
   - [ ] Ensure the chat response includes the effective `agent_id` and `model` from the selected Agent.
   - [ ] Keep command effect semantics (`NewSession`, `ClearHistory`, etc.) unchanged.

7. **Legacy Endpoint Behavior**
   - [ ] Decide and implement the behavior of `/api/v1/model` and `/api/v1/commands` in a multi-agent environment (e.g., default Agent only).
   - [ ] Add tests or documentation notes to discourage new clients from relying on global-only endpoints.

8. **Testing and Validation**
   - [ ] Add unit and integration tests for the new endpoints and multi-agent routing behavior.
   - [ ] Run `cargo fmt`, `cargo clippy`, and `cargo test` to validate the implementation.
   - [ ] Validate the OpenSpec change with `openspec validate add-multi-agent-server-endpoints --strict`.

