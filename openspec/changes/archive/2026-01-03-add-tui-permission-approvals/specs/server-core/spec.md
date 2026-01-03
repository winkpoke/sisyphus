## ADDED Requirements

### Requirement: Permission requests are streamed to clients
The server SHALL stream `PermissionRequest` system events to connected clients via the SSE endpoint.

#### Scenario: Client receives permission request event
- **GIVEN** a running server
- **AND** a client is connected to `/api/v1/events`
- **WHEN** the agent emits a `PermissionRequest` system event
- **THEN** the client SHALL receive the event data as JSON via the stream
