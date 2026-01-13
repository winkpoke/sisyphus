# slash-commands Specification

## Purpose
Defines server-side slash command system that expands user commands via Jinja2 templates before LLM completion. Supports custom commands loaded from Markdown files, built-in commands, and reserved UiCommand name enforcement to prevent client-side command collisions.
## Requirements
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

### Requirement: Command Loading Separation
The system SHALL separate command loading logic from the runtime registry to ensure clean architecture and testability.

#### Scenario: Loading Commands
Given a directory of Markdown command files
When the application starts
Then `CommandLoader` should scan and parse the files
And return a collection of command configurations
And `CommandRegistry` should be populated from this collection
But `CommandRegistry` should not contain any file I/O logic

### Requirement: Command Registry Responsibility
The system SHALL ensure the CommandRegistry is responsible only for storage and retrieval.

#### Scenario: Command Registry Responsibility
Given a `CommandRegistry`
When `register_builtin` or `register_custom` is called
Then it should store the command in memory
And it should allow retrieval by name

### Requirement: Command Parsing
The command parsing logic MUST be encapsulated in a dedicated component, separating syntax analysis from command execution, and it MUST expose enough information for reliable custom command expansion.

#### Scenario: Parsing quoted arguments
- **GIVEN** a command string `/cmd "arg 1" arg2`
- **WHEN** parsed by the command parser
- **THEN** it returns command `/cmd` and arguments `["arg 1", "arg2"]`

#### Scenario: Preserving raw args for expansion
- **GIVEN** a command string `/cmd "arg 1" arg2`
- **WHEN** parsed by the command parser
- **THEN** it also returns `raw_args` equal to `"arg 1" arg2`

### Requirement: Help output reflects the command registry
The system SHALL generate `/help` output from the runtime command registry, including both built-in and loaded custom commands.

#### Scenario: Help lists all registered commands
- **GIVEN** the registry contains built-in commands and custom commands
- **WHEN** the user executes `/help`
- **THEN** the output includes every registered command name

The output MUST be deterministic:
- The output MUST start with the exact header line: `Available commands:`
- Each subsequent line MUST be of the form: `- /<name>`
- Command lines MUST be sorted in ascending lexicographic order by `<name>`

### Requirement: Slash command arguments support quoted strings
The system SHALL parse slash command arguments with support for quoted multi-word values.

#### Scenario: Quoted argument preserves whitespace
- **WHEN** the user executes a slash command with a quoted argument (e.g., `/cmd "two words"`)
- **THEN** the parsed argument list contains a single argument with the full value `two words`

The parser MUST additionally support these behaviors:
- `\"` inside a quoted argument represents a literal `"`
- `\\` inside a quoted argument represents a literal `\`

#### Scenario: Unterminated quote is a parse error
- **WHEN** the user executes a slash command with an unterminated quote (e.g., `/cmd "two words`)
- **THEN** the command MUST NOT be executed
- **AND** the user receives a deterministic error message with exact content:
  - `Invalid command syntax: unterminated quote.`

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

### Requirement: SlashCommand discovery is exposed to clients
The server SHALL provide a deterministic SlashCommand discovery mechanism so clients can render palettes and completion.

#### Scenario: Client receives SlashCommand metadata for discovery
- **GIVEN** a running server
- **WHEN** a client requests `GET /api/v1/slash-commands`
- **THEN** the server MUST return a list of SlashCommand metadata including name and description
- **AND** the list MUST include both built-in and custom SlashCommands

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

