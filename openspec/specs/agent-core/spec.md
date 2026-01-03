# agent-core Specification

## Purpose
Defines the core architecture for Sisyphus Agents, including configuration, permissions, and dynamic prompt generation. This specification ensures agents are secure, configurable, and context-aware, matching OpenCode's capabilities.
## Requirements
### Requirement: Agent Configuration
The system SHALL support configuring Agents with strictly typed metadata including name, mode, permissions, and model settings.

#### Scenario: Load Valid Config
- **WHEN** a valid agent configuration JSON/TOML is loaded
- **THEN** the system correctly parses the name, mode, and permission rules

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

### Requirement: Built-in Repository Search Tools
The system SHALL expose `glob` and `grep` as callable tools in the agent tool definitions.

#### Scenario: Tools are available to the model
- **GIVEN** an agent session is started
- **WHEN** the system creates the completion request tool definitions
- **THEN** the tool list includes `glob`
- **AND** the tool list includes `grep`

### Requirement: Repository Search Tool Schemas
The system SHALL expose stable tool schemas for `glob` and `grep` so models can call them reliably.

#### Scenario: Tool schemas include required arguments
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** `glob` schema requires `pattern`
- **AND** `grep` schema requires `pattern`

#### Scenario: Tool schemas support selective ignore overrides
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** both schemas include an `include_ignored` argument

#### Scenario: Tool schemas include output cap arguments
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** both schemas include a `max_results` argument

#### Scenario: Tool schemas include optional base path
- **GIVEN** `glob` and `grep` are registered
- **WHEN** the system exposes their tool schemas
- **THEN** both schemas include an optional `path` argument

#### Scenario: Grep schema supports output modes
- **GIVEN** `grep` is registered
- **WHEN** the system exposes its tool schema
- **THEN** the schema includes an `output_mode` argument
- **AND** `output_mode` supports `files_with_matches`, `content`, and `count`

### Requirement: System Prompt Construction
The system prompt builder MUST NOT block the execution thread when reading environment context or external files.

#### Scenario: Reading custom rules
- **GIVEN** a large `AGENTS.md` file
- **WHEN** the agent builds the system prompt for a new user turn
- **THEN** it reads the file asynchronously without blocking the agent runtime

### Requirement: Environment Snapshotting
The agent MUST capture a snapshot of the environment (OS, workspace root, date, custom rules) asynchronously before generating the system prompt for each user turn.

#### Scenario: Capturing snapshot
- **GIVEN** a running agent
- **WHEN** a new user turn begins
- **THEN** it captures the environment state asynchronously before the first model call

#### Scenario: Snapshot reuse within a turn
- **GIVEN** a single user turn that triggers tool calls
- **WHEN** the agent makes multiple model calls to complete the turn
- **THEN** it reuses the same environment snapshot for that turn

### Requirement: Workspace Rule Resolution
The agent MUST resolve `AGENTS.md` from the workspace root used for tool sandboxing, not from an implicit process working directory.

#### Scenario: Deterministic rule discovery
- **GIVEN** a workspace root containing `AGENTS.md`
- **WHEN** the agent builds the system prompt
- **THEN** it includes the contents of that workspace-root `AGENTS.md`

