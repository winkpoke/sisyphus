## ADDED Requirements

### Requirement: Permission Approval Overlay Actions
When the TUI receives a `PermissionRequest` event, it SHALL present a permission overlay with Approve and Deny actions.

#### Scenario: Overlay opens on PermissionRequest
- **GIVEN** the TUI is running
- **WHEN** the backend emits a `PermissionRequest` event
- **THEN** the TUI SHALL show an overlay that includes `operation`, `tool_name`, and `call_id`
- **AND** the overlay SHALL indicate that the agent is blocked pending a decision

#### Scenario: Approve resumes the assistant turn
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **WHEN** the user selects Approve
- **THEN** the TUI SHALL submit an approval decision to the server
- **AND** the TUI SHALL append the resulting assistant response to the transcript
- **AND** the overlay SHALL close

#### Scenario: Deny resumes the assistant turn
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **WHEN** the user selects Deny
- **THEN** the TUI SHALL submit a denial decision to the server
- **AND** the TUI SHALL append the resulting assistant response to the transcript
- **AND** the overlay SHALL close

