# Change: Remove CommandEffect

## Why
The current implementation still carries a `CommandEffect` concept (e.g., `NewSession`, `ClearHistory`, `Exit`) that couples chat turns to UI/session lifecycle control flow.

However, the current OpenSpec direction already separates responsibilities:
- **SlashCommands** are prompt templates expanded server-side and MUST NOT mutate session lifecycle.
- **UiCommands** are handled by the client UI runtime and may call explicit server endpoints (e.g., clear session, create session).

Keeping `CommandEffect` in the core chat path creates duplicate control planes (effects vs endpoints), increases client/server coupling, and risks inconsistent behavior across clients.

## What Changes
- Remove `CommandEffect` from core command outcomes and server chat response metadata.
- Ensure chat orchestration is limited to SlashCommand expansion and normal chat turns.
- Move any remaining lifecycle behaviors to explicit APIs and client UiCommand routing.
- Remove/replace any spec requirements that imply slash-command-driven effects or effect application in HTTP handlers.

## Impact
- Affected specs: `server-core`, `session-core`
- Affected code (implementation stage):
  - `crates/core/src/command/*` (remove `CommandEffect`, adjust outcomes)
  - `crates/core/src/service.rs` (remove effect interpretation in chat flow)
  - `crates/server/src/lib.rs` (remove `effect` from chat response DTO)
  - `crates/client/src/*` and `crates/cli/src/ui/*` (stop consuming effect metadata; rely on UiCommands + endpoints)

**BREAKING**: `POST /api/v1/sessions/:id/chat` responses will no longer include an `effect` field.
