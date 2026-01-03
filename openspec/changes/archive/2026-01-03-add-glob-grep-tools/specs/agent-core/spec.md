## ADDED Requirements

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
