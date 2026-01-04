## ADDED Requirements

### Requirement: Exit commands are session-scoped
The system SHALL treat `/exit` as a session/client-scoped command.

Executing `/exit` MUST NOT shut down the server process.

#### Scenario: Exit returns an exit effect without server shutdown
- **GIVEN** a running server
- **AND** a client is connected and has an active session `S1`
- **WHEN** the client sends `/exit` to `/api/v1/sessions/S1/chat`
- **THEN** the command MUST be executed by the core command system
- **AND** the chat response MUST include a command effect indicating exit
- **AND** the server MUST continue to accept subsequent requests for other sessions

### Requirement: Quit is an alias of exit
The system SHALL treat `/quit` as an alias of `/exit`.

Executing `/quit` MUST NOT shut down the server process.

#### Scenario: Quit resolves to exit
- **GIVEN** a running server
- **AND** a client is connected and has an active session `S1`
- **WHEN** the client sends `/quit` to `/api/v1/sessions/S1/chat`
- **THEN** the command MUST be executed by the core command system as if the user sent `/exit`
- **AND** the chat response MUST include a command effect indicating exit
- **AND** the server MUST continue to accept subsequent requests for other sessions
