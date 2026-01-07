## MODIFIED Requirements

### Requirement: Dynamic System Prompt
The system SHALL generate system prompts dynamically at the start of a user chat turn using Jinja2 templating with XML-structured semantic sections, including current environment context and custom project rules.

#### Scenario: Environment injection uses XML tags
- **WHEN** a system prompt is generated
- **THEN** it includes current Working Directory, Platform (OS), and Today's Date in an `<environment>` XML tag

The `<environment>` tag MUST contain these labels exactly:
- `Operating system:`
- `Working directory:`
- `Today's date:`

#### Scenario: System prompt uses semantic XML structure
- **WHEN** a system prompt is generated
- **THEN** it MUST use the following XML tags for semantic chunking:
  - `<role>`: Agent description/persona
  - `<task>`: Task description derived from AgentConfig
  - `<instructions>`: Core instructions from AgentConfig
  - `<environment>`: Environment context (OS, CWD, Date)
  - `<project_rules>`: AGENTS.md content if present
  - `<output_format>`: Response format guidelines

#### Scenario: Custom rules injection from `AGENTS.md`
- **GIVEN** `AGENTS.md` exists in the workspace root
- **WHEN** a system prompt is generated
- **THEN** its content is wrapped in a `<project_rules>` XML tag and appended to the system prompt

#### Scenario: Prompt is stable within a user turn
- **GIVEN** a single user input triggers multiple completion requests due to tool calls
- **WHEN** the agent rebuilds completion requests for that user input
- **THEN** the system prompt content (including all XML tags) remains unchanged across those requests

This stability requirement includes:
- The `<environment>` tag values, which are snapshotted at the start of the user chat turn
- The `<project_rules>` tag content, which is snapshotted at the start of the user chat turn
- All other XML tag content, which must not change during the turn

#### Scenario: XML tag escaping in code blocks
- **GIVEN** system prompt content contains XML special characters (e.g., `<`, `>`, `&`)
- **WHEN** the system prompt is generated
- **THEN** XML special characters within code blocks or instruction sections MUST be escaped to prevent tag confusion
- **AND** the system prompt MUST render correctly when sent to LLM providers

### Requirement: System Prompt Construction
The system prompt builder MUST use Jinja2 templating for all variable interpolation and MUST NOT block the execution thread when reading environment context or external files.

#### Scenario: Template engine supports variable interpolation
- **GIVEN** a system prompt template containing variables (e.g., `{{os}}`, `{{cwd}}`, `{{date}}`)
- **WHEN** the system prompt is built
- **THEN** all template variables are replaced with their corresponding values
- **AND** undefined variables cause a clear error to be returned

#### Scenario: Template engine supports advanced features
- **GIVEN** a system prompt template using Jinja2 features:
  - Conditionals: `{% if has_project_rules %}...{% endif %}`
  - Loops: `{% for rule in custom_rules %}...{% endfor %}`
  - Filters: `{{date|date('YYYY-MM-DD')}}`
- **WHEN** the system prompt is built
- **THEN** all Jinja2 features are rendered correctly
- **AND** invalid Jinja2 syntax causes a clear error with line number and context

#### Scenario: Reading custom rules
- **GIVEN** a large `AGENTS.md` file
- **WHEN** the agent builds the system prompt for a new user turn
- **THEN** it reads the file asynchronously without blocking the agent runtime
- **AND** the file content is available as a template variable (e.g., `{{custom_rules}}`)

#### Scenario: Template compilation happens at load time
- **GIVEN** system prompt templates are loaded from configuration
- **WHEN** the agent starts
- **THEN** templates are compiled and validated immediately
- **AND** invalid template syntax causes startup failure with clear error message
- **AND** valid templates are cached for fast rendering

## ADDED Requirements

### Requirement: Jinja2 Template Engine
The system SHALL use the minijinja crate for Jinja2-compatible template processing in both system prompts and command templates.

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
- **GIVEN** a custom SlashCommand with template `Hello {{args}}, {{upper(args)}}`
- **AND** user executes `/command World`
- **WHEN** the template is rendered
- **THEN** both `{{args}}` and the Jinja2 filter work correctly
- **AND** the output is `Hello World, WORLD`
