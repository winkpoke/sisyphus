## Context
The server exposes a chat endpoint that forwards user input into the agent. Slash commands (e.g., `/new`, `/clear`, `/exit`) are parsed and executed in the agent command system, but the resulting `CommandEffect` is also interpreted and applied in the server HTTP handler. In addition, some effects are applied directly inside the agent command path.

This split ownership creates:
- Duplicate and potentially conflicting state transitions.
- Client inconsistencies (REPL vs TUI) that require local fallbacks.
- Unsafe semantics for process-wide behavior (e.g., treating `/exit` as server shutdown).

## Goals / Non-Goals
- Goals:
  - Provide a single server-side place to handle slash commands and apply their effects.
  - Make the HTTP layer a thin adapter and keep orchestration in core.
  - Ensure session-scoped effects are applied exactly once.
  - Define safe semantics for `/exit` and `/quit` that work in multi-session/multi-agent deployments.
- Non-Goals:
  - Implement multi-agent routing in this change.
  - Change the slash command parser semantics.
  - Redesign the event bus protocol.

## Decisions
- Decision: Introduce a core application service (e.g., `ChatService`) as the only entrypoint for chat turns.
  - It receives `(session_id, input)` and returns a response DTO containing:
    - assistant output
    - effective session id (for lifecycle changes)
    - explicit command effect metadata (for consistent client UX)
    - usage/model metadata (as currently returned)
- Decision: Centralize `CommandEffect` application in the core service.
  - Session-scoped effects (`ClearHistory`, `NewSession`) mutate session state via `Session`/`SessionManager` in exactly one place.
  - Client-scoped effects (`Exit`) do not shut down the server process.
- Decision: Implement `/quit` as a command alias for `/exit` in the command registry.
  - `/exit` remains the canonical implementation.
  - `/quit` resolves to `/exit` before execution.
  - Help/command discovery may list both spellings, but execution behavior is identical.
- Decision: Keep process shutdown out of user-level exit commands.
  - If needed, introduce an explicit admin-only shutdown command or rely on OS signals.

## Alternatives Considered
- Keep effect application in HTTP handlers.
  - Rejected: makes server layer responsible for business logic and complicates multi-agent.
- Apply effects only inside the agent.
  - Rejected: the agent currently does not own session creation and should remain stateless beyond operating on a provided `Session` reference.

## Risks / Trade-offs
- API change risk: adding effect metadata to chat responses may require updating clients.
  - Mitigation: make new fields optional/forward-compatible during migration.
- Behavior change risk: `/exit` no longer shuts down the server.
  - Mitigation: provide a separate admin shutdown mechanism if required.

## Migration Plan
1. Add the core `ChatService` and route server chat requests through it.
2. Remove `CommandEffect` handling from HTTP handlers.
3. Remove duplicate effect application from the agent path so effects apply exactly once.
4. Extend chat response DTO to include explicit effect metadata.
5. Update clients to rely on server-provided metadata and stop executing slash commands locally.

## Open Questions
- Should `/exit` end only the client connection, or also mark the session as inactive?
- Should a process-wide shutdown command exist, and if so, how should it be permission-gated?
