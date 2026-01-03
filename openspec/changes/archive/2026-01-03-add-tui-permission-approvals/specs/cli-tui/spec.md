## ADDED Requirements

### Requirement: Permission Request Prompt
The TUI SHALL render `PermissionRequest` events as a first-class permission prompt instead of raw event JSON.

#### Scenario: Permission request opens an overlay
- **GIVEN** the TUI is connected to the server event stream
- **WHEN** a `PermissionRequest` event is received
- **THEN** the TUI SHALL display an overlay describing the request
- **AND** it SHALL include `operation`, `tool_name`, and `call_id`

#### Scenario: Permission request indicates the agent is blocked
- **GIVEN** a permission request overlay is visible
- **WHEN** the user reads the overlay
- **THEN** the UI SHALL indicate that tool execution is blocked pending user action
