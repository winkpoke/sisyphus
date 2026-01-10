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
The system SHALL generate system prompts dynamically at the start of a user chat turn using Jinja2 templates with semantic XML-style tags, including current environment context and custom project rules.

#### Scenario: Environment injection uses XML tags
- **GIVEN** a system prompt template with environment variables
- **WHEN** a system prompt is generated
- **THEN** it includes current Working Directory, Platform (OS), and Today's Date in an `<environment>` XML tag

The `<environment>` tag MUST contain these labels exactly:
- `OS:`
- `CWD:`
- `Date:`

#### Scenario: System prompt uses semantic XML structure
- **WHEN** a system prompt is generated
- **THEN** it MUST use following XML tags for semantic chunking:
  - `<instructions>`: Core instructions from AgentConfig
  - `<environment>`: Environment context (OS, CWD, Date, workspace_root)
  - `<project_rules>`: AGENTS.md content if present

#### Scenario: Custom rules injection from AGENTS.md
- **GIVEN** `AGENTS.md` exists in workspace root
- **WHEN** a system prompt is generated
- **THEN** its content is wrapped in a `<project_rules>` XML tag and appended to system prompt

#### Scenario: Project rules content is escaped
- **GIVEN** `AGENTS.md` contains XML-special characters (`<`, `>`, `&`)
- **WHEN** its content is embedded into `<project_rules>`
- **THEN** XML-special characters are escaped so they cannot create new tags

#### Scenario: Prompt is stable within a user turn
- **GIVEN** a single user input triggers multiple completion requests due to tool calls
- **WHEN** agent rebuilds completion requests for that user input
- **THEN** system prompt content (including all XML tags) remains unchanged across those requests

This stability requirement includes:
- The `<environment>` tag values, which are snapshotted at the start of user chat turn
- The `<project_rules>` tag content, which is snapshotted at the start of user chat turn
- All other XML tag content, which must not change during turn

#### Scenario: Template variable interpolation works
- **GIVEN** a system prompt template `You are {{instructions}}. OS: {{os}}`
- **AND** context variables `instructions = "a helpful assistant"`, `os = "linux"`
- **WHEN** the system prompt template is rendered
- **THEN** output is `You are a helpful assistant. OS: linux`

#### Scenario: Template conditionals work
- **GIVEN** a system prompt template with conditional section
- **WHEN** the system prompt template is rendered with appropriate context
- **THEN** conditional sections are included or excluded based on Jinja2 `{% if %}` syntax

#### Scenario: Template loops work
- **GIVEN** a system prompt template with a loop over a list variable
- **WHEN** the system prompt template is rendered
- **THEN** the loop iterates correctly and outputs multiple items

#### Scenario: Template error includes context
- **GIVEN** a system prompt template with undefined variable or syntax error
- **WHEN** the system prompt template is compiled or rendered
- **THEN** a clear error message is returned indicating:
  - The undefined variable name or syntax error description
  - The line number where the error occurred (if available)
  - A suggested fix or reference to documentation

#### Scenario: Template validation at load time
- **GIVEN** a custom system prompt template file is provided
- **WHEN** the agent starts
- **THEN** the template is validated for Jinja2 syntax before first use
- **AND** if validation fails, a clear error message is logged indicating the template source and error details

#### Scenario: Backward compatibility (no template specified)
- **GIVEN** an agent configuration with no custom template specified
- **WHEN** the system prompt is generated
- **THEN** the system prompt builder uses a default embedded template
- **AND** the default template renders with the same behavior as the current format! macro approach

#### Scenario: Migration path is seamless
- **GIVEN** existing agent configuration with only `instructions` field (no template)
- **WHEN** system prompt is generated without template specified
- **THEN** output matches current behavior exactly (same format, same content)
- **AND** no configuration changes are required
- **AND** existing agent behavior is preserved

#### Scenario: Template discovery order respects configuration
- **GIVEN** an agent configuration with `system_prompt_template` field set
- **AND** a custom template file exists at `.sisyphus/templates/system_prompt.jinja`
- **WHEN** the system prompt is generated
- **THEN** the inline `system_prompt_template` field is used (highest priority)
- **AND** the file-based template is ignored

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

### Requirement: Expose Agent Metadata via Server API
The system SHALL expose Agent configuration metadata, including model settings, via the server Agent discovery API.

#### Scenario: Agent Metadata Reflects Config
- **GIVEN** an Agent configured with a specific name, description, mode, permissions, and model settings
- **WHEN** the server returns that Agent via `GET /api/v1/agents` or `GET /api/v1/agents/:id`
- **THEN** the returned metadata MUST include the configured name, description, and model
- **AND** it MUST NOT expose internal-only fields that are not part of the Agent configuration contract.

### Requirement: Jinja2 Template Engine
The system SHALL use the minijinja crate for Jinja2-compatible template processing in SlashCommand templates.

#### Scenario: Jinja2 variable interpolation works
- **GIVEN** a template string `Hello {{name}}!`
- **AND** a context variable `name = "World"`
- **WHEN** the template is rendered
- **THEN** the output is `Hello World!`

#### Scenario: Jinja2 conditionals work
- **GIVEN** a template string `{% if show_message %}Message shown{% endif %}`
- **AND** a context variable `show_message = true`
- **WHEN** the template is rendered
- **THEN** the output includes `Message shown`
- **AND** when `show_message = false`, the output is empty

#### Scenario: Jinja2 loops work
- **GIVEN** a template string `{% for item in items %}{{item}} {% endfor %}`
- **AND** a context variable `items = ["a", "b", "c"]`
- **WHEN** the template is rendered
- **THEN** the output is `a b c `

#### Scenario: Jinja2 filters work
- **GIVEN** a template string `{{text|upper}}`
- **AND** a context variable `text = "hello"`
- **WHEN** the template is rendered
- **THEN** the output is `HELLO`

#### Scenario: Template error includes context
- **GIVEN** a template string with undefined variable `{{undefined_var}}`
- **WHEN** the template is rendered
- **THEN** a clear error is returned indicating:
  - The undefined variable name
  - The line number where the error occurred
  - The template source (if available)

### Requirement: Backward Compatibility for Template Syntax
The system MUST maintain backward compatibility with existing `{{args}}` syntax in custom SlashCommand templates.

#### Scenario: Existing `{{args}}` templates continue to work
- **GIVEN** a custom SlashCommand with template `Hello {{args}}`
- **AND** user executes `/command World`
- **WHEN** the template is rendered
- **THEN** the output is `Hello World`
- **AND** no migration or modification is required

#### Scenario: Mix of old and new syntax works
- **GIVEN** a custom SlashCommand with template `Hello {{args}}, {{args|upper}}`
- **AND** user executes `/command World`
- **WHEN** the template is rendered
- **THEN** both `{{args}}` and the Jinja2 filter work correctly
- **AND** the output is `Hello World, WORLD`

