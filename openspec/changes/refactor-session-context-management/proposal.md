# Change: Refactor Session Context Management

## Why
`Session` currently exposes raw message history as `Vec<Message>`. This makes it easy for call sites to:
- Break tool-calling correctness by separating an assistant tool-call message from its tool results.
- Duplicate prompt-building logic across the codebase.
- Add ad-hoc pruning rules that become inconsistent over time.

This change introduces a dedicated `Context` component to own the message lifecycle and to construct a safe, provider-ready context window on demand.

## What Changes
- Replace `Session.history: Vec<Message>` with `Session.context: Context`.
- Provide invariant-enforcing append APIs so tool exchanges are always recorded and compacted atomically.
- Add a single render entrypoint that builds the provider-ready message list for a completion request.
- Add minimal, deterministic compaction based on a prompt token budget and a pluggable token estimator.
- **BREAKING**: Remove broad direct mutation of session history (internal API change). Call sites must use the context API.

## Impact
- Affected specs: `session-core`
- Related specs: `agent-core` (system prompt generation is injected at request time)
- Affected code:
  - `crates/core/src/session.rs`
  - `crates/core/src/session/` (context module)
  - `crates/core/src/agent.rs` (prompt construction and history access)
  - Command handlers that mutate `Session.history` directly

