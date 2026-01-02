# Refactor Session Context Management Design

## Context
Sisyphus currently stores session messages as a raw `Vec<Message>` inside `Session`. This is simple but does not protect key invariants (especially around tool calling) and does not provide a clear integration point for context-window compaction.

## Goals / Non-Goals

### Goals
- Encapsulate message lifecycle behind a small API.
- Ensure tool call messages and tool result messages are compacted as a unit.
- Make compaction request-time and provider-limit-aware.
- Keep initial implementation minimal and predictable.

### Non-Goals
- Summarization or semantic pruning.
- Introducing new external dependencies for exact token counting.

## Decisions

### Decision: Store context as structured blocks, not raw messages
Represent context as a private sequence of blocks, where blocks model atomic units of conversation that must not be split.

**Why**
- Prevents accidental invariant breaks (e.g., dropping the assistant tool-call message but retaining tool results).
- Makes pruning rules straightforward: drop whole blocks from the front.

### Decision: Compaction is explicit at prompt-build time
Compaction should run when building a completion request, because that is when the system knows the effective model budget and the reserved completion tokens.

**Why**
- Avoids hard-coded token limits in session state.
- Avoids repeated compaction work during message appends.

## Proposed API

### Types
- `Context`: owned by `Session`, stores the conversation.
- `ContextLimits`: `max_prompt_tokens` and optional `reserved_completion_tokens`.
- `TokenEstimator`: trait for estimating token usage.

### Block Model
`Context` stores a private `Vec<Block>` where `Block` is one of:
- `Pinned(Message)`: never dropped (e.g., system prompt).
- `Turn { user: Message, assistant: Option<Message> }`: a normal conversation turn.
- `ToolExchange { assistant: Message, tool_results: Vec<Message> }`: atomic tool-calling exchange.

### Key Operations
- Append operations that enforce invariants:
  - push pinned system message
  - push user message
  - push assistant message
  - begin tool exchange (assistant with tool calls)
  - push tool result (must attach to most recent tool exchange)
- `render(limits, estimator, policy) -> Vec<Message>`:
  - builds a provider-ready linear message list
  - runs compaction if `limits` are provided

## Compaction Policy (Initial)

### Inputs
- `ContextLimits.max_prompt_tokens`
- `TokenEstimator` (heuristic implementation to start)

### Algorithm
1. Render the context into a linear `Vec<Message>`.
2. Estimate tokens for the rendered prompt.
3. If it exceeds `max_prompt_tokens`, drop the oldest non-pinned blocks and re-render.
4. Repeat until within budget or only pinned blocks remain.

### Invariants
- Pinned blocks are always preserved.
- Tool exchanges are dropped or preserved as a unit.
- Message ordering is stable and deterministic.

## Alternatives Considered
- Storing raw `Vec<Message>` with `get_history_mut`: rejected due to invariant leakage.
- Implementing summarization now: rejected to keep scope minimal.

## Risks / Trade-offs
- Heuristic token estimation may under/over-estimate, causing compaction to be slightly aggressive or insufficient.
  - Mitigation: keep estimator pluggable so exact tokenization can be added later.

## Migration Plan
1. Introduce `Context` and refactor `Session` to use it.
2. Update call sites to use `Session::add_message` / context append APIs.
3. Add unit tests around tool exchange atomicity and compaction behavior.

