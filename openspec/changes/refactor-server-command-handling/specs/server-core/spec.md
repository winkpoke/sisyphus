## MODIFIED Requirements

### Requirement: Chat Session Lifecycle Signaling
The server SHALL return enough information from the chat endpoint for clients to handle command-driven session lifecycle and UX changes deterministically.

The chat response MUST include:
- The assistant response content
- The effective session id to use for subsequent requests
- The command effect (if any) produced by executing slash commands for that request

#### Scenario: Chat response includes effective session id
- **GIVEN** a running server
- **AND** an existing session with id `S1`
- **WHEN** a client sends a chat request to `/api/v1/sessions/S1/chat`
- **THEN** the server SHALL return a successful response containing the assistant response
- **AND** it SHALL include the effective session id for subsequent requests

#### Scenario: New session command returns new session id
- **GIVEN** a running server
- **AND** an existing session with id `S1`
- **WHEN** a client sends "/new" to `/api/v1/sessions/S1/chat`
- **THEN** the server SHALL create a new session with id `S2`
- **AND** it SHALL return `S2` as the effective session id in the chat response
- **AND** `S2` SHALL have an empty message history
- **AND** it SHALL return a command effect indicating a new session was created

#### Scenario: Command effect is returned for client UX
- **GIVEN** a running server
- **AND** an existing session with id `S1`
- **WHEN** a client sends a slash command that produces a command effect to `/api/v1/sessions/S1/chat`
- **THEN** the server SHALL return the command effect in the chat response
- **AND** the command effect value MUST be deterministic for that command

### Requirement: HTTP Handlers Delegate Command Handling
The server HTTP layer MUST NOT interpret or apply slash command effects directly.

Instead, it MUST delegate chat request handling to a single core application service that:
- Executes slash commands via the core command system
- Applies the resulting command effects to session state
- Produces the response DTO returned by the HTTP handler

#### Scenario: Server handlers are thin adapters
- **GIVEN** a running server
- **WHEN** a chat request is handled
- **THEN** the HTTP handler MUST delegate command handling and effect application to the core service
- **AND** the HTTP handler MUST NOT independently mutate session state based on command effects

