## ADDED Requirements

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
