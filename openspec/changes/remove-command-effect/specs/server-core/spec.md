## MODIFIED Requirements

### Requirement: HTTP Handlers Delegate Command Handling
The server HTTP layer MUST NOT interpret or apply slash command effects directly.

Instead, it MUST delegate chat request handling to a single core application service that:
- Executes SlashCommand parsing and expansion
- Executes normal chat turns for expanded or non-command input
- Produces the response DTO returned by the HTTP handler

#### Scenario: Server handlers are thin adapters
- **GIVEN** a running server
- **WHEN** a chat request is handled
- **THEN** the HTTP handler MUST delegate chat handling to the core service
- **AND** the HTTP handler MUST NOT independently mutate session state based on parsed command input

## REMOVED Requirements

### Requirement: Chat Session Lifecycle Signaling

## ADDED Requirements

### Requirement: Chat responses do not include command effects
The server chat response MUST NOT include any command-effect metadata.

#### Scenario: Chat response omits effect metadata
- **GIVEN** a running server
- **AND** an existing session with id `S1`
- **WHEN** a client sends a chat request to `/api/v1/sessions/S1/chat`
- **THEN** the server MUST return a successful response containing the assistant response
- **AND** the response MUST NOT include an `effect` field

### Requirement: Chat Response Session Context
The server SHALL return the session context identifier from the chat endpoint for clients to maintain conversation continuity.

The chat response MUST include:
- The assistant response content
- The session id to use for subsequent requests

The chat response MUST NOT include command execution effects or lifecycle signaling.

#### Scenario: Chat response provides session continuity
- **GIVEN** a running server
- **AND** an existing session with id `S1`
- **WHEN** a client sends a chat request to `/api/v1/sessions/S1/chat`
- **THEN** the server SHALL return a successful response containing the assistant response
- **AND** it SHALL include `S1` as the session id for subsequent requests
