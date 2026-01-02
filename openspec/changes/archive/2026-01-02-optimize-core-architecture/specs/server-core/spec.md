## ADDED Requirements

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
