# Spec: Session Core

## ADDED Requirements

### Requirement: Session State Management
The system SHALL maintain the state of a conversation independently of the agent execution logic.

#### Scenario: Create new session
- **WHEN** a new session is requested via SessionManager
- **THEN** a new Session object is created with a unique ID
- **AND** the session status is initialized to "Idle"
- **AND** the history is empty

#### Scenario: Session Locking
- **WHEN** an agent begins processing a turn for a session
- **THEN** the session status transitions to "Busy"
- **WHEN** the agent finishes processing
- **THEN** the session status transitions back to "Idle"

#### Scenario: Prevent concurrent access
- **WHEN** a request is made to process a session that is "Busy"
- **THEN** the system throws/returns a `SessionBusy` error

### Requirement: Session History
The system SHALL store the linear history of messages (User, Assistant, System, Tool) within the Session object.

#### Scenario: Append message
- **WHEN** a message is added to the session
- **THEN** it is appended to the history list
- **AND** available for the next context window generation

## MODIFIED Requirements

### Requirement: Stateless Agent Execution
The Agent SHALL NOT maintain internal session state.

#### Scenario: Execute Turn
- **WHEN** the Agent executes a chat turn
- **THEN** it accepts a `Session` reference as an input
- **AND** reads history from that Session
- **AND** writes new messages to that Session
