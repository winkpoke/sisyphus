# Server Core Specifications

## ADDED Requirements

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
