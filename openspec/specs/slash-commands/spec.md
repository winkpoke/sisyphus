# slash-commands Specification

## Purpose
TBD - created by archiving change implement-slash-commands. Update Purpose after archive.
## Requirements
### Requirement: Slash Command Support
The system SHALL support executing SlashCommands triggered by a forward slash `/` at the start of a message.

SlashCommands MUST be prompt templates expanded by the agent runtime before the model call.

SlashCommands MUST NOT directly cause session lifecycle changes (e.g. new session, clear history, exit) or UI-only side effects.

#### Scenario: SlashCommand expands to prompt input
- **GIVEN** a SlashCommand `/greet` is registered with template `Hello {{args}}`
- **WHEN** the user sends `/greet world`
- **THEN** the agent MUST expand the command into `Hello world` prior to the model call
- **AND** the expanded text MUST be treated as normal chat input for that turn

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

Loaded custom SlashCommands MUST be treated as prompt templates and MUST support substituting `{{args}}` with the raw argument string.

If a custom SlashCommand name collides with a reserved UiCommand name, the system MUST reject the custom SlashCommand.

#### Scenario: Custom SlashCommand loads from Markdown frontmatter
- **GIVEN** a command directory contains `summarize.md` with frontmatter `description`
- **WHEN** the agent starts
- **THEN** the system SHALL register a SlashCommand named `/summarize` with the provided description
- **AND** the template content SHALL be used for expansion

#### Scenario: Custom SlashCommand colliding with reserved UiCommand is rejected
- **GIVEN** `/exit` is a reserved UiCommand name
- **AND** a command directory contains `exit.md`
- **WHEN** the agent starts
- **THEN** the system MUST NOT register `/exit` as a SlashCommand

### Requirement: SlashCommand discovery is exposed to clients
The server SHALL provide a deterministic SlashCommand discovery mechanism so clients can render palettes and completion.

#### Scenario: Client receives SlashCommand metadata for discovery
- **GIVEN** a running server
- **WHEN** a client requests `GET /api/v1/slash-commands`
- **THEN** the server MUST return a list of SlashCommand metadata including name and description
- **AND** the list MUST include both built-in and custom SlashCommands

