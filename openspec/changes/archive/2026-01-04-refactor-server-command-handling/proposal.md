# Change: Refactor Server-Side Command Handling

## Why
Server-side chat currently applies command-driven effects in multiple places (agent and HTTP handler), which creates unclear ownership and inconsistent behavior across clients. This also makes future multi-session/multi-agent operation riskier, especially for process-wide side effects like shutdown.

## What Changes
- Introduce a single application-layer entrypoint in core (e.g., `ChatService`) for handling chat requests and command effects.
- Ensure the HTTP server becomes a thin transport adapter that delegates chat handling to this core service.
- Define a single, deterministic place where `CommandEffect` is interpreted and applied to session state.
- Clarify semantics for `/exit` and `/quit` so they are session/client-scoped, not process-scoped.
- Treat `/quit` as an alias of `/exit` (single canonical implementation).
- Extend the chat response contract to include enough metadata for clients to handle command-driven UX consistently (e.g., an explicit command effect, effective session id).

## Impact
- Affected specs: `server-core`, `session-core`, `slash-commands`
- Affected code (implementation stage):
  - `crates/server/src/lib.rs` (remove effect interpretation from HTTP handlers)
  - `crates/core/src/agent.rs` (remove duplicate effect application logic)
  - `crates/core/src/session/*` and `crates/core/src/command/*` (integrate via a unified core service)
  - `crates/client/src/*` and `crates/cli/src/ui/*` (consume explicit effect metadata; reduce local command handling)

