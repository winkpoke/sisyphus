# server-core Specification (Delta)

## ADDED Requirements

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
