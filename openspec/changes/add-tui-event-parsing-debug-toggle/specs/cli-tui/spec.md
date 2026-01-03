# cli-tui Delta Specification

## ADDED Requirements

### Requirement: Human-readable backend event rendering
The TUI SHALL render backend SSE system events as concise, end-user-readable transcript entries by default.

#### Scenario: Tool execution event is summarized
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** a `ToolExecuted` event is received
- **THEN** the TUI SHALL append a short system message summarizing the tool completion

#### Scenario: Backend error event is surfaced
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** an `Error` event is received
- **THEN** the TUI SHALL append an error message suitable for end users

#### Scenario: Unparseable event payload does not crash the UI
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** an SSE message is received that cannot be parsed as a `SystemEvent`
- **THEN** the TUI MUST remain responsive
- **AND** it SHALL append a minimal system message indicating an unparsed event was received

### Requirement: Debug toggle for raw backend event payloads
The TUI SHALL provide a local `/debug` command to toggle visibility of raw backend event payloads.

#### Scenario: Debug is disabled by default
- **GIVEN** the TUI starts in a new session
- **THEN** the TUI SHALL NOT display raw backend event payloads in the transcript

#### Scenario: /debug toggles raw payload visibility
- **GIVEN** the TUI is running and debug mode is disabled
- **WHEN** the user runs `/debug`
- **THEN** the TUI SHALL enable debug mode
- **AND** subsequent backend events SHALL include raw payload output

- **GIVEN** the TUI is running and debug mode is enabled
- **WHEN** the user runs `/debug`
- **THEN** the TUI SHALL disable debug mode
- **AND** subsequent backend events SHALL NOT include raw payload output

#### Scenario: /debug is discoverable in the command palette
- **GIVEN** the user opens the command palette
- **THEN** `/debug` SHALL appear in the available commands list

### Requirement: Raw payload redaction and truncation
When debug mode is enabled, raw backend event payloads MUST be displayed in a redacted and truncated form.

#### Scenario: Raw payload redacts sensitive fields
- **GIVEN** debug mode is enabled
- **AND** an SSE event payload contains sensitive values
- **WHEN** the TUI displays the raw payload
- **THEN** sensitive values MUST be replaced with a redaction marker

#### Scenario: Raw payload is truncated to a fixed maximum size
- **GIVEN** debug mode is enabled
- **AND** an SSE event payload exceeds the maximum raw display size
- **WHEN** the TUI displays the raw payload
- **THEN** the displayed payload MUST be truncated

## MODIFIED Requirements

### Requirement: Permission Request Prompt
The TUI SHALL render `PermissionRequest` events as a first-class permission prompt instead of raw event JSON.

#### Scenario: Permission request does not spam raw JSON by default
- **GIVEN** the TUI is connected to the server event stream
- **AND** debug mode is disabled
- **WHEN** a `PermissionRequest` event is received
- **THEN** the TUI SHALL display the permission overlay
- **AND** the transcript SHALL NOT include the raw JSON payload for that event
