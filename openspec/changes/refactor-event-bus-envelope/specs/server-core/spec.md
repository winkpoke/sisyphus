# server-core Delta Specification

## MODIFIED Requirements

### Requirement: Real-time Event Streaming
The server SHALL provide an SSE endpoint to stream system events as JSON-encoded `EventEnvelope<SystemEvent>` values.

#### Scenario: Subscribe to envelope events
- **GIVEN** a running server
- **WHEN** a client connects to `/api/v1/events`
- **AND** a `MessageReceived` system event is published on the internal bus
- **THEN** the client should receive a JSON envelope via the stream

#### Scenario: SSE event id matches envelope id
- **GIVEN** a running server
- **WHEN** the server streams a system event via SSE
- **THEN** the SSE `id` field SHOULD equal the envelope `id`

### Requirement: Permission requests are streamed to clients
The server SHALL stream `PermissionRequest` system events to connected clients via the SSE endpoint as `EventEnvelope<SystemEvent>`.

#### Scenario: Client receives permission request envelope
- **GIVEN** a running server
- **AND** a client is connected to `/api/v1/events`
- **WHEN** the agent emits a `PermissionRequest` system event
- **THEN** the client SHALL receive the event data as JSON-encoded `EventEnvelope<SystemEvent>` via the stream

