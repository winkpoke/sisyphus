## ADDED Requirements

### Requirement: Permission Approval API
The server SHALL provide an endpoint to submit approval decisions for Ask-gated tool execution.

#### Scenario: Approve a pending tool call
- **GIVEN** a running server
- **AND** a session `S1` has a pending permission request for tool call id `C1`
- **WHEN** the client sends `POST /api/v1/sessions/S1/approvals/C1` with decision `approve`
- **THEN** the server MUST execute the pending tool call
- **AND** the server MUST continue the assistant turn and return the assistant response

#### Scenario: Deny a pending tool call
- **GIVEN** a running server
- **AND** a session `S1` has a pending permission request for tool call id `C1`
- **WHEN** the client sends `POST /api/v1/sessions/S1/approvals/C1` with decision `deny`
- **THEN** the server MUST NOT execute the pending tool call
- **AND** the server MUST append a deterministic denial tool-result correlated to `C1`
- **AND** the server MUST continue the assistant turn and return the assistant response

#### Scenario: Reject unknown approvals
- **GIVEN** a running server
- **WHEN** the client submits an approval decision for a `(session_id, call_id)` pair with no pending request
- **THEN** the server MUST return a deterministic client error response

