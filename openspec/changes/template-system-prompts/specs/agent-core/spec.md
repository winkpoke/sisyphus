## MODIFIED Requirements

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
