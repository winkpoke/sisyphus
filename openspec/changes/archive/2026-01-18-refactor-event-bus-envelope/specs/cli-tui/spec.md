# cli-tui Delta Specification

## ADDED Requirements

### Requirement: Backend SSE envelope parsing
The TUI SHALL parse backend SSE event `data` as `EventEnvelope<SystemEvent>` and dispatch the inner `SystemEvent` into the existing UI update flow.

#### Scenario: Envelope is parsed and dispatched
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** an SSE message is received with JSON-encoded `EventEnvelope<SystemEvent>` in `data`
- **THEN** the TUI SHALL parse the envelope
- **AND** it SHALL dispatch the inner `SystemEvent` for handling

## MODIFIED Requirements

### Requirement: Permission Request Prompt
The TUI SHALL render `PermissionRequest` events as a first-class permission prompt instead of raw event JSON.

#### Scenario: Permission request does not spam raw JSON by default
- **GIVEN** the TUI is connected to the server event stream
- **AND** debug mode is disabled
- **WHEN** a `PermissionRequest` event is received via the SSE stream as an `EventEnvelope<SystemEvent>`
- **THEN** the TUI SHALL display the permission overlay
- **AND** the transcript SHALL NOT include the raw JSON payload for that event

### Requirement: Human-readable backend event rendering
The TUI SHALL render backend SSE system events as concise, end-user-readable transcript entries by default.

#### Scenario: Unparseable event payload does not crash the UI
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** an SSE message is received that cannot be parsed as an `EventEnvelope<SystemEvent>`
- **THEN** the TUI MUST remain responsive
- **AND** it SHALL append a minimal system message indicating an unparsed event was received
