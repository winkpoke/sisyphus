# server-core Specification

## Purpose
Defines the HTTP server that exposes REST APIs for session management, agent discovery, slash commands, and tool approvals. Provides Server-Sent Events (SSE) endpoint for real-time event streaming to connected clients.
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
- Executes SlashCommand parsing and expansion
- Executes normal chat turns for expanded or non-command input
- Produces the response DTO returned by the HTTP handler

#### Scenario: Server handlers are thin adapters
- **GIVEN** a running server
- **WHEN** a chat request is handled
- **THEN** the HTTP handler MUST delegate chat handling to the core service
- **AND** the HTTP handler MUST NOT independently mutate session state based on parsed command input

### Requirement: Agent Discovery API
The server SHALL provide endpoints to discover available Agents and their model settings.

#### Scenario: List Agents
- **GIVEN** a running server with one or more configured Agents
- **WHEN** a client sends `GET /api/v1/agents`
- **THEN** the server MUST return a JSON array of Agent objects
- **AND** each object MUST include `id`, `name`, `description`, and `model` fields.

#### Scenario: Get Agent by ID
- **GIVEN** a running server with a configured Agent `A1`
- **WHEN** a client sends `GET /api/v1/agents/A1`
- **THEN** the server MUST return a JSON object with `id`, `name`, `description`, and `model` for `A1`
- **AND** the server MUST return a suitable 4xx error if `A1` does not exist.

### Requirement: Session Creation with Agent Selection
The server SHALL allow sessions to be created with an explicitly selected Agent.

#### Scenario: Create Session with Default Agent
- **GIVEN** a running server with a configured default Agent `A_default`
- **WHEN** a client sends `POST /api/v1/sessions` with no body or without `agent_id`
- **THEN** the server MUST create a new session associated with `A_default`
- **AND** the response MUST include the session id and the `agent_id` `A_default`.

#### Scenario: Create Session with Specific Agent
- **GIVEN** a running server with configured Agents `A1` and `A2`
- **WHEN** a client sends `POST /api/v1/sessions` with body `{ "agent_id": "A2" }`
- **THEN** the server MUST create a new session associated with `A2`
- **AND** the response MUST include the session id and the `agent_id` `A2`
- **AND** the server MUST return a suitable 4xx error if `agent_id` does not match any configured Agent.

### Requirement: Session Agent Management API
The server SHALL provide endpoints to inspect and change the Agent associated with an existing session.

#### Scenario: Get Session Agent
- **GIVEN** a running server and an existing session `S1` associated with Agent `A1`
- **WHEN** a client sends `GET /api/v1/sessions/S1`
- **THEN** the response MUST include the `agent_id` `A1` in the session metadata.

#### Scenario: Change Session Agent
- **GIVEN** a running server and an existing session `S1` associated with Agent `A1`
- **AND** the session is not currently processing a chat turn
- **WHEN** a client sends `PUT /api/v1/sessions/S1/agent` with body `{ "agent_id": "A2" }`
- **THEN** the server MUST update `S1` to be associated with `A2`
- **AND** the response MUST confirm the new `agent_id` `A2`.

#### Scenario: Prevent Agent Change While Busy
- **GIVEN** a running server and an existing session `S1` whose status is `Busy`
- **WHEN** a client sends `PUT /api/v1/sessions/S1/agent` with any body
- **THEN** the server MUST NOT change the session's `agent_id`
- **AND** the server MUST return a deterministic 4xx error indicating the session is busy.

### Requirement: Chat Response Includes Agent Metadata
The chat response SHALL include the effective Agent identity used for the turn.

#### Scenario: Chat Response Agent ID
- **GIVEN** a running server and an existing session `S1` associated with Agent `A1`
- **WHEN** a client sends `POST /api/v1/sessions/S1/chat`
- **THEN** the response MUST include the `agent_id` `A1`
- **AND** the `model` field in the response MUST reflect the model configured for `A1`.

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

### Requirement: Default Server Runtime Exposes Built-in Agents
When started with a default configuration, the server SHALL expose both built-in Agents via the Agent discovery API.

#### Scenario: Agent discovery includes plan and build
- **GIVEN** a running server started with default configuration
- **WHEN** a client sends `GET /api/v1/agents`
- **THEN** the response MUST include an Agent object with id `plan`
- **AND** the response MUST include an Agent object with id `build`

### Requirement: Default Agent is Plan
When no agent id is provided, the server MUST default sessions to the Plan agent.

#### Scenario: Create session defaults to plan
- **GIVEN** a running server started with default configuration
- **WHEN** a client sends `POST /api/v1/sessions` with no body or without `agent_id`
- **THEN** the server MUST treat the session as associated with agent id `plan`

### Requirement: Server Handler Testing
The system SHALL provide unit tests for HTTP handlers in `server/src/lib.rs` to verify request processing, validation, and response formatting.

#### Scenario: Session creation handler
- **GIVEN** a running server
- **WHEN** POST request to `/api/v1/sessions` is made
- **THEN** unique session ID is generated
- **AND** session is registered in session manager
- **AND** response includes session ID and metadata
- **AND** status code is 201 (Created)

