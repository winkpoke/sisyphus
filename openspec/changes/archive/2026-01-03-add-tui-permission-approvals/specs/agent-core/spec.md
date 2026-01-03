## MODIFIED Requirements

### Requirement: Agent Permissions
The system SHALL enforce granular permissions for privileged operations, supporting Allow, Ask, and Deny levels.

When a tool call requires approval (`Ask`), the agent MUST stop the current user turn after emitting the permission-request signal, returning a deterministic response without further model calls.

#### Scenario: Ask still emits a permission request
- **GIVEN** tool execution is set to Ask
- **WHEN** the agent attempts to execute a tool call
- **THEN** the tool MUST NOT be executed automatically
- **AND** the system SHALL emit a permission-request signal with fields:
  - `operation`: `tool_execution`
  - `tool_name`: the requested tool name
  - `call_id`: the tool call id
- **AND** the user receives a deterministic message with exact content:
  - `Permission required: approve tool execution to continue.`

#### Scenario: Ask hard-stops the current turn
- **GIVEN** tool execution is set to Ask
- **WHEN** the agent attempts to execute a tool call
- **THEN** the agent MUST NOT call the model again for that user turn
- **AND** it MUST return the deterministic permission-required message as the turn outcome
