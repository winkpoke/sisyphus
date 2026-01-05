## MODIFIED Requirements

### Requirement: Session State Management
The system SHALL maintain the state of a conversation independently of the agent execution logic.

#### Scenario: Create new session
- **WHEN** a new session is requested via SessionManager
- **THEN** a new Session object is created with a unique ID
- **AND** the session status is initialized to "Idle"
- **AND** the session context is empty

#### Scenario: Session Locking
- **WHEN** an agent begins processing a turn for a session
- **THEN** the session status transitions to "Busy"
- **WHEN** the agent finishes processing
- **THEN** the session status transitions back to "Idle"

#### Scenario: Prevent concurrent access
- **WHEN** a request is made to process a session that is "Busy"
- **THEN** the system throws/returns a `SessionBusy` error

#### Scenario: Clear operation preserves lifecycle correctness
- **GIVEN** an existing session with prior context
- **WHEN** the session context is cleared via an explicit clear operation
- **THEN** the session context is cleared before the next completion request
- **AND** the session status remains consistent with the operation lifecycle
