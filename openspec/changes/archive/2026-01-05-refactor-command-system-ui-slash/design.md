## Context
Today, slash-prefixed commands serve multiple purposes:
- Some behave like prompt templates expanded by the agent.
- Some drive session lifecycle (e.g. new session / clear history).
- Some are purely local UI behavior (e.g. debug toggles, quitting the UI).

This mixing of concerns results in inconsistent client behavior, unclear source of truth, and accidental coupling between chat turns and lifecycle operations.

## Goals / Non-Goals
- Goals:
  - Define two command kinds with a clear owner and execution model.
  - Preserve custom command loading from a directory as prompt templates.
  - Make command discovery deterministic and consistent across UIs.
  - Move lifecycle behavior to explicit server APIs invoked by UiCommands.
- Non-Goals:
  - Redesign the agent tool-permission system.
  - Introduce a new templating language beyond the current `{{args}}` substitution.
  - Change the fundamental chat turn loop semantics beyond command routing.

## Decisions

### Decision: Two command kinds with explicit ownership
- **SlashCommand** is expanded by the agent and becomes normal chat input before the model call.
- **UiCommand** is interpreted by the client UI runtime and is never injected into the model prompt unless the UI explicitly sends a follow-up chat message.

### Decision: UiCommand precedence on name collisions
If an input begins with `/` and matches a known UiCommand name, the client MUST treat it as a UiCommand, even if a SlashCommand with the same name exists.

Rationale:
- Ensures stable UX for lifecycle and local UI actions.
- Prevents custom templates from accidentally overriding local safety controls.

### Decision: Escape hatch for literal slash
Clients SHALL support `//` as an escape prefix meaning a literal leading slash should be sent as chat text.

Rationale:
- Users need a deterministic way to send literal `/something` without triggering command routing.

### Decision: Reserved UiCommand names cannot be overridden by custom SlashCommands
When loading custom SlashCommands from the command directory, the server MUST reject any custom command whose name collides with a reserved UiCommand name.

Rationale:
- Prevents custom templates from shadowing stable lifecycle and local UI behavior.
- Ensures deterministic routing and help/palette output.

### Decision: Server exposes SlashCommand metadata; clients merge
The server is the source of truth for SlashCommands available for the running server configuration.
Clients combine this list with their own UiCommands for discovery (palette, completion, help).

Rationale:
- SlashCommands include custom templates discovered server-side.
- UiCommands are UI-specific and can differ between CLI/TUI.

## Architecture

### Data model
SlashCommand discovery metadata returned by the server uses a stable shape:
- `name`: String (includes leading `/`)
- `description`: String
- `source`: `builtin` | `custom`

Clients MAY project the merged command list into a UI-oriented shape by adding a `kind` field with values `slash` or `ui`.

Templates are not returned to clients.

### Routing flow

#### Client input routing
1. If input starts with `//`, strip one leading `/` and send as chat text.
2. Else if input starts with `/` and the command name matches a UiCommand:
   - Execute the UiCommand locally.
   - If the UiCommand requires server state changes, call explicit endpoints.
3. Else if input starts with `/`:
   - Send the input to the chat endpoint unchanged.
   - The server-side agent performs SlashCommand expansion (built-in + custom) before the model call.
4. Else:
   - Send as normal chat input.

#### Server handling
- The chat endpoint processes chat input and performs SlashCommand expansion.
- Lifecycle and history changes are performed by explicit REST APIs called by UiCommands.

## Risks / Trade-offs
- Breaking the prior meaning of `/new` and `/clear` as chat-driven effects requires a migration path.
- Clients must implement routing consistently; discovery helps minimize mismatch.
- Name collisions between UiCommands and custom SlashCommands must be deterministic.

## Migration Plan
1. Add discovery endpoint for SlashCommands.
2. Add clear-history endpoint.
3. Implement UiCommand routing in clients with UiCommand precedence.
4. Deprecate chat-driven lifecycle effects for `/new`, `/clear`, `/exit`, `/quit`.
5. Optionally keep server-side compatibility for a limited transition period, but document that clients should prefer UiCommands + endpoints.

## Open Questions
None.
