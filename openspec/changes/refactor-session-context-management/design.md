# Refactor Session Context Management Design

## Context
Sisyphus stores session messages as a raw `Vec<Message>` inside `Session`. The Agent builds completion requests by injecting a system prompt at request time and then cloning `Session.history` into the provider request.

This is simple, but it:
- Exposes invariants to call sites (especially tool-calling exchanges).
- Provides no single integration point for context window construction or prompt-size compaction.
- Does not model the actual execution flow where one user input can trigger multiple assistant/tool steps.

## Goals / Non-Goals

### Goals
- Encapsulate message lifecycle behind a small API that enforces correctness.
- Preserve tool exchange atomicity during storage and compaction.
- Support request-time, model-budget-aware compaction without storing model limits in session state.
- Keep initial implementation deterministic and provider-agnostic.

### Non-Goals
- Summarization or semantic pruning.
- Introducing new external dependencies for exact token counting.
- Persisted session storage, pagination, or cross-process session sharing.

## Decisions

### Decision: Model context as turns with steps (not raw messages)
Represent context as a sequence of entries, where the primary unit is a user turn that may contain multiple assistant/tool steps.

**Why**
- Matches current runtime behavior: a single user message can lead to an assistant response, tool calls, tool results, and follow-up assistant messages.
- Allows compaction to drop whole turns while preserving within-turn invariants.

### Decision: Keep system prompt injection outside stored context
The system prompt is generated dynamically (environment/date/custom rules) and injected at completion request build time.

**Why**
- Avoids freezing dynamic prompt fields inside long-lived session state.
- Aligns with the Agent's current request-building pattern.

### Decision: Compaction is explicit at prompt-build time
Compaction runs when building a completion request because that is when the effective model budget and reserved completion tokens are known.

**Why**
- Avoids hard-coded token limits in session state.
- Prevents repeated compaction work on every append.

## Proposed API

### Types
- `Context`: owned by `Session`, stores conversation state.
- `ContextLimits`: `max_prompt_tokens` and optional `reserved_completion_tokens`.
- `TokenEstimator`: trait for estimating token usage.

### Data Model
`Context` stores a private `Vec<Entry>`, where `Entry` is one of:
- `Pinned(Message)`: never dropped by compaction.
- `UserTurn { user: Message, steps: Vec<Step> }`.

`Step` is one of:
- `Assistant(Message)`: an assistant message without tool calls.
- `ToolExchange { assistant: Message, tool_results: Vec<Message> }`: an assistant tool-call message plus its tool results.

### Key Operations
- Append operations that enforce invariants:
  - push pinned message
  - push user message (starts a new `UserTurn`)
  - push assistant message (appends to current `UserTurn`)
  - begin tool exchange (assistant with tool calls; appends a `ToolExchange` step)
  - push tool result (must attach to the most recent open `ToolExchange`)
- `render(system_messages, limits, estimator) -> RenderedContext`:
  - builds a provider-ready linear message list
  - runs compaction if `limits` are provided

`RenderedContext` includes:
- `messages: Vec<Message>`
- `estimated_prompt_tokens: u32`
- `dropped_turns: u32`

## Compaction Policy (Initial)

### Budget Calculation
If `reserved_completion_tokens` is provided, the effective prompt budget is:

`prompt_budget = max_prompt_tokens - reserved_completion_tokens`

If subtraction underflows, treat `prompt_budget` as `0`.

### Algorithm
1. Estimate token usage per entry (pinned and turns) using `TokenEstimator`.
2. If the total exceeds `prompt_budget`, drop the oldest non-pinned `UserTurn` entries until within budget.
3. Render the remaining entries exactly once into a linear message list.

### Failure Modes
- If pinned entries plus injected system messages exceed `prompt_budget`, rendering MUST fail with a deterministic error (e.g., `PromptBudgetExceeded`).

### Invariants
- Pinned entries are always preserved by compaction.
- Tool exchanges are dropped or preserved as a unit.
- A tool result MUST NOT be appended unless the most recent step is an open tool exchange.
- Message ordering is stable and deterministic.

## Alternatives Considered
- Storing raw `Vec<Message>` with `get_history_mut`: rejected due to invariant leakage.
- Summarization now: rejected to keep scope minimal and deterministic.
- Re-render loop compaction (render/estimate/drop/re-render): rejected due to avoidable O(n^2) behavior.

## Risks / Trade-offs
- Heuristic token estimation may under/over-estimate.
  - Mitigation: keep the estimator pluggable; add provider-error-triggered backoff compaction during request execution.
- Moving prompt construction into a central API may require touching multiple call sites.
  - Mitigation: provide temporary compatibility helpers during the migration.

## Migration Plan
1. Introduce `Context` and refactor `Session` to use it.
2. Migrate Agent request-building to use `Context::render` while continuing to inject system messages at request time.
3. Migrate commands and any call sites that mutate `Session.history` directly.
4. Add unit tests around tool exchange atomicity, multi-step turns, and compaction behavior.