#### Scenario: Session retrieval handler
- **GIVEN** a server with existing session
- **WHEN** GET request to `/api/v1/sessions/:id` is made
- **THEN** session metadata is returned
- **AND** response includes session history (if requested)
- **AND** status code is 200 (OK)

#### Scenario: Session not found handler
- **GIVEN** a server without session ID
- **WHEN** GET request to `/api/v1/sessions/:id` is made for non-existent ID
- **THEN** status code is 404 (Not Found)
- **AND** error message indicates session not found
- **AND** response is valid JSON

#### Scenario: Chat message handler
- **GIVEN** a server with active session
- **WHEN** POST request to `/api/v1/sessions/:id/chat` is made
- **THEN** request is forwarded to agent
- **AND** response includes agent response or permission request
- **AND** session history is updated
- **AND** status code is 200 (OK)

#### Scenario: Approval submission handler
- **GIVEN** a server with pending approval
- **WHEN** POST request to `/api/v1/sessions/:id/approvals/:call_id` is made
- **THEN** approval decision is processed
- **AND** tool execution is resumed or blocked accordingly
- **AND** response includes final agent response
- **AND** status code is 200 (OK)

#### Scenario: Agent discovery handler
- **GIVEN** a server with registered agents
- **WHEN** GET request to `/api/v1/agents` is made
- **THEN** all registered agents are returned
- **AND** each agent includes ID, name, description, model
- **AND** status code is 200 (OK)

#### Scenario: Invalid request validation
- **GIVEN** a server with HTTP handlers
- **WHEN** request is missing required fields
- **THEN** status code is 400 (Bad Request)
- **AND** error message indicates missing field
- **AND** response is valid JSON

### Requirement: Middleware Testing
The system SHALL provide tests for middleware components (CORS, error handling, logging) to ensure request/response processing is correct.

#### Scenario: CORS headers
- **GIVEN** a server with CORS configured
- **WHEN** OPTIONS request is made
- **THEN** appropriate CORS headers are returned
- **AND** Access-Control-Allow-Origin is set correctly
- **AND** preflight requests succeed

#### Scenario: Error response formatting
- **GIVEN** a server encountering error
- **WHEN** error response is generated
- **THEN** response is valid JSON
- **AND** response includes error message
- **AND** status code is appropriate (4xx, 5xx)
- **AND** no stack traces are leaked

#### Scenario: Request logging
- **GIVEN** a server with tracing configured
- **WHEN** request is processed
- **THEN** request details are logged
- **AND** response details are logged
- **AND** logs include correlation ID (if configured)

### Requirement: WebSocket Testing (if applicable)
The system SHALL provide tests for WebSocket upgrade and message dispatch if WebSocket support is implemented.

#### Scenario: WebSocket upgrade handling
- **GIVEN** a server with WebSocket support
- **WHEN** WebSocket upgrade request is made
- **THEN** connection is upgraded successfully
- **AND** session is associated with connection
- **AND** upgrade headers are correct

#### Scenario: WebSocket message dispatch
- **GIVEN** a server with active WebSocket connection
- **WHEN** message is received from client
- **THEN** message is dispatched to appropriate handler
- **AND** session context is maintained
- **AND** response is sent back over WebSocket

#### Scenario: WebSocket error handling
- **GIVEN** a server with active WebSocket connection
- **WHEN** invalid message is received
- **THEN** error response is sent over WebSocket
- **AND** connection remains open (unless critical error)
- **AND** session state is not corrupted

### Requirement: SSE Event Streaming Testing
The system SHALL provide tests for Server-Sent Events endpoint to ensure real-time event streaming works correctly.

#### Scenario: SSE connection handling
- **GIVEN** a server with SSE endpoint
- **WHEN** client connects to SSE endpoint
- **THEN** connection is accepted
- **AND** appropriate headers are set (Content-Type: text/event-stream)
- **AND** connection stays open

#### Scenario: SSE event formatting
- **GIVEN** a server with active session
- **WHEN** events are emitted (PermissionRequest, ToolResult, ChatResponse)
- **THEN** events are formatted correctly as SSE
- **AND** event type is correctly set
- **AND** event data is valid JSON
- **AND** event sequence is maintained

#### Scenario: SSE chunk parsing (client-side)
- **GIVEN** a client receiving SSE stream
- **WHEN** chunks are received
- **THEN** split network chunks are reassembled correctly
- **AND** multi-byte characters are handled
- **AND** events are complete before parsing

### Requirement: Server Integration Testing
The system SHALL provide integration tests for full request lifecycles and concurrent access to verify server behavior under realistic conditions.

#### Scenario: Full request lifecycle integration test
- **GIVEN** a running server
- **WHEN** client creates session, sends message, receives response
- **THEN** session is created successfully
- **AND** message is processed by agent
- **AND** response is returned correctly
- **AND** session history is updated

#### Scenario: Concurrent request handling
- **GIVEN** a running server
- **WHEN** multiple clients make simultaneous requests
- **THEN** all requests are processed correctly
- **AND** no data corruption occurs
- **AND** responses are returned to correct clients

#### Scenario: Load test with multiple sessions
- **GIVEN** a running server
- **WHEN** multiple clients create sessions and send messages
- **THEN** sessions are isolated from each other
- **AND** no cross-session contamination occurs
- **AND** server remains stable under load

