## Context
The current system contains legacy lifecycle signaling via `CommandEffect` returned from chat turns. At the same time, the spec suite already establishes a split between:
- **SlashCommands**: server-side prompt expansion only.
- **UiCommands**: client-side commands that may call server endpoints.

In this model, session lifecycle changes (create new session, clear context, exit UI) are not properties of a chat turn result; they are explicit user-driven operations.

## Goals / Non-Goals
- Goals:
  - Remove `CommandEffect` as an API and runtime concept from chat orchestration.
  - Make chat responses purely conversational (assistant content + session id + usage/model metadata as applicable).
  - Prevent any accidental session mutations triggered by SlashCommand execution.
  - Reduce coupling between core/server and specific client UX workflows.
- Non-Goals:
  - Redesign the UiCommand router UX or palette behavior.
  - Rename or redesign existing endpoints beyond what is necessary to eliminate effect metadata.
  - Introduce new server authorization models for lifecycle endpoints.

## Decisions
- Decision: Treat lifecycle as explicit operations, not chat outcomes.
  - Session creation occurs via `POST /api/v1/sessions`.
  - Session clearing occurs via `POST /api/v1/sessions/:id/clear`.
  - UI exit/debug remain local to the client UI.

- Decision: Chat endpoint is not a control channel.
  - `POST /api/v1/sessions/:id/chat` MUST NOT embed or imply lifecycle effects.

- Decision: Server handlers delegate chat work without applying command effects.
  - The HTTP layer delegates to a core chat service for:
    - SlashCommand parsing/expansion
    - normal chat execution
  - The HTTP layer does not independently apply lifecycle effects.

## Risks / Trade-offs
- Client migration risk: Some clients may currently rely on `effect` metadata.
  - Mitigation: Update client routing to treat lifecycle commands as UiCommands and call endpoints.

- Behavior drift risk: Users sending `/clear` as chat text might expect clearing.
  - Mitigation: UiCommand routers take precedence for reserved UiCommand names; users can send literal slash text via escaping.

## Migration Plan
1. Remove `effect` from chat response contracts.
2. Remove `CommandEffect` from core orchestration.
3. Ensure clients execute lifecycle via UiCommands and session endpoints.
4. Update tests to validate absence of effect metadata and correct endpoint behavior.

## Open Questions
- Should the server provide any explicit “client should close” response for `/exit`, or is that strictly local?
