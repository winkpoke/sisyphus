# Design: Event envelopes for system events

## Overview
Wrap every `SystemEvent` in an `EventEnvelope` that provides stable metadata for ordering, correlation, and logging.

## Envelope schema
`EventEnvelope<SystemEvent>` SHOULD be JSON-serializable and include:
- `id`: a process-local, monotonically increasing integer identifier
- `timestamp_ms`: Unix epoch time in milliseconds
- `event`: the existing `SystemEvent` payload (internally tagged)

The envelope MAY include additional optional fields in the future (e.g., correlation identifiers, source component) without changing existing event payload variants.

## Ordering and delivery
- The system SHOULD assign `id` at publish time.
- Subscribers SHOULD treat `id` as the primary ordering key.
- Consumers MUST tolerate lossy delivery when the underlying broadcast channel lags.

## SSE mapping
The server SSE endpoint SHOULD stream the envelope JSON as the SSE data payload. The server SHOULD also set the SSE `id` field to the envelope `id` so clients can reason about gaps and correlate logs.

## Backwards compatibility
This change is breaking for clients that expect SSE `data` to be a raw `SystemEvent`. The CLI/TUI SHOULD be updated in the same change to parse envelopes.

