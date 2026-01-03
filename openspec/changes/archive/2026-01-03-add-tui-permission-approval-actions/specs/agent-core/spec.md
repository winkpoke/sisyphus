## MODIFIED Requirements

### Requirement: Agent Permissions
The system SHALL enforce granular permissions for privileged operations, supporting Allow, Ask, and Deny levels.

#### Scenario: Ask requires explicit user approval and is resumable
- **GIVEN** tool execution is set to Ask
- **WHEN** the agent attempts to execute a tool call with id `C1`
- **THEN** the tool MUST NOT be executed automatically
- **AND** the system MUST emit a permission-request signal with fields:
  - `operation`: `tool_execution`
  - `tool_name`: the requested tool name
  - `call_id`: `C1`
- **AND** the system MUST stop the current assistant turn until an explicit approval decision is received

#### Scenario: Approve executes tool and continues the same assistant turn
- **GIVEN** a pending permission request for tool call id `C1`
- **WHEN** the user approves execution for `C1`
- **THEN** the system MUST execute the tool call
- **AND** the system MUST append a Tool message correlated to `C1` containing the tool output
- **AND** the system MUST perform the next model call to continue the same assistant turn

#### Scenario: Deny appends deterministic tool-result and continues the same assistant turn
- **GIVEN** a pending permission request for tool call id `C1`
- **WHEN** the user denies execution for `C1`
- **THEN** the system MUST NOT execute the tool call
- **AND** the system MUST append a Tool message correlated to `C1` with exact content:
  - `Permission denied: user rejected tool execution.`
- **AND** the system MUST perform the next model call to continue the same assistant turn

