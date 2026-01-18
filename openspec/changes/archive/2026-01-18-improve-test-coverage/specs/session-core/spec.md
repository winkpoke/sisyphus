# session-core Specification (Delta)

## ADDED Requirements

### Requirement: Session Lifecycle Testing
The system SHALL provide comprehensive tests for session initialization, status transitions, and persistence to ensure session state is correctly managed throughout its lifecycle.

#### Scenario: New session initialization
- **GIVEN** no existing session
- **WHEN** a new session is created with ID
- **THEN** session ID is unique
- **AND** session status is `Idle`
- **AND** message history is empty
- **AND** pending approvals is empty
- **AND** pending batch is empty

#### Scenario: Session status transitions
- **GIVEN** a session in `Idle` status
- **WHEN** chat request is received
- **THEN** status transitions to `Running`
- **AND** session status is atomic (no intermediate states)

#### Scenario: Idle status recovery
- **GIVEN** a session in `Error` status
- **WHEN** session is recovered or reset
- **THEN** status transitions to `Idle`
- **AND** session can accept new requests
- **AND** error state is cleared

#### Scenario: Running status blocking
- **GIVEN** a session in `Running` status
- **WHEN** a concurrent chat request is attempted
- **THEN** request is rejected or queued
- **AND** no data corruption occurs
- **AND** session state remains consistent

### Requirement: Message History Testing
The system SHALL provide tests for message history append, retrieval, and limit enforcement to ensure conversation context is correctly maintained.

#### Scenario: Message history append
- **GIVEN** a session with existing messages
- **WHEN** a new message is appended
- **THEN** message is added to end of history
- **AND** message order is preserved
- **AND** history is not truncated unless at limit

#### Scenario: History limit enforcement
- **GIVEN** a session configured with max_messages limit
- **AND** history has reached limit
- **WHEN** a new message is appended
- **THEN** oldest messages are removed to maintain limit
- **AND** recent messages are preserved
- **AND** pinned messages are never removed

#### Scenario: Message retrieval by role
- **GIVEN** a session with mixed message history
- **WHEN** messages are filtered by role
- **THEN** correct messages are returned
- **AND** order is preserved
- **AND** tool calls and responses are correlated

#### Scenario: Tool result injection
- **GIVEN** a session with pending tool call
- **WHEN** tool result is received
- **THEN** result is appended with correct role
- **AND** result is correlated to tool call
- **AND** message history maintains turn structure

### Requirement: Persistence Testing
The system SHALL provide tests for session save and load operations to ensure conversation state can be persisted and restored.

#### Scenario: Session save to disk
- **GIVEN** a session with conversation history
- **AND** configured persistence path
- **WHEN** session is saved
- **THEN** session data is written to disk
- **AND** file contains all history
- **AND** file contains session metadata (ID, status)
- **AND** write is atomic (no corruption on crash)

#### Scenario: Session load from disk
- **GIVEN** a persisted session file
- **WHEN** session is loaded
- **THEN** all history is restored
- **AND** session metadata is restored
- **AND** session ID matches persisted value
- **AND** session status is restored correctly

#### Scenario: Invalid session load
- **GIVEN** a corrupted or malformed session file
- **WHEN** session is loaded
- **THEN** appropriate error is returned
- **AND** no invalid state is loaded
- **AND** error message is descriptive

#### Scenario: Concurrent session access
- **GIVEN** a session loaded by multiple tasks
- **WHEN** both tasks attempt to save simultaneously
- **THEN** saves are serialized safely
- **AND** no data corruption occurs
- **AND** last write wins (or appropriate locking)

### Requirement: Context Compaction Testing
The system SHALL provide tests for context compaction algorithm to ensure conversation history is correctly summarized when token limits are reached.

#### Scenario: Compaction trigger on token limit
- **GIVEN** a session with message history
- **AND** context approaching token limit
- **WHEN** a new message would exceed limit
- **THEN** compaction is triggered
- **AND** older messages are summarized or removed
- **AND** pinned messages are preserved
- **AND** token count is within limit

#### Scenario: Pinned message preservation
- **GIVEN** a session with pinned messages
- **AND** context compaction triggered
- **WHEN** messages are summarized
- **THEN** pinned messages are not removed
- **AND** pinned messages remain in context
- **AND** pinned messages retain original content

#### Scenario: Deterministic compaction
- **GIVEN** a session with identical message history
- **WHEN** compaction is triggered multiple times
- **THEN** result is deterministic each time
- **AND** no randomness in message selection
- **AND** summary format is consistent

### Requirement: Error Recovery Testing
The system SHALL provide tests for error state recovery to ensure sessions can recover from failures without corruption.

#### Scenario: Error state after provider failure
- **GIVEN** a session in `Running` status
- **AND** LLM provider fails
- **WHEN** error is encountered
- **THEN** session transitions to `Error` status
- **AND** partial history is preserved
- **AND** session can recover on retry

#### Scenario: Error state after tool failure
- **GIVEN** a session in `Running` status
- **AND** tool execution fails
- **WHEN** error is encountered
- **THEN** session transitions to `Error` status
- **AND** tool error is captured in history
- **AND** session can recover on retry

#### Scenario: Session reset after unrecoverable error
- **GIVEN** a session in `Error` status
- **WHEN** session is reset
- **THEN** status transitions to `Idle`
- **AND** session is ready for new requests
- **AND** history is not unnecessarily cleared

#### Scenario: Test cleanup on failure
- **GIVEN** a test with session state
- **WHEN** test fails during session operation
- **THEN** all temporary resources are cleaned up
- **AND** no file handles remain open
- **AND** no memory leaks occur

#### Scenario: Temp directory cleanup verification
- **GIVEN** a session test using tempfile
- **WHEN** test completes (success or failure)
- **THEN** temp directory is cleaned up automatically
- **AND** no test artifacts remain on disk
- **AND** cleanup is verified in test teardown
