# Refactor Session Context Management

## Summary
Introduce a dedicated session context component that encapsulates message history storage, safe context-window construction, and prompt-size compaction without exposing raw mutable message vectors.

## Motivation
The current `Session` model stores `history: Vec<Message>` directly and exposes it broadly. This makes it difficult to:
- Enforce invariants required by tool-calling (tool call messages must not be separated from their tool results).
- Add pruning/compaction logic in a single place.
- Evolve token estimation and model-specific limits without threading ad-hoc rules across call sites.

## Goals
- Keep `Session` focused on session identity and lifecycle state.
- Store messages inside a dedicated `Context` type owned by `Session`.
- Prevent callers from mutating message history in ways that break invariants.
- Provide a single method to build a provider-ready prompt message list.
- Provide a minimal, safe compaction mechanism based on estimated token limits.

## Non-Goals
- LLM-based summarization.
- Provider-specific exact tokenization.
- Persisted session storage or pagination of history.

## What Changes
- Replace `Session.history: Vec<Message>` with `Session.context: Context`.
- Introduce `ContextLimits` and a pluggable token estimation interface.
- Introduce an initial compaction policy that drops oldest non-pinned context blocks to fit a prompt budget.
- Treat tool-call exchanges as atomic blocks so compaction cannot break tool-call correctness.

## Impact
- Affected specs: `session-core`
- Affected code:
  - `crates/core/src/session.rs`
  - `crates/core/src/session/` (new module for context management)
  - Any code that reads/writes `Session.history`

