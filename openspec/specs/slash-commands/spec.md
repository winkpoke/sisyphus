# slash-commands Specification

## Purpose
TBD - created by archiving change implement-slash-commands. Update Purpose after archive.
## Requirements
### Requirement: Slash Command Support
The system SHALL support executing commands triggered by a forward slash `/` at the start of a message.

#### Scenario: Discoverable command listing
- **Given** the user is using the interactive TUI
- **When** the user requests command discovery (e.g., typing `/`)
- **Then** the UI SHALL present available slash commands without requiring the user to memorize names

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

