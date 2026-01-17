# Change: Refactor system event bus to use event envelopes

## Why
The current system event stream provides typed `SystemEvent` payloads but lacks stable, uniform metadata (ordering and timestamps) across the EventBus, server SSE, and event logging. As a result, clients cannot reliably detect dropped events, correlate log lines with streamed events, or reason about event ordering. Introducing a standard event envelope improves traceability and enables extensible metadata without forcing per-event changes to each `SystemEvent` variant.

## What Changes
- Introduce a typed `EventEnvelope<SystemEvent>` that carries stable metadata alongside the existing `SystemEvent` JSON shape.
- Update the internal `EventBus` to publish envelopes instead of raw events, while preserving topic-based routing and global auditing semantics.
- Update the server SSE endpoint to stream envelope JSON and set the SSE `id` field to the envelope id.
- **BREAKING**: Change `/api/v1/events` SSE `data` payload from raw `SystemEvent` JSON to `EventEnvelope<SystemEvent>` JSON.
- Update the TUI backend event listener (currently decoding SSE `msg.data` as `SystemEvent`) to parse envelopes and dispatch the inner `SystemEvent` to existing UI logic.
- Update event logging to include envelope metadata consistently.

## Impact
- Affected specs: common-infra, server-core, cli-tui
- Affected code: `crates/common/src/bus.rs`, `crates/common/src/logging.rs`, `crates/server/src/lib.rs` (SSE `/api/v1/events`), `crates/tui/src/tui/mod.rs`
- **BREAKING**: Changes `EventBus` subscription callback signatures from `FnMut(SystemEvent)` to `FnMut(EventEnvelope<SystemEvent>)`, affecting all internal EventBus consumers (requires audit and migration)
