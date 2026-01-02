## MODIFIED Requirements

### Requirement: Granular Session Locking
The system SHALL support granular locking for sessions to prevent global blocking during long-running operations.

#### Scenario: Concurrent Session Access
Given a running server with multiple active sessions
When one user initiates a chat request that triggers a long-running LLM inference
Then other users should still be able to create new sessions
And other users should still be able to list sessions
And other users should still be able to chat in their own sessions

### Requirement: Session Isolation
The system SHALL isolate session locks so that accessing one session does not block others.

#### Scenario: Session Isolation
Given a `SessionManager` using `DashMap`
When a session is accessed via `get_session`
Then the returned reference should be an `Arc<RwLock<Session>>`
And locking this session should not block access to other sessions in the map
