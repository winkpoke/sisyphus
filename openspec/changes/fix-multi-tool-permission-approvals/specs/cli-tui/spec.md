# cli-tui Delta Specification

## MODIFIED Requirements

### Requirement: Permission Approval Overlay Actions
When the TUI receives a `PermissionRequest` event, it SHALL present a permission overlay with Approve and Deny actions.

#### Scenario: Overlay opens on PermissionRequest
- **GIVEN** the TUI is running
- **WHEN** the backend emits a `PermissionRequest` event
- **THEN** the TUI SHALL show an overlay that includes `operation`, `tool_name`, and `call_id`
- **AND** the overlay SHALL indicate that the agent is blocked pending a decision

#### Scenario: Multiple permission requests are queued and actionable
- **GIVEN** the TUI is running
- **WHEN** the backend emits PermissionRequest events for call ids `C1` and `C2` in that order
- **THEN** the TUI MUST enqueue both requests
- **AND** the overlay MUST present `C1` before `C2`
- **AND** after `C1` is approved or denied, the overlay MUST present `C2`

#### Scenario: Approve advances the pending approval queue
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **AND** there is at least one other pending permission request
- **WHEN** the user selects Approve
- **THEN** the TUI SHALL submit an approval decision to the server
- **AND** the overlay MUST remain available to present the next pending request

#### Scenario: Deny advances the pending approval queue
- **GIVEN** a permission overlay is open for `call_id` `C1`
- **AND** there is at least one other pending permission request
- **WHEN** the user selects Deny
- **THEN** the TUI SHALL submit a denial decision to the server
- **AND** the overlay MUST remain available to present the next pending request

