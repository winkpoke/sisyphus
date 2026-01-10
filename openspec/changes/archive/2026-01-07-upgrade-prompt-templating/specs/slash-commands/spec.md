## MODIFIED Requirements

### Requirement: Slash Command Support
The system SHALL support executing SlashCommands triggered by a forward slash `/` at the start of a message.

SlashCommands MUST be Jinja2 templates expanded by the agent runtime before the model call.

SlashCommands MUST NOT directly cause session lifecycle changes (e.g. new session, clear history, exit) or UI-only side effects.

#### Scenario: SlashCommand expands to prompt input
- **GIVEN** a SlashCommand `/greet` is registered with template `Hello {{args}}`
- **WHEN** the user sends `/greet World`
- **THEN** the agent MUST expand the command into `Hello World` using Jinja2 templating
- **AND** the expanded text MUST be treated as normal chat input for that turn

#### Scenario: Jinja2 features work in templates
- **GIVEN** a SlashCommand `/format` is registered with template `{{args|upper}}`
- **WHEN** the user sends `/format hello`
- **THEN** the agent MUST expand the command into `HELLO` using a Jinja2 filter

#### Scenario: Template loops iterate over argv
- **GIVEN** a SlashCommand `/list` is registered with template `{% for item in argv %}- {{item}}\n{% endfor %}`
- **WHEN** the user sends `/list a b c`
- **THEN** the agent MUST expand the command into:
  ```
  - a
  - b
  - c
  ```

### Requirement: Custom SlashCommands are loadable from a command directory
The system SHALL support loading custom SlashCommands from a configured directory of Markdown files.

Loaded custom SlashCommands MUST be treated as Jinja2 templates and MUST support substituting variables including `{{args}}` and any context-provided variables.

If a custom SlashCommand name collides with a reserved UiCommand name, system MUST reject the custom SlashCommand.

#### Scenario: Custom SlashCommand loads from Markdown frontmatter
- **GIVEN** a command directory contains `summarize.md` with frontmatter `description`
- **WHEN** the agent starts
- **THEN** the system SHALL register a SlashCommand named `/summarize` with the provided description
- **AND** the template content SHALL be compiled and validated as a Jinja2 template
- **AND** invalid template syntax causes the command to be skipped with a clear warning

#### Scenario: Custom SlashCommand colliding with reserved UiCommand is rejected
- **GIVEN** `/exit` is a reserved UiCommand name
- **AND** a command directory contains `exit.md`
- **WHEN** the agent starts
- **THEN** the system MUST NOT register `/exit` as a SlashCommand
- **AND** a warning or error message MUST indicate the name collision

#### Scenario: Template compilation validates syntax at load time
- **GIVEN** a command directory contains `bad-syntax.md` with template `{% if condition %}...` (missing endif)
- **WHEN** the agent starts
- **THEN** the system MUST skip registering the command
- **AND** a clear error message MUST be displayed indicating:
  - The file name with syntax error
  - The expected syntax correction

## ADDED Requirements

### Requirement: Template Context Variables
The system SHALL provide context variables to SlashCommand templates at render time, including but not limited to:

- `{{args}}`: Raw argument string from command invocation
- `{{argv}}`: Parsed argument list (quoted arguments preserved)
- `{{command}}`: Command name (without the `/` prefix)
- `{{cwd}}`: Current working directory
- `{{workspace_root}}`: Workspace root path

#### Scenario: Template uses cwd variable
- **GIVEN** a SlashCommand with template `Current directory: {{cwd}}`
- **WHEN** the user executes the command
- **THEN** the template is rendered with the current working directory path

#### Scenario: Template uses workspace_root variable
- **GIVEN** a SlashCommand with template `Workspace: {{workspace_root}}`
- **WHEN** the user executes the command
- **THEN** the template is rendered with the workspace root path

#### Scenario: Multiple variables work together
- **GIVEN** a SlashCommand with template `Command: {{command}} in {{cwd}} with args: {{args}}`
- **WHEN** the user executes `/test arg1 arg2`
- **THEN** the template is rendered with all variables replaced:
  - `{{command}}` → `test`
  - `{{cwd}}` → actual directory path
  - `{{args}}` → `arg1 arg2`

### Requirement: Template Error Handling
The system SHALL provide clear, actionable error messages when template compilation or rendering fails.

#### Scenario: Undefined variable error is clear
- **GIVEN** a SlashCommand template with `{{undefined_var}}`
- **WHEN** the user executes the command
- **THEN** a clear error message MUST indicate:
  - The undefined variable name
  - The command name

#### Scenario: Syntax error includes context
- **GIVEN** a SlashCommand template with invalid Jinja2 syntax
- **WHEN** the agent starts (load time) or user executes the command (render time)
- **THEN** a clear error message MUST include:
  - The syntax error description
  - The file name (when loading from disk)
  - A suggested fix or reference to documentation

#### Scenario: Runtime rendering failure is logged
- **GIVEN** a SlashCommand template that fails at render time (e.g., division by zero in expression)
- **WHEN** the user executes the command
- **THEN** the error MUST be logged with full context
- **AND** the user MUST receive a user-friendly error message
- **AND** the agent MUST continue operating (not crash)
