## REMOVED Requirements

### Requirement: Session State Management
### Requirement: Start a new session via command
### Requirement: Clear history via command

## ADDED Requirements

### Requirement: Session Lifecycle Management
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

#### Scenario: Start a new session via API
- **GIVEN** an existing session with prior context
- **WHEN** a "new session" operation is executed
- **THEN** the session context is cleared
- **AND** the session status remains consistent with the operation lifecycle

#### Scenario: Clear operation preserves lifecycle correctness
- **GIVEN** an existing session with prior context
- **WHEN** the session context is cleared via an explicit clear operation
- **THEN** the session context is cleared before the next completion request
- **AND** the session status remains consistent with the operation lifecycle

### Requirement: Start a new session via API
The system SHALL support starting a new session via an explicit session-creation API.

Session creation MUST produce a new session id and an empty session context.

#### Scenario: Create new session produces empty context
- **GIVEN** an existing session `S1` with prior context
- **WHEN** a new session is created via the session manager
- **THEN** the new session `S2` MUST have an empty message history
- **AND** the new session `S2` MUST have an empty tool-result history

### Requirement: Clear history via API
The system SHALL support clearing an existing session context via a dedicated clear operation.

#### Scenario: Clear operation clears context
- **GIVEN** an existing session with prior context
- **WHEN** the clear operation is executed for that session
- **THEN** the session context MUST be cleared before the next completion request
