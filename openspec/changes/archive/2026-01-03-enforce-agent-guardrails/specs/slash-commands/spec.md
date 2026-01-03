# slash-commands Delta Specification

## ADDED Requirements

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
