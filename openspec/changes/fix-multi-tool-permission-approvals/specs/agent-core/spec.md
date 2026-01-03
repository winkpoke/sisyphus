# agent-core Delta Specification

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

#### Scenario: Multiple tool calls respect order and block on permission
- **GIVEN** a single assistant message includes tool calls `C1`, `C2`, and `C3` in that order
- **AND** `C1` is Ask-gated
- **AND** `C2` is Allow-gated
- **AND** `C3` is Ask-gated
- **WHEN** the agent processes the assistant message tool calls
- **THEN** the system MUST emit a permission request for `C1`
- **AND** the system MUST NOT execute `C2` or `C3`
- **AND** the system MUST NOT emit a permission request for `C3`
- **AND** the system MUST stop the current assistant turn waiting for `C1`

#### Scenario: Resuming batch executes subsequent allowed tools
- **GIVEN** a blocked tool batch `[C1(Ask), C2(Allow), C3(Ask)]` stopped at `C1`
- **WHEN** the user approves `C1`
- **THEN** the system MUST execute `C1`
- **AND** the system MUST execute `C2` immediately (as it is Allow-gated)
- **AND** the system MUST emit a permission request for `C3`
- **AND** the system MUST stop the current assistant turn waiting for `C3`

#### Scenario: Deny appends deterministic tool-result and continues the same assistant turn
- **GIVEN** a pending permission request for tool call id `C1`
- **WHEN** the user denies execution for `C1`
- **THEN** the system MUST NOT execute the tool call
- **AND** the system MUST append a Tool message correlated to `C1` with exact content:
  - `Permission denied: user rejected tool execution.`
- **AND** the system MUST perform the next model call to continue the same assistant turn

