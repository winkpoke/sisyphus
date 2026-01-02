## Context
Codex’s UI maintains a transcript composed of “cells” that can be appended and updated as the agent streams responses. It renders on `Draw` ticks and keeps input editing separate from transcript updates.

## Goals / Non-Goals
- Goals:
  - Progressive assistant output without flicker.
  - Predictable wrapping/reflow on terminal resize.
  - Correct scrolling semantics for long conversations.
- Non-Goals:
  - Full Codex parity (selection/copy, overlays, rich widgets) in this change.

## Decisions
- Decision: Model the transcript as a sequence of typed cells (user message, assistant message, status/event).
- Decision: Allow in-place updates for the active assistant cell while streaming.
- Decision: Use a “stick to bottom unless user scrolls” policy.

## Risks / Trade-offs
- Reflowing wrapped content on resize can be expensive.
  - Mitigation: cache layout metadata and only recompute on width change.

## Migration Plan
1. Add transcript model and rendering without changing backend protocol.
2. Map current SSE events into streamed deltas vs discrete events.
3. Expand supported event types as the server emits richer activity.

