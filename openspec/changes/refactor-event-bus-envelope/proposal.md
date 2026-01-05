# Change: Refactor system event bus to use event envelopes

## Why
The current system event stream lacks stable metadata (ordering, timestamps, correlation) and relies on stringly-typed payload fields in multiple places. Introducing a standard event envelope improves traceability and makes the event stream extensible without forcing ad-hoc per-event changes.

## What Changes
- Introduce a typed `EventEnvelope<SystemEvent>` that carries stable metadata alongside the event payload.
- Update the internal `EventBus` to publish envelopes instead of raw events.
- Update the server SSE endpoint to stream envelope JSON.
- Update CLI/TUI event consumers to parse envelopes.
- Update event logging to include envelope metadata.

## Impact
- Affected specs: common-infra, server-core, cli-tui
- Affected code: crates/common (bus + logging), crates/server SSE stream, CLI/TUI SSE parsing

