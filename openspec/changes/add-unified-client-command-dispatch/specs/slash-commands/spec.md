## MODIFIED Requirements

### Requirement: Slash Command Support
The system SHALL support executing commands triggered by a forward slash `/` at the start of a message.

The system SHALL support both:
- server-registered slash commands executed by the core command system
- client-local slash commands executed by the active client frontend

#### Scenario: Client-local command executes without server roundtrip
- **GIVEN** a client frontend registers a client-local slash command
- **WHEN** the user executes that slash command
- **THEN** the client SHALL execute it locally
- **AND** it SHALL NOT require a server roundtrip

#### Scenario: Client-local command forwards to server
- **GIVEN** a client-local slash command is configured to forward
- **WHEN** the user executes the command
- **THEN** the client SHALL forward a (possibly rewritten) message to the server
- **AND** the forwarded message SHALL be handled as a normal server chat input

