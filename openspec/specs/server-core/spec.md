# server-core Specification

## Purpose
TBD - created by archiving change implement-server. Update Purpose after archive.
## Requirements
### Requirement: HTTP Server Initialization
The system SHALL provide a mechanism to start an HTTP server binding to a configured port.

#### Scenario: Start Server
Given a valid configuration
When the server start command is executed
Then the server should listen on the specified port (default 3000)
And a health check endpoint `/health` should return 200 OK.

### Requirement: Session Management API
The server SHALL provide REST endpoints to create and list sessions.

#### Scenario: Create and List Session
Given a running server
When a POST request is sent to `/api/v1/sessions`
Then a new session ID should be returned
And a subsequent GET request to `/api/v1/sessions` should include this ID.

### Requirement: Real-time Event Streaming
The server SHALL provide an SSE endpoint to stream system events.

#### Scenario: Subscribe to Events
Given a running server
When a client connects to `/api/v1/events`
And a `MessageReceived` event is published on the internal bus
Then the client should receive this event via the stream.

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

### Requirement: Efficient Session Listing
The system SHALL provide a lightweight representation of sessions for listing endpoints to optimize performance.

#### Scenario: Listing Sessions
Given a server with multiple sessions containing long chat histories
When a client requests `GET /api/v1/sessions`
Then the server should return a list of `SessionSummary` objects
And the response should not include the full message history
And the response size should remain small regardless of chat length

### Requirement: Session Summary Content
The system SHALL define a `SessionSummary` DTO containing only essential metadata.

#### Scenario: Session Summary Content
Given a `SessionSummary` object
It should contain the session ID
And it should contain the session status
And it should contain metadata (e.g., message count)
But it should not contain the `messages` array

### Requirement: Permission requests are streamed to clients
The server SHALL stream `PermissionRequest` system events to connected clients via the SSE endpoint.

#### Scenario: Client receives permission request event
- **GIVEN** a running server
- **AND** a client is connected to `/api/v1/events`
- **WHEN** the agent emits a `PermissionRequest` system event
- **THEN** the client SHALL receive the event data as JSON via the stream

### Requirement: Permission Approval API
The server SHALL provide an endpoint to submit approval decisions for Ask-gated tool execution.

#### Scenario: Approve a pending tool call
- **GIVEN** a running server
- **AND** a session `S1` has a pending permission request for tool call id `C1`
- **WHEN** the client sends `POST /api/v1/sessions/S1/approvals/C1` with decision `approve`
- **THEN** the server MUST execute the pending tool call
- **AND** the server MUST continue the assistant turn and return the assistant response

#### Scenario: Deny a pending tool call
- **GIVEN** a running server
- **AND** a session `S1` has a pending permission request for tool call id `C1`
- **WHEN** the client sends `POST /api/v1/sessions/S1/approvals/C1` with decision `deny`
- **THEN** the server MUST NOT execute the pending tool call
- **AND** the server MUST append a deterministic denial tool-result correlated to `C1`
- **AND** the server MUST continue the assistant turn and return the assistant response

#### Scenario: Reject unknown approvals
- **GIVEN** a running server
- **WHEN** the client submits an approval decision for a `(session_id, call_id)` pair with no pending request
- **THEN** the server MUST return a deterministic client error response

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

