# agent-core Delta Specification

## MODIFIED Requirements

### Requirement: Agent Permissions
The system SHALL enforce granular permissions for privileged operations, supporting Allow, Ask, and Deny levels.

#### Scenario: Deny blocks tool execution
- **GIVEN** tool execution is set to Deny
- **WHEN** the agent attempts to execute a tool call
- **THEN** the tool MUST NOT be executed
- **AND** the user receives a deterministic permission-denied response with exact content:
  - `Permission denied: tool execution is set to Deny.`

#### Scenario: Ask requires explicit user approval
- **GIVEN** tool execution is set to Ask
- **WHEN** the agent attempts to execute a tool call
- **THEN** the tool MUST NOT be executed automatically
- **AND** the system signals that user approval is required before continuing by emitting a permission-request signal with fields:
  - `operation`: `tool_execution`
  - `tool_name`: the requested tool name
  - `call_id`: the tool call id
- **AND** the user receives a deterministic message with exact content:
  - `Permission required: approve tool execution to continue.`

### Requirement: Dynamic System Prompt
The system SHALL generate system prompts dynamically at the start of a user chat turn, including current environment context and custom project rules.

#### Scenario: Environment injection uses an `<env>` block
- **WHEN** a system prompt is generated
- **THEN** it includes the current Working Directory, Platform (OS), and Today's Date in a formatted `<env>` block

The `<env>` block MUST contain these labels exactly:
- `Operating system:`
- `Working directory:`
- `Today's date:`

#### Scenario: Custom rules injection from `AGENTS.md`
- **GIVEN** `AGENTS.md` exists in the workspace
- **WHEN** a system prompt is generated
- **THEN** its content is appended to the system prompt

#### Scenario: Prompt is stable within a user turn
- **GIVEN** a single user input triggers multiple completion requests due to tool calls
- **WHEN** the agent rebuilds completion requests for that user input
- **THEN** the system prompt content remains unchanged across those requests

This stability requirement includes:
- The `<env>` values, which are snapshotted at the start of the user chat turn
- The `AGENTS.md` existence check and content, which are snapshotted at the start of the user chat turn
