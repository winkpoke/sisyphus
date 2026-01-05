## ADDED Requirements

### Requirement: UiCommand routing and precedence
The CLI client SHALL route slash-prefixed user input through a UiCommand router before sending input to the server.

If the command name matches a known UiCommand, the CLI MUST execute it locally.

#### Scenario: UiCommand takes precedence over SlashCommand name collisions
- **GIVEN** the CLI registers a UiCommand `/exit`
- **AND** the server reports a SlashCommand named `/exit`
- **WHEN** the user inputs `/exit`
- **THEN** the CLI MUST execute the UiCommand locally
- **AND** it MUST NOT send the input to the chat endpoint

### Requirement: Escaping leading slash for chat
The CLI client SHALL support escaping a leading slash so users can send literal slash text.

#### Scenario: Double slash sends literal slash
- **GIVEN** the CLI is connected to a server
- **WHEN** the user inputs `//help`
- **THEN** the CLI MUST send `/help` as normal chat text
- **AND** it MUST NOT treat it as a command

## MODIFIED Requirements

### Requirement: REPL Session Switching
The CLI REPL SHALL adopt new session IDs returned by the server after UiCommand-driven lifecycle operations.

#### Scenario: REPL Updates Session ID After /new
Given the CLI is connected to a server using session id `S1`
When the user runs the UiCommand `/new`
And the CLI creates a new session via `POST /api/v1/sessions`
Then the CLI SHALL store the new session id `S2` as the active session id
And subsequent chat requests SHALL use `S2`

