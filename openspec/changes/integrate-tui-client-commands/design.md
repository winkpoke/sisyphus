# Design: Integrate TUI Client Commands (Without EventBus)

## Context
The TUI implements MVU-style state transitions, where user inputs and backend events are translated into `Action`s and then mapped to a small set of side-effectful `TuiInstruction`s executed by the outer event loop.

The core `Command` trait is text-oriented and is primarily a server-side abstraction (tools + slash commands). UI-scoped commands are different: they must cause local UI effects and/or client API calls (create session, clear session, quit).

## Problems
- The proposal-level design using `SystemEvent` / `EventBus` as a control plane conflates two different event streams:
  - backend-to-client SSE events (`SystemEvent`)
  - client-local UI effects (quit, route command, invoke client API)
- Introducing UI-only `SystemEvent` variants creates protocol coupling and leaks UI concerns into the common event schema.
- The TUI has multiple slash routing paths; inconsistent parsing (whitespace splitting vs quoted args) can diverge from server semantics.

## Goals
- UI-scoped commands trigger explicit local instructions (MVU control plane), not `SystemEvent`.
- Slash command parsing is consistent with server semantics by reusing the shared core parser.
- Command palette shows a merged, deterministic list of local UI commands and remote server commands.

## Solution Architecture

### 1. Use MVU Instructions as the Control Plane
UI-scoped commands return a `TuiInstruction` (plus optional user-facing output such as a system transcript entry). The outer event loop executes the instruction (e.g., call `client.create_session()`, then dispatch `Action::SessionCreated`).

This keeps UI state changes local and testable and avoids introducing protocol-level event types for UI-only effects.

### 2. Shared Parsing + Deterministic Routing
For any input starting with `/`, parse the command using the shared core slash parser. The router then produces one of:
- Not a slash command (e.g., `//literal` escape)
- Local UI command (consume locally and produce a `TuiInstruction`)
- Remote slash command (forward to server)

This matches the model described in the related change “add-unified-client-command-dispatch” and avoids duplicated ad-hoc parsing.

### 3. Clear Ownership of Side Effects
- The update function decides *what* should happen (`TuiInstruction`).
- The outer event loop performs *how* it happens (async client calls) and emits follow-up `Action`s.

### 4. Command Discovery
The palette provides a merged command list:
- local UI commands (client-side)
- remote commands fetched from `/api/v1/commands` (server-side)

The UI may optionally annotate origin, but the minimum requirement is a stable merged list.

## Non-Goals
- Adding UI-only `SystemEvent` variants.
- Reworking the server-side slash command system.
- Redesigning the TUI’s MVU architecture.

## Alternatives Considered
- **EventBus control plane**: Rejected because it couples UI effects to a protocol event type and has weaker delivery semantics for “must happen” UI control flow.
- **Direct `App` mutation from commands**: Rejected because it creates ownership cycles and makes state mutation harder to test.
