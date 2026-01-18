# session-core Specification

## Purpose
Defines session management architecture including conversation history storage, context compaction with token budgeting, turn-based message tracking, and pending approval state. Sessions track associated agents and provide concurrent-safe access via granular locking.
## Requirements
### Requirement: Session History
The system SHALL store the linear history of conversation messages (User, Assistant, Tool) within the Session object via a dedicated context component.

The system prompt is generated dynamically by the Agent and injected at completion request time; it is not required to be stored in the session context.

#### Scenario: Append message
- **WHEN** a message is added to the session
- **THEN** it is appended to the session context
- **AND** it is available for the next context window generation

#### Scenario: Preserve multi-step turn ordering
- **GIVEN** a single user input that triggers multiple assistant/tool steps
- **WHEN** those steps are appended to the session
- **THEN** the session context preserves their order deterministically
- **AND** the context window generation reflects the same order

### Requirement: Stateless Agent Execution
The Agent SHALL NOT maintain internal session state.

#### Scenario: Execute Turn
- **WHEN** the Agent executes a chat turn
- **THEN** it accepts a `Session` reference as an input
- **AND** reads history from that Session
- **AND** writes new messages to that Session

### Requirement: Context Window Construction
The system SHALL construct a provider-ready context window from the session context for each LLM completion request.

#### Scenario: Render context window
- **GIVEN** a session context containing prior messages
- **WHEN** the system builds a completion request
- **THEN** the system MUST produce an ordered list of messages suitable for the LLM provider

#### Scenario: Inject system messages at request time
- **GIVEN** a session context containing prior conversation messages
- **WHEN** the system builds a completion request
- **THEN** system messages (such as the dynamic system prompt) are prepended for that request
- **AND** they do not need to be stored in the session context

### Requirement: Safe Context Compaction
The system SHALL support compacting the session context to fit within a configurable prompt token budget.

#### Scenario: Compact by dropping oldest blocks
- **GIVEN** a session context that exceeds the prompt token budget
- **WHEN** the system compacts the context
- **THEN** the system MUST remove the oldest non-pinned context blocks until within budget
- **AND** pinned context blocks MUST be preserved

#### Scenario: Preserve tool exchange atomicity
- **GIVEN** a session context containing an Assistant message with tool calls and the corresponding Tool result messages
- **WHEN** the system compacts the context
- **THEN** it MUST NOT remove only part of the tool exchange

#### Scenario: Fail deterministically when pinned content exceeds budget
- **GIVEN** pinned content and injected system messages exceed the prompt token budget
- **WHEN** the system attempts to build a completion request
- **THEN** it MUST return a deterministic error describing the budget violation

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

### Requirement: Session Tracks Current Agent
The Session entity SHALL track the identity of the Agent responsible for executing its chat turns.

#### Scenario: Session Stores Agent ID
- **WHEN** a new session is created by the SessionManager
- **THEN** the Session MUST store an `agent_id` field representing the associated Agent
- **AND** this field MUST be included in the Session's serialized representation used by the server API.

#### Scenario: Agent ID is Stable During Turn
- **GIVEN** an existing session `S1` with `agent_id` `A1`
- **WHEN** the Agent begins processing a chat turn for `S1`
- **THEN** the `agent_id` used for that turn MUST remain `A1` for the duration of the turn
- **AND** any concurrent request to change `S1`'s `agent_id` MUST be rejected according to the server API.

### Requirement: Session context mutations are not driven by SlashCommands
The system SHALL NOT require SlashCommands to mutate session lifecycle state.

#### Scenario: SlashCommand expansion does not clear context
- **GIVEN** a session with prior context
- **WHEN** the user executes a SlashCommand that expands to prompt text
- **THEN** the session context MUST remain intact unless an explicit session operation is invoked

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

