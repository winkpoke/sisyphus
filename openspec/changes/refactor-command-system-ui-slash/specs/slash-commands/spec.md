## MODIFIED Requirements

### Requirement: Slash Command Support
The system SHALL support executing SlashCommands triggered by a forward slash `/` at the start of a message.

SlashCommands MUST be prompt templates expanded by the agent runtime before the model call.

SlashCommands MUST NOT directly cause session lifecycle changes (e.g. new session, clear history, exit) or UI-only side effects.

#### Scenario: SlashCommand expands to prompt input
- **GIVEN** a SlashCommand `/greet` is registered with template `Hello {{args}}`
- **WHEN** the user sends `/greet world`
- **THEN** the agent MUST expand the command into `Hello world` prior to the model call
- **AND** the expanded text MUST be treated as normal chat input for that turn

## ADDED Requirements

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

## REMOVED Requirements

### Requirement: Exit commands are session-scoped
**Reason**: `/exit` is a UiCommand handled by the client UI runtime.

**Migration**: Clients should execute `/exit` locally and stop the active UI session without calling the chat endpoint.

#### Scenario: /exit is not a SlashCommand
- **GIVEN** a client UI registers `/exit` as a UiCommand
- **WHEN** the user inputs `/exit`
- **THEN** the client MUST NOT route the input through SlashCommand expansion
- **AND** it MUST NOT send the input to the chat endpoint

### Requirement: Quit is an alias of exit
**Reason**: `/quit` is a UiCommand handled by the client UI runtime.

**Migration**: Clients should treat `/quit` as an alias of the `/exit` UiCommand.

#### Scenario: /quit is not a SlashCommand
- **GIVEN** a client UI registers `/quit` as a UiCommand
- **WHEN** the user inputs `/quit`
- **THEN** the client MUST NOT route the input through SlashCommand expansion
- **AND** it MUST NOT send the input to the chat endpoint
