# server-core Specification

## ADDED Requirements

### Requirement: Chat Session Lifecycle Signaling
The server SHALL return enough information from the chat endpoint for clients to handle command-driven session lifecycle changes.

#### Scenario: Chat Response Includes Effective Session ID
Given a running server
And an existing session with id `S1`
When a client sends a chat request to `/api/v1/sessions/S1/chat`
Then the server SHALL return a successful response containing the assistant response
And it SHALL include the effective session id for subsequent requests

#### Scenario: New Session Command Returns New Session ID
Given a running server
And an existing session with id `S1`
When a client sends "/new" to `/api/v1/sessions/S1/chat`
Then the server SHALL create a new session with id `S2`
And it SHALL return `S2` in the chat response
And `S2` SHALL have an empty message history
