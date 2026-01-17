# Design: Event envelopes for system events

## Context
The current `EventBus` broadcasts raw `SystemEvent` values and the server streams raw `SystemEvent` JSON over SSE. This provides typed payloads but does not provide stable, uniform metadata (monotonic ids and timestamps) at the system-event transport boundary.

## Goals / Non-Goals
- Goals:
  - Provide a stable monotonic identifier per published system event for ordering and gap detection.
  - Provide a timestamp for consistent logging and correlation with streamed events.
  - Preserve the existing `SystemEvent` JSON shape inside the envelope.
- Non-Goals:
  - Durable event storage or replay on reconnect (this change does not implement `Last-Event-ID` replay).
  - Global uniqueness across process restarts or across multiple servers.

## Envelope schema
`EventEnvelope<SystemEvent>` SHOULD be JSON-serializable and include:
- `id`: a process-local, monotonically increasing integer identifier (e.g., `u64`)
  - Counter is owned by `EventBus` as `AtomicU64` and increments on each `publish()` call
  - Counter resets to 0 when `EventBus` is created (cross-process restart uniqueness is out of scope)
- `timestamp_ms`: Unix epoch time in milliseconds (UTC)
  - Uses `chrono::Utc::now().timestamp_millis()` for explicit timezone handling
- `event`: the existing `SystemEvent` payload (internally tagged as `{ "type": ..., "payload": ... }`)

Example payload:
```json
{
  "id": 42,
  "timestamp_ms": 1768631000000,
  "event": {
    "type": "PermissionRequest",
    "payload": {
      "operation": "shell",
      "tool_name": "RunCommand",
      "call_id": "call_123"
    }
  }
}
```

The envelope MAY include additional optional fields in the future (e.g., correlation identifiers, source component) without changing existing event payload variants.

## Ordering and delivery
- The system SHOULD assign `id` and `timestamp_ms` at publish time.
- Subscribers SHOULD treat `id` as the primary ordering key.
- Consumers MUST tolerate lossy delivery when the underlying `tokio::sync::broadcast` channel lags.

## SSE mapping
The server SSE endpoint SHOULD stream the envelope JSON as the SSE data payload. The server SHOULD also set the SSE `id` field to the envelope `id` (string-encoded) so clients can reason about gaps and correlate logs.

## Backwards compatibility
This change is breaking for clients that expect SSE `data` to be a raw `SystemEvent`. The TUI backend event listener SHOULD be updated in the same change to parse envelopes and dispatch the inner `SystemEvent`.

### Internal consumer impact
The `EventBus` subscription callback signatures change from:
- `FnMut(SystemEvent)` → `FnMut(EventEnvelope<SystemEvent>)`

This affects ALL internal EventBus consumers including:
- Event logger (`crates/common/src/logging.rs`)
- Any other components using `subscribe_all`, `subscribe`, or `subscribe_raw`

All consumers must be audited and migrated to extract `SystemEvent` from envelopes where needed.
