## MODIFIED Requirements

### Requirement: Chat Session Lifecycle Signaling
The server SHALL return enough information from the chat endpoint for clients to render responses deterministically.

The chat response MUST include:
- The assistant response content
- The session id to use for subsequent requests

Chat responses MUST NOT be used to apply UiCommand-driven lifecycle changes.

#### Scenario: Chat response is stable for normal turns
- **GIVEN** a running server
- **AND** an existing session with id `S1`
- **WHEN** a client sends a chat request to `/api/v1/sessions/S1/chat`
- **THEN** the server SHALL return a successful response containing the assistant response
- **AND** it SHALL include `S1` as the session id for subsequent requests

## ADDED Requirements

### Requirement: SlashCommand discovery API
The server SHALL provide an endpoint to list discoverable SlashCommands available for the running server configuration.

The response MUST include, per command:
- `name` (including the leading `/`)
- `description`
- `source` with values `builtin` or `custom`

#### Scenario: List SlashCommands
- **GIVEN** a running server
- **WHEN** a client requests `GET /api/v1/slash-commands`
- **THEN** the server MUST return a JSON array of SlashCommand metadata
- **AND** command names MUST be unique within the response

### Requirement: Clear session history API
The server SHALL provide an endpoint to clear the message and tool-result history for a session.

#### Scenario: Clear session history
- **GIVEN** a running server
- **AND** an existing session `S1` with prior context
- **WHEN** a client sends `POST /api/v1/sessions/S1/clear`
- **THEN** the server MUST clear the session context
- **AND** a subsequent chat request for `S1` MUST NOT include prior context
